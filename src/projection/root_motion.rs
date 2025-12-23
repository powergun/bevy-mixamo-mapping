//! Root motion extraction from animations

use crate::core::RootMotion;
use crate::gltf_parser::{ChannelProperty, GltfAnimation3D, KeyframeValue3D};
use crate::projection::transform::{ProjectionConfig, ViewAxis};
use glam::Vec2;

/// Extract root motion from a 3D animation
///
/// Finds the root bone's translation channel and computes:
/// - Total displacement over the animation
/// - Average velocity
/// - Per-frame deltas (if sample_rate matches)
pub fn extract_root_motion(
    anim: &GltfAnimation3D,
    root_node_index: usize,
    config: &ProjectionConfig,
) -> Option<RootMotion> {
    // Find root translation channel
    let root_channel = anim.channels.iter().find(|c| {
        c.node_index == root_node_index && c.property == ChannelProperty::Translation
    })?;

    if root_channel.keyframes.is_empty() {
        return None;
    }

    // Get first and last positions
    let first_pos = extract_translation(&root_channel.keyframes.first()?.value)?;
    let last_pos = extract_translation(&root_channel.keyframes.last()?.value)?;

    // Project to 2D
    let first_2d = project_vec3_to_2d(first_pos, config);
    let last_2d = project_vec3_to_2d(last_pos, config);

    let total_displacement = (last_2d - first_2d) * config.scale;
    let velocity = if anim.duration > 0.0 {
        total_displacement / anim.duration
    } else {
        Vec2::ZERO
    };

    // Compute per-frame deltas
    let deltas = compute_deltas(&root_channel.keyframes, anim.duration, config);

    Some(RootMotion {
        total_displacement,
        velocity,
        deltas,
    })
}

/// Strip root motion from animation by zeroing X translation
///
/// Returns the animation with root bone translations modified to stay in place
/// while preserving vertical movement (Y).
pub fn strip_root_motion_horizontal(
    anim: &mut GltfAnimation3D,
    root_node_index: usize,
) {
    for channel in &mut anim.channels {
        if channel.node_index == root_node_index
            && channel.property == ChannelProperty::Translation
        {
            // Get first frame X position to use as anchor
            let anchor_x = channel
                .keyframes
                .first()
                .and_then(|kf| extract_translation(&kf.value))
                .map(|v| v.x)
                .unwrap_or(0.0);

            // Zero out X translation (keep Y and Z)
            for kf in &mut channel.keyframes {
                if let KeyframeValue3D::Translation(ref mut v) = kf.value {
                    v.x = anchor_x;
                }
            }
        }
    }
}

/// Helper to extract Vec3 from keyframe value
fn extract_translation(value: &KeyframeValue3D) -> Option<glam::Vec3> {
    match value {
        KeyframeValue3D::Translation(v) => Some(*v),
        _ => None,
    }
}

/// Project 3D position to 2D based on view axis
fn project_vec3_to_2d(v: glam::Vec3, config: &ProjectionConfig) -> Vec2 {
    match config.view_axis {
        ViewAxis::X => Vec2::new(v.y, v.z),
        ViewAxis::Y => Vec2::new(v.x, v.z),
        ViewAxis::Z => Vec2::new(v.x, v.y),
    }
}

/// Compute per-frame deltas from keyframes
fn compute_deltas(
    keyframes: &[crate::gltf_parser::GltfKeyframe3D],
    duration: f32,
    config: &ProjectionConfig,
) -> Vec<Vec2> {
    if keyframes.len() < 2 || duration <= 0.0 {
        return vec![];
    }

    let frame_count = (duration * config.sample_rate).ceil() as usize;
    let mut deltas = Vec::with_capacity(frame_count);
    let mut prev_pos = Vec2::ZERO;

    for frame in 0..frame_count {
        let time = frame as f32 / config.sample_rate;
        let pos = sample_translation_at_time(keyframes, time, config);
        deltas.push(pos - prev_pos);
        prev_pos = pos;
    }

    deltas
}

/// Sample translation at a specific time using linear interpolation
fn sample_translation_at_time(
    keyframes: &[crate::gltf_parser::GltfKeyframe3D],
    time: f32,
    config: &ProjectionConfig,
) -> Vec2 {
    if keyframes.is_empty() {
        return Vec2::ZERO;
    }

    // Find surrounding keyframes
    let idx = keyframes.partition_point(|k| k.time <= time);

    if idx == 0 {
        return extract_translation(&keyframes[0].value)
            .map(|v| project_vec3_to_2d(v, config) * config.scale)
            .unwrap_or(Vec2::ZERO);
    }

    if idx >= keyframes.len() {
        return extract_translation(&keyframes.last().unwrap().value)
            .map(|v| project_vec3_to_2d(v, config) * config.scale)
            .unwrap_or(Vec2::ZERO);
    }

    let prev = &keyframes[idx - 1];
    let next = &keyframes[idx];

    let prev_pos = extract_translation(&prev.value)
        .map(|v| project_vec3_to_2d(v, config) * config.scale)
        .unwrap_or(Vec2::ZERO);
    let next_pos = extract_translation(&next.value)
        .map(|v| project_vec3_to_2d(v, config) * config.scale)
        .unwrap_or(Vec2::ZERO);

    let t = if next.time > prev.time {
        (time - prev.time) / (next.time - prev.time)
    } else {
        0.0
    };

    prev_pos.lerp(next_pos, t)
}
