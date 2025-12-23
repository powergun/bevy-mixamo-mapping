//! Example: Play 2D Skeletal Animation
//!
//! This example demonstrates how to:
//! 1. Load a skeleton and animation from RON files
//! 2. Spawn a skeleton entity with animation player
//! 3. Control animation playback with keyboard
//! 4. Toggle debug visualization
//!
//! # Usage
//!
//! First, convert a Mixamo FBX file to RON format:
//! ```bash
//! cargo run --features cli -- convert path/to/animation.fbx -o assets/
//! ```
//!
//! Then run this example:
//! ```bash
//! cargo run --example play_animation
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

use bevy::prelude::*;
use bevy_mixamo_2d::bevy_plugin::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "2D Skeletal Animation - bevy_mixamo_2d".into(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Mixamo2DPlugin)
        // Enable debug visualization by default
        .insert_resource(SkeletonDebugEnabled(true))
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input, update_ui))
        .run();
}

#[derive(Component)]
struct MainSkeleton;

#[derive(Component)]
struct UiText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Try to load skeleton and animation assets
    // These should be created by running the CLI converter first
    let skeleton_handle: Handle<Skeleton2DAsset> =
        asset_server.load("Sprint.skeleton2d.ron");
    let animation_handle: Handle<Animation2DAsset> =
        asset_server.load("mixamo_com.anim2d.ron");

    // Spawn skeleton entity at center of screen
    commands.spawn((
        SkeletonBundle {
            skeleton_instance: SkeletonInstance::new(skeleton_handle),
            transform: Transform::from_xyz(0.0, -200.0, 0.0).with_scale(Vec3::splat(0.02)),
            ..default()
        },
        SkeletonDebugConfig {
            draw_bones: true,
            draw_shapes: true,
            bone_color: Color::srgb(0.2, 0.8, 0.2),
            shape_color: Color::srgb(0.8, 0.8, 0.2),
            bone_thickness: 2.0,
        },
        MainSkeleton,
        AnimationToLoad(animation_handle),
    ));

    // Spawn UI text
    commands.spawn((
        Text::new("Loading assets...\n\nControls:\nSpace: Play/Pause\nR: Reset\nLeft/Right: Seek\nUp/Down: Speed\nF3: Toggle Debug"),
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

fn update_ui(
    skeleton_query: Query<(&SkeletonInstance, &AnimationPlayer2D), With<MainSkeleton>>,
    debug_enabled: Res<SkeletonDebugEnabled>,
    skeletons: Res<Assets<Skeleton2DAsset>>,
    animations: Res<Assets<Animation2DAsset>>,
    mut text_query: Query<&mut Text, With<UiText>>,
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

    for mut text in text_query.iter_mut() {
        **text = format!(
            "Skeleton: {}\n\
             Animation: {}\n\
             State: {}\n\
             Time: {:.2}s / {:.2}s\n\
             Speed: {:.2}x\n\
             Debug: {}\n\
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
            if debug_enabled.0 { "ON" } else { "OFF" }
        );
    }
}
