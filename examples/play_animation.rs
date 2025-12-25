//! Example: Play 2D Skeletal Animation
//!
//! This example demonstrates how to:
//! 1. Load a skeleton and animation from RON files
//! 2. Spawn a skeleton entity with animation player
//! 3. Control animation playback with keyboard
//! 4. Toggle debug visualization
//! 5. Capture screenshots of the animation (optional)
//!
//! # Usage
//!
//! First, convert a Mixamo FBX file to RON format:
//! ```bash
//! cargo run --features cli -- convert path/to/animation.fbx -o assets/
//! ```
//!
//! Then run this example with skeleton and animation files:
//! ```bash
//! cargo run --example play_animation -- -s assets/Sprint.skeleton2d.ron -a assets/mixamo_com.anim2d.ron
//! ```
//!
//! Files can be in any directory (not just assets/):
//! ```bash
//! cargo run --example play_animation -- -s ws/Sprint.skeleton2d.ron -a ws/mixamo_com.anim2d.ron
//! ```
//!
//! # Screenshot Mode
//!
//! Capture screenshots of the animation:
//! ```bash
//! # Take screenshots every 3 frames for 1 second
//! cargo run --example play_animation -- -s assets/Sprint.skeleton2d.ron -a assets/mixamo_com.anim2d.ron --screenshot
//!
//! # Customize interval and duration
//! cargo run --example play_animation -- -s assets/Sprint.skeleton2d.ron -a assets/mixamo_com.anim2d.ron \
//!   --screenshot --screenshot-interval 2 --screenshot-duration 2.0
//!
//! # Change output directory
//! cargo run --example play_animation -- -s assets/Sprint.skeleton2d.ron -a assets/mixamo_com.anim2d.ron \
//!   --screenshot --screenshot-output ./my_screenshots
//! ```
//!
//! # Controls
//!
//! - Space: Play/Pause animation
//! - R: Reset animation to beginning
//! - Left/Right: Seek backward/forward
//! - Up/Down: Speed up/slow down
//! - F3: Toggle debug visualization
//! - Escape: Quit

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy_mixamo_2d::bevy_plugin::{
    Animation2DAsset, AnimationPlayer2D, BoneColorMode, Mixamo2DPlugin, PlaybackState,
    Skeleton2DAsset, SkeletonBundle, SkeletonDebugConfig, SkeletonDebugEnabled, SkeletonInstance,
};
use bevy_simple_screenshot::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// Play 2D Skeletal Animation example
#[derive(Parser, Debug)]
#[command(name = "play_animation")]
#[command(about = "Play 2D skeletal animations from RON files")]
struct Args {
    /// Path to the skeleton file (*.skeleton2d.ron).
    /// Specify the full path to the file (e.g., "assets/Sprint.skeleton2d.ron" or "ws/Sprint.skeleton2d.ron").
    #[arg(short, long)]
    skeleton: PathBuf,

    /// Path to the animation file (*.anim2d.ron).
    /// Specify the full path to the file (e.g., "assets/mixamo_com.anim2d.ron" or "ws/mixamo_com.anim2d.ron").
    #[arg(short, long)]
    animation: PathBuf,

    /// Scale factor for the skeleton (Mixamo skeletons are large)
    #[arg(long, default_value = "0.02")]
    scale: f32,

    /// Enable screenshot mode to capture animation frames
    #[arg(long)]
    screenshot: bool,

    /// Frames between each screenshot (e.g., 3 means capture every 3rd frame)
    #[arg(long, default_value = "3")]
    screenshot_interval: u32,

    /// Duration in seconds to capture screenshots
    #[arg(long, default_value = "1.0")]
    screenshot_duration: f32,

    /// Output directory for screenshots
    #[arg(long, default_value = ".screenshots")]
    screenshot_output: PathBuf,
}

#[derive(Resource)]
struct AnimationPaths {
    skeleton: PathBuf,
    animation: PathBuf,
    scale: f32,
}

#[derive(Resource)]
struct ScreenshotConfig {
    enabled: bool,
    interval: u32,
    duration: f32,
}

#[derive(Resource)]
struct ScreenshotState {
    frame_counter: u32,
    elapsed_time: f32,
    screenshot_count: u32,
    total_frames: u32,
    finished: bool,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        Self {
            frame_counter: 0,
            elapsed_time: 0.0,
            screenshot_count: 0,
            total_frames: 0,
            finished: false,
        }
    }
}

impl ScreenshotState {
    /// Estimate max frames based on current frame rate and remaining duration
    fn estimate_max_frames(&self, duration: f32) -> u32 {
        if self.elapsed_time > 0.0 {
            let fps = self.total_frames as f32 / self.elapsed_time;
            (duration * fps).ceil() as u32
        } else {
            // Estimate 60 FPS if we don't have data yet
            (duration * 60.0).ceil() as u32
        }
    }
}

fn main() {
    let args = Args::parse();

    // Validate that files exist (paths are used as-is)
    if !args.skeleton.exists() {
        eprintln!(
            "Error: Skeleton file not found: {}\n\n\
             Hint: Convert a Mixamo FBX file first:\n  \
             cargo run --features cli -- convert path/to/animation.fbx -o assets/",
            args.skeleton.display()
        );
        std::process::exit(1);
    }

    if !args.animation.exists() {
        eprintln!(
            "Error: Animation file not found: {}",
            args.animation.display()
        );
        std::process::exit(1);
    }

    println!("Loading skeleton: {}", args.skeleton.display());
    println!("Loading animation: {}", args.animation.display());
    println!("Scale: {}", args.scale);

    if args.screenshot {
        println!("\nScreenshot mode enabled:");
        println!("  Interval: every {} frame(s)", args.screenshot_interval);
        println!("  Duration: {}s", args.screenshot_duration);
        println!("  Output: {}", args.screenshot_output.display());
    }

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "2D Skeletal Animation - bevy_mixamo_2d".into(),
                    resolution: bevy::window::WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // Allow loading assets from any file path (not just assets/)
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            }),
    )
    .add_plugins(Mixamo2DPlugin)
    // Store paths as a resource
    .insert_resource(AnimationPaths {
        skeleton: args.skeleton,
        animation: args.animation,
        scale: args.scale,
    })
    .insert_resource(ScreenshotConfig {
        enabled: args.screenshot,
        interval: args.screenshot_interval,
        duration: args.screenshot_duration,
    })
    // Enable debug visualization by default
    .insert_resource(SkeletonDebugEnabled(true))
    .add_systems(Startup, setup)
    .add_systems(Update, (keyboard_input, update_ui));

    // Add screenshot plugin and systems only when screenshot mode is enabled
    if args.screenshot {
        app.add_plugins(ScreenshotBufferPlugin::with_config(
            bevy_simple_screenshot::ScreenshotConfig::default()
                .with_output_dir(args.screenshot_output.to_string_lossy().to_string()),
        ))
        .init_resource::<ScreenshotState>()
        .add_systems(Update, capture_screenshots);
    }

    app.run();
}

#[derive(Component)]
struct MainSkeleton;

#[derive(Component)]
struct UiText;

/// Convert a file path to a Bevy asset path.
///
/// With UnapprovedPathMode::Allow, Bevy can load assets from any path.
/// We convert relative paths to absolute paths for consistency.
fn to_asset_path(path: &std::path::Path) -> String {
    // Convert to absolute path for consistent behavior
    if let Ok(abs_path) = path.canonicalize() {
        return abs_path.to_string_lossy().to_string();
    }

    // Fallback: use path as-is
    path.to_string_lossy().to_string()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, paths: Res<AnimationPaths>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Convert PathBuf to asset path string
    // Bevy expects paths relative to the assets directory
    let skeleton_path = to_asset_path(&paths.skeleton);
    let animation_path = to_asset_path(&paths.animation);

    // Load skeleton and animation assets
    let skeleton_handle: Handle<Skeleton2DAsset> = asset_server.load(skeleton_path);
    let animation_handle: Handle<Animation2DAsset> = asset_server.load(animation_path);

    // Spawn skeleton entity at center of screen
    commands.spawn((
        SkeletonBundle {
            skeleton_instance: SkeletonInstance::new(skeleton_handle),
            transform: Transform::from_xyz(0.0, -200.0, 0.0).with_scale(Vec3::splat(paths.scale)),
            ..default()
        },
        SkeletonDebugConfig {
            draw_bones: true,
            draw_shapes: true,
            // Default bone color (Green) for bones that are neither left nor right
            bone_color: Color::srgb(0.2, 0.8, 0.2),
            shape_color: Color::srgb(0.8, 0.8, 0.2),
            bone_thickness: 2.0,
            // Left bones in Red, Right bones in Blue
            bone_color_mode: BoneColorMode::left_right(
                Color::srgb(1.0, 0.2, 0.2), // Red for left bones
                Color::srgb(0.2, 0.2, 1.0), // Blue for right bones
            ),
        },
        MainSkeleton,
        AnimationToLoad(animation_handle),
    ));

    // Spawn UI text
    commands.spawn((
        Text::new(
            "Loading assets...\n\nControls:\nSpace: Play/Pause\nR: Reset\nLeft/Right: Seek\nUp/Down: Speed\nF3: Toggle Debug",
        ),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        UiText,
    ));
}

#[derive(Component)]
struct AnimationToLoad(Handle<Animation2DAsset>);

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_enabled: ResMut<SkeletonDebugEnabled>,
    mut skeleton_query: Query<(&mut AnimationPlayer2D, Option<&AnimationToLoad>)>,
    animations: Res<Assets<Animation2DAsset>>,
) {
    for (mut player, animation_to_load) in skeleton_query.iter_mut() {
        // Load animation if not already loaded
        if player.current_animation.is_none() {
            if let Some(anim_to_load) = animation_to_load {
                if animations.get(&anim_to_load.0).is_some() {
                    player.play(anim_to_load.0.clone());
                }
            }
        }

        // Play/Pause toggle
        if keys.just_pressed(KeyCode::Space) {
            if player.is_playing() {
                player.pause();
            } else if player.is_paused() {
                player.resume();
            } else {
                // If stopped, restart with current animation
                let current = player.current_animation.clone();
                if let Some(handle) = current {
                    player.play(handle);
                }
            }
        }

        // Reset
        if keys.just_pressed(KeyCode::KeyR) {
            let current = player.current_animation.clone();
            player.stop();
            if let Some(handle) = current {
                player.play(handle);
            }
        }

        // Seek
        if keys.just_pressed(KeyCode::ArrowLeft) {
            let new_time = (player.elapsed() - 0.1).max(0.0);
            player.seek(new_time);
        }
        if keys.just_pressed(KeyCode::ArrowRight) {
            let new_time = player.elapsed() + 0.1;
            player.seek(new_time);
        }

        // Speed control
        if keys.just_pressed(KeyCode::ArrowUp) {
            let new_speed = (player.speed + 0.25).min(3.0);
            player.set_speed(new_speed);
        }
        if keys.just_pressed(KeyCode::ArrowDown) {
            let new_speed = (player.speed - 0.25).max(0.25);
            player.set_speed(new_speed);
        }
    }

    // Toggle debug visualization
    if keys.just_pressed(KeyCode::F3) {
        debug_enabled.0 = !debug_enabled.0;
    }
}

#[allow(deprecated)]
fn capture_screenshots(
    trigger: ScreenshotTrigger,
    time: Res<Time>,
    config: Res<ScreenshotConfig>,
    mut state: ResMut<ScreenshotState>,
    mut exit: EventWriter<AppExit>,
    skeleton_query: Query<&AnimationPlayer2D, With<MainSkeleton>>,
) {
    // Don't capture if already finished
    if state.finished {
        return;
    }

    // Wait for animation to be playing
    let Ok(player) = skeleton_query.single() else {
        return;
    };

    if !player.is_playing() {
        return;
    }

    // Increment total frame counter
    state.total_frames += 1;

    // Update elapsed time
    state.elapsed_time += time.delta_secs();

    // Check if we've exceeded the duration
    if state.elapsed_time >= config.duration {
        if !state.finished {
            println!(
                "\nScreenshot capture complete: {} screenshots taken in {} frames",
                state.screenshot_count, state.total_frames
            );
            state.finished = true;
            exit.write(AppExit::Success);
        }
        return;
    }

    // Increment interval frame counter
    state.frame_counter += 1;

    // Capture screenshot at the specified interval
    if state.frame_counter >= config.interval {
        state.frame_counter = 0;
        state.screenshot_count += 1;

        let desc = format!("frame_{:04}", state.screenshot_count);
        screenshot!(&trigger, "animation", &desc);

        // Print progress with frame count
        let max_frames = state.estimate_max_frames(config.duration);
        print!(
            "\rCapturing: {} / {} frames, {} screenshots",
            state.total_frames, max_frames, state.screenshot_count
        );
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
}

fn update_ui(
    skeleton_query: Query<(&SkeletonInstance, &AnimationPlayer2D), With<MainSkeleton>>,
    debug_enabled: Res<SkeletonDebugEnabled>,
    skeletons: Res<Assets<Skeleton2DAsset>>,
    animations: Res<Assets<Animation2DAsset>>,
    mut text_query: Query<&mut Text, With<UiText>>,
    screenshot_config: Res<ScreenshotConfig>,
    screenshot_state: Option<Res<ScreenshotState>>,
) {
    // Get the first skeleton (there should only be one with MainSkeleton marker)
    let Some((instance, player)) = skeleton_query.iter().next() else {
        return;
    };

    let skeleton_status = if skeletons.get(&instance.skeleton).is_some() {
        "Loaded"
    } else {
        "Loading..."
    };

    let animation_status = player
        .current_animation
        .as_ref()
        .map(|h| {
            if animations.get(h).is_some() {
                "Loaded"
            } else {
                "Loading..."
            }
        })
        .unwrap_or("None");

    let state = match player.state {
        PlaybackState::Playing => "Playing",
        PlaybackState::Paused => "Paused",
        PlaybackState::Stopped => "Stopped",
    };

    let screenshot_info = if screenshot_config.enabled {
        if let Some(ref ss_state) = screenshot_state {
            let max_frames = ss_state.estimate_max_frames(screenshot_config.duration);
            format!(
                "\n\nScreenshot: {} captured\nFrame: {} / {}\nTime: {:.1}s / {:.1}s",
                ss_state.screenshot_count,
                ss_state.total_frames,
                max_frames,
                ss_state.elapsed_time,
                screenshot_config.duration
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    for mut text in text_query.iter_mut() {
        **text = format!(
            "Skeleton: {}\n\
             Animation: {}\n\
             State: {}\n\
             Time: {:.2}s / {:.2}s\n\
             Speed: {:.2}x\n\
             Debug: {}{}\n\
             \n\
             Controls:\n\
             Space: Play/Pause\n\
             R: Reset\n\
             Left/Right: Seek\n\
             Up/Down: Speed\n\
             F3: Toggle Debug",
            skeleton_status,
            animation_status,
            state,
            player.elapsed(),
            player.duration(),
            player.speed,
            if debug_enabled.0 { "ON" } else { "OFF" },
            screenshot_info
        );
    }
}
