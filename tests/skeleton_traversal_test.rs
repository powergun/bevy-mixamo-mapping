//! Integration tests for skeleton hierarchy traversal
//!
//! Tests the fbx_parser::traversal module against real Mixamo FBX files.
//! Test files: testdata/Sprint.fbx, testdata/fbx/Idle.fbx

use bevy_mixamo_2d::fbx_parser::{extract_skeleton, traverse_skeleton_hierarchy, FbxData};
use std::path::Path;

const SPRINT_FBX_PATH: &str = "testdata/Sprint.fbx";
const IDLE_FBX_PATH: &str = "testdata/fbx/Idle.fbx";

// ═══════════════════════════════════════════════════════════════════════════════
// SPRINT.FBX TRAVERSAL TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_traverse_sprint_skeleton() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // Sprint skeleton should have 65 bones
    assert_eq!(hierarchy.bone_count, 65, "Sprint skeleton should have 65 bones");

    // Root should be mixamorig:Hips
    assert_eq!(hierarchy.root_name, "mixamorig:Hips", "Root should be Hips");

    // Max depth should be 11 (Hips -> LeftUpLeg -> LeftLeg -> LeftFoot -> LeftToeBase -> LeftToe_End)
    // or finger chains
    assert_eq!(hierarchy.max_depth, 11, "Max depth should be 11");
}

#[test]
fn test_sprint_traversal_order() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // First bone should be root (depth 0)
    assert_eq!(hierarchy.bones[0].depth, 0, "First bone should be at depth 0");
    assert!(
        hierarchy.bones[0].name.contains("Hips"),
        "First bone should be Hips"
    );

    // Verify Spine comes after Hips (direct child)
    let spine_pos = hierarchy
        .bones
        .iter()
        .position(|b| b.name.contains("Spine") && !b.name.contains("Spine1") && !b.name.contains("Spine2"))
        .expect("Should find Spine bone");

    assert!(spine_pos > 0, "Spine should come after Hips");
    assert_eq!(hierarchy.bones[spine_pos].depth, 1, "Spine should be at depth 1");
}

#[test]
fn test_sprint_skeleton_structure() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // Verify key bones are present
    let bone_names: Vec<&str> = hierarchy.bones.iter().map(|b| b.name.as_str()).collect();

    let required_bones = [
        "mixamorig:Hips",
        "mixamorig:Spine",
        "mixamorig:Head",
        "mixamorig:LeftArm",
        "mixamorig:RightArm",
        "mixamorig:LeftLeg",
        "mixamorig:RightLeg",
        "mixamorig:LeftHand",
        "mixamorig:RightHand",
        "mixamorig:LeftFoot",
        "mixamorig:RightFoot",
    ];

    for required in &required_bones {
        assert!(
            bone_names.contains(required),
            "Missing required bone: {}",
            required
        );
    }
}

#[test]
fn test_sprint_format_tree() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);
    let tree = hierarchy.format_tree("  ");

    // Root should have no indentation
    assert!(tree.contains("[0] mixamorig:Hips"), "Tree should contain root");

    // Spine should be indented once
    assert!(
        tree.contains("  [1] mixamorig:Spine"),
        "Tree should show Spine with one level of indentation"
    );

    // Head should be deeply indented (child of Neck -> Spine2 -> Spine1 -> Spine -> Hips)
    assert!(tree.contains("mixamorig:Head"), "Tree should contain Head");
}

#[test]
fn test_sprint_format_with_summary() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);
    let summary = hierarchy.format_with_summary("  ");

    // Summary should contain key information
    assert!(summary.contains("Bones: 65"), "Summary should show bone count");
    assert!(
        summary.contains("Root: mixamorig:Hips"),
        "Summary should show root name"
    );
    assert!(summary.contains("Max depth: 11"), "Summary should show max depth");
    assert!(summary.contains("Hierarchy:"), "Summary should include hierarchy header");
}

// ═══════════════════════════════════════════════════════════════════════════════
// IDLE.FBX TRAVERSAL TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_traverse_idle_skeleton() {
    let path = Path::new(IDLE_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Idle.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // Idle skeleton should also have 65 bones (same Mixamo character)
    assert_eq!(hierarchy.bone_count, 65, "Idle skeleton should have 65 bones");

    // Root should be mixamorig:Hips
    assert_eq!(hierarchy.root_name, "mixamorig:Hips", "Root should be Hips");

    // Max depth should be 11 (same skeleton structure as Sprint)
    assert_eq!(hierarchy.max_depth, 11, "Max depth should be 11");
}

#[test]
fn test_idle_skeleton_matches_sprint() {
    // Load both skeletons
    let sprint_data = FbxData::load(Path::new(SPRINT_FBX_PATH)).expect("Failed to load Sprint.fbx");
    let sprint_nodes = extract_skeleton(&sprint_data).expect("Failed to extract Sprint skeleton");
    let sprint_hierarchy = traverse_skeleton_hierarchy(&sprint_nodes);

    let idle_data = FbxData::load(Path::new(IDLE_FBX_PATH)).expect("Failed to load Idle.fbx");
    let idle_nodes = extract_skeleton(&idle_data).expect("Failed to extract Idle skeleton");
    let idle_hierarchy = traverse_skeleton_hierarchy(&idle_nodes);

    // Both skeletons should have the same structure
    assert_eq!(
        sprint_hierarchy.bone_count, idle_hierarchy.bone_count,
        "Both skeletons should have same bone count"
    );

    // Both should have the same bone names in the same order
    for (sprint_bone, idle_bone) in sprint_hierarchy.bones.iter().zip(idle_hierarchy.bones.iter()) {
        assert_eq!(
            sprint_bone.name, idle_bone.name,
            "Bone names should match: {} vs {}",
            sprint_bone.name, idle_bone.name
        );
        assert_eq!(
            sprint_bone.depth, idle_bone.depth,
            "Bone depths should match for {}",
            sprint_bone.name
        );
    }
}

#[test]
fn test_idle_format_tree() {
    let path = Path::new(IDLE_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Idle.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);
    let tree = hierarchy.format_tree("    "); // 4-space indent

    // Verify tree structure with 4-space indent
    assert!(tree.contains("[0] mixamorig:Hips"), "Tree should contain root");
    assert!(
        tree.contains("    [1] mixamorig:Spine"),
        "Tree should show Spine with 4-space indentation"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CROSS-SKELETON TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_all_bones_have_unique_indices() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // All bones should have unique indices
    let mut seen_indices = std::collections::HashSet::new();
    for bone in &hierarchy.bones {
        assert!(
            seen_indices.insert(bone.index),
            "Duplicate bone index found: {}",
            bone.index
        );
    }
}

#[test]
fn test_depth_monotonically_reasonable() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // Depth should never exceed max_depth
    for bone in &hierarchy.bones {
        assert!(
            bone.depth <= hierarchy.max_depth,
            "Bone {} has depth {} exceeding max_depth {}",
            bone.name,
            bone.depth,
            hierarchy.max_depth
        );
    }
}

#[test]
fn test_finger_bones_deeply_nested() {
    let path = Path::new(SPRINT_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load Sprint.fbx");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let hierarchy = traverse_skeleton_hierarchy(&nodes);

    // Finger bones should be deeply nested (at least depth 8)
    // Hips -> Spine -> Spine1 -> Spine2 -> Shoulder -> Arm -> ForeArm -> Hand -> Finger
    let finger_bones: Vec<_> = hierarchy
        .bones
        .iter()
        .filter(|b| {
            let name = b.name.to_lowercase();
            name.contains("thumb") || name.contains("index") || name.contains("middle")
                || name.contains("ring") || name.contains("pinky")
        })
        .collect();

    assert!(!finger_bones.is_empty(), "Should have finger bones");

    for finger in &finger_bones {
        assert!(
            finger.depth >= 8,
            "Finger bone {} should be at least depth 8, got {}",
            finger.name,
            finger.depth
        );
    }
}
