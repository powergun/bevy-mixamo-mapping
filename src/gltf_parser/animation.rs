//! Animation extraction from GLTF files

use super::{ChannelProperty, GltfAnimation3D, GltfChannel3D, GltfData, GltfKeyframe3D, KeyframeValue3D};
use crate::error::{Mixamo2dError, Result};
use glam::{Quat, Vec3};

/// Extract all animations from GLTF document
pub fn extract_animations(data: &GltfData) -> Result<Vec<GltfAnimation3D>> {
    let mut animations = Vec::new();

    for anim in data.document.animations() {
        let extracted = extract_single_animation(&anim, &data.buffers)?;
        animations.push(extracted);
    }

    if animations.is_empty() {
        return Err(Mixamo2dError::NoAnimation);
    }

    Ok(animations)
}

/// Extract a single animation
fn extract_single_animation(
    anim: &gltf::Animation,
    buffers: &[gltf::buffer::Data],
) -> Result<GltfAnimation3D> {
    let name = anim.name().unwrap_or("unnamed").to_string();
    let mut channels = Vec::new();
    let mut max_time: f32 = 0.0;

    for channel in anim.channels() {
        let target = channel.target();
        let node_index = target.node().index();

        let property = match target.property() {
            gltf::animation::Property::Translation => ChannelProperty::Translation,
            gltf::animation::Property::Rotation => ChannelProperty::Rotation,
            gltf::animation::Property::Scale => ChannelProperty::Scale,
            gltf::animation::Property::MorphTargetWeights => continue, // Skip morph targets
        };

        let sampler = channel.sampler();
        let interpolation = sampler.interpolation();

        // Read timestamps
        let input = sampler.input();
        let times = read_accessor_f32(&input, buffers);

        // Track max time for duration
        if let Some(&last_time) = times.last() {
            max_time = max_time.max(last_time);
        }

        // Read output values
        let output = sampler.output();
        let keyframes = match property {
            ChannelProperty::Translation => {
                let values = read_accessor_vec3(&output, buffers);
                times
                    .iter()
                    .zip(values.iter())
                    .map(|(&time, &value)| GltfKeyframe3D {
                        time,
                        value: KeyframeValue3D::Translation(value),
                        interpolation,
                    })
                    .collect()
            }
            ChannelProperty::Rotation => {
                let values = read_accessor_quat(&output, buffers);
                times
                    .iter()
                    .zip(values.iter())
                    .map(|(&time, &value)| GltfKeyframe3D {
                        time,
                        value: KeyframeValue3D::Rotation(value),
                        interpolation,
                    })
                    .collect()
            }
            ChannelProperty::Scale => {
                let values = read_accessor_vec3(&output, buffers);
                times
                    .iter()
                    .zip(values.iter())
                    .map(|(&time, &value)| GltfKeyframe3D {
                        time,
                        value: KeyframeValue3D::Scale(value),
                        interpolation,
                    })
                    .collect()
            }
        };

        channels.push(GltfChannel3D {
            node_index,
            property,
            keyframes,
        });
    }

    Ok(GltfAnimation3D {
        name,
        duration: max_time,
        channels,
    })
}

/// Read f32 values from accessor
fn read_accessor_f32(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Vec<f32> {
    let view = accessor.view().expect("Accessor has no buffer view");
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(4);

    (0..accessor.count())
        .map(|i| {
            let start = offset + i * stride;
            let bytes = &buffer[start..start + 4];
            f32::from_le_bytes(bytes.try_into().unwrap())
        })
        .collect()
}

/// Read Vec3 values from accessor
fn read_accessor_vec3(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Vec<Vec3> {
    let view = accessor.view().expect("Accessor has no buffer view");
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(12);

    (0..accessor.count())
        .map(|i| {
            let start = offset + i * stride;
            let x = f32::from_le_bytes(buffer[start..start + 4].try_into().unwrap());
            let y = f32::from_le_bytes(buffer[start + 4..start + 8].try_into().unwrap());
            let z = f32::from_le_bytes(buffer[start + 8..start + 12].try_into().unwrap());
            Vec3::new(x, y, z)
        })
        .collect()
}

/// Read Quat values from accessor (stored as Vec4: x, y, z, w)
fn read_accessor_quat(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Vec<Quat> {
    let view = accessor.view().expect("Accessor has no buffer view");
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(16);

    (0..accessor.count())
        .map(|i| {
            let start = offset + i * stride;
            let x = f32::from_le_bytes(buffer[start..start + 4].try_into().unwrap());
            let y = f32::from_le_bytes(buffer[start + 4..start + 8].try_into().unwrap());
            let z = f32::from_le_bytes(buffer[start + 8..start + 12].try_into().unwrap());
            let w = f32::from_le_bytes(buffer[start + 12..start + 16].try_into().unwrap());
            Quat::from_xyzw(x, y, z, w)
        })
        .collect()
}
