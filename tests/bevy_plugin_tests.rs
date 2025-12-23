//! Integration tests for the Bevy plugin
//!
//! Tests asset loading, skeleton initialization, and animation playback.

use bevy::prelude::*;
use bevy_mixamo_2d::bevy_plugin::*;
use bevy_mixamo_2d::core::{
    Animation2D, Bone2D, BoneTimeline, Keyframe, PrimitiveShape, Skeleton2D, Transform2D,
};
use glam::Vec2;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn make_test_skeleton() -> Skeleton2D {
    let bones = vec![
        Bone2D::new(
            "hips",
            None,
            10.0,
            Transform2D::new(Vec2::new(0.0, 100.0), 0.0, Vec2::ONE),
            PrimitiveShape::Circle { radius: 5.0 },
        ),
        Bone2D::new(
            "spine",
            Some(0),
            20.0,
            Transform2D::new(Vec2::new(0.0, 20.0), 0.0, Vec2::ONE),
            PrimitiveShape::Rectangle {
                width: 10.0,
                height: 20.0,
            },
        ),
        Bone2D::new(
            "head",
            Some(1),
            10.0,
            Transform2D::new(Vec2::new(0.0, 10.0), 0.0, Vec2::ONE),
            PrimitiveShape::Circle { radius: 8.0 },
        ),
    ];
    Skeleton2D::new("test_skeleton", bones).unwrap()
}

fn make_test_animation() -> Animation2D {
    let mut timelines = HashMap::new();

    // Hips timeline with rotation and translation
    timelines.insert(
        0,
        BoneTimeline::new(vec![
            Keyframe::linear(0.0, 0.0),
            Keyframe::linear(0.5, 0.1),
            Keyframe::linear(1.0, 0.0),
        ])
        .with_translations(vec![
            Keyframe::linear(0.0, Vec2::new(0.0, 100.0)),
            Keyframe::linear(0.5, Vec2::new(5.0, 105.0)),
            Keyframe::linear(1.0, Vec2::new(0.0, 100.0)),
        ]),
    );

    // Spine timeline with just rotation
    timelines.insert(
        1,
        BoneTimeline::new(vec![
            Keyframe::linear(0.0, 0.0),
            Keyframe::linear(0.5, -0.1),
            Keyframe::linear(1.0, 0.0),
        ]),
    );

    // Head timeline
    timelines.insert(
        2,
        BoneTimeline::new(vec![
            Keyframe::linear(0.0, 0.0),
            Keyframe::linear(1.0, 0.0),
        ]),
    );

    Animation2D::new(
        "test_walk",
        "test_skeleton",
        1.0,
        30.0,
        true,
        None,
        timelines,
        3,
    )
    .unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// ASSET WRAPPER TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_asset_wraps_skeleton() {
    let skeleton = make_test_skeleton();
    let asset = Skeleton2DAsset(skeleton.clone());

    assert_eq!(asset.name, "test_skeleton");
    assert_eq!(asset.bone_count(), 3);
    assert_eq!(asset.skeleton().bones[0].name, "hips");
}

#[test]
fn test_animation_asset_wraps_animation() {
    let animation = make_test_animation();
    let asset = Animation2DAsset(animation.clone());

    assert_eq!(asset.name, "test_walk");
    assert_eq!(asset.skeleton_name, "test_skeleton");
    assert_eq!(asset.duration, 1.0);
    assert!(asset.looping);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION PLAYER TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_player_state_transitions() {
    let mut player = AnimationPlayer2D::default();

    // Initial state
    assert!(player.is_stopped());
    assert!(!player.is_playing());
    assert!(!player.is_paused());

    // Play
    player.play(Handle::default());
    assert!(player.is_playing());
    assert!(!player.is_stopped());

    // Pause
    player.pause();
    assert!(player.is_paused());
    assert!(!player.is_playing());

    // Resume
    player.resume();
    assert!(player.is_playing());

    // Stop
    player.stop();
    assert!(player.is_stopped());
    assert_eq!(player.elapsed(), 0.0);
}

#[test]
fn test_animation_player_speed_control() {
    let mut player = AnimationPlayer2D::default();
    player.play(Handle::default());
    player.update_cache(1.0, true);

    // Normal speed
    player.set_speed(1.0);
    player.advance_time(0.5);
    assert!((player.elapsed() - 0.5).abs() < 0.001);

    // Double speed
    player.stop();
    player.play(Handle::default());
    player.update_cache(1.0, true);
    player.set_speed(2.0);
    player.advance_time(0.25);
    assert!((player.elapsed() - 0.5).abs() < 0.001);

    // Half speed
    player.stop();
    player.play(Handle::default());
    player.update_cache(1.0, true);
    player.set_speed(0.5);
    player.advance_time(1.0);
    assert!((player.elapsed() - 0.5).abs() < 0.001);
}

#[test]
fn test_animation_player_seek() {
    let mut player = AnimationPlayer2D::default();
    player.update_cache(1.0, true);

    player.seek(0.5);
    assert!((player.elapsed() - 0.5).abs() < 0.001);

    // Seek beyond duration wraps for looping
    player.seek(1.5);
    assert!((player.elapsed() - 0.5).abs() < 0.001);

    // Negative seek clamps to 0
    player.seek(-1.0);
    assert_eq!(player.elapsed(), 0.0);
}

#[test]
fn test_animation_player_looping() {
    let mut player = AnimationPlayer2D::default();
    player.play(Handle::default());
    player.update_cache(1.0, true);

    // Advance past duration
    player.advance_time(1.5);

    // Should wrap
    assert!((player.elapsed() - 0.5).abs() < 0.001);
    assert!(player.is_playing()); // Still playing
}

#[test]
fn test_animation_player_non_looping_stops_at_end() {
    let mut player = AnimationPlayer2D::default();
    player.play(Handle::default());
    player.update_cache(1.0, false); // Non-looping

    // Advance past duration
    player.advance_time(1.5);

    // Should clamp and stop
    assert_eq!(player.elapsed(), 1.0);
    assert!(player.is_stopped());
    assert!(player.is_finished());
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSE TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pose2d_new() {
    let pose = Pose2D::new(Vec2::new(10.0, 20.0), 0.5, Vec2::new(1.5, 2.0));

    assert_eq!(pose.translation, Vec2::new(10.0, 20.0));
    assert_eq!(pose.rotation, 0.5);
    assert_eq!(pose.scale, Vec2::new(1.5, 2.0));
}

#[test]
fn test_pose2d_identity() {
    let pose = Pose2D::identity();

    assert_eq!(pose.translation, Vec2::ZERO);
    assert_eq!(pose.rotation, 0.0);
    assert_eq!(pose.scale, Vec2::ONE);
}

// ═══════════════════════════════════════════════════════════════════════════════
// BONE ENTITY TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_bone_entity_creation() {
    let bone = BoneEntity::new(5, "test_bone", 15.0);

    assert_eq!(bone.bone_index, 5);
    assert_eq!(bone.name, "test_bone");
    assert_eq!(bone.length, 15.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON DEBUG CONFIG TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_debug_config_default() {
    let config = SkeletonDebugConfig::default();

    assert!(config.draw_bones);
    assert!(config.draw_shapes);
    assert_eq!(config.bone_thickness, 2.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON INSTANCE TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_instance_new() {
    let handle: Handle<Skeleton2DAsset> = Handle::default();
    let instance = SkeletonInstance::new(handle.clone());

    assert!(!instance.initialized);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PLUGIN REGISTRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_plugin_registers_assets() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());

    // Add our plugin
    app.add_plugins(Mixamo2DPlugin);

    // Verify assets are registered
    assert!(app.world().contains_resource::<Assets<Skeleton2DAsset>>());
    assert!(app.world().contains_resource::<Assets<Animation2DAsset>>());
}

#[test]
fn test_plugin_initializes_debug_resource() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(Mixamo2DPlugin);

    // Debug should be disabled by default
    let debug = app.world().resource::<SkeletonDebugEnabled>();
    assert!(!debug.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION SAMPLING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_sampling_at_keyframes() {
    let animation = make_test_animation();

    // Sample at start
    let sample = animation.sample_bone(0, 0.0).unwrap();
    assert!((sample.rotation - 0.0).abs() < 0.001);

    // Sample at middle
    let sample = animation.sample_bone(0, 0.5).unwrap();
    assert!((sample.rotation - 0.1).abs() < 0.001);

    // Sample at end
    let sample = animation.sample_bone(1, 1.0).unwrap();
    assert!((sample.rotation - 0.0).abs() < 0.001);
}

#[test]
fn test_animation_sampling_interpolation() {
    let animation = make_test_animation();

    // Sample between keyframes (0.25 is between 0.0 and 0.5)
    let sample = animation.sample_bone(0, 0.25).unwrap();
    // Expected: linear interpolation between 0.0 and 0.1 at t=0.5 -> 0.05
    assert!((sample.rotation - 0.05).abs() < 0.001);

    // Sample translation between keyframes
    let sample = animation.sample_bone(0, 0.25).unwrap();
    let trans = sample.translation.unwrap();
    // Expected: linear between (0,100) and (5,105) at t=0.5 -> (2.5, 102.5)
    assert!((trans.x - 2.5).abs() < 0.01);
    assert!((trans.y - 102.5).abs() < 0.01);
}

#[test]
fn test_animation_sampling_bone_without_timeline() {
    let animation = make_test_animation();

    // Bone index 99 doesn't exist in the animation
    let sample = animation.sample_bone(99, 0.5);
    assert!(sample.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZATION ROUNDTRIP TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_serialization_roundtrip() {
    let skeleton = make_test_skeleton();

    // Serialize to RON
    let ron_str = ron::to_string(&skeleton).unwrap();

    // Deserialize
    let parsed: Skeleton2D = ron::from_str(&ron_str).unwrap();

    // Verify
    assert_eq!(parsed.name, skeleton.name);
    assert_eq!(parsed.bone_count(), skeleton.bone_count());
    for (i, bone) in parsed.bones.iter().enumerate() {
        assert_eq!(bone.name, skeleton.bones[i].name);
        assert_eq!(bone.parent, skeleton.bones[i].parent);
    }
}

#[test]
fn test_animation_serialization_roundtrip() {
    let animation = make_test_animation();

    // Serialize to RON
    let ron_str = ron::to_string(&animation).unwrap();

    // Deserialize
    let parsed: Animation2D = ron::from_str(&ron_str).unwrap();

    // Verify
    assert_eq!(parsed.name, animation.name);
    assert_eq!(parsed.skeleton_name, animation.skeleton_name);
    assert_eq!(parsed.duration, animation.duration);
    assert_eq!(parsed.looping, animation.looping);
    assert_eq!(parsed.timelines.len(), animation.timelines.len());
}
