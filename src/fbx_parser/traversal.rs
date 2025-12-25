//! Skeleton hierarchy traversal and formatting utilities
//!
//! Provides functions for displaying skeleton hierarchies in human-readable form.

use crate::gltf_parser::GltfNode3D;

/// A single bone in a formatted hierarchy output
#[derive(Debug, Clone)]
pub struct FormattedBone {
    /// The name of the bone
    pub name: String,
    /// Bone index in the skeleton array
    pub index: usize,
    /// Depth level in the hierarchy (0 = root)
    pub depth: usize,
}

/// Result of skeleton hierarchy traversal
#[derive(Debug, Clone)]
pub struct SkeletonHierarchy {
    /// The formatted bones in traversal order (depth-first)
    pub bones: Vec<FormattedBone>,
    /// Total number of bones
    pub bone_count: usize,
    /// Maximum depth in the hierarchy
    pub max_depth: usize,
    /// Name of the root bone
    pub root_name: String,
}

impl SkeletonHierarchy {
    /// Format the hierarchy as a string with tree-like indentation
    ///
    /// # Arguments
    /// * `indent` - The string to use for each level of indentation (e.g., "  " or "    ")
    ///
    /// # Returns
    /// A string with the hierarchy formatted as a tree
    pub fn format_tree(&self, indent: &str) -> String {
        let mut output = String::new();
        for bone in &self.bones {
            let prefix = indent.repeat(bone.depth);
            output.push_str(&format!("{}[{}] {}\n", prefix, bone.index, bone.name));
        }
        output
    }

    /// Format the hierarchy with summary header
    pub fn format_with_summary(&self, indent: &str) -> String {
        let mut output = String::new();
        output.push_str(&format!("Bones: {}\n", self.bone_count));
        output.push_str(&format!("Root: {}\n", self.root_name));
        output.push_str(&format!("Max depth: {}\n", self.max_depth));
        output.push_str("\nHierarchy:\n");
        output.push_str(&self.format_tree(indent));
        output
    }
}

/// Traverse skeleton nodes and build a formatted hierarchy
///
/// The nodes must be in topological order (parents before children).
/// This function performs a depth-first traversal from the root bone.
///
/// # Arguments
/// * `nodes` - The skeleton nodes extracted from FBX
///
/// # Returns
/// A `SkeletonHierarchy` with bones in depth-first traversal order
pub fn traverse_skeleton_hierarchy(nodes: &[GltfNode3D]) -> SkeletonHierarchy {
    if nodes.is_empty() {
        return SkeletonHierarchy {
            bones: Vec::new(),
            bone_count: 0,
            max_depth: 0,
            root_name: String::new(),
        };
    }

    let mut result = Vec::new();
    let mut max_depth = 0;

    // Start from root (first node which has no parent)
    traverse_recursive(nodes, 0, 0, &mut result, &mut max_depth);

    let root_name = nodes
        .first()
        .map(|n| n.name.clone())
        .unwrap_or_default();

    SkeletonHierarchy {
        bone_count: result.len(),
        bones: result,
        max_depth,
        root_name,
    }
}

/// Recursive helper for depth-first traversal
fn traverse_recursive(
    nodes: &[GltfNode3D],
    node_idx: usize,
    depth: usize,
    result: &mut Vec<FormattedBone>,
    max_depth: &mut usize,
) {
    if node_idx >= nodes.len() {
        return;
    }

    let node = &nodes[node_idx];
    *max_depth = (*max_depth).max(depth);

    result.push(FormattedBone {
        name: node.name.clone(),
        index: node_idx,
        depth,
    });

    // Find and traverse children (nodes whose parent_index is this node)
    for (child_idx, child) in nodes.iter().enumerate() {
        if child.parent_index == Some(node_idx) {
            traverse_recursive(nodes, child_idx, depth + 1, result, max_depth);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    fn make_test_node(name: &str, parent: Option<usize>) -> GltfNode3D {
        GltfNode3D {
            name: name.to_string(),
            index: 0, // Not used in traversal
            parent_index: parent,
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            children: vec![],
        }
    }

    #[test]
    fn test_empty_skeleton() {
        let hierarchy = traverse_skeleton_hierarchy(&[]);
        assert_eq!(hierarchy.bone_count, 0);
        assert_eq!(hierarchy.max_depth, 0);
        assert!(hierarchy.root_name.is_empty());
    }

    #[test]
    fn test_single_bone() {
        let nodes = vec![make_test_node("Hips", None)];
        let hierarchy = traverse_skeleton_hierarchy(&nodes);

        assert_eq!(hierarchy.bone_count, 1);
        assert_eq!(hierarchy.max_depth, 0);
        assert_eq!(hierarchy.root_name, "Hips");
        assert_eq!(hierarchy.bones[0].depth, 0);
    }

    #[test]
    fn test_linear_chain() {
        let nodes = vec![
            make_test_node("Hips", None),
            make_test_node("Spine", Some(0)),
            make_test_node("Chest", Some(1)),
            make_test_node("Head", Some(2)),
        ];
        let hierarchy = traverse_skeleton_hierarchy(&nodes);

        assert_eq!(hierarchy.bone_count, 4);
        assert_eq!(hierarchy.max_depth, 3);

        // Verify depths
        assert_eq!(hierarchy.bones[0].depth, 0); // Hips
        assert_eq!(hierarchy.bones[1].depth, 1); // Spine
        assert_eq!(hierarchy.bones[2].depth, 2); // Chest
        assert_eq!(hierarchy.bones[3].depth, 3); // Head
    }

    #[test]
    fn test_branching_skeleton() {
        // Hips -> Spine, LeftLeg, RightLeg
        let nodes = vec![
            make_test_node("Hips", None),
            make_test_node("Spine", Some(0)),
            make_test_node("LeftLeg", Some(0)),
            make_test_node("RightLeg", Some(0)),
        ];
        let hierarchy = traverse_skeleton_hierarchy(&nodes);

        assert_eq!(hierarchy.bone_count, 4);
        assert_eq!(hierarchy.max_depth, 1);

        // All children should be at depth 1
        let depths: Vec<_> = hierarchy.bones.iter().map(|b| (b.name.as_str(), b.depth)).collect();
        assert!(depths.contains(&("Hips", 0)));
        assert!(depths.contains(&("Spine", 1)));
        assert!(depths.contains(&("LeftLeg", 1)));
        assert!(depths.contains(&("RightLeg", 1)));
    }

    #[test]
    fn test_format_tree() {
        let nodes = vec![
            make_test_node("Hips", None),
            make_test_node("Spine", Some(0)),
            make_test_node("Head", Some(1)),
        ];
        let hierarchy = traverse_skeleton_hierarchy(&nodes);
        let tree = hierarchy.format_tree("  ");

        assert!(tree.contains("[0] Hips"));
        assert!(tree.contains("  [1] Spine"));
        assert!(tree.contains("    [2] Head"));
    }
}
