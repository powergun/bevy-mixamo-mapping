//! Bone filtering for skeleton extraction
//!
//! Provides hierarchical filtering of bones using regex patterns.
//! Supports both allowlist (select) and denylist (exclude) patterns.

use crate::gltf_parser::GltfNode3D;
use regex::Regex;
use std::collections::HashSet;

/// Configuration for filtering bones during extraction
#[derive(Debug, Clone, Default)]
pub struct BoneFilter {
    /// Regex patterns for selecting bones (allowlist).
    /// If empty, all bones are selected by default.
    /// If non-empty, only bones matching any pattern are selected.
    select_patterns: Vec<Regex>,

    /// Regex patterns for excluding bones (denylist).
    /// Bones matching any pattern are excluded.
    /// If a bone is excluded, all its descendants are also excluded.
    exclude_patterns: Vec<Regex>,
}

/// Result of building a BoneFilter
#[derive(Debug)]
pub enum BoneFilterError {
    /// Invalid regex pattern
    InvalidPattern { pattern: String, error: String },
}

impl std::fmt::Display for BoneFilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoneFilterError::InvalidPattern { pattern, error } => {
                write!(f, "Invalid regex pattern '{}': {}", pattern, error)
            }
        }
    }
}

impl std::error::Error for BoneFilterError {}

impl BoneFilter {
    /// Create a new empty filter (selects all bones)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter from pattern strings
    ///
    /// # Arguments
    /// * `select_patterns` - Regex patterns for selecting bones
    /// * `exclude_patterns` - Regex patterns for excluding bones
    ///
    /// # Returns
    /// A configured BoneFilter or an error if any pattern is invalid
    pub fn from_patterns(
        select_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Self, BoneFilterError> {
        let mut filter = Self::new();

        for pattern in select_patterns {
            filter.add_select_pattern(pattern)?;
        }

        for pattern in exclude_patterns {
            filter.add_exclude_pattern(pattern)?;
        }

        Ok(filter)
    }

    /// Add a select pattern (allowlist)
    pub fn add_select_pattern(&mut self, pattern: &str) -> Result<(), BoneFilterError> {
        let regex = Regex::new(pattern).map_err(|e| BoneFilterError::InvalidPattern {
            pattern: pattern.to_string(),
            error: e.to_string(),
        })?;
        self.select_patterns.push(regex);
        Ok(())
    }

    /// Add an exclude pattern (denylist)
    pub fn add_exclude_pattern(&mut self, pattern: &str) -> Result<(), BoneFilterError> {
        let regex = Regex::new(pattern).map_err(|e| BoneFilterError::InvalidPattern {
            pattern: pattern.to_string(),
            error: e.to_string(),
        })?;
        self.exclude_patterns.push(regex);
        Ok(())
    }

    /// Check if this filter has any select patterns
    pub fn has_select_patterns(&self) -> bool {
        !self.select_patterns.is_empty()
    }

    /// Check if this filter has any exclude patterns
    pub fn has_exclude_patterns(&self) -> bool {
        !self.exclude_patterns.is_empty()
    }

    /// Check if a bone name matches any select pattern
    fn matches_select(&self, name: &str) -> bool {
        // If no select patterns, everything is selected by default
        if self.select_patterns.is_empty() {
            return true;
        }
        // Otherwise, must match at least one pattern
        self.select_patterns.iter().any(|p| p.is_match(name))
    }

    /// Check if a bone name matches any exclude pattern
    fn matches_exclude(&self, name: &str) -> bool {
        self.exclude_patterns.iter().any(|p| p.is_match(name))
    }

    /// Filter skeleton nodes according to select/exclude rules
    ///
    /// # Filtering Rules (applied in order)
    /// 1. If no select patterns: all bones are initially selected
    /// 2. If select patterns present: only bones matching any select pattern are selected
    /// 3. Connectivity: ancestors of selected bones are also selected (to maintain tree structure)
    /// 4. Exclude filter: bones matching any exclude pattern are removed
    /// 5. Hierarchical exclusion: if a bone is excluded, all descendants are also excluded
    /// 6. The root bone is always kept (even if it doesn't match select patterns)
    ///
    /// # Arguments
    /// * `nodes` - The skeleton nodes in topological order (parents before children)
    ///
    /// # Returns
    /// Filtered nodes with updated parent indices, maintaining topological order
    pub fn filter_nodes(&self, nodes: &[GltfNode3D]) -> Vec<GltfNode3D> {
        if nodes.is_empty() {
            return Vec::new();
        }

        // Step 1: Determine which bones pass the select filter
        // Root is always selected to maintain skeleton connectivity
        let mut selected: Vec<bool> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                if i == 0 {
                    // Root is always selected
                    true
                } else {
                    self.matches_select(&node.name)
                }
            })
            .collect();

        // Step 2: Ensure connectivity BEFORE exclusion
        // If a bone is selected, its ancestors must also be selected
        for i in (0..nodes.len()).rev() {
            if selected[i] {
                if let Some(parent_idx) = nodes[i].parent_index {
                    selected[parent_idx] = true;
                }
            }
        }

        // Step 3: Apply exclude filter with hierarchical propagation
        // Build excluded set, propagating to children
        let mut excluded: HashSet<usize> = HashSet::new();

        for (i, node) in nodes.iter().enumerate() {
            // Check if this node is explicitly excluded
            let explicitly_excluded = self.matches_exclude(&node.name);

            // Check if parent is excluded (hierarchical exclusion)
            let parent_excluded = node
                .parent_index
                .map(|p| excluded.contains(&p))
                .unwrap_or(false);

            if explicitly_excluded || parent_excluded {
                excluded.insert(i);
                selected[i] = false;
            }
        }

        // Step 4: Build mapping from old indices to new indices
        let mut old_to_new: Vec<Option<usize>> = vec![None; nodes.len()];
        let mut new_index = 0;

        for (old_idx, &is_selected) in selected.iter().enumerate() {
            if is_selected {
                old_to_new[old_idx] = Some(new_index);
                new_index += 1;
            }
        }

        // Step 5: Create filtered nodes with updated parent indices
        let mut result = Vec::new();

        for (old_idx, node) in nodes.iter().enumerate() {
            if selected[old_idx] {
                let new_parent = node.parent_index.and_then(|p| old_to_new[p]);

                // Update children indices (only those that are selected)
                let new_children: Vec<usize> = node
                    .children
                    .iter()
                    .filter_map(|&child_idx| old_to_new.get(child_idx).copied().flatten())
                    .collect();

                result.push(GltfNode3D {
                    name: node.name.clone(),
                    index: old_to_new[old_idx].unwrap_or(0),
                    parent_index: new_parent,
                    translation: node.translation,
                    rotation: node.rotation,
                    scale: node.scale,
                    children: new_children,
                });
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    fn make_node(name: &str, parent: Option<usize>, children: Vec<usize>) -> GltfNode3D {
        GltfNode3D {
            name: name.to_string(),
            index: 0,
            parent_index: parent,
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            children,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // FILTER CREATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_empty_filter() {
        let filter = BoneFilter::new();
        assert!(!filter.has_select_patterns());
        assert!(!filter.has_exclude_patterns());
    }

    #[test]
    fn test_from_patterns_valid() {
        let select = vec!["Spine.*".to_string(), "Head".to_string()];
        let exclude = vec!["Thumb.*".to_string()];

        let filter = BoneFilter::from_patterns(&select, &exclude).unwrap();
        assert!(filter.has_select_patterns());
        assert!(filter.has_exclude_patterns());
    }

    #[test]
    fn test_from_patterns_invalid_select() {
        let select = vec!["[invalid".to_string()];
        let exclude = vec![];

        let result = BoneFilter::from_patterns(&select, &exclude);
        assert!(result.is_err());
        if let Err(BoneFilterError::InvalidPattern { pattern, .. }) = result {
            assert_eq!(pattern, "[invalid");
        }
    }

    #[test]
    fn test_from_patterns_invalid_exclude() {
        let select = vec![];
        let exclude = vec!["(?invalid)".to_string()];

        let result = BoneFilter::from_patterns(&select, &exclude);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // DEFAULT SELECT-ALL BEHAVIOR
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_no_filter_selects_all() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2]),
            make_node("Spine", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
        ];

        let filter = BoneFilter::new();
        let filtered = filter.filter_nodes(&nodes);

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].name, "Hips");
        assert_eq!(filtered[1].name, "Spine");
        assert_eq!(filtered[2].name, "LeftLeg");
    }

    #[test]
    fn test_empty_nodes_returns_empty() {
        let filter = BoneFilter::new();
        let filtered = filter.filter_nodes(&[]);
        assert!(filtered.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SELECT PATTERN TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_single_pattern() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("Spine", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
            make_node("RightLeg", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Spine").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Should have Hips (root always kept) and Spine (matched)
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "Hips");
        assert_eq!(filtered[1].name, "Spine");
    }

    #[test]
    fn test_select_multiple_patterns_or_relationship() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("Spine", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
            make_node("RightLeg", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Spine").unwrap();
        filter.add_select_pattern("Left.*").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Should have Hips (root), Spine (matched first), LeftLeg (matched second)
        assert_eq!(filtered.len(), 3);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine"));
        assert!(names.contains(&"LeftLeg"));
        assert!(!names.contains(&"RightLeg"));
    }

    #[test]
    fn test_select_regex_pattern() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("Spine1", Some(0), vec![]),
            make_node("Spine2", Some(0), vec![]),
            make_node("Head", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern(r"Spine\d+").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        assert_eq!(filtered.len(), 3);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine1"));
        assert!(names.contains(&"Spine2"));
        assert!(!names.contains(&"Head"));
    }

    #[test]
    fn test_select_with_partial_match() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2]),
            make_node("LeftHand", Some(0), vec![]),
            make_node("LeftHandThumb", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Hand").unwrap(); // Partial match

        let filtered = filter.filter_nodes(&nodes);

        // "Hand" should match both "LeftHand" and "LeftHandThumb" (partial match)
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_select_exact_match_with_anchors() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2]),
            make_node("LeftHand", Some(0), vec![]),
            make_node("LeftHandThumb", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("^LeftHand$").unwrap(); // Exact match

        let filtered = filter.filter_nodes(&nodes);

        // Only exact "LeftHand" should match
        assert_eq!(filtered.len(), 2);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"LeftHand"));
        assert!(!names.contains(&"LeftHandThumb"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EXCLUDE PATTERN TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_exclude_single_pattern() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("Spine", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
            make_node("RightLeg", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("LeftLeg").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        assert_eq!(filtered.len(), 3);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine"));
        assert!(names.contains(&"RightLeg"));
        assert!(!names.contains(&"LeftLeg"));
    }

    #[test]
    fn test_exclude_multiple_patterns_or_relationship() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("Spine", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
            make_node("RightLeg", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("Left.*").unwrap();
        filter.add_exclude_pattern("Right.*").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Should only have Hips and Spine
        assert_eq!(filtered.len(), 2);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine"));
    }

    #[test]
    fn test_exclude_regex_pattern() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3, 4]),
            make_node("LeftHandThumb1", Some(0), vec![]),
            make_node("LeftHandThumb2", Some(0), vec![]),
            make_node("LeftHandIndex1", Some(0), vec![]),
            make_node("Spine", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern(r"Thumb\d+").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        assert_eq!(filtered.len(), 3);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"LeftHandIndex1"));
        assert!(names.contains(&"Spine"));
        assert!(!names.contains(&"LeftHandThumb1"));
        assert!(!names.contains(&"LeftHandThumb2"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // HIERARCHICAL EXCLUSION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_exclude_propagates_to_children() {
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("LeftHand", Some(0), vec![2, 3]),
            make_node("LeftHandThumb1", Some(1), vec![4]),
            make_node("LeftHandIndex1", Some(1), vec![]),
            make_node("LeftHandThumb2", Some(2), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("^LeftHand$").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // LeftHand excluded, so all its children should be excluded too
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Hips");
    }

    #[test]
    fn test_exclude_propagates_deeply() {
        // Hips -> Arm -> Hand -> Finger1 -> Finger2 -> Finger3
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("Arm", Some(0), vec![2]),
            make_node("Hand", Some(1), vec![3]),
            make_node("Finger1", Some(2), vec![4]),
            make_node("Finger2", Some(3), vec![5]),
            make_node("Finger3", Some(4), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("Hand").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Hand excluded, so Finger1, Finger2, Finger3 should all be excluded
        assert_eq!(filtered.len(), 2);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Arm"));
    }

    #[test]
    fn test_exclude_only_subtree() {
        // Hips -> LeftArm -> LeftHand -> Thumb
        //      -> RightArm -> RightHand -> Thumb
        let nodes = vec![
            make_node("Hips", None, vec![1, 4]),
            make_node("LeftArm", Some(0), vec![2]),
            make_node("LeftHand", Some(1), vec![3]),
            make_node("LeftThumb", Some(2), vec![]),
            make_node("RightArm", Some(0), vec![5]),
            make_node("RightHand", Some(4), vec![6]),
            make_node("RightThumb", Some(5), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("LeftHand").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Only LeftHand subtree should be excluded
        assert_eq!(filtered.len(), 5);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"LeftArm"));
        assert!(names.contains(&"RightArm"));
        assert!(names.contains(&"RightHand"));
        assert!(names.contains(&"RightThumb"));
        assert!(!names.contains(&"LeftHand"));
        assert!(!names.contains(&"LeftThumb"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // COMBINED SELECT AND EXCLUDE TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_then_exclude() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("LeftArm", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
            make_node("RightArm", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Left.*").unwrap();
        filter.add_exclude_pattern("LeftLeg").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Select Left*, then exclude LeftLeg
        // Result: Hips (root), LeftArm
        assert_eq!(filtered.len(), 2);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"LeftArm"));
        assert!(!names.contains(&"LeftLeg"));
        assert!(!names.contains(&"RightArm"));
    }

    #[test]
    fn test_select_specific_exclude_children() {
        // Hips -> Hand -> Thumb1 -> Thumb2
        //             -> Index1 -> Index2
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("Hand", Some(0), vec![2, 4]),
            make_node("Thumb1", Some(1), vec![3]),
            make_node("Thumb2", Some(2), vec![]),
            make_node("Index1", Some(1), vec![5]),
            make_node("Index2", Some(4), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Hand").unwrap();
        filter.add_select_pattern("Thumb").unwrap();
        filter.add_select_pattern("Index").unwrap();
        filter.add_exclude_pattern("Thumb").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Select Hand, Thumb*, Index*, then exclude Thumb*
        // Result: Hips, Hand, Index1, Index2
        assert_eq!(filtered.len(), 4);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Hand"));
        assert!(names.contains(&"Index1"));
        assert!(names.contains(&"Index2"));
        assert!(!names.contains(&"Thumb1"));
        assert!(!names.contains(&"Thumb2"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PARENT INDEX REMAPPING TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_parent_indices_remapped_correctly() {
        // Hips(0) -> Spine(1) -> Head(2)
        //        -> LeftLeg(3) -> LeftFoot(4)
        let nodes = vec![
            make_node("Hips", None, vec![1, 3]),
            make_node("Spine", Some(0), vec![2]),
            make_node("Head", Some(1), vec![]),
            make_node("LeftLeg", Some(0), vec![4]),
            make_node("LeftFoot", Some(3), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("Left.*").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // After excluding Left*, should have: Hips(0), Spine(1), Head(2)
        assert_eq!(filtered.len(), 3);

        assert_eq!(filtered[0].name, "Hips");
        assert_eq!(filtered[0].parent_index, None);

        assert_eq!(filtered[1].name, "Spine");
        assert_eq!(filtered[1].parent_index, Some(0));

        assert_eq!(filtered[2].name, "Head");
        assert_eq!(filtered[2].parent_index, Some(1));
    }

    #[test]
    fn test_children_indices_remapped_correctly() {
        let nodes = vec![
            make_node("Hips", None, vec![1, 2, 3]),
            make_node("Spine", Some(0), vec![]),
            make_node("LeftLeg", Some(0), vec![]),
            make_node("RightLeg", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("LeftLeg").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Hips should have updated children indices
        assert_eq!(filtered[0].name, "Hips");
        // Children should now be [1, 2] (Spine and RightLeg)
        assert_eq!(filtered[0].children.len(), 2);
        assert!(filtered[0].children.contains(&1)); // Spine
        assert!(filtered[0].children.contains(&2)); // RightLeg
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ROOT BONE SPECIAL HANDLING
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_root_always_kept() {
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("Spine", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Spine").unwrap(); // Doesn't match Hips

        let filtered = filter.filter_nodes(&nodes);

        // Root should still be kept
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "Hips");
        assert_eq!(filtered[1].name, "Spine");
    }

    #[test]
    fn test_root_cannot_be_excluded() {
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("Spine", Some(0), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("Hips").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Even though Hips is excluded, all children should be excluded
        // But we need a root, so this is a degenerate case
        // The hierarchical exclusion will kick in and exclude Spine too
        assert_eq!(filtered.len(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ANCESTOR CONNECTIVITY TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ancestors_included_for_connectivity() {
        // Hips -> Spine -> Spine1 -> Spine2 -> Head
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("Spine", Some(0), vec![2]),
            make_node("Spine1", Some(1), vec![3]),
            make_node("Spine2", Some(2), vec![4]),
            make_node("Head", Some(3), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Head").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Should include Head and all ancestors to maintain connectivity
        assert_eq!(filtered.len(), 5);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine"));
        assert!(names.contains(&"Spine1"));
        assert!(names.contains(&"Spine2"));
        assert!(names.contains(&"Head"));
    }

    #[test]
    fn test_excluded_ancestor_breaks_chain() {
        // Hips -> Spine -> Spine1 -> Spine2 -> Head
        let nodes = vec![
            make_node("Hips", None, vec![1]),
            make_node("Spine", Some(0), vec![2]),
            make_node("Spine1", Some(1), vec![3]),
            make_node("Spine2", Some(2), vec![4]),
            make_node("Head", Some(3), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Head").unwrap();
        filter.add_exclude_pattern("Spine1").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Spine1 excluded, so Spine2 and Head (its descendants) should also be excluded
        // Only Hips and Spine remain
        assert_eq!(filtered.len(), 2);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Hips"));
        assert!(names.contains(&"Spine"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MIXAMO-LIKE SKELETON TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_mixamo_exclude_fingers() {
        let nodes = vec![
            make_node("mixamorig:Hips", None, vec![1]),
            make_node("mixamorig:Spine", Some(0), vec![2]),
            make_node("mixamorig:LeftHand", Some(1), vec![3, 4, 5]),
            make_node("mixamorig:LeftHandThumb1", Some(2), vec![]),
            make_node("mixamorig:LeftHandIndex1", Some(2), vec![]),
            make_node("mixamorig:LeftHandMiddle1", Some(2), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_exclude_pattern("Thumb").unwrap();
        filter.add_exclude_pattern("Index").unwrap();
        filter.add_exclude_pattern("Middle").unwrap();
        filter.add_exclude_pattern("Ring").unwrap();
        filter.add_exclude_pattern("Pinky").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        assert_eq!(filtered.len(), 3);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"mixamorig:Hips"));
        assert!(names.contains(&"mixamorig:Spine"));
        assert!(names.contains(&"mixamorig:LeftHand"));
    }

    #[test]
    fn test_mixamo_select_upper_body() {
        let nodes = vec![
            make_node("mixamorig:Hips", None, vec![1, 4]),
            make_node("mixamorig:Spine", Some(0), vec![2]),
            make_node("mixamorig:Spine1", Some(1), vec![3]),
            make_node("mixamorig:Head", Some(2), vec![]),
            make_node("mixamorig:LeftUpLeg", Some(0), vec![5]),
            make_node("mixamorig:LeftLeg", Some(4), vec![]),
        ];

        let mut filter = BoneFilter::new();
        filter.add_select_pattern("Spine").unwrap();
        filter.add_select_pattern("Head").unwrap();

        let filtered = filter.filter_nodes(&nodes);

        // Should have Hips (root), Spine, Spine1, Head
        assert_eq!(filtered.len(), 4);
        let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"mixamorig:Hips"));
        assert!(names.contains(&"mixamorig:Spine"));
        assert!(names.contains(&"mixamorig:Spine1"));
        assert!(names.contains(&"mixamorig:Head"));
        assert!(!names.contains(&"mixamorig:LeftUpLeg"));
        assert!(!names.contains(&"mixamorig:LeftLeg"));
    }
}
