//! FBX file parsing for Mixamo animations
//!
//! Extracts 3D skeleton and animation data from FBX files using ufbx.
//! Outputs the same data structures as gltf_parser for unified downstream processing.

pub mod animation;
pub mod filter;
pub mod skeleton;
pub mod traversal;

pub use animation::extract_animations;
pub use filter::{BoneFilter, BoneFilterError};
pub use skeleton::extract_skeleton;
pub use traversal::{traverse_skeleton_hierarchy, FormattedBone, SkeletonHierarchy};

use crate::error::{Mixamo2dError, Result};
use std::path::Path;

/// Loaded FBX data with scene access
pub struct FbxData {
    pub scene: ufbx::SceneRoot,
}

impl FbxData {
    /// Load an FBX file from disk
    pub fn load(path: &Path) -> Result<Self> {
        let opts = ufbx::LoadOpts::default();
        let path_str = path.to_str().ok_or_else(|| {
            Mixamo2dError::FbxParse("Invalid path encoding".to_string())
        })?;
        let scene = ufbx::load_file(path_str, opts)
            .map_err(|e| Mixamo2dError::FbxParse(e.description.to_string()))?;
        Ok(Self { scene })
    }

    /// Load from FBX bytes in memory
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let opts = ufbx::LoadOpts::default();
        let scene = ufbx::load_memory(bytes, opts)
            .map_err(|e| Mixamo2dError::FbxParse(e.description.to_string()))?;
        Ok(Self { scene })
    }
}
