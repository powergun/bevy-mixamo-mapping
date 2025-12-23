use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Interpolation method between keyframes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Interpolation {
    #[default]
    Linear,
    Step,
    CubicSpline,
}

/// A single keyframe with interpolation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    /// Time in seconds from animation start
    pub time: f32,
    /// Value at this keyframe
    pub value: T,
    /// How to interpolate to the next keyframe
    pub interpolation: Interpolation,
}

impl<T> Keyframe<T> {
    pub fn new(time: f32, value: T, interpolation: Interpolation) -> Self {
        Self { time, value, interpolation }
    }

    pub fn linear(time: f32, value: T) -> Self {
        Self::new(time, value, Interpolation::Linear)
    }
}

/// Animation timeline for a single bone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoneTimeline {
    /// Rotation keyframes (time -> radians) - REQUIRED
    pub rotations: Vec<Keyframe<f32>>,
    /// Translation keyframes (time -> Vec2) - optional, typically only for root
    pub translations: Option<Vec<Keyframe<Vec2>>>,
    /// Scale keyframes (time -> Vec2) - optional, for foreshortening
    pub scales: Option<Vec<Keyframe<Vec2>>>,
}

impl BoneTimeline {
    /// Create a new BoneTimeline with just rotation keyframes
    pub fn new(rotations: Vec<Keyframe<f32>>) -> Self {
        Self {
            rotations,
            translations: None,
            scales: None,
        }
    }

    /// Builder method to add translations
    pub fn with_translations(mut self, translations: Vec<Keyframe<Vec2>>) -> Self {
        self.translations = Some(translations);
        self
    }

    /// Builder method to add scales
    pub fn with_scales(mut self, scales: Vec<Keyframe<Vec2>>) -> Self {
        self.scales = Some(scales);
        self
    }

    /// Check if keyframes are sorted by time
    pub fn are_keyframes_sorted(&self) -> bool {
        Self::check_sorted(&self.rotations)
            && self.translations.as_ref().is_none_or(|t| Self::check_sorted(t))
            && self.scales.as_ref().is_none_or(|s| Self::check_sorted(s))
    }

    fn check_sorted<T>(keyframes: &[Keyframe<T>]) -> bool {
        keyframes.windows(2).all(|w| w[0].time <= w[1].time)
    }
}
