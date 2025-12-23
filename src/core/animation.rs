use super::keyframe::{BoneTimeline, Interpolation, Keyframe};
use super::skeleton::Skeleton2D;
use crate::error::ValidationError;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root motion data extracted from the hip/root bone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootMotion {
    /// Total displacement over one animation cycle
    pub total_displacement: Vec2,
    /// Average velocity (units per second)
    pub velocity: Vec2,
    /// Per-frame displacement deltas (for precise motion matching)
    pub deltas: Vec<Vec2>,
}

/// A 2D animation clip, saved as `*.anim2d.ron`
///
/// # Invariants
/// - `duration` > 0
/// - `sample_rate` > 0
/// - `timelines` is non-empty
/// - All timeline bone indices are valid (< target skeleton bone count)
/// - All keyframes have times in range [0, duration]
/// - All keyframes within a timeline are sorted by time
/// - Each timeline has at least one rotation keyframe
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animation2D {
    /// Animation name
    pub name: String,
    /// Reference to the skeleton this animation targets
    pub skeleton_name: String,
    /// Total duration in seconds
    pub duration: f32,
    /// Sample rate (frames per second)
    pub sample_rate: f32,
    /// Whether this animation should loop
    pub looping: bool,
    /// Extracted root motion data
    pub root_motion: Option<RootMotion>,
    /// Per-bone animation timelines, keyed by bone index
    pub timelines: HashMap<usize, BoneTimeline>,
}

impl Animation2D {
    /// Create a new Animation2D with full validation
    ///
    /// # Arguments
    /// * `bone_count` - Number of bones in the target skeleton (for validation)
    ///
    /// # Errors
    /// Returns `ValidationError` if any invariant is violated
    pub fn new(
        name: impl Into<String>,
        skeleton_name: impl Into<String>,
        duration: f32,
        sample_rate: f32,
        looping: bool,
        root_motion: Option<RootMotion>,
        timelines: HashMap<usize, BoneTimeline>,
        bone_count: usize,
    ) -> Result<Self, ValidationError> {
        let animation = Self {
            name: name.into(),
            skeleton_name: skeleton_name.into(),
            duration,
            sample_rate,
            looping,
            root_motion,
            timelines,
        };
        animation.validate(bone_count)?;
        Ok(animation)
    }

    /// Validate animation against a known bone count
    pub fn validate(&self, bone_count: usize) -> Result<(), ValidationError> {
        self.validate_duration()?;
        self.validate_sample_rate()?;
        self.validate_non_empty_timelines()?;
        self.validate_timeline_bone_indices(bone_count)?;
        self.validate_keyframe_times()?;
        self.validate_keyframe_sorting()?;
        self.validate_rotation_keyframes()?;
        self.validate_root_motion()?;
        Ok(())
    }

    /// Validate animation is compatible with a specific skeleton
    pub fn validate_compatibility(&self, skeleton: &Skeleton2D) -> Result<(), ValidationError> {
        // Check skeleton name matches
        if self.skeleton_name != skeleton.name {
            return Err(ValidationError::SkeletonNameMismatch {
                animation_skeleton: self.skeleton_name.clone(),
                actual_skeleton: skeleton.name.clone(),
            });
        }

        // Check all timeline bone indices are valid
        for &bone_idx in self.timelines.keys() {
            if bone_idx >= skeleton.bone_count() {
                return Err(ValidationError::TimelineBoneIndexOutOfRange {
                    index: bone_idx,
                    bone_count: skeleton.bone_count(),
                });
            }
        }

        Ok(())
    }

    /// Get expected frame count based on duration and sample rate
    pub fn expected_frame_count(&self) -> usize {
        (self.duration * self.sample_rate).ceil() as usize
    }

    /// Sample the animation at a given time for a specific bone
    pub fn sample_bone(&self, bone_index: usize, time: f32) -> Option<SampledPose> {
        let timeline = self.timelines.get(&bone_index)?;
        let clamped_time = time.clamp(0.0, self.duration);

        let rotation = Self::sample_keyframes(&timeline.rotations, clamped_time);
        let translation = timeline.translations.as_ref()
            .map(|t| Self::sample_keyframes_vec2(t, clamped_time));
        let scale = timeline.scales.as_ref()
            .map(|s| Self::sample_keyframes_vec2(s, clamped_time));

        Some(SampledPose { rotation, translation, scale })
    }

    fn sample_keyframes(keyframes: &[Keyframe<f32>], time: f32) -> f32 {
        if keyframes.is_empty() {
            return 0.0;
        }
        if keyframes.len() == 1 || time <= keyframes[0].time {
            return keyframes[0].value;
        }
        if time >= keyframes.last().unwrap().time {
            return keyframes.last().unwrap().value;
        }

        // Binary search for surrounding keyframes
        let idx = keyframes.partition_point(|k| k.time <= time);
        let prev = &keyframes[idx.saturating_sub(1)];
        let next = &keyframes[idx.min(keyframes.len() - 1)];

        if prev.time == next.time {
            return prev.value;
        }

        let t = (time - prev.time) / (next.time - prev.time);

        match prev.interpolation {
            Interpolation::Step => prev.value,
            Interpolation::Linear => prev.value + (next.value - prev.value) * t,
            Interpolation::CubicSpline => {
                // Simplified: fall back to linear for now
                prev.value + (next.value - prev.value) * t
            }
        }
    }

    fn sample_keyframes_vec2(keyframes: &[Keyframe<Vec2>], time: f32) -> Vec2 {
        if keyframes.is_empty() {
            return Vec2::ZERO;
        }
        if keyframes.len() == 1 || time <= keyframes[0].time {
            return keyframes[0].value;
        }
        if time >= keyframes.last().unwrap().time {
            return keyframes.last().unwrap().value;
        }

        let idx = keyframes.partition_point(|k| k.time <= time);
        let prev = &keyframes[idx.saturating_sub(1)];
        let next = &keyframes[idx.min(keyframes.len() - 1)];

        if prev.time == next.time {
            return prev.value;
        }

        let t = (time - prev.time) / (next.time - prev.time);

        match prev.interpolation {
            Interpolation::Step => prev.value,
            Interpolation::Linear => prev.value.lerp(next.value, t),
            Interpolation::CubicSpline => prev.value.lerp(next.value, t),
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Private validation methods
    // ─────────────────────────────────────────────────────────────

    fn validate_duration(&self) -> Result<(), ValidationError> {
        if self.duration <= 0.0 {
            return Err(ValidationError::InvalidDuration { duration: self.duration });
        }
        Ok(())
    }

    fn validate_sample_rate(&self) -> Result<(), ValidationError> {
        if self.sample_rate <= 0.0 {
            return Err(ValidationError::InvalidSampleRate { sample_rate: self.sample_rate });
        }
        Ok(())
    }

    fn validate_non_empty_timelines(&self) -> Result<(), ValidationError> {
        if self.timelines.is_empty() {
            return Err(ValidationError::EmptyTimelines);
        }
        Ok(())
    }

    fn validate_timeline_bone_indices(&self, bone_count: usize) -> Result<(), ValidationError> {
        for &bone_idx in self.timelines.keys() {
            if bone_idx >= bone_count {
                return Err(ValidationError::TimelineTargetsInvalidBone { bone_index: bone_idx });
            }
        }
        Ok(())
    }

    fn validate_keyframe_times(&self) -> Result<(), ValidationError> {
        for (&bone_idx, timeline) in &self.timelines {
            Self::check_keyframe_times(&timeline.rotations, self.duration, bone_idx)?;
            if let Some(ref translations) = timeline.translations {
                Self::check_keyframe_times_vec2(translations, self.duration, bone_idx)?;
            }
            if let Some(ref scales) = timeline.scales {
                Self::check_keyframe_times_vec2(scales, self.duration, bone_idx)?;
            }
        }
        Ok(())
    }

    fn check_keyframe_times(
        keyframes: &[Keyframe<f32>],
        duration: f32,
        _bone_idx: usize,
    ) -> Result<(), ValidationError> {
        for (i, kf) in keyframes.iter().enumerate() {
            if kf.time < 0.0 || kf.time > duration {
                return Err(ValidationError::KeyframeTimeOutOfRange {
                    index: i,
                    time: kf.time,
                    duration,
                });
            }
        }
        Ok(())
    }

    fn check_keyframe_times_vec2(
        keyframes: &[Keyframe<Vec2>],
        duration: f32,
        _bone_idx: usize,
    ) -> Result<(), ValidationError> {
        for (i, kf) in keyframes.iter().enumerate() {
            if kf.time < 0.0 || kf.time > duration {
                return Err(ValidationError::KeyframeTimeOutOfRange {
                    index: i,
                    time: kf.time,
                    duration,
                });
            }
        }
        Ok(())
    }

    fn validate_keyframe_sorting(&self) -> Result<(), ValidationError> {
        for timeline in self.timelines.values() {
            Self::check_sorted(&timeline.rotations)?;
            if let Some(ref translations) = timeline.translations {
                Self::check_sorted_vec2(translations)?;
            }
            if let Some(ref scales) = timeline.scales {
                Self::check_sorted_vec2(scales)?;
            }
        }
        Ok(())
    }

    fn check_sorted(keyframes: &[Keyframe<f32>]) -> Result<(), ValidationError> {
        for i in 1..keyframes.len() {
            if keyframes[i].time < keyframes[i - 1].time {
                return Err(ValidationError::KeyframesNotSorted {
                    index: i,
                    time: keyframes[i].time,
                    prev_time: keyframes[i - 1].time,
                });
            }
        }
        Ok(())
    }

    fn check_sorted_vec2(keyframes: &[Keyframe<Vec2>]) -> Result<(), ValidationError> {
        for i in 1..keyframes.len() {
            if keyframes[i].time < keyframes[i - 1].time {
                return Err(ValidationError::KeyframesNotSorted {
                    index: i,
                    time: keyframes[i].time,
                    prev_time: keyframes[i - 1].time,
                });
            }
        }
        Ok(())
    }

    fn validate_rotation_keyframes(&self) -> Result<(), ValidationError> {
        for (&bone_idx, timeline) in &self.timelines {
            if timeline.rotations.is_empty() {
                return Err(ValidationError::EmptyRotationKeyframes { bone_index: bone_idx });
            }
        }
        Ok(())
    }

    fn validate_root_motion(&self) -> Result<(), ValidationError> {
        if let Some(ref root_motion) = self.root_motion {
            let expected = self.expected_frame_count();
            if root_motion.deltas.len() != expected && !root_motion.deltas.is_empty() {
                return Err(ValidationError::RootMotionDeltasMismatch {
                    deltas_count: root_motion.deltas.len(),
                    expected_count: expected,
                });
            }
        }
        Ok(())
    }
}

/// Result of sampling animation at a specific time
#[derive(Debug, Clone, Copy)]
pub struct SampledPose {
    pub rotation: f32,
    pub translation: Option<Vec2>,
    pub scale: Option<Vec2>,
}
