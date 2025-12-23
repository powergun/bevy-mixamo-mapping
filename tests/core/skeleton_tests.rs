use bevy_mixamo_2d::core::*;
use bevy_mixamo_2d::error::ValidationError;
use glam::Vec2;

// ═══════════════════════════════════════════════════════════════════════════════
// VALID SKELETON CONSTRUCTION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_valid_minimal_skeleton() {
    let bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("minimal", bones);
    assert!(skeleton.is_ok());
    assert_eq!(skeleton.unwrap().bone_count(), 1);
}

#[test]
fn test_valid_two_bone_skeleton() {
    let bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("child", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("two_bone", bones);
    assert!(skeleton.is_ok());
}

#[test]
fn test_valid_humanoid_skeleton() {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::Circle { radius: 0.15 }),
        Bone2D::new("spine", Some(0), 0.3, Transform2D::default(), PrimitiveShape::Rectangle { width: 0.2, height: 0.3 }),
        Bone2D::new("head", Some(1), 0.1, Transform2D::default(), PrimitiveShape::Circle { radius: 0.1 }),
        Bone2D::new("leg_left", Some(0), 0.4, Transform2D::default(), PrimitiveShape::Rectangle { width: 0.1, height: 0.4 }),
        Bone2D::new("leg_right", Some(0), 0.4, Transform2D::default(), PrimitiveShape::Rectangle { width: 0.1, height: 0.4 }),
    ];
    let skeleton = Skeleton2D::new("humanoid", bones);
    assert!(skeleton.is_ok());
    let s = skeleton.unwrap();
    assert_eq!(s.bone_count(), 5);
    assert_eq!(s.root_bone().name, "hips");
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: EMPTY SKELETON
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_skeleton_rejected() {
    let result = Skeleton2D::new("empty", vec![]);
    assert!(matches!(result, Err(ValidationError::EmptyBoneList)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: DUPLICATE BONE NAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_duplicate_bone_names_rejected() {
    let bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("arm", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("arm", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None), // duplicate!
    ];
    let result = Skeleton2D::new("duplicate", bones);
    assert!(matches!(
        result,
        Err(ValidationError::DuplicateBoneName { name, first: 1, second: 2 }) if name == "arm"
    ));
}

#[test]
fn test_duplicate_root_name_rejected() {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("hips", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None), // same name as root
    ];
    let result = Skeleton2D::new("duplicate_root", bones);
    assert!(matches!(
        result,
        Err(ValidationError::DuplicateBoneName { name, first: 0, second: 1 }) if name == "hips"
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: MULTIPLE ROOT BONES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_roots_rejected() {
    let bones = vec![
        Bone2D::new("root1", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("root2", None, 0.0, Transform2D::default(), PrimitiveShape::None), // second root!
    ];
    let result = Skeleton2D::new("multi_root", bones);
    assert!(matches!(
        result,
        Err(ValidationError::MultipleRootBones { first, second })
            if first == "root1" && second == "root2"
    ));
}

#[test]
fn test_no_root_rejected() {
    let bones = vec![
        Bone2D::new("orphan1", Some(1), 1.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("orphan2", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let result = Skeleton2D::new("no_root", bones);
    // This will fail either with NoRootBone or with topological order violation
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID PARENT INDICES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_parent_index_out_of_bounds_rejected() {
    let bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("child", Some(99), 1.0, Transform2D::default(), PrimitiveShape::None), // invalid index
    ];
    let result = Skeleton2D::new("bad_parent", bones);
    assert!(matches!(
        result,
        Err(ValidationError::InvalidParentIndex { name, index: 1, parent_index: 99 }) if name == "child"
    ));
}

#[test]
fn test_parent_after_child_rejected() {
    let bones = vec![
        Bone2D::new("child", Some(1), 1.0, Transform2D::default(), PrimitiveShape::None), // parent index 1, but we're at index 0
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let result = Skeleton2D::new("bad_order", bones);
    assert!(matches!(
        result,
        Err(ValidationError::ParentAfterChild { name, index: 0, parent_index: 1 }) if name == "child"
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: CIRCULAR REFERENCES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_self_referential_bone_rejected() {
    // This would be caught by topological order check first
    let bones = vec![
        Bone2D::new("self_ref", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let result = Skeleton2D::new("self_ref", bones);
    assert!(result.is_err()); // Either NoRootBone or ParentAfterChild
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID BONE PROPERTIES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_negative_bone_length_rejected() {
    let bones = vec![
        Bone2D::new("root", None, -1.0, Transform2D::default(), PrimitiveShape::None), // negative length
    ];
    let result = Skeleton2D::new("neg_length", bones);
    assert!(matches!(
        result,
        Err(ValidationError::InvalidBoneLength { name, length }) if name == "root" && length == -1.0
    ));
}

#[test]
fn test_zero_scale_rejected() {
    let transform = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(0.0, 1.0)); // zero x scale
    let bones = vec![
        Bone2D::new("root", None, 1.0, transform, PrimitiveShape::None),
    ];
    let result = Skeleton2D::new("zero_scale", bones);
    assert!(matches!(
        result,
        Err(ValidationError::InvalidBoneScale { name, x: 0.0, .. }) if name == "root"
    ));
}

#[test]
fn test_negative_scale_rejected() {
    let transform = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(-1.0, 1.0)); // negative scale
    let bones = vec![
        Bone2D::new("root", None, 1.0, transform, PrimitiveShape::None),
    ];
    let result = Skeleton2D::new("neg_scale", bones);
    assert!(matches!(
        result,
        Err(ValidationError::InvalidBoneScale { .. })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER METHOD TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_get_bone_by_name() {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("spine", Some(0), 0.3, Transform2D::default(), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("test", bones).unwrap();

    let (idx, bone) = skeleton.get_bone_by_name("spine").unwrap();
    assert_eq!(idx, 1);
    assert_eq!(bone.name, "spine");

    assert!(skeleton.get_bone_by_name("nonexistent").is_none());
}

#[test]
fn test_children_of() {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("spine", Some(0), 0.3, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("leg_left", Some(0), 0.4, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("leg_right", Some(0), 0.4, Transform2D::default(), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("test", bones).unwrap();

    let children = skeleton.children_of(0);
    assert_eq!(children, vec![1, 2, 3]);

    let spine_children = skeleton.children_of(1);
    assert!(spine_children.is_empty());
}
