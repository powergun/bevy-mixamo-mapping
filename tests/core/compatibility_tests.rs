use bevy_mixamo_2d::core::*;
use bevy_mixamo_2d::error::ValidationError;
use std::collections::HashMap;

fn make_skeleton(name: &str, bone_count: usize) -> Skeleton2D {
    let mut bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    for i in 1..bone_count {
        bones.push(Bone2D::new(
            format!("bone_{}", i),
            Some(0),
            1.0,
            Transform2D::default(),
            PrimitiveShape::None,
        ));
    }
    Skeleton2D::new(name, bones).unwrap()
}

fn make_animation(skeleton_name: &str, bone_indices: &[usize], bone_count: usize) -> Animation2D {
    let mut timelines = HashMap::new();
    for &idx in bone_indices {
        timelines.insert(idx, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));
    }
    Animation2D::new(
        "test_anim",
        skeleton_name,
        1.0,
        30.0,
        true,
        None,
        timelines,
        bone_count,
    ).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON NAME MISMATCH
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_name_mismatch_rejected() {
    let skeleton = make_skeleton("skeleton_a", 3);
    let animation = make_animation("skeleton_b", &[0, 1, 2], 3); // different name

    let result = check_compatibility(&skeleton, &animation);
    assert!(matches!(
        result,
        Err(ValidationError::SkeletonNameMismatch {
            animation_skeleton,
            actual_skeleton
        }) if animation_skeleton == "skeleton_b" && actual_skeleton == "skeleton_a"
    ));
}

#[test]
fn test_skeleton_name_match_accepted() {
    let skeleton = make_skeleton("my_skeleton", 3);
    let animation = make_animation("my_skeleton", &[0, 1], 3);

    let result = check_compatibility(&skeleton, &animation);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// BONE INDEX OUT OF RANGE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_timeline_bone_index_exceeds_skeleton_rejected() {
    let skeleton = make_skeleton("test", 2); // only indices 0 and 1 valid

    // Animation was created with bone_count=5 (for its internal validation)
    // but skeleton only has 2 bones
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));
    timelines.insert(4, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)])); // index 4 invalid for skeleton

    let animation = Animation2D::new("anim", "test", 1.0, 30.0, true, None, timelines, 5).unwrap();

    let result = check_compatibility(&skeleton, &animation);
    assert!(matches!(
        result,
        Err(ValidationError::TimelineBoneIndexOutOfRange { index: 4, bone_count: 2 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// STRICT COMPATIBILITY (BONE COUNT MATCH)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_strict_compatibility_bone_count_mismatch() {
    let skeleton = make_skeleton("test", 5);
    let animation = make_animation("test", &[0, 1, 2], 5); // only 3 timelines, but 5 bones

    // Basic compatibility should pass
    assert!(check_compatibility(&skeleton, &animation).is_ok());

    // Strict compatibility should fail
    let result = check_strict_compatibility(&skeleton, &animation);
    assert!(matches!(
        result,
        Err(ValidationError::BoneCountMismatch { expected: 5, actual: 3 })
    ));
}

#[test]
fn test_strict_compatibility_all_bones_covered() {
    let skeleton = make_skeleton("test", 3);
    let animation = make_animation("test", &[0, 1, 2], 3); // all 3 bones covered

    let result = check_strict_compatibility(&skeleton, &animation);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTEGRATION: validate_compatibility METHOD
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_validate_compatibility_method() {
    let skeleton = make_skeleton("my_skel", 3);
    let animation = make_animation("my_skel", &[0, 1], 3);

    // Using the method directly on Animation2D
    assert!(animation.validate_compatibility(&skeleton).is_ok());
}

#[test]
fn test_animation_validate_compatibility_fails_on_mismatch() {
    let skeleton = make_skeleton("skeleton_x", 3);
    let animation = make_animation("skeleton_y", &[0], 3);

    assert!(animation.validate_compatibility(&skeleton).is_err());
}
