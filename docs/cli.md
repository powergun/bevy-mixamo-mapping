# CLI Reference

This document describes the `mixamo2d` command-line tool for converting Mixamo 3D animations to 2D skeletal animations. Designed for coding agents to understand the CLI architecture and usage.

## Overview

The CLI orchestrates the full conversion pipeline:

```
FBX/GLB File → Parse → Project → Serialize → RON Files
                 ↓        ↓          ↓
              FbxData  Skeleton2D  *.skeleton2d.ron
              GltfData Animation2D *.anim2d.ron
```

## Module Location

```
src/bin/cli.rs  (~540 lines)
```

## Building

The CLI requires the `cli` feature:

```bash
# Build
cargo build --features cli

# Install locally
cargo install --features cli --path .

# Run directly
cargo run --features cli -- <command>
```

## Commands

### `convert` - Single File Conversion

Converts one FBX or GLB file to 2D skeleton and animation RON files.

```bash
mixamo2d convert <INPUT> [OPTIONS]
```

**Arguments:**
- `<INPUT>` - Path to FBX, GLB, or GLTF file

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output` | `-o` | Output directory | `.` (current dir) |
| `--name` | `-n` | Custom skeleton name | Input filename |
| `--view-axis` | | Projection axis (x, y, z) | `z` |
| `--scale` | | 3D to 2D scale factor | `100.0` |
| `--foreshortening` | | Enable depth scaling | `true` |
| `--sample-rate` | | Output FPS | `30.0` |
| `--verbose` | `-v` | Verbose output | `false` |

**Examples:**

```bash
# Basic conversion
mixamo2d convert walk.fbx -o output/

# Custom name and settings
mixamo2d convert walk.fbx -o output/ -n player --scale 50 -v

# Side view projection (default)
mixamo2d convert walk.fbx --view-axis z

# Top-down view projection
mixamo2d convert walk.fbx --view-axis y
```

**Output Files:**
- `{name}.skeleton2d.ron` - Skeleton definition
- `{animation_name}.anim2d.ron` - Animation clip(s)

### `batch` - Batch Directory Conversion

Converts all FBX/GLB files in a directory.

```bash
mixamo2d batch <INPUT> [OPTIONS]
```

**Arguments:**
- `<INPUT>` - Directory containing FBX/GLB files

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output` | `-o` | Output directory | `.` |
| `--recursive` | `-r` | Search subdirectories | `false` |
| `--view-axis` | | Projection axis | `z` |
| `--scale` | | Scale factor | `100.0` |
| `--foreshortening` | | Depth scaling | `true` |
| `--sample-rate` | | Output FPS | `30.0` |
| `--verbose` | `-v` | Verbose output | `false` |

**Examples:**

```bash
# Convert all files in directory
mixamo2d batch animations/ -o output/

# Recursive with verbose
mixamo2d batch animations/ -o output/ -r -v
```

### `info` - File Inspection

Displays information about a file without converting.

```bash
mixamo2d info <INPUT> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--bones` | Show only bone hierarchy |
| `--animations` | Show only animation details |

By default, shows both bones and animations.

**Examples:**

```bash
# Full info
mixamo2d info walk.fbx

# Bones only
mixamo2d info walk.fbx --bones

# Animations only
mixamo2d info walk.fbx --animations
```

**Sample Output:**

```
File: testdata/Sprint.fbx
Format: FBX
Parser: FBX (ufbx)

Skeleton:
  Bones: 65
  Root: mixamorig:Hips
  Max depth: 11

Bone hierarchy:
  [0] mixamorig:Hips
    [1] mixamorig:Spine
      [2] mixamorig:Spine1
        ...

Animations: 1

  [0] mixamo.com
      Duration: 0.533s
      Channels: 156
      Keyframes: 931
      Properties: 52 translation, 52 rotation, 52 scale
```

## Architecture

### Code Structure

```rust
// src/bin/cli.rs

// Clap argument parsing
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Convert { /* ... */ },
    Batch { /* ... */ },
    Info { /* ... */ },
}

// Shared conversion options
struct ConversionOptions {
    view_axis: ViewAxis,
    scale: f32,
    foreshortening: bool,
    sample_rate: f32,
    verbose: bool,
}
```

### Conversion Flow

```rust
fn convert_single(
    input: &Path,
    output: &Path,
    name: Option<&str>,
    opts: &ConversionOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Detect format by extension
    let extension = input.extension()...;

    // 2. Parse input file
    let (nodes_3d, animations_3d) = match extension {
        "fbx" => {
            let data = FbxData::load(input)?;
            let nodes = fbx_parser::extract_skeleton(&data)?;
            let anims = fbx_parser::extract_animations(&data)?;
            (nodes, anims)
        }
        "glb" | "gltf" => {
            let data = GltfData::load(input)?;
            let nodes = gltf_parser::extract_skeleton(&data)?;
            let anims = gltf_parser::extract_animations(&data)?;
            (nodes, anims)
        }
        _ => return Err(...)
    };

    // 3. Configure projection
    let config = ProjectionConfig {
        view_axis: opts.view_axis,
        scale: opts.scale,
        enable_foreshortening: opts.foreshortening,
        sample_rate: opts.sample_rate,
    };

    // 4. Project to 2D
    let skeleton_2d = project_skeleton(&nodes_3d, &skeleton_name, &config)?;

    // 5. Save skeleton
    save_skeleton(&skeleton_2d, &skeleton_path)?;

    // 6. Project and save each animation
    for anim_3d in &animations_3d {
        let anim_2d = project_animation(&anim_3d, &nodes_3d, &skeleton_name, &config)?;
        save_animation(&anim_2d, &anim_path)?;
    }

    Ok(())
}
```

### Parser Selection

The CLI prefers FBX over GLB because:
1. FBX comes directly from Mixamo with clean bone names (`mixamorig:Hips`)
2. GLB files from FBX conversion may have duplicate bones
3. GLB conversion may lose animation data

```rust
match extension.as_str() {
    "fbx" => /* FbxData::load - primary */ ,
    "glb" | "gltf" => /* GltfData::load - fallback */ ,
    _ => /* error */
}
```

## View Axis Explanation

The `--view-axis` option controls how 3D coordinates project to 2D:

| View Axis | Camera Looks Along | Visible Plane | Use Case |
|-----------|-------------------|---------------|----------|
| `x` | +X axis | YZ plane | Front/back view |
| `y` | +Y axis | XZ plane | Top-down view |
| `z` | +Z axis | XY plane | **Side view (default)** |

For side-scrolling games, use `z` (default):
- 3D X → 2D X (horizontal movement)
- 3D Y → 2D Y (vertical movement)
- 3D Z → depth (foreshortening)

## Error Handling

The CLI uses `Box<dyn std::error::Error>` for ergonomic error propagation:

```rust
fn main() {
    let result = match cli.command {
        Commands::Convert { .. } => convert_single(...),
        Commands::Batch { .. } => convert_batch(...),
        Commands::Info { .. } => show_info(...),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

Errors bubble up from:
- File I/O (`std::io::Error`)
- FBX parsing (`Mixamo2dError::FbxParse`)
- GLTF parsing (`Mixamo2dError::GltfParse`)
- Validation (`ValidationError`)
- RON serialization (`ron::error::SpannedError`)

## Batch Processing

Batch mode processes files sequentially, continuing on errors:

```rust
fn convert_batch(...) {
    let files = collect_input_files(input, recursive)?;

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

    println!("Batch complete: {} succeeded, {} failed", success_count, error_count);
}
```

## File Detection

Supported file extensions:

```rust
fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_lowercase().as_str(), "fbx" | "glb" | "gltf"))
}
```

## Filename Sanitization

Animation names are sanitized for filesystem safety:

```rust
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
```

Example: `"mixamo.com"` → `"mixamo_com"`

## Dependencies

```toml
# Cargo.toml
[dependencies]
clap = { version = "4.5", features = ["derive"], optional = true }

[features]
cli = ["dep:clap"]

[[bin]]
name = "mixamo2d"
path = "src/bin/cli.rs"
required-features = ["cli"]
```

## Testing the CLI

```bash
# Help
cargo run --features cli -- --help
cargo run --features cli -- convert --help

# Convert test file
cargo run --features cli -- convert testdata/Sprint.fbx -o /tmp/test -v

# Inspect file
cargo run --features cli -- info testdata/Sprint.fbx

# Batch convert (if you have multiple files)
cargo run --features cli -- batch testdata/ -o /tmp/batch
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parsing, validation, I/O) |

## Future Considerations

Potential enhancements (not yet implemented):
- `--format` option for JSON output
- `--dry-run` to preview without writing
- `--strip-root-motion` option
- Progress bar for batch operations
- Parallel batch processing
