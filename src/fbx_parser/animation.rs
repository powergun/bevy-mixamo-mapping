//! Animation extraction from FBX files

use super::FbxData;
use crate::error::{Mixamo2dError, Result};
use crate::gltf_parser::{ChannelProperty, GltfAnimation3D, GltfChannel3D, GltfKeyframe3D, KeyframeValue3D};
use glam::{Quat, Vec3};
use std::ops::Deref;

/// Extract all animations from FBX scene
///
/// Returns animation data in the same format as GLTF parser for unified processing.
pub fn extract_animations(data: &FbxData) -> Result<Vec<GltfAnimation3D>> {
    let scene: &ufbx::Scene = data.scene.deref();

    if scene.anim_stacks.is_empty() {
        return Err(Mixamo2dError::NoAnimation);
    }

    let mut animations = Vec::new();

    for stack in &scene.anim_stacks {
        // Skip empty animation stacks
        if stack.layers.is_empty() {
            continue;
        }

        let duration = (stack.time_end - stack.time_begin) as f32;
        if duration <= 0.0 {
            continue;
        }

        // Use baked animation for cleaner data
        let bake_opts = ufbx::BakeOpts {
            resample_rate: 30.0, // 30 FPS
            ..Default::default()
        };

        let baked = match ufbx::bake_anim(scene, &stack.anim, bake_opts) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let mut channels = Vec::new();

        // Extract baked animation for each bone node
        for baked_node in &baked.nodes {
            // Find the original node
            let node = &scene.nodes[baked_node.typed_id as usize];
            if node.bone.is_none() {
                continue;
            }

            let node_index = node.element.typed_id as usize;

            // Translation keyframes
            if !baked_node.translation_keys.is_empty() {
                let keyframes: Vec<GltfKeyframe3D> = baked_node
                    .translation_keys
                    .iter()
                    .map(|key| GltfKeyframe3D {
                        time: key.time as f32,
                        value: KeyframeValue3D::Translation(Vec3::new(
                            key.value.x as f32,
                            key.value.y as f32,
                            key.value.z as f32,
                        )),
                        interpolation: gltf::animation::Interpolation::Linear,
                    })
                    .collect();

                if !keyframes.is_empty() {
                    channels.push(GltfChannel3D {
                        node_index,
                        property: ChannelProperty::Translation,
                        keyframes,
                    });
                }
            }

            // Rotation keyframes
            if !baked_node.rotation_keys.is_empty() {
                let keyframes: Vec<GltfKeyframe3D> = baked_node
                    .rotation_keys
                    .iter()
                    .map(|key| GltfKeyframe3D {
                        time: key.time as f32,
                        value: KeyframeValue3D::Rotation(Quat::from_xyzw(
                            key.value.x as f32,
                            key.value.y as f32,
                            key.value.z as f32,
                            key.value.w as f32,
                        )),
                        interpolation: gltf::animation::Interpolation::Linear,
                    })
                    .collect();

                if !keyframes.is_empty() {
                    channels.push(GltfChannel3D {
                        node_index,
                        property: ChannelProperty::Rotation,
                        keyframes,
                    });
                }
            }

            // Scale keyframes
            if !baked_node.scale_keys.is_empty() {
                let keyframes: Vec<GltfKeyframe3D> = baked_node
                    .scale_keys
                    .iter()
                    .map(|key| GltfKeyframe3D {
                        time: key.time as f32,
                        value: KeyframeValue3D::Scale(Vec3::new(
                            key.value.x as f32,
                            key.value.y as f32,
                            key.value.z as f32,
                        )),
                        interpolation: gltf::animation::Interpolation::Linear,
                    })
                    .collect();

                if !keyframes.is_empty() {
                    channels.push(GltfChannel3D {
                        node_index,
                        property: ChannelProperty::Scale,
                        keyframes,
                    });
                }
            }
        }

        // Only add animations that have actual content
        if !channels.is_empty() {
            animations.push(GltfAnimation3D {
                name: stack.element.name.to_string(),
                duration,
                channels,
            });
        }
    }

    if animations.is_empty() {
        return Err(Mixamo2dError::NoAnimation);
    }

    Ok(animations)
}
