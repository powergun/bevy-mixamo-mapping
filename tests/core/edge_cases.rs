//! Additional edge case tests for comprehensive coverage

use bevy_mixamo_2d::core::*;
use glam::Vec2;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSFORM2D EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_transform2d_rotation_normalization() {
    // Test that rotations are normalized to [-π, π]
    let t1 = Transform2D::new(Vec2::ZERO, 7.0, Vec2::ONE); // > 2π
    assert!(t1.rotation >= -std::f32::consts::PI && t1.rotation <= std::f32::consts::PI);

    let t2 = Transform2D::new(Vec2::ZERO, -7.0, Vec2::ONE); // < -2π
    assert!(t2.rotation >= -std::f32::consts::PI && t2.rotation <= std::f32::consts::PI);
}

#[test]
fn test_primitive_shape_validation() {
    assert!(PrimitiveShape::Circle { radius: 1.0 }.is_valid());
    assert!(!PrimitiveShape::Circle { radius: 0.0 }.is_valid());
    assert!(!PrimitiveShape::Circle { radius: -1.0 }.is_valid());

    assert!(PrimitiveShape::Rectangle { width: 1.0, height: 1.0 }.is_valid());
    assert!(!PrimitiveShape::Rectangle { width: 0.0, height: 1.0 }.is_valid());
    assert!(!PrimitiveShape::Rectangle { width: 1.0, height: -1.0 }.is_valid());

    assert!(PrimitiveShape::None.is_valid());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON DEEP HIERARCHY
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_deep_bone_hierarchy() {
    // Create a chain of 100 bones
    let mut bones = vec![
        Bone2D::new("bone_0", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    for i in 1..100 {
        bones.push(Bone2D::new(
            format!("bone_{}", i),
            Some(i - 1), // chain: each bone's parent is the previous one
            1.0,
            Transform2D::default(),
            PrimitiveShape::None,
        ));
    }

    let skeleton = Skeleton2D::new("deep_chain", bones);
    assert!(skeleton.is_ok());
    assert_eq!(skeleton.unwrap().bone_count(), 100);
}

#[test]
fn test_wide_bone_hierarchy() {
    // Root with 50 direct children
    let mut bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    for i in 1..=50 {
        bones.push(Bone2D::new(
            format!("child_{}", i),
            Some(0), // all children of root
            1.0,
            Transform2D::default(),
            PrimitiveShape::None,
        ));
    }

    let skeleton = Skeleton2D::new("wide", bones);
    assert!(skeleton.is_ok());
    let s = skeleton.unwrap();
    assert_eq!(s.children_of(0).len(), 50);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION BOUNDARY CONDITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_keyframe_at_exact_duration() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 1.0), // exactly at duration
    ]));

    let animation = Animation2D::new("exact", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(animation.is_ok());
}

#[test]
fn test_animation_single_keyframe() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.5, 1.0), // single keyframe in the middle
    ]));

    let animation = Animation2D::new("single", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(animation.is_ok());

    let anim = animation.unwrap();
    // Sampling should return the single keyframe value regardless of time
    assert_eq!(anim.sample_bone(0, 0.0).unwrap().rotation, 1.0);
    assert_eq!(anim.sample_bone(0, 0.5).unwrap().rotation, 1.0);
    assert_eq!(anim.sample_bone(0, 1.0).unwrap().rotation, 1.0);
}

#[test]
fn test_animation_many_keyframes() {
    let mut timelines = HashMap::new();
    let keyframes: Vec<_> = (0..1000)
        .map(|i| Keyframe::linear(i as f32 * 0.001, i as f32))
        .collect();
    timelines.insert(0, BoneTimeline::new(keyframes));

    let animation = Animation2D::new("many", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(animation.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZATION ROUNDTRIP
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_ron_roundtrip() {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::Circle { radius: 0.1 }),
        Bone2D::new("spine", Some(0), 0.3, Transform2D::new(Vec2::new(0.0, 0.3), 1.57, Vec2::ONE), PrimitiveShape::Rectangle { width: 0.2, height: 0.3 }),
    ];
    let skeleton = Skeleton2D::new("roundtrip", bones).unwrap();

    let ron_str = ron::to_string(&skeleton).unwrap();
    let parsed: Skeleton2D = ron::from_str(&ron_str).unwrap();

    assert_eq!(skeleton, parsed);
}

#[test]
fn test_animation_ron_roundtrip() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 3.14),
    ]).with_translations(vec![
        Keyframe::linear(0.0, Vec2::ZERO),
        Keyframe::linear(1.0, Vec2::new(1.0, 0.0)),
    ]));

    let animation = Animation2D::new(
        "roundtrip",
        "test_skel",
        1.0,
        30.0,
        true,
        Some(RootMotion {
            total_displacement: Vec2::new(1.0, 0.0),
            velocity: Vec2::new(1.0, 0.0),
            deltas: vec![],
        }),
        timelines,
        1,
    ).unwrap();

    let ron_str = ron::to_string(&animation).unwrap();
    let parsed: Animation2D = ron::from_str(&ron_str).unwrap();

    assert_eq!(animation.name, parsed.name);
    assert_eq!(animation.duration, parsed.duration);
    // Note: Full equality check depends on PartialEq implementation
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNICODE AND SPECIAL CHARACTERS IN NAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unicode_bone_names() {
    let bones = vec![
        Bone2D::new("根骨", None, 0.0, Transform2D::default(), PrimitiveShape::None), // Chinese
        Bone2D::new("κεφαλή", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None), // Greek
        Bone2D::new("кость", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None), // Russian
    ];
    let skeleton = Skeleton2D::new("unicode_test", bones);
    assert!(skeleton.is_ok());
}

#[test]
fn test_special_characters_in_names() {
    let bones = vec![
        Bone2D::new("bone-with-dashes", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("bone_with_underscores", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("bone.with.dots", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("bone:with:colons", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("special_chars", bones);
    assert!(skeleton.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// FLOATING POINT EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_very_small_duration() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    // Very small but positive duration should be valid
    let animation = Animation2D::new("tiny", "skel", 0.001, 1000.0, true, None, timelines, 1);
    assert!(animation.is_ok());
}

#[test]
fn test_very_large_values() {
    let bones = vec![
        Bone2D::new("root", None, 1000000.0, Transform2D::new(
            Vec2::new(1e6, 1e6),
            0.0,
            Vec2::ONE,
        ), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("large", bones);
    assert!(skeleton.is_ok());
}
