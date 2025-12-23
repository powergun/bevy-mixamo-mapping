//! GLTF/GLB file parsing for Mixamo animations
//!
//! Extracts 3D skeleton and animation data from GLTF files.

pub mod animation;
pub mod skeleton;

pub use animation::extract_animations;
pub use skeleton::extract_skeleton;

use crate::error::Result;
use std::path::Path;

/// Loaded GLTF data with buffer access
pub struct GltfData {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
}

impl GltfData {
    /// Load a GLTF/GLB file from disk
    pub fn load(path: &Path) -> Result<Self> {
        let (document, buffers, _images) = gltf::import(path)?;
        Ok(Self { document, buffers })
    }

    /// Load from GLB bytes in memory
    pub fn from_glb_bytes(bytes: &[u8]) -> Result<Self> {
        let (document, buffers, _images) = gltf::import_slice(bytes)?;
        Ok(Self { document, buffers })
    }
}

/// 3D skeleton node extracted from GLTF
#[derive(Debug, Clone)]
pub struct GltfNode3D {
    pub name: String,
    pub index: usize,
    pub parent_index: Option<usize>,
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    pub children: Vec<usize>,
}

/// 3D animation data extracted from GLTF
#[derive(Debug, Clone)]
pub struct GltfAnimation3D {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<GltfChannel3D>,
}

/// Single animation channel (one property of one node)
#[derive(Debug, Clone)]
pub struct GltfChannel3D {
    pub node_index: usize,
    pub property: ChannelProperty,
    pub keyframes: Vec<GltfKeyframe3D>,
}

/// Animation property type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelProperty {
    Translation,
    Rotation,
    Scale,
}

/// 3D keyframe data
#[derive(Debug, Clone)]
pub struct GltfKeyframe3D {
    pub time: f32,
    pub value: KeyframeValue3D,
    pub interpolation: gltf::animation::Interpolation,
}

/// Keyframe value variants
#[derive(Debug, Clone)]
pub enum KeyframeValue3D {
    Translation(glam::Vec3),
    Rotation(glam::Quat),
    Scale(glam::Vec3),
}
