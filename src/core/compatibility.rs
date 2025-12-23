use super::animation::Animation2D;
use super::skeleton::Skeleton2D;
use crate::error::ValidationError;

/// Check if an animation is compatible with a skeleton
pub fn check_compatibility(
    skeleton: &Skeleton2D,
    animation: &Animation2D,
) -> Result<(), ValidationError> {
    // 1. Check skeleton name matches
    if animation.skeleton_name != skeleton.name {
        return Err(ValidationError::SkeletonNameMismatch {
            animation_skeleton: animation.skeleton_name.clone(),
            actual_skeleton: skeleton.name.clone(),
        });
    }

    // 2. Check all timeline bone indices are within skeleton bounds
    for &bone_idx in animation.timelines.keys() {
        if bone_idx >= skeleton.bone_count() {
            return Err(ValidationError::TimelineBoneIndexOutOfRange {
                index: bone_idx,
                bone_count: skeleton.bone_count(),
            });
        }
    }

    Ok(())
}

/// Strict compatibility check - animation must have timelines for ALL skeleton bones
pub fn check_strict_compatibility(
    skeleton: &Skeleton2D,
    animation: &Animation2D,
) -> Result<(), ValidationError> {
    // First, do basic compatibility check
    check_compatibility(skeleton, animation)?;

    // Then verify bone count matches
    if animation.timelines.len() != skeleton.bone_count() {
        return Err(ValidationError::BoneCountMismatch {
            expected: skeleton.bone_count(),
            actual: animation.timelines.len(),
        });
    }

    Ok(())
}
