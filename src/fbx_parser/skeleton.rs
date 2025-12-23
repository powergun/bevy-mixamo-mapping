//! 3D skeleton extraction from FBX files

use super::FbxData;
use crate::error::{Mixamo2dError, Result};
use crate::gltf_parser::GltfNode3D;
use glam::{Quat, Vec3};
use std::collections::HashMap;
use std::ops::Deref;

/// Extract skeleton hierarchy from FBX scene
///
/// Finds bone nodes and extracts them in topological order (parents before children).
/// Uses Mixamo naming convention (mixamorig:Hips as root).
pub fn extract_skeleton(data: &FbxData) -> Result<Vec<GltfNode3D>> {
    let scene: &ufbx::Scene = data.scene.deref();

    // Find the skeleton root (Hips bone)
    let root_idx = find_skeleton_root_index(scene).ok_or(Mixamo2dError::NoSkeleton)?;

    // Build the skeleton in topological order (BFS)
    let mut result = Vec::new();
    let mut node_id_to_bone_index: HashMap<u32, usize> = HashMap::new();
    let mut queue = vec![(root_idx, None::<usize>)];

    while let Some((node_idx, parent_bone_idx)) = queue.pop() {
        let node = &scene.nodes[node_idx];

        // Only include nodes that are bones (or root)
        if node.bone.is_none() && parent_bone_idx.is_some() {
            continue;
        }

        let bone_index = result.len();
        node_id_to_bone_index.insert(node.element.element_id, bone_index);

        // Extract transform
        let local_transform = &node.local_transform;
        let translation = Vec3::new(
            local_transform.translation.x as f32,
            local_transform.translation.y as f32,
            local_transform.translation.z as f32,
        );
        let rotation = Quat::from_xyzw(
            local_transform.rotation.x as f32,
            local_transform.rotation.y as f32,
            local_transform.rotation.z as f32,
            local_transform.rotation.w as f32,
        );
        let scale = Vec3::new(
            local_transform.scale.x as f32,
            local_transform.scale.y as f32,
            local_transform.scale.z as f32,
        );

        let children_indices: Vec<usize> = node
            .children
            .iter()
            .filter(|c| c.bone.is_some())
            .map(|c| c.element.typed_id as usize)
            .collect();

        result.push(GltfNode3D {
            name: node.element.name.to_string(),
            index: node.element.typed_id as usize,
            parent_index: parent_bone_idx,
            translation,
            rotation,
            scale,
            children: children_indices,
        });

        // Queue children in reverse order to maintain left-to-right processing
        let child_indices: Vec<usize> = node
            .children
            .iter()
            .filter(|c| c.bone.is_some())
            .map(|c| c.element.typed_id as usize)
            .collect();
        for child_idx in child_indices.into_iter().rev() {
            queue.push((child_idx, Some(bone_index)));
        }
    }

    if result.is_empty() {
        return Err(Mixamo2dError::NoSkeleton);
    }

    Ok(result)
}

/// Find the skeleton root node index (typically "mixamorig:Hips")
fn find_skeleton_root_index(scene: &ufbx::Scene) -> Option<usize> {
    // First try to find by name (Mixamo convention)
    let root_candidates = [
        "mixamorig:Hips",
        "mixamorig_Hips",
        "mixamorigHips",
        "Hips",
    ];

    for candidate in &root_candidates {
        if let Some(idx) = scene
            .nodes
            .iter()
            .position(|n| n.element.name.as_ref() == *candidate && n.bone.is_some())
        {
            return Some(idx);
        }
    }

    // Fallback: find first bone node with children that are also bones
    scene.nodes.iter().position(|n| {
        n.bone.is_some()
            && n.children.iter().any(|c| c.bone.is_some())
            && n.parent
                .as_ref()
                .map(|p| p.bone.is_none())
                .unwrap_or(true)
    })
}

/// Get skeleton name from FBX (uses anim stack name or filename)
pub fn get_skeleton_name(data: &FbxData, fallback: &str) -> String {
    let scene: &ufbx::Scene = data.scene.deref();

    // Try to get name from animation stack (Mixamo typically names it)
    if let Some(stack) = scene.anim_stacks.iter().find(|s| {
        let name = s.element.name.as_ref();
        name.contains("mixamo") || name.contains("Take")
    }) {
        let name = stack.element.name.as_ref();
        if !name.is_empty() && name != "Take 001" {
            return name.to_string();
        }
    }

    fallback.to_string()
}
