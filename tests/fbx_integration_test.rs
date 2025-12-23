//! Integration tests for FBX parsing with real Mixamo files
//!
//! Test file: testdata/Sprint.fbx
//! This is a direct Mixamo download with proper bone names and animation data.

use bevy_mixamo_2d::fbx_parser::{extract_animations, extract_skeleton};
use bevy_mixamo_2d::projection::{project_skeleton, ProjectionConfig};
use bevy_mixamo_2d::FbxData;
use std::path::Path;

const TEST_FBX_PATH: &str = "testdata/Sprint.fbx";

// ═══════════════════════════════════════════════════════════════════════════════
// FBX LOADING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_load_mixamo_fbx_no_panic() {
    let path = Path::new(TEST_FBX_PATH);
    assert!(path.exists(), "Test file not found: {}", TEST_FBX_PATH);

    let data = FbxData::load(path).expect("Failed to load FBX file");
    assert!(data.scene.nodes.len() > 0, "No nodes in scene");
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON EXTRACTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_extract_skeleton_from_fbx() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");

    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    // Should have ~65 bones (Mixamo humanoid)
    assert!(
        nodes.len() >= 50,
        "Expected at least 50 bones for Mixamo humanoid, got {}",
        nodes.len()
    );

    println!("Extracted {} bones from FBX", nodes.len());

    // Root bone should have no parent
    let root = &nodes[0];
    assert!(
        root.parent_index.is_none(),
        "First bone should be root (no parent)"
    );

    // Root should be named "mixamorig:Hips" (Mixamo convention with colon)
    assert!(
        root.name.contains("Hips"),
        "Root bone should be Hips, got: {}",
        root.name
    );

    // Verify essential Mixamo bones are present
    let bone_names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
    let required_bones = ["Spine", "Head", "Arm", "Leg", "Hand", "Foot"];
    for required in &required_bones {
        let found = bone_names.iter().any(|n| n.contains(required));
        assert!(
            found,
            "Missing required bone containing '{}'. Found: {:?}",
            required,
            bone_names.iter().take(20).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_fbx_bone_naming_convention() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    // FBX uses colon separator: "mixamorig:Hips"
    let bones_with_colon: Vec<_> = nodes
        .iter()
        .filter(|n| n.name.contains("mixamorig:"))
        .collect();

    println!(
        "{}/{} bones have 'mixamorig:' prefix (with colon)",
        bones_with_colon.len(),
        nodes.len()
    );

    // Most bones should have the mixamorig: prefix
    let ratio = bones_with_colon.len() as f32 / nodes.len() as f32;
    assert!(
        ratio > 0.9,
        "Expected most bones to have 'mixamorig:' prefix, got {:.1}%",
        ratio * 100.0
    );

    // Document the naming convention
    println!("\nMixamo FBX bone naming examples:");
    for node in nodes.iter().take(10) {
        println!("  {}", node.name);
    }
}

#[test]
fn test_skeleton_topological_order() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");
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

    println!("Verified topological order for {} bones", nodes.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2D PROJECTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_project_fbx_skeleton_to_2d() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    let config = ProjectionConfig::default();
    let skeleton_2d = project_skeleton(&nodes, "mixamo_sprint", &config);

    let skeleton = skeleton_2d.expect("Projected skeleton should be valid");

    println!(
        "2D Skeleton '{}' with {} bones",
        skeleton.name,
        skeleton.bone_count()
    );

    // Skeleton should pass validation
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
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");
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
// ANIMATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_extract_animations_from_fbx() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");

    let animations = extract_animations(&data).expect("Failed to extract animations");

    // Should have at least one animation
    assert!(
        !animations.is_empty(),
        "Expected at least one animation in FBX"
    );

    println!("Found {} animations:", animations.len());
    for anim in &animations {
        println!(
            "  '{}': {:.2}s, {} channels",
            anim.name,
            anim.duration,
            anim.channels.len()
        );

        // Animation should have positive duration
        assert!(
            anim.duration > 0.0,
            "Animation '{}' has invalid duration: {}",
            anim.name,
            anim.duration
        );

        // Animation should have channels
        assert!(
            !anim.channels.is_empty(),
            "Animation '{}' has no channels",
            anim.name
        );
    }
}

#[test]
fn test_animation_has_keyframes() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");

    let animations = extract_animations(&data).expect("Failed to extract animations");

    // Find the main animation (typically "mixamo.com" or similar)
    let main_anim = animations
        .iter()
        .find(|a| a.name.contains("mixamo"))
        .or(animations.first())
        .expect("No animation found");

    println!(
        "Checking animation '{}' with {} channels",
        main_anim.name,
        main_anim.channels.len()
    );

    // Count keyframes
    let total_keyframes: usize = main_anim
        .channels
        .iter()
        .map(|c| c.keyframes.len())
        .sum();

    println!("Total keyframes: {}", total_keyframes);

    // Should have many keyframes for a motion
    assert!(
        total_keyframes > 100,
        "Expected more than 100 keyframes for a motion animation, got {}",
        total_keyframes
    );

    // Check keyframe times are within bounds
    for channel in &main_anim.channels {
        for kf in &channel.keyframes {
            assert!(
                kf.time >= 0.0 && kf.time <= main_anim.duration + 0.01,
                "Keyframe time {} out of bounds [0, {}]",
                kf.time,
                main_anim.duration
            );
        }
    }
}

#[test]
fn test_animation_bone_coverage() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");

    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");
    let animations = extract_animations(&data).expect("Failed to extract animations");

    let main_anim = animations
        .iter()
        .find(|a| a.name.contains("mixamo"))
        .or(animations.first())
        .expect("No animation found");

    // Collect animated node indices
    let animated_nodes: std::collections::HashSet<usize> = main_anim
        .channels
        .iter()
        .map(|c| c.node_index)
        .collect();

    println!(
        "Animation covers {} of {} bones",
        animated_nodes.len(),
        nodes.len()
    );

    // Most bones should have animation data
    let coverage = animated_nodes.len() as f32 / nodes.len() as f32;
    assert!(
        coverage > 0.5,
        "Expected more than 50% of bones to be animated, got {:.1}%",
        coverage * 100.0
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON SUMMARY (for documentation)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_print_fbx_skeleton_summary() {
    let path = Path::new(TEST_FBX_PATH);
    let data = FbxData::load(path).expect("Failed to load FBX file");
    let nodes = extract_skeleton(&data).expect("Failed to extract skeleton");

    println!("\n=== Mixamo FBX Skeleton Summary ===");
    println!("Total bones: {}", nodes.len());
    println!(
        "Root: {} (trans: [{:.1}, {:.1}, {:.1}])",
        nodes[0].name, nodes[0].translation.x, nodes[0].translation.y, nodes[0].translation.z
    );

    // Count bones by category
    let spine_bones = nodes
        .iter()
        .filter(|n| n.name.to_lowercase().contains("spine"))
        .count();
    let arm_bones = nodes
        .iter()
        .filter(|n| n.name.to_lowercase().contains("arm"))
        .count();
    let leg_bones = nodes
        .iter()
        .filter(|n| n.name.to_lowercase().contains("leg"))
        .count();
    let hand_bones = nodes
        .iter()
        .filter(|n| n.name.to_lowercase().contains("hand"))
        .count();
    let finger_bones = nodes
        .iter()
        .filter(|n| {
            let name = n.name.to_lowercase();
            name.contains("thumb")
                || name.contains("index")
                || name.contains("middle")
                || name.contains("ring")
                || name.contains("pinky")
        })
        .count();

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
