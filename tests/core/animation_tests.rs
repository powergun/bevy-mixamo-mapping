use bevy_mixamo_2d::core::*;
use bevy_mixamo_2d::error::ValidationError;
use glam::Vec2;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// VALID ANIMATION CONSTRUCTION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_valid_minimal_animation() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let animation = Animation2D::new(
        "walk",
        "test_skeleton",
        1.0,
        30.0,
        true,
        None,
        timelines,
        2, // bone_count
    );
    assert!(animation.is_ok());
}

#[test]
fn test_valid_animation_with_all_bones() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(0.5, 1.0),
        Keyframe::linear(1.0, 0.0),
    ]));
    timelines.insert(1, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 0.5),
    ]));

    let animation = Animation2D::new(
        "run",
        "test_skeleton",
        1.0,
        60.0,
        true,
        None,
        timelines,
        2,
    );
    assert!(animation.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID DURATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_zero_duration_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", 0.0, 30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidDuration { duration: 0.0 })));
}

#[test]
fn test_negative_duration_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", -1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidDuration { .. })));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID SAMPLE RATE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_zero_sample_rate_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", 1.0, 0.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidSampleRate { sample_rate: 0.0 })));
}

#[test]
fn test_negative_sample_rate_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", 1.0, -30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidSampleRate { .. })));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: EMPTY TIMELINES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_timelines_rejected() {
    let timelines = HashMap::new();

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::EmptyTimelines)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID BONE INDEX IN TIMELINE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_timeline_targets_nonexistent_bone_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));
    timelines.insert(99, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)])); // bone 99 doesn't exist

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 2);
    assert!(matches!(
        result,
        Err(ValidationError::TimelineTargetsInvalidBone { bone_index: 99 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: KEYFRAME TIME OUT OF RANGE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframe_time_exceeds_duration_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(2.0, 1.0), // duration is 1.0, this is 2.0
    ]));

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::KeyframeTimeOutOfRange { time: 2.0, duration: 1.0, .. })
    ));
}

#[test]
fn test_negative_keyframe_time_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(-0.5, 0.0), // negative time
    ]));

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::KeyframeTimeOutOfRange { time, .. }) if time == -0.5
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: UNSORTED KEYFRAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unsorted_keyframes_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.5, 0.0),
        Keyframe::linear(0.2, 1.0), // 0.2 < 0.5, out of order
    ]));

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::KeyframesNotSorted { index: 1, time: 0.2, prev_time: 0.5 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: EMPTY ROTATION KEYFRAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_rotation_keyframes_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![])); // empty rotations

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::EmptyRotationKeyframes { bone_index: 0 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: ROOT MOTION DELTAS MISMATCH
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_root_motion_deltas_mismatch_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let root_motion = RootMotion {
        total_displacement: Vec2::ZERO,
        velocity: Vec2::ZERO,
        deltas: vec![Vec2::ZERO, Vec2::ZERO], // 2 deltas, but duration=1.0, rate=30 means 30 frames
    };

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, Some(root_motion), timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::RootMotionDeltasMismatch { deltas_count: 2, expected_count: 30 })
    ));
}

#[test]
fn test_empty_root_motion_deltas_allowed() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let root_motion = RootMotion {
        total_displacement: Vec2::new(1.0, 0.0),
        velocity: Vec2::new(1.0, 0.0),
        deltas: vec![], // empty is OK - means "not computed"
    };

    let result = Animation2D::new("good", "skel", 1.0, 30.0, true, Some(root_motion), timelines, 1);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SAMPLING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_sample_bone_linear_interpolation() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 2.0),
    ]));

    let animation = Animation2D::new("test", "skel", 1.0, 30.0, true, None, timelines, 1).unwrap();

    let sample = animation.sample_bone(0, 0.5).unwrap();
    assert!((sample.rotation - 1.0).abs() < 0.001); // midpoint = 1.0
}

#[test]
fn test_sample_bone_step_interpolation() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::new(0.0, 0.0, Interpolation::Step),
        Keyframe::new(1.0, 2.0, Interpolation::Step),
    ]));

    let animation = Animation2D::new("test", "skel", 1.0, 30.0, true, None, timelines, 1).unwrap();

    let sample = animation.sample_bone(0, 0.5).unwrap();
    assert!((sample.rotation - 0.0).abs() < 0.001); // step holds previous value
}

#[test]
fn test_sample_bone_clamps_time() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 1.0),
        Keyframe::linear(1.0, 2.0),
    ]));

    let animation = Animation2D::new("test", "skel", 1.0, 30.0, true, None, timelines, 1).unwrap();

    // Before start
    let sample = animation.sample_bone(0, -1.0).unwrap();
    assert!((sample.rotation - 1.0).abs() < 0.001);

    // After end
    let sample = animation.sample_bone(0, 5.0).unwrap();
    assert!((sample.rotation - 2.0).abs() < 0.001);
}
