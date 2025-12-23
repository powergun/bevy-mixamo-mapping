//! # bevy_mixamo_2d
//!
//! Convert Mixamo 3D animations to 2D skeletal animations for Bevy.
//!
//! This crate provides:
//! - Core data structures for 2D skeletons and animations
//! - GLB/GLTF parsing for Mixamo animations
//! - FBX parsing for direct Mixamo downloads
//! - 3D to 2D projection
//! - RON serialization for skeleton and animation data
//! - Bevy plugin for runtime animation playback
//!
//! # Feature Flags
//!
//! - `visualization` (default): Enables the Bevy plugin for animation playback and debug visualization
//! - `cli`: Enables the command-line interface for converting animations
//!
//! # Quick Start with Bevy
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
//!     commands.spawn(Camera2d);
//!
//!     let skeleton = asset_server.load("player.skeleton2d.ron");
//!     commands.spawn(SkeletonBundle {
//!         skeleton_instance: SkeletonInstance::new(skeleton),
//!         ..default()
//!     });
//! }
//! ```

pub mod core;
pub mod error;
pub mod export;
pub mod fbx_parser;
pub mod gltf_parser;
pub mod projection;

#[cfg(feature = "visualization")]
pub mod bevy_plugin;

// Re-export commonly used types at the crate root
pub use core::{
    Animation2D, Bone2D, BoneTimeline, Interpolation, Keyframe, PrimitiveShape, RootMotion,
    SampledPose, Skeleton2D, Transform2D,
};
pub use error::{Mixamo2dError, Result, ValidationError};
pub use export::{load_animation, load_skeleton, save_animation, save_skeleton};
pub use fbx_parser::FbxData;
pub use gltf_parser::GltfData;
pub use projection::{project_animation, project_skeleton, ProjectionConfig, ViewAxis};

// Re-export Bevy plugin types when the visualization feature is enabled
#[cfg(feature = "visualization")]
pub use bevy_plugin::{
    Animation2DAsset, AnimationPlayer2D, BoneEntity, Mixamo2DPlugin, PlaybackState, Pose2D,
    Skeleton2DAsset, SkeletonBundle, SkeletonDebugConfig, SkeletonDebugEnabled, SkeletonInstance,
};
