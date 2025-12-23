//! Integration tests for GLTF parsing with real Mixamo files
//!
//! Test file: testdata/Sprint.fbx.glb
//! Note: This GLB was converted from FBX and may not contain animation data.
//! The skeleton hierarchy is present and valid.

use bevy_mixamo_2d::gltf_parser::{extract_animations, extract_skeleton, GltfData};
use bevy_mixamo_2d::projection::{project_skeleton, ProjectionConfig};
use std::path::Path;

const TEST_GLB_PATH: &str = "testdata/Sprint.fbx.glb";

// ═══════════════════════════════════════════════════════════════════════════════
// GLTF LOADING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_load_mixamo_glb_no_panic() {
    let path = Path::new(TEST_GLB_PATH);
    assert!(path.exists(), "Test file not found: {}", TEST_GLB_PATH);

    let data = GltfData::load(path).expect("Failed to load GLB file");
    assert!(data.document.nodes().count() > 0, "No nodes in document");
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON EXTRACTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_extract_skeleton_from_mixamo() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");

    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    // Should have many bones (Mixamo humanoid has 50+ bones)
    assert!(
        nodes.len() > 50,
        "Expected at least 50 bones for Mixamo humanoid, got {}",
        nodes.len()
    );

    println!("Extracted {} bones from Mixamo skeleton", nodes.len());

    // Root bone should have no parent
    let root = &nodes[0];
    assert!(
        root.parent_index.is_none(),
        "First bone should be root (no parent)"
    );

    // Root should be named "Hips" (Mixamo convention)
    let root_name = root.name.to_lowercase();
    assert!(
        root_name.contains("hips"),
        "Root bone should be Hips, got: {}",
        root.name
    );

    // Collect unique bone names (some may be duplicated in the export)
    let bone_names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();

    // Verify essential Mixamo bones are present
    let required_bones = ["Spine", "Head", "Arm", "Leg", "Hand", "Foot"];
    for required in &required_bones {
        let found = bone_names
            .iter()
            .any(|n| n.to_lowercase().contains(&required.to_lowercase()));
        assert!(
            found,
            "Missing required bone containing '{}'. Found: {:?}",
            required,
            bone_names.iter().take(20).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_skeleton_topological_order() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    // Verify topological order: parent index < child index
    for (idx, node) in nodes.iter().enumerate() {
        if let Some(parent_idx) = node.parent_index {
            assert!(
                parent_idx < idx,
                "Bone '{}' at index {} has parent at index {} (violates topological order)",
                node.name,
                idx,
                parent_idx
            );
        }
    }

    println!(
        "Verified topological order for {} bones",
        nodes.len()
    );
}

#[test]
fn test_skeleton_hierarchy_structure() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    // Count bones at each level
    let root_count = nodes.iter().filter(|n| n.parent_index.is_none()).count();
    assert_eq!(root_count, 1, "Should have exactly one root bone");

    // Count direct children of root
    let root_children: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.parent_index == Some(0))
        .collect();

    println!("Root '{}' has {} direct children:", nodes[0].name, root_children.len());
    for (idx, child) in &root_children {
        println!("  [{}] {}", idx, child.name);
    }

    // Mixamo Hips should have Spine and both legs as children (at least)
    assert!(
        root_children.len() >= 3,
        "Root should have at least 3 children (spine, left leg, right leg), got {}",
        root_children.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2D PROJECTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_project_skeleton_produces_valid_2d() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let config = ProjectionConfig::default();
    let skeleton_2d = project_skeleton(&nodes, "mixamo_sprint", &config);

    // Should produce valid skeleton that passes validation
    let skeleton = skeleton_2d.expect("Projected skeleton should be valid");

    println!(
        "2D Skeleton '{}' with {} bones",
        skeleton.name,
        skeleton.bone_count()
    );

    // Verify skeleton passes all validations
    skeleton.validate().expect("Skeleton should pass validation");

    // Bone count should match
    assert_eq!(
        skeleton.bone_count(),
        nodes.len(),
        "2D skeleton bone count should match 3D"
    );
}

#[test]
fn test_2d_skeleton_has_valid_transforms() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let config = ProjectionConfig::default();
    let skeleton = project_skeleton(&nodes, "test", &config).expect("Failed to project skeleton");

    // Check that transforms are reasonable (not NaN, not infinite)
    for (idx, bone) in skeleton.bones.iter().enumerate() {
        assert!(
            bone.setup.translation.is_finite(),
            "Bone {} '{}' has non-finite translation",
            idx,
            bone.name
        );
        assert!(
            bone.setup.rotation.is_finite(),
            "Bone {} '{}' has non-finite rotation",
            idx,
            bone.name
        );
        assert!(
            bone.setup.scale.is_finite(),
            "Bone {} '{}' has non-finite scale",
            idx,
            bone.name
        );
        assert!(
            bone.length.is_finite() && bone.length >= 0.0,
            "Bone {} '{}' has invalid length: {}",
            idx,
            bone.name,
            bone.length
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION TESTS (may be skipped if GLB has no animations)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_extraction_handles_no_animations() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");

    let result = extract_animations(&data);

    // This GLB may not have animations (FBX conversion issue)
    // The test should not panic either way
    match result {
        Ok(animations) => {
            println!("Found {} animations", animations.len());
            for anim in &animations {
                println!(
                    "  '{}': {:.2}s, {} channels",
                    anim.name,
                    anim.duration,
                    anim.channels.len()
                );

                // If animations exist, verify they have content
                assert!(anim.duration > 0.0, "Animation duration should be positive");
                assert!(
                    anim.channels.len() > 0,
                    "Animation should have at least one channel"
                );
            }
        }
        Err(bevy_mixamo_2d::Mixamo2dError::NoAnimation) => {
            println!("GLB has no embedded animations (this is expected for some FBX conversions)");
            // This is acceptable - not all GLBs have animations
        }
        Err(e) => {
            panic!("Unexpected error extracting animations: {}", e);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BONE NAMING TESTS (verify Mixamo conventions)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_mixamo_bone_naming_convention() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    // Check for Mixamo naming pattern (mixamorig prefix)
    let mixamo_bones: Vec<_> = nodes
        .iter()
        .filter(|n| n.name.starts_with("mixamorig"))
        .collect();

    println!(
        "{}/{} bones have 'mixamorig' prefix",
        mixamo_bones.len(),
        nodes.len()
    );

    // Most bones should have the mixamorig prefix
    let ratio = mixamo_bones.len() as f32 / nodes.len() as f32;
    assert!(
        ratio > 0.5,
        "Expected most bones to have 'mixamorig' prefix, got {:.1}%",
        ratio * 100.0
    );

    // Document the actual naming convention found
    println!("\nBone naming examples:");
    for node in nodes.iter().take(10) {
        println!("  {}", node.name);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HIERARCHY PRINT (for documentation)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_print_skeleton_summary() {
    let path = Path::new(TEST_GLB_PATH);
    let data = GltfData::load(path).expect("Failed to load GLB file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    println!("\n=== Mixamo Skeleton Summary ===");
    println!("Total bones: {}", nodes.len());
    println!("Root: {} (trans: [{:.1}, {:.1}, {:.1}])",
        nodes[0].name,
        nodes[0].translation.x,
        nodes[0].translation.y,
        nodes[0].translation.z
    );

    // Count bones by category
    let spine_bones = nodes.iter().filter(|n| n.name.to_lowercase().contains("spine")).count();
    let arm_bones = nodes.iter().filter(|n| n.name.to_lowercase().contains("arm")).count();
    let leg_bones = nodes.iter().filter(|n| n.name.to_lowercase().contains("leg")).count();
    let hand_bones = nodes.iter().filter(|n| n.name.to_lowercase().contains("hand")).count();
    let finger_bones = nodes.iter().filter(|n| {
        let name = n.name.to_lowercase();
        name.contains("thumb") || name.contains("index") || name.contains("middle")
            || name.contains("ring") || name.contains("pinky")
    }).count();

    println!("\nBone categories:");
    println!("  Spine: {}", spine_bones);
    println!("  Arms: {}", arm_bones);
    println!("  Legs: {}", leg_bones);
    println!("  Hands: {}", hand_bones);
    println!("  Fingers: {}", finger_bones);

    // Verify skeleton is complete
    assert!(spine_bones >= 2, "Should have at least 2 spine bones");
    assert!(arm_bones >= 4, "Should have at least 4 arm bones (2 per side)");
    assert!(leg_bones >= 4, "Should have at least 4 leg bones (2 per side)");
}
