//! 3D skeleton extraction from GLTF files

use super::{GltfData, GltfNode3D};
use crate::error::{Mixamo2dError, Result};
use glam::{Quat, Vec3};
use std::collections::{HashMap, HashSet};

/// Extract skeleton hierarchy from GLTF document
///
/// Finds the skeleton root and extracts all descendant bones in topological order.
/// Supports Mixamo naming conventions (with or without colons).
/// Duplicate bone names are made unique by appending `.1`, `.2`, etc.
pub fn extract_skeleton(data: &GltfData) -> Result<Vec<GltfNode3D>> {
    let document = &data.document;

    // Build node lookup
    let nodes: Vec<_> = document.nodes().collect();

    // Try to find skeleton from skin first (most reliable)
    let skeleton_root = find_skeleton_from_skin(document, &nodes)
        .or_else(|| find_skeleton_root_by_name(&nodes))
        .ok_or(Mixamo2dError::NoSkeleton)?;

    // Extract bones in topological order (BFS)
    let mut result = Vec::new();
    let mut node_to_bone_index: HashMap<usize, usize> = HashMap::new();
    let mut seen_names: HashSet<String> = HashSet::new();
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    let mut queue = vec![(skeleton_root, None::<usize>)];

    while let Some((node_idx, parent_bone_idx)) = queue.pop() {
        let node = &nodes[node_idx];
        let (translation, rotation, scale) = node.transform().decomposed();

        let bone_index = result.len();
        node_to_bone_index.insert(node_idx, bone_index);

        // Make bone name unique if it's a duplicate
        let base_name = node.name().unwrap_or("unnamed").to_string();
        let unique_name = if seen_names.contains(&base_name) {
            let count = name_counts.entry(base_name.clone()).or_insert(1);
            *count += 1;
            format!("{}.{}", base_name, count)
        } else {
            seen_names.insert(base_name.clone());
            name_counts.insert(base_name.clone(), 1);
            base_name
        };

        result.push(GltfNode3D {
            name: unique_name,
            index: node_idx,
            parent_index: parent_bone_idx,
            translation: Vec3::from_array(translation),
            rotation: Quat::from_array(rotation),
            scale: Vec3::from_array(scale),
            children: node.children().map(|c| c.index()).collect(),
        });

        // Add children to queue (reverse order to maintain left-to-right)
        let children: Vec<_> = node.children().collect();
        for child in children.into_iter().rev() {
            queue.push((child.index(), Some(bone_index)));
        }
    }

    if result.is_empty() {
        return Err(Mixamo2dError::NoSkeleton);
    }

    Ok(result)
}

/// Find skeleton root from skin definition (most reliable method)
fn find_skeleton_from_skin<'a>(
    document: &'a gltf::Document,
    nodes: &[gltf::Node<'a>],
) -> Option<usize> {
    // Find skin with joints that have a proper hierarchy
    for skin in document.skins() {
        // Check if skin has explicit skeleton root
        if let Some(skeleton_root) = skin.skeleton() {
            // Verify it has children (is actually a skeleton root)
            if skeleton_root.children().count() > 0 {
                return Some(skeleton_root.index());
            }
        }

        // Find root among joints (the one that's not a child of another joint)
        let joint_indices: std::collections::HashSet<usize> =
            skin.joints().map(|j| j.index()).collect();

        for joint in skin.joints() {
            // A root joint is one whose parent (if any) is not in the joint set
            // Also must have children to be a skeleton root
            if joint.children().count() > 0 {
                // Check if this joint's parent is NOT in the skin's joint set
                let is_root_candidate = nodes.iter().all(|parent_node| {
                    !parent_node
                        .children()
                        .any(|c| c.index() == joint.index() && joint_indices.contains(&parent_node.index()))
                });

                // For Mixamo, the root is typically named "Hips" and has children
                let name = joint.name().unwrap_or("");
                if name.to_lowercase().contains("hips") && joint.children().count() > 0 {
                    return Some(joint.index());
                }

                if is_root_candidate {
                    return Some(joint.index());
                }
            }
        }
    }

    None
}

/// Find skeleton root by name (fallback method)
fn find_skeleton_root_by_name(nodes: &[gltf::Node]) -> Option<usize> {
    // Common Mixamo root bone names (with and without colons/underscores)
    let root_candidates = [
        "mixamorig:Hips",
        "mixamorig_Hips",
        "mixamorigHips",
        "Hips",
        "Armature",
        "Root",
        "root",
    ];

    // Try exact match first, preferring nodes with children
    for candidate in &root_candidates {
        // First pass: exact match with children
        if let Some(node) = nodes
            .iter()
            .find(|n| n.name() == Some(*candidate) && n.children().count() > 0)
        {
            return Some(node.index());
        }
    }

    // Second pass: case-insensitive "hips" with children
    if let Some(node) = nodes.iter().find(|n| {
        n.name()
            .map(|name| name.to_lowercase().contains("hips"))
            .unwrap_or(false)
            && n.children().count() > 0
    }) {
        return Some(node.index());
    }

    // Third pass: any node with children that looks like a bone
    if let Some(node) = nodes.iter().find(|n| {
        n.children().count() > 0
            && n.name()
                .map(|name| {
                    let lower = name.to_lowercase();
                    lower.contains("mixamo")
                        || lower.contains("armature")
                        || lower.contains("skeleton")
                })
                .unwrap_or(false)
    }) {
        return Some(node.index());
    }

    // Last resort: first node with children
    nodes
        .iter()
        .find(|n| n.children().count() > 0)
        .map(|n| n.index())
}

/// Get skeleton name from GLTF (uses scene name, skin name, or filename)
pub fn get_skeleton_name(data: &GltfData, fallback: &str) -> String {
    // Try scene name
    if let Some(scene) = data
        .document
        .default_scene()
        .or_else(|| data.document.scenes().next())
    {
        if let Some(name) = scene.name() {
            return name.to_string();
        }
    }

    // Try skin name
    if let Some(skin) = data.document.skins().next() {
        if let Some(name) = skin.name() {
            return name.to_string();
        }
    }

    fallback.to_string()
}
