//! Asset types and loaders for Skeleton2D and Animation2D
//!
//! Provides Bevy asset integration for loading `.skeleton2d.ron` and `.anim2d.ron` files.

use crate::core::{Animation2D, Skeleton2D};
use crate::error::ValidationError;
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use thiserror::Error;

// ═══════════════════════════════════════════════════════════════════════════════
// ASSET TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Bevy asset wrapper for [`Skeleton2D`]
///
/// Load using the asset server:
/// ```ignore
/// let handle: Handle<Skeleton2DAsset> = asset_server.load("player.skeleton2d.ron");
/// ```
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Skeleton2DAsset(pub Skeleton2D);

impl Skeleton2DAsset {
    /// Get a reference to the inner skeleton
    #[inline]
    pub fn skeleton(&self) -> &Skeleton2D {
        &self.0
    }
}

impl std::ops::Deref for Skeleton2DAsset {
    type Target = Skeleton2D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Bevy asset wrapper for [`Animation2D`]
///
/// Load using the asset server:
/// ```ignore
/// let handle: Handle<Animation2DAsset> = asset_server.load("walk.anim2d.ron");
/// ```
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Animation2DAsset(pub Animation2D);

impl Animation2DAsset {
    /// Get a reference to the inner animation
    #[inline]
    pub fn animation(&self) -> &Animation2D {
        &self.0
    }
}

impl std::ops::Deref for Animation2DAsset {
    type Target = Animation2D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Error type for skeleton asset loading
#[derive(Debug, Error)]
pub enum Skeleton2DAssetLoaderError {
    /// IO error reading the file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// RON parsing error
    #[error("RON parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),

    /// Skeleton validation error
    #[error("Skeleton validation error: {0}")]
    Validation(#[from] ValidationError),
}

/// Error type for animation asset loading
#[derive(Debug, Error)]
pub enum Animation2DAssetLoaderError {
    /// IO error reading the file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// RON parsing error
    #[error("RON parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),

    /// Animation validation error (partial - without bone count)
    #[error("Animation validation error: {0}")]
    Validation(#[from] ValidationError),
}

// ═══════════════════════════════════════════════════════════════════════════════
// ASSET LOADERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Asset loader for `.skeleton2d.ron` files
///
/// Loads and validates [`Skeleton2D`] data from RON format.
#[derive(Default)]
pub struct Skeleton2DAssetLoader;

impl AssetLoader for Skeleton2DAssetLoader {
    type Asset = Skeleton2DAsset;
    type Settings = ();
    type Error = Skeleton2DAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // Read file contents
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        // Parse RON to Skeleton2D
        let contents = std::str::from_utf8(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let skeleton: Skeleton2D = ron::from_str(contents)?;

        // Validate skeleton (defense in depth - file may be hand-edited)
        skeleton.validate()?;

        Ok(Skeleton2DAsset(skeleton))
    }

    fn extensions(&self) -> &[&str] {
        &["skeleton2d.ron"]
    }
}

/// Asset loader for `.anim2d.ron` files
///
/// Loads [`Animation2D`] data from RON format. Performs partial validation
/// (duration, sample rate, keyframe times) but cannot validate bone indices
/// without knowing the target skeleton's bone count. Full compatibility
/// validation happens at runtime when paired with a skeleton.
#[derive(Default)]
pub struct Animation2DAssetLoader;

impl AssetLoader for Animation2DAssetLoader {
    type Asset = Animation2DAsset;
    type Settings = ();
    type Error = Animation2DAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // Read file contents
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        // Parse RON to Animation2D
        let contents = std::str::from_utf8(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let animation: Animation2D = ron::from_str(contents)?;

        // Perform partial validation (what we can validate without bone count)
        // Note: We use a large bone count to pass bone index validation,
        // since full validation happens at runtime with the actual skeleton
        validate_animation_partial(&animation)?;

        Ok(Animation2DAsset(animation))
    }

    fn extensions(&self) -> &[&str] {
        &["anim2d.ron"]
    }
}

/// Partial animation validation (without bone count)
///
/// Validates properties that don't depend on the target skeleton:
/// - Duration > 0
/// - Sample rate > 0
/// - Non-empty timelines
/// - Keyframes sorted and within duration bounds
/// - Each timeline has at least one rotation keyframe
fn validate_animation_partial(animation: &Animation2D) -> Result<(), ValidationError> {
    // Validate duration
    if animation.duration <= 0.0 {
        return Err(ValidationError::InvalidDuration {
            duration: animation.duration,
        });
    }

    // Validate sample rate
    if animation.sample_rate <= 0.0 {
        return Err(ValidationError::InvalidSampleRate {
            sample_rate: animation.sample_rate,
        });
    }

    // Validate non-empty timelines
    if animation.timelines.is_empty() {
        return Err(ValidationError::EmptyTimelines);
    }

    // Validate keyframe properties for each timeline
    for (&bone_idx, timeline) in &animation.timelines {
        // Check rotation keyframes exist
        if timeline.rotations.is_empty() {
            return Err(ValidationError::EmptyRotationKeyframes { bone_index: bone_idx });
        }

        // Check rotation keyframes are sorted and in bounds
        for (i, kf) in timeline.rotations.iter().enumerate() {
            if kf.time < 0.0 || kf.time > animation.duration {
                return Err(ValidationError::KeyframeTimeOutOfRange {
                    index: i,
                    time: kf.time,
                    duration: animation.duration,
                });
            }
            if i > 0 && kf.time < timeline.rotations[i - 1].time {
                return Err(ValidationError::KeyframesNotSorted {
                    index: i,
                    time: kf.time,
                    prev_time: timeline.rotations[i - 1].time,
                });
            }
        }

        // Check optional translation keyframes
        if let Some(ref translations) = timeline.translations {
            for (i, kf) in translations.iter().enumerate() {
                if kf.time < 0.0 || kf.time > animation.duration {
                    return Err(ValidationError::KeyframeTimeOutOfRange {
                        index: i,
                        time: kf.time,
                        duration: animation.duration,
                    });
                }
                if i > 0 && kf.time < translations[i - 1].time {
                    return Err(ValidationError::KeyframesNotSorted {
                        index: i,
                        time: kf.time,
                        prev_time: translations[i - 1].time,
                    });
                }
            }
        }

        // Check optional scale keyframes
        if let Some(ref scales) = timeline.scales {
            for (i, kf) in scales.iter().enumerate() {
                if kf.time < 0.0 || kf.time > animation.duration {
                    return Err(ValidationError::KeyframeTimeOutOfRange {
                        index: i,
                        time: kf.time,
                        duration: animation.duration,
                    });
                }
                if i > 0 && kf.time < scales[i - 1].time {
                    return Err(ValidationError::KeyframesNotSorted {
                        index: i,
                        time: kf.time,
                        prev_time: scales[i - 1].time,
                    });
                }
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Bone2D, BoneTimeline, Keyframe, PrimitiveShape, Transform2D};
    use std::collections::HashMap;

    fn make_valid_skeleton() -> Skeleton2D {
        let bones = vec![
            Bone2D::new("root", None, 1.0, Transform2D::default(), PrimitiveShape::None),
            Bone2D::new("child", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
        ];
        Skeleton2D::new("test", bones).unwrap()
    }

    fn make_valid_animation() -> Animation2D {
        let mut timelines = HashMap::new();
        timelines.insert(
            0,
            BoneTimeline::new(vec![
                Keyframe::linear(0.0, 0.0),
                Keyframe::linear(1.0, 1.0),
            ]),
        );
        Animation2D::new("test_anim", "test", 1.0, 30.0, true, None, timelines, 2).unwrap()
    }

    #[test]
    fn test_skeleton_asset_deref() {
        let skeleton = make_valid_skeleton();
        let asset = Skeleton2DAsset(skeleton.clone());
        assert_eq!(asset.name, skeleton.name);
        assert_eq!(asset.bone_count(), skeleton.bone_count());
    }

    #[test]
    fn test_animation_asset_deref() {
        let animation = make_valid_animation();
        let asset = Animation2DAsset(animation.clone());
        assert_eq!(asset.name, animation.name);
        assert_eq!(asset.duration, animation.duration);
    }

    #[test]
    fn test_partial_validation_valid_animation() {
        let animation = make_valid_animation();
        assert!(validate_animation_partial(&animation).is_ok());
    }

    #[test]
    fn test_partial_validation_zero_duration() {
        let mut timelines = HashMap::new();
        timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

        // We need to bypass the normal constructor to create an invalid animation
        let animation = Animation2D {
            name: "test".to_string(),
            skeleton_name: "test".to_string(),
            duration: 0.0, // Invalid!
            sample_rate: 30.0,
            looping: true,
            root_motion: None,
            timelines,
        };

        let result = validate_animation_partial(&animation);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidDuration { duration: 0.0 })
        ));
    }

    #[test]
    fn test_partial_validation_empty_timelines() {
        let animation = Animation2D {
            name: "test".to_string(),
            skeleton_name: "test".to_string(),
            duration: 1.0,
            sample_rate: 30.0,
            looping: true,
            root_motion: None,
            timelines: HashMap::new(), // Empty!
        };

        let result = validate_animation_partial(&animation);
        assert!(matches!(result, Err(ValidationError::EmptyTimelines)));
    }

    #[test]
    fn test_partial_validation_unsorted_keyframes() {
        let mut timelines = HashMap::new();
        timelines.insert(
            0,
            BoneTimeline::new(vec![
                Keyframe::linear(0.5, 0.0),
                Keyframe::linear(0.2, 1.0), // Out of order!
            ]),
        );

        let animation = Animation2D {
            name: "test".to_string(),
            skeleton_name: "test".to_string(),
            duration: 1.0,
            sample_rate: 30.0,
            looping: true,
            root_motion: None,
            timelines,
        };

        let result = validate_animation_partial(&animation);
        assert!(matches!(
            result,
            Err(ValidationError::KeyframesNotSorted { .. })
        ));
    }

    #[test]
    fn test_partial_validation_keyframe_out_of_range() {
        let mut timelines = HashMap::new();
        timelines.insert(
            0,
            BoneTimeline::new(vec![
                Keyframe::linear(0.0, 0.0),
                Keyframe::linear(2.0, 1.0), // Exceeds duration!
            ]),
        );

        let animation = Animation2D {
            name: "test".to_string(),
            skeleton_name: "test".to_string(),
            duration: 1.0,
            sample_rate: 30.0,
            looping: true,
            root_motion: None,
            timelines,
        };

        let result = validate_animation_partial(&animation);
        assert!(matches!(
            result,
            Err(ValidationError::KeyframeTimeOutOfRange { .. })
        ));
    }
}
