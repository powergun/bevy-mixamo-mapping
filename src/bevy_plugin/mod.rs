//! Bevy plugin for 2D skeletal animation
//!
//! This module provides Bevy integration for loading and playing 2D skeletal animations
//! converted from Mixamo 3D animations.
//!
//! # Features
//!
//! - **Asset Loading**: Load `.skeleton2d.ron` and `.anim2d.ron` files as Bevy assets
//! - **Animation Playback**: Play, pause, stop, and seek through animations
//! - **Debug Visualization**: Draw skeleton bones and shapes using gizmos
//!
//! # Quick Start
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_mixamo_2d::bevy_plugin::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(Mixamo2DPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     // Spawn camera
//!     commands.spawn(Camera2d);
//!
//!     // Load skeleton and animation
//!     let skeleton = asset_server.load("player.skeleton2d.ron");
//!     let walk_anim = asset_server.load("walk.anim2d.ron");
//!
//!     // Spawn skeleton entity
//!     commands.spawn((
//!         SkeletonBundle {
//!             skeleton_instance: SkeletonInstance::new(skeleton),
//!             ..default()
//!         },
//!         SkeletonDebugConfig::default(),
//!     ));
//! }
//! ```
//!
//! # Animation Control
//!
//! ```ignore
//! fn control_animation(
//!     mut query: Query<&mut AnimationPlayer2D>,
//!     asset_server: Res<AssetServer>,
//! ) {
//!     for mut player in query.iter_mut() {
//!         // Play an animation
//!         player.play(asset_server.load("run.anim2d.ron"));
//!
//!         // Control playback
//!         player.pause();
//!         player.resume();
//!         player.set_speed(0.5); // Half speed
//!         player.seek(1.0);      // Jump to 1 second
//!         player.stop();         // Stop and reset
//!     }
//! }
//! ```
//!
//! # Debug Visualization
//!
//! Enable debug drawing by inserting the [`SkeletonDebugEnabled`] resource:
//!
//! ```ignore
//! app.insert_resource(SkeletonDebugEnabled(true));
//! ```
//!
//! Configure per-skeleton visualization with [`SkeletonDebugConfig`]:
//!
//! ```ignore
//! commands.spawn((
//!     SkeletonBundle { .. },
//!     SkeletonDebugConfig {
//!         draw_bones: true,
//!         draw_shapes: true,
//!         bone_color: Color::srgb(0.0, 1.0, 0.0),
//!         ..default()
//!     },
//! ));
//! ```

pub mod assets;
pub mod components;
pub mod primitives;
pub mod systems;

// Re-export commonly used types
pub use assets::{Animation2DAsset, Animation2DAssetLoader, Skeleton2DAsset, Skeleton2DAssetLoader};
pub use components::{
    AnimationPlayer2D, BoneEntity, PlaybackState, Pose2D, SkeletonBundle, SkeletonDebugConfig,
    SkeletonInstance,
};
pub use primitives::SkeletonDebugEnabled;

use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// PLUGIN
// ═══════════════════════════════════════════════════════════════════════════════

/// Main plugin for 2D skeletal animation.
///
/// Registers asset loaders, components, and systems for loading and playing
/// 2D skeletal animations converted from Mixamo 3D animations.
///
/// # Systems
///
/// The plugin adds systems in the following schedule order:
///
/// - **PreUpdate**: `advance_animation_time` - Updates playback time
/// - **Update**: (in order)
///   1. `initialize_skeleton_hierarchy` - Spawns bone entities when skeleton loads
///   2. `sample_animations` - Samples animation keyframes
///   3. `apply_poses_to_transforms` - Updates Transform components from poses
/// - **PostUpdate**: `draw_skeleton_debug` - Draws debug visualization (when enabled)
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use bevy_mixamo_2d::bevy_plugin::Mixamo2DPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(Mixamo2DPlugin)
///         .run();
/// }
/// ```
pub struct Mixamo2DPlugin;

impl Plugin for Mixamo2DPlugin {
    fn build(&self, app: &mut App) {
        // Register asset types
        app.init_asset::<Skeleton2DAsset>()
            .init_asset::<Animation2DAsset>();

        // Register asset loaders
        app.register_asset_loader(assets::Skeleton2DAssetLoader)
            .register_asset_loader(assets::Animation2DAssetLoader);

        // Register components for reflection (enables inspector and serialization)
        app.register_type::<SkeletonInstance>()
            .register_type::<AnimationPlayer2D>()
            .register_type::<Pose2D>()
            .register_type::<BoneEntity>()
            .register_type::<SkeletonDebugConfig>()
            .register_type::<PlaybackState>();

        // Initialize debug resource (disabled by default)
        app.init_resource::<SkeletonDebugEnabled>();

        // Add animation systems
        app.add_systems(PreUpdate, systems::advance_animation_time);

        app.add_systems(
            Update,
            (
                systems::initialize_skeleton_hierarchy,
                systems::sample_animations,
                systems::apply_poses_to_transforms,
            )
                .chain(),
        );

        // Add debug visualization system (conditional on SkeletonDebugEnabled)
        app.add_systems(PostUpdate, primitives::draw_skeleton_debug);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_builds_without_panic() {
        // Create a minimal app and add the plugin
        let mut app = App::new();

        // Add minimal plugins required for our plugin
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());

        // This should not panic
        app.add_plugins(Mixamo2DPlugin);

        // Verify resources were added
        assert!(app.world().contains_resource::<SkeletonDebugEnabled>());
    }
}
