//! 3D to 2D transform projection

use crate::core::{
    Animation2D, Bone2D, BoneTimeline, Interpolation, Keyframe, PrimitiveShape, Skeleton2D,
    Transform2D,
};
use crate::gltf_parser::{
    ChannelProperty, GltfAnimation3D, GltfKeyframe3D, GltfNode3D, KeyframeValue3D,
};
use glam::{Quat, Vec2, Vec3};
use std::collections::HashMap;

/// Configuration for 3D to 2D projection
#[derive(Debug, Clone)]
pub struct ProjectionConfig {
    /// View axis for projection (which 3D axis faces the camera)
    /// Default: Z axis (profile view from the side)
    pub view_axis: ViewAxis,
    /// Scale factor to convert 3D units to 2D units
    pub scale: f32,
    /// Whether to compute foreshortening scale from depth
    pub enable_foreshortening: bool,
    /// Sample rate for output animation (FPS)
    pub sample_rate: f32,
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            view_axis: ViewAxis::Z,
            scale: 100.0, // Mixamo uses meters, convert to pixels
            enable_foreshortening: true,
            sample_rate: 30.0,
        }
    }
}

/// Which axis the camera looks along
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAxis {
    /// Camera looks along +X (YZ plane visible)
    X,
    /// Camera looks along +Y (XZ plane visible)
    Y,
    /// Camera looks along +Z (XY plane visible) - typical side view
    Z,
}

/// Project 3D skeleton to 2D
pub fn project_skeleton(
    nodes: &[GltfNode3D],
    name: &str,
    config: &ProjectionConfig,
) -> Result<Skeleton2D, crate::error::ValidationError> {
    let bones: Vec<Bone2D> = nodes
        .iter()
        .map(|node| {
            let translation_2d = project_vec3(node.translation, config);
            let rotation_2d = project_rotation(node.rotation, config);
            let scale_2d = if config.enable_foreshortening {
                compute_foreshortening_scale(node.translation, config)
            } else {
                Vec2::ONE
            };

            // Compute bone length from translation magnitude
            let length = translation_2d.length() * config.scale;

            Bone2D::new(
                node.name.clone(),
                node.parent_index,
                length,
                Transform2D::new(translation_2d * config.scale, rotation_2d, scale_2d),
                PrimitiveShape::None, // Can be customized per-bone later
            )
        })
        .collect();

    Skeleton2D::new(name, bones)
}

/// Project 3D animation to 2D
pub fn project_animation(
    anim: &GltfAnimation3D,
    nodes: &[GltfNode3D],
    skeleton_name: &str,
    config: &ProjectionConfig,
) -> Result<Animation2D, crate::error::ValidationError> {
    // Build node index to bone index mapping
    let node_to_bone: HashMap<usize, usize> = nodes
        .iter()
        .enumerate()
        .map(|(bone_idx, node)| (node.index, bone_idx))
        .collect();

    // Group channels by bone
    let mut bone_channels: HashMap<usize, BoneChannels> = HashMap::new();

    for channel in &anim.channels {
        if let Some(&bone_idx) = node_to_bone.get(&channel.node_index) {
            let entry = bone_channels.entry(bone_idx).or_default();
            match channel.property {
                ChannelProperty::Translation => {
                    entry.translations = Some(&channel.keyframes);
                }
                ChannelProperty::Rotation => {
                    entry.rotations = Some(&channel.keyframes);
                }
                ChannelProperty::Scale => {
                    entry.scales = Some(&channel.keyframes);
                }
            }
        }
    }

    // Convert to 2D timelines
    let mut timelines: HashMap<usize, BoneTimeline> = HashMap::new();

    for (bone_idx, channels) in bone_channels {
        let timeline = project_bone_timeline(&channels, config);
        if !timeline.rotations.is_empty() {
            timelines.insert(bone_idx, timeline);
        }
    }

    // Ensure at least root bone has a timeline
    if timelines.is_empty() && !nodes.is_empty() {
        timelines.insert(
            0,
            BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]),
        );
    }

    Animation2D::new(
        anim.name.clone(),
        skeleton_name,
        anim.duration.max(0.001), // Ensure positive duration
        config.sample_rate,
        true, // Default to looping
        None, // Root motion extracted separately
        timelines,
        nodes.len(),
    )
}

/// Temporary struct to group channels for a bone
#[derive(Default)]
struct BoneChannels<'a> {
    translations: Option<&'a [GltfKeyframe3D]>,
    rotations: Option<&'a [GltfKeyframe3D]>,
    scales: Option<&'a [GltfKeyframe3D]>,
}

/// Project bone channels to 2D timeline
fn project_bone_timeline(channels: &BoneChannels, config: &ProjectionConfig) -> BoneTimeline {
    // Project rotations (required)
    let rotations = channels
        .rotations
        .map(|kfs| {
            kfs.iter()
                .map(|kf| {
                    let rotation = match &kf.value {
                        KeyframeValue3D::Rotation(q) => project_rotation(*q, config),
                        _ => 0.0,
                    };
                    Keyframe::new(kf.time, rotation, convert_interpolation(kf.interpolation))
                })
                .collect()
        })
        .unwrap_or_else(|| vec![Keyframe::linear(0.0, 0.0)]);

    let mut timeline = BoneTimeline::new(rotations);

    // Project translations (optional)
    if let Some(kfs) = channels.translations {
        let translations: Vec<Keyframe<Vec2>> = kfs
            .iter()
            .map(|kf| {
                let translation = match &kf.value {
                    KeyframeValue3D::Translation(v) => project_vec3(*v, config) * config.scale,
                    _ => Vec2::ZERO,
                };
                Keyframe::new(kf.time, translation, convert_interpolation(kf.interpolation))
            })
            .collect();
        timeline = timeline.with_translations(translations);
    }

    // Project scales with foreshortening (optional)
    if config.enable_foreshortening {
        if let Some(trans_kfs) = channels.translations {
            let scales: Vec<Keyframe<Vec2>> = trans_kfs
                .iter()
                .map(|kf| {
                    let scale = match &kf.value {
                        KeyframeValue3D::Translation(v) => compute_foreshortening_scale(*v, config),
                        _ => Vec2::ONE,
                    };
                    Keyframe::new(kf.time, scale, convert_interpolation(kf.interpolation))
                })
                .collect();
            timeline = timeline.with_scales(scales);
        }
    }

    timeline
}

/// Project Vec3 to Vec2 based on view axis
///
/// Mixamo coordinate convention:
/// - Z axis: forward (positive) / backward (negative)
/// - X axis: left (positive) / right (negative) from character's perspective
/// - Y axis: up (positive) / down (negative)
///
/// When viewing along an axis, we project the perpendicular plane to 2D.
/// The sign conventions ensure:
/// - Positive forward motion (in 3D) appears as leftward motion (in 2D) for side-scrollers
/// - Positive upward motion (in 3D) appears as upward motion (in 2D)
fn project_vec3(v: Vec3, config: &ProjectionConfig) -> Vec2 {
    match config.view_axis {
        // Looking along +X axis (from right side of character):
        // 3D +Z (forward) → 2D -X (leftward on screen)
        // 3D +Y (up) → 2D +Y (upward on screen)
        ViewAxis::X => Vec2::new(-v.z, v.y),
        ViewAxis::Y => Vec2::new(v.x, v.z), // Looking along Y, see XZ (top-down view)
        ViewAxis::Z => Vec2::new(v.x, v.y), // Looking along Z, see XY (front/back view)
    }
}

/// Project 3D rotation to 2D angle
///
/// Computes the angle of the bone's direction when projected to 2D.
/// Bones in skeletal systems typically point along local +Y.
/// We rotate Vec3::Y by the bone's rotation, project to 2D, and compute the angle.
fn project_rotation(q: Quat, config: &ProjectionConfig) -> f32 {
    // Bones typically point along local +Y axis
    // Compute where this direction points in world space after rotation
    let bone_direction_3d = q * Vec3::Y;

    // Project the direction to 2D
    let bone_direction_2d = project_vec3(bone_direction_3d, config);

    // Compute the angle this direction makes with the 2D +Y axis
    // atan2(x, y) gives the angle from +Y axis, counterclockwise positive
    bone_direction_2d.x.atan2(bone_direction_2d.y)
}

/// Compute foreshortening scale based on depth
fn compute_foreshortening_scale(translation: Vec3, config: &ProjectionConfig) -> Vec2 {
    // Get depth along view axis
    let depth = match config.view_axis {
        ViewAxis::X => translation.x,
        ViewAxis::Y => translation.y,
        ViewAxis::Z => translation.z,
    };

    // Simple linear foreshortening: closer = larger, further = smaller
    // Using a reference distance of 1.0 units
    let scale_factor = 1.0 / (1.0 + depth.abs() * 0.1);

    Vec2::new(scale_factor, scale_factor)
}

/// Convert GLTF interpolation to our interpolation type
fn convert_interpolation(interp: gltf::animation::Interpolation) -> Interpolation {
    match interp {
        gltf::animation::Interpolation::Linear => Interpolation::Linear,
        gltf::animation::Interpolation::Step => Interpolation::Step,
        gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
    }
}
