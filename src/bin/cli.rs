//! CLI tool for converting Mixamo 3D animations to 2D skeletal animations
//!
//! This binary requires the `cli` feature to be enabled.
//!
//! # Usage
//!
//! ```bash
//! # Convert a single FBX file
//! mixamo2d convert animation.fbx -o output/
//!
//! # Convert with custom settings
//! mixamo2d convert animation.fbx -o output/ --scale 100 --view-axis z
//!
//! # Batch convert all FBX files in a directory
//! mixamo2d batch input_dir/ -o output/
//!
//! # Inspect a file without converting
//! mixamo2d info animation.fbx
//! ```

use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

use bevy_mixamo_2d::{
    fbx_parser::{self, FbxData},
    gltf_parser::{self, GltfData},
    project_animation, project_skeleton, save_animation, save_skeleton, ProjectionConfig, ViewAxis,
};

/// Convert Mixamo 3D animations to 2D skeletal animations for Bevy
#[derive(Parser)]
#[command(name = "mixamo2d")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a single FBX or GLB file to 2D skeleton and animation
    Convert {
        /// Input file path (FBX or GLB)
        input: PathBuf,

        /// Output directory for generated RON files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Custom skeleton name (defaults to input filename)
        #[arg(short, long)]
        name: Option<String>,

        /// View axis for projection (which axis the camera looks along)
        #[arg(long, value_enum, default_value = "z")]
        view_axis: ViewAxisArg,

        /// Scale factor for 3D to 2D conversion (e.g., 100 for meters to pixels)
        #[arg(long, default_value = "100.0")]
        scale: f32,

        /// Enable foreshortening scale based on depth
        #[arg(long, default_value = "true")]
        foreshortening: bool,

        /// Output animation sample rate in FPS
        #[arg(long, default_value = "30.0")]
        sample_rate: f32,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Batch convert all FBX/GLB files in a directory
    Batch {
        /// Input directory containing FBX/GLB files
        input: PathBuf,

        /// Output directory for generated RON files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// View axis for projection
        #[arg(long, value_enum, default_value = "z")]
        view_axis: ViewAxisArg,

        /// Scale factor for 3D to 2D conversion
        #[arg(long, default_value = "100.0")]
        scale: f32,

        /// Enable foreshortening scale based on depth
        #[arg(long, default_value = "true")]
        foreshortening: bool,

        /// Output animation sample rate in FPS
        #[arg(long, default_value = "30.0")]
        sample_rate: f32,

        /// Recursive search in subdirectories
        #[arg(short, long)]
        recursive: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Display information about an FBX or GLB file without converting
    Info {
        /// Input file path (FBX or GLB)
        input: PathBuf,

        /// Show detailed bone hierarchy
        #[arg(long)]
        bones: bool,

        /// Show animation details
        #[arg(long)]
        animations: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ViewAxisArg {
    /// Camera looks along X axis (sees YZ plane)
    X,
    /// Camera looks along Y axis (sees XZ plane)
    Y,
    /// Camera looks along Z axis (sees XY plane) - typical side view
    Z,
}

impl From<ViewAxisArg> for ViewAxis {
    fn from(arg: ViewAxisArg) -> Self {
        match arg {
            ViewAxisArg::X => ViewAxis::X,
            ViewAxisArg::Y => ViewAxis::Y,
            ViewAxisArg::Z => ViewAxis::Z,
        }
    }
}

/// Conversion options shared between convert and batch commands
struct ConversionOptions {
    view_axis: ViewAxis,
    scale: f32,
    foreshortening: bool,
    sample_rate: f32,
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Convert {
            input,
            output,
            name,
            view_axis,
            scale,
            foreshortening,
            sample_rate,
            verbose,
        } => {
            let opts = ConversionOptions {
                view_axis: view_axis.into(),
                scale,
                foreshortening,
                sample_rate,
                verbose,
            };
            convert_single(&input, &output, name.as_deref(), &opts)
        }
        Commands::Batch {
            input,
            output,
            view_axis,
            scale,
            foreshortening,
            sample_rate,
            recursive,
            verbose,
        } => {
            let opts = ConversionOptions {
                view_axis: view_axis.into(),
                scale,
                foreshortening,
                sample_rate,
                verbose,
            };
            convert_batch(&input, &output, &opts, recursive)
        }
        Commands::Info {
            input,
            bones,
            animations,
        } => show_info(&input, bones, animations),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Convert a single file
fn convert_single(
    input: &Path,
    output: &Path,
    name: Option<&str>,
    opts: &ConversionOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let extension = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let skeleton_name = name.map(|s| s.to_string()).unwrap_or_else(|| {
        input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("skeleton")
            .to_string()
    });

    let config = ProjectionConfig {
        view_axis: opts.view_axis,
        scale: opts.scale,
        enable_foreshortening: opts.foreshortening,
        sample_rate: opts.sample_rate,
    };

    if opts.verbose {
        println!("Converting: {}", input.display());
        println!("  Skeleton name: {}", skeleton_name);
        println!("  View axis: {:?}", opts.view_axis);
        println!("  Scale: {}", opts.scale);
        println!("  Foreshortening: {}", opts.foreshortening);
        println!("  Sample rate: {} FPS", opts.sample_rate);
    }

    // Parse input file
    let (nodes_3d, animations_3d) = match extension.as_str() {
        "fbx" => {
            if opts.verbose {
                println!("  Parser: FBX (ufbx)");
            }
            let data = FbxData::load(input)?;
            let nodes = fbx_parser::extract_skeleton(&data)?;
            let anims = fbx_parser::extract_animations(&data)?;
            (nodes, anims)
        }
        "glb" | "gltf" => {
            if opts.verbose {
                println!("  Parser: GLTF/GLB");
            }
            let data = GltfData::load(input)?;
            let nodes = gltf_parser::extract_skeleton(&data)?;
            let anims = gltf_parser::extract_animations(&data).unwrap_or_default();
            (nodes, anims)
        }
        _ => {
            return Err(format!(
                "Unsupported file format: {}. Use .fbx, .glb, or .gltf",
                extension
            )
            .into());
        }
    };

    if opts.verbose {
        println!("  3D bones: {}", nodes_3d.len());
        println!("  3D animations: {}", animations_3d.len());
    }

    // Project to 2D
    let skeleton_2d = project_skeleton(&nodes_3d, &skeleton_name, &config)?;

    if opts.verbose {
        println!("  2D bones: {}", skeleton_2d.bone_count());
    }

    // Ensure output directory exists
    std::fs::create_dir_all(output)?;

    // Save skeleton
    let skeleton_path = output.join(format!("{}.skeleton2d.ron", skeleton_name));
    save_skeleton(&skeleton_2d, &skeleton_path)?;
    println!("Saved skeleton: {}", skeleton_path.display());

    // Project and save each animation
    for anim_3d in &animations_3d {
        let anim_2d = project_animation(anim_3d, &nodes_3d, &skeleton_name, &config)?;

        let anim_name = if anim_3d.name.is_empty() || anim_3d.name == "Take 001" {
            skeleton_name.clone()
        } else {
            sanitize_filename(&anim_3d.name)
        };

        let anim_path = output.join(format!("{}.anim2d.ron", anim_name));
        save_animation(&anim_2d, &anim_path)?;
        println!(
            "Saved animation: {} ({:.2}s, {} timelines)",
            anim_path.display(),
            anim_2d.duration,
            anim_2d.timelines.len()
        );
    }

    Ok(())
}

/// Batch convert all files in a directory
fn convert_batch(
    input: &Path,
    output: &Path,
    opts: &ConversionOptions,
    recursive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let files = collect_input_files(input, recursive)?;

    if files.is_empty() {
        println!("No FBX or GLB files found in: {}", input.display());
        return Ok(());
    }

    println!("Found {} files to convert", files.len());

    let mut success_count = 0;
    let mut error_count = 0;

    for file in &files {
        match convert_single(file, output, None, opts) {
            Ok(()) => success_count += 1,
            Err(e) => {
                eprintln!("Failed to convert {}: {}", file.display(), e);
                error_count += 1;
            }
        }
    }

    println!(
        "\nBatch conversion complete: {} succeeded, {} failed",
        success_count, error_count
    );

    Ok(())
}

/// Collect input files from directory
fn collect_input_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    if recursive {
        collect_files_recursive(dir, &mut files)?;
    } else {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if is_supported_file(&path) {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, files)?;
        } else if is_supported_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_lowercase().as_str(), "fbx" | "glb" | "gltf"))
}

/// Show information about a file
fn show_info(
    input: &Path,
    show_bones: bool,
    show_animations: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let extension = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    println!("File: {}", input.display());
    println!("Format: {}", extension.to_uppercase());

    let (nodes_3d, animations_3d, parser_name) = match extension.as_str() {
        "fbx" => {
            let data = FbxData::load(input)?;
            let nodes = fbx_parser::extract_skeleton(&data)?;
            let anims = fbx_parser::extract_animations(&data).unwrap_or_default();
            (nodes, anims, "FBX (ufbx)")
        }
        "glb" | "gltf" => {
            let data = GltfData::load(input)?;
            let nodes = gltf_parser::extract_skeleton(&data)?;
            let anims = gltf_parser::extract_animations(&data).unwrap_or_default();
            (nodes, anims, "GLTF/GLB")
        }
        _ => {
            return Err(format!(
                "Unsupported file format: {}. Use .fbx, .glb, or .gltf",
                extension
            )
            .into());
        }
    };

    println!("Parser: {}", parser_name);
    println!();
    println!("Skeleton:");
    println!("  Bones: {}", nodes_3d.len());

    if let Some(root) = nodes_3d.first() {
        println!("  Root: {}", root.name);
    }

    // Count bones by depth
    let max_depth = calculate_max_depth(&nodes_3d);
    println!("  Max depth: {}", max_depth);

    // Show bones by default if neither flag is set
    let show_both = !show_bones && !show_animations;

    if show_bones || show_both {
        println!();
        println!("Bone hierarchy:");
        print_bone_hierarchy(&nodes_3d, 0, 0);
    }

    println!();
    println!("Animations: {}", animations_3d.len());

    if show_animations || show_both {
        for (i, anim) in animations_3d.iter().enumerate() {
            println!();
            println!("  [{}] {}", i, anim.name);
            println!("      Duration: {:.3}s", anim.duration);
            println!("      Channels: {}", anim.channels.len());

            // Count keyframes
            let total_keyframes: usize = anim.channels.iter().map(|c| c.keyframes.len()).sum();
            println!("      Keyframes: {}", total_keyframes);

            // Count by property type
            let translation_channels = anim
                .channels
                .iter()
                .filter(|c| matches!(c.property, gltf_parser::ChannelProperty::Translation))
                .count();
            let rotation_channels = anim
                .channels
                .iter()
                .filter(|c| matches!(c.property, gltf_parser::ChannelProperty::Rotation))
                .count();
            let scale_channels = anim
                .channels
                .iter()
                .filter(|c| matches!(c.property, gltf_parser::ChannelProperty::Scale))
                .count();

            println!(
                "      Properties: {} translation, {} rotation, {} scale",
                translation_channels, rotation_channels, scale_channels
            );
        }
    }

    Ok(())
}

fn calculate_max_depth(nodes: &[gltf_parser::GltfNode3D]) -> usize {
    let mut max_depth = 0;

    for node in nodes {
        let mut depth = 0;
        let mut current_parent = node.parent_index;
        while let Some(parent_idx) = current_parent {
            depth += 1;
            if parent_idx < nodes.len() {
                current_parent = nodes[parent_idx].parent_index;
            } else {
                break;
            }
        }
        max_depth = max_depth.max(depth);
    }

    max_depth
}

fn print_bone_hierarchy(nodes: &[gltf_parser::GltfNode3D], node_idx: usize, depth: usize) {
    if node_idx >= nodes.len() {
        return;
    }

    let node = &nodes[node_idx];
    let indent = "  ".repeat(depth + 1);
    println!("{}[{}] {}", indent, node_idx, node.name);

    // Find children
    for (child_idx, child) in nodes.iter().enumerate() {
        if child.parent_index == Some(node_idx) {
            print_bone_hierarchy(nodes, child_idx, depth + 1);
        }
    }
}

/// Sanitize a string for use as a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
