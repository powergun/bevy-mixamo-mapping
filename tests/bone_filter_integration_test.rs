//! Integration tests for bone filtering with real Mixamo FBX files
//!
//! Tests the fbx_parser::filter module against real Mixamo FBX files.
//! Test files: testdata/Sprint.fbx, testdata/fbx/Idle.fbx

use bevy_mixamo_2d::fbx_parser::{extract_skeleton, BoneFilter, FbxData};
use std::path::Path;

const SPRINT_FBX_PATH: &str = "testdata/Sprint.fbx";
const IDLE_FBX_PATH: &str = "testdata/fbx/Idle.fbx";

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

fn load_sprint_skeleton() -> Vec<bevy_mixamo_2d::gltf_parser::GltfNode3D> {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    extract_skeleton(&data).expect("Failed to extract skeleton")
}

fn load_idle_skeleton() -> Vec<bevy_mixamo_2d::gltf_parser::GltfNode3D> {
    let path = Path::new(IDLE_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Idle.fbx");
    extract_skeleton(&data).expect("Failed to extract skeleton")
}

// ═══════════════════════════════════════════════════════════════════════════════
// NO FILTER (BASELINE)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_no_filter_keeps_all_bones() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::new();

    let filtered = filter.filter_nodes(&nodes);

    assert_eq!(filtered.len(), 65, "Should keep all 65 Mixamo bones");
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCLUDE FINGER BONES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_exclude_all_finger_bones() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(
        &[],
        &[
            "Thumb".to_string(),
            "Index".to_string(),
            "Middle".to_string(),
            "Ring".to_string(),
            "Pinky".to_string(),
        ],
    )
    .unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Each hand has 5 fingers x 4 bones = 20 bones per hand, 40 total
    // 65 - 40 = 25 bones
    assert_eq!(filtered.len(), 25, "Should have 25 bones after excluding fingers");

    // Verify no finger bones remain
    let finger_names = ["Thumb", "Index", "Middle", "Ring", "Pinky"];
    for node in &filtered {
        for finger in &finger_names {
            assert!(
                !node.name.contains(finger),
                "Bone '{}' should not contain '{}'",
                node.name,
                finger
            );
        }
    }

    // Verify hands remain
    let hand_bones: Vec<_> = filtered.iter().filter(|n| n.name.contains("Hand")).collect();
    assert_eq!(hand_bones.len(), 2, "Should have 2 hand bones (left and right)");
}

#[test]
fn test_exclude_left_hand_and_descendants() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(&[], &["LeftHand".to_string()]).unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // LeftHand has 20 finger bones + 1 hand = 21 bones excluded
    // 65 - 21 = 44 bones
    assert_eq!(filtered.len(), 44, "Should have 44 bones after excluding left hand");

    // Verify no left hand or finger bones remain
    for node in &filtered {
        assert!(
            !node.name.contains("LeftHand"),
            "Bone '{}' should not contain 'LeftHand'",
            node.name
        );
    }

    // RightHand should still exist
    let right_hand = filtered.iter().find(|n| n.name.contains("RightHand"));
    assert!(right_hand.is_some(), "RightHand should remain");
}

#[test]
fn test_exclude_toes() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(&[], &["Toe".to_string()]).unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Each foot has 2 toe bones (ToeBase, Toe_End), 4 total
    // 65 - 4 = 61 bones
    assert_eq!(filtered.len(), 61, "Should have 61 bones after excluding toes");

    // Verify no toe bones remain
    for node in &filtered {
        assert!(
            !node.name.contains("Toe"),
            "Bone '{}' should not contain 'Toe'",
            node.name
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SELECT SPECIFIC BONES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_select_spine_chain() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(
        &[r"Spine\d*".to_string()],
        &[],
    )
    .unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Should have: Hips (root), Spine, Spine1, Spine2 = 4 bones
    assert_eq!(filtered.len(), 4, "Should have 4 bones in spine chain");

    let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"mixamorig:Hips"), "Should contain Hips (root)");
    assert!(names.contains(&"mixamorig:Spine"), "Should contain Spine");
    assert!(names.contains(&"mixamorig:Spine1"), "Should contain Spine1");
    assert!(names.contains(&"mixamorig:Spine2"), "Should contain Spine2");
}

#[test]
fn test_select_head() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(&["Head".to_string()], &[]).unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Should have entire ancestor chain: Hips -> Spine -> Spine1 -> Spine2 -> Neck -> Head -> HeadTop_End
    // = 7 bones
    assert_eq!(filtered.len(), 7, "Should have 7 bones for head chain");

    let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"mixamorig:Hips"), "Should contain Hips");
    assert!(names.contains(&"mixamorig:Head"), "Should contain Head");
    assert!(names.contains(&"mixamorig:HeadTop_End"), "Should contain HeadTop_End");
}

#[test]
fn test_select_upper_body_only() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(
        &[
            "Spine".to_string(),
            "Neck".to_string(),
            "Head".to_string(),
            "Shoulder".to_string(),
            "Arm".to_string(),
        ],
        &[
            "Hand".to_string(), // Exclude hands and fingers
        ],
    )
    .unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Verify no leg bones
    for node in &filtered {
        assert!(
            !node.name.contains("Leg") && !node.name.contains("Foot"),
            "Bone '{}' should not be a leg bone",
            node.name
        );
    }

    // Verify no hand/finger bones
    for node in &filtered {
        assert!(
            !node.name.contains("Hand"),
            "Bone '{}' should not be a hand bone",
            node.name
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMBINED SELECT AND EXCLUDE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_select_arms_exclude_fingers() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(
        &[
            "Shoulder".to_string(),
            "Arm".to_string(),
            "Hand".to_string(),
        ],
        &[
            "Thumb".to_string(),
            "Index".to_string(),
            "Middle".to_string(),
            "Ring".to_string(),
            "Pinky".to_string(),
        ],
    )
    .unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Should have: Hips (root), Spine chain (4), Shoulder, Arm, ForeArm, Hand x2 sides
    // But wait, we selected Shoulder/Arm/Hand which matches Left/Right variants
    // Let's verify we have the arm bones but no fingers

    // Check hands exist
    let hand_bones: Vec<_> = filtered.iter().filter(|n| n.name.contains("Hand")).collect();
    assert_eq!(hand_bones.len(), 2, "Should have 2 hand bones");

    // Check no finger bones
    for node in &filtered {
        let finger_names = ["Thumb", "Index", "Middle", "Ring", "Pinky"];
        for finger in &finger_names {
            assert!(
                !node.name.contains(finger),
                "Should not have finger bone '{}'",
                node.name
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARENT INDEX VALIDATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_filtered_skeleton_has_valid_parent_indices() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(
        &[],
        &["Thumb".to_string(), "Index".to_string()],
    )
    .unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Verify all parent indices are valid
    for (idx, node) in filtered.iter().enumerate() {
        if let Some(parent_idx) = node.parent_index {
            assert!(
                parent_idx < idx,
                "Parent index {} should be less than child index {} for bone '{}'",
                parent_idx,
                idx,
                node.name
            );
            assert!(
                parent_idx < filtered.len(),
                "Parent index {} should be less than total bones {} for bone '{}'",
                parent_idx,
                filtered.len(),
                node.name
            );
        }
    }

    // Verify root has no parent
    assert!(
        filtered[0].parent_index.is_none(),
        "Root bone should have no parent"
    );
}

#[test]
fn test_filtered_skeleton_maintains_hierarchy() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(&["Head".to_string()], &[]).unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Verify the chain is connected: Hips -> Spine -> Spine1 -> Spine2 -> Neck -> Head -> HeadTop_End
    let names: Vec<&str> = filtered.iter().map(|n| n.name.as_str()).collect();

    // Find indices
    let hips_idx = names.iter().position(|n| *n == "mixamorig:Hips").unwrap();
    let spine_idx = names.iter().position(|n| *n == "mixamorig:Spine").unwrap();
    let head_idx = names.iter().position(|n| *n == "mixamorig:Head").unwrap();

    // Verify topological order
    assert!(hips_idx < spine_idx, "Hips should come before Spine");
    assert!(spine_idx < head_idx, "Spine should come before Head");

    // Verify Spine's parent is Hips
    assert_eq!(
        filtered[spine_idx].parent_index,
        Some(hips_idx),
        "Spine's parent should be Hips"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CROSS-FILE CONSISTENCY
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_same_filter_on_different_files() {
    let sprint_nodes = load_sprint_skeleton();
    let idle_nodes = load_idle_skeleton();

    let filter = BoneFilter::from_patterns(
        &[],
        &["Thumb".to_string(), "Index".to_string()],
    )
    .unwrap();

    let sprint_filtered = filter.filter_nodes(&sprint_nodes);
    let idle_filtered = filter.filter_nodes(&idle_nodes);

    // Both should have the same number of bones after filtering
    assert_eq!(
        sprint_filtered.len(),
        idle_filtered.len(),
        "Same filter should produce same bone count on same character"
    );

    // Both should have the same bone names
    let sprint_names: Vec<&str> = sprint_filtered.iter().map(|n| n.name.as_str()).collect();
    let idle_names: Vec<&str> = idle_filtered.iter().map(|n| n.name.as_str()).collect();

    assert_eq!(
        sprint_names, idle_names,
        "Same filter should produce same bone names on same character"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// REGEX PATTERN TESTS WITH REAL DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_regex_select_left_side_only() {
    let nodes = load_sprint_skeleton();
    let filter = BoneFilter::from_patterns(&[r"^mixamorig:Left".to_string()], &[]).unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // All bones (except Hips which is root) should start with "mixamorig:Left" or be ancestors
    for node in &filtered {
        let is_left = node.name.starts_with("mixamorig:Left");
        let is_ancestor = node.name.contains("Hips")
            || node.name.contains("Spine")
            || node.name.contains("Neck")
            || node.name.contains("Shoulder");

        assert!(
            is_left || is_ancestor,
            "Bone '{}' should be left-side or ancestor",
            node.name
        );
    }
}

#[test]
fn test_regex_number_suffix() {
    let nodes = load_sprint_skeleton();
    // Select only bones ending in a number
    let filter = BoneFilter::from_patterns(&[r"\d$".to_string()], &[]).unwrap();

    let filtered = filter.filter_nodes(&nodes);

    // Verify we got Spine1, Spine2, Thumb1, Thumb2, etc.
    let numbered_bones: Vec<_> = filtered
        .iter()
        .filter(|n| n.name.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false))
        .collect();

    assert!(
        !numbered_bones.is_empty(),
        "Should have bones ending in numbers"
    );
}
