//! RON serialization for skeleton and animation data

use crate::core::{Animation2D, Skeleton2D};
use crate::error::{Mixamo2dError, Result};
use ron::ser::PrettyConfig;
use std::path::Path;

/// Save skeleton to RON file
pub fn save_skeleton(skeleton: &Skeleton2D, path: &Path) -> Result<()> {
    let config = PrettyConfig::new()
        .struct_names(true)
        .separate_tuple_members(true)
        .enumerate_arrays(false);

    let ron_string = ron::ser::to_string_pretty(skeleton, config)
        .map_err(|e| Mixamo2dError::InvalidMapping(e.to_string()))?;
    std::fs::write(path, ron_string)?;
    Ok(())
}

/// Load skeleton from RON file with validation
pub fn load_skeleton(path: &Path) -> Result<Skeleton2D> {
    let contents = std::fs::read_to_string(path)?;
    let skeleton: Skeleton2D = ron::from_str(&contents)?;
    // Re-validate after deserialization (defense in depth)
    skeleton.validate()?;
    Ok(skeleton)
}

/// Save animation to RON file
pub fn save_animation(animation: &Animation2D, path: &Path) -> Result<()> {
    let config = PrettyConfig::new()
        .struct_names(true)
        .separate_tuple_members(true)
        .enumerate_arrays(false);

    let ron_string = ron::ser::to_string_pretty(animation, config)
        .map_err(|e| Mixamo2dError::InvalidMapping(e.to_string()))?;
    std::fs::write(path, ron_string)?;
    Ok(())
}

/// Load animation from RON file with validation
pub fn load_animation(path: &Path, bone_count: usize) -> Result<Animation2D> {
    let contents = std::fs::read_to_string(path)?;
    let animation: Animation2D = ron::from_str(&contents)?;
    // Re-validate after deserialization
    animation.validate(bone_count)?;
    Ok(animation)
}

/// Load animation and validate against a specific skeleton
pub fn load_animation_for_skeleton(path: &Path, skeleton: &Skeleton2D) -> Result<Animation2D> {
    let animation = load_animation(path, skeleton.bone_count())?;
    animation.validate_compatibility(skeleton)?;
    Ok(animation)
}
