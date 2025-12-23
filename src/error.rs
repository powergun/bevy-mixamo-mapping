use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    // Skeleton validation errors
    #[error("Skeleton has no bones")]
    EmptyBoneList,

    #[error("Duplicate bone name '{name}' found at indices {first} and {second}")]
    DuplicateBoneName { name: String, first: usize, second: usize },

    #[error("Bone '{name}' at index {index} references non-existent parent index {parent_index}")]
    InvalidParentIndex { name: String, index: usize, parent_index: usize },

    #[error("Bone '{name}' at index {index} references parent index {parent_index} which is >= its own index (violates topological order)")]
    ParentAfterChild { name: String, index: usize, parent_index: usize },

    #[error("Circular parent reference detected involving bone '{name}'")]
    CircularParentReference { name: String },

    #[error("Multiple root bones found: '{first}' and '{second}' (only one bone may have parent=None)")]
    MultipleRootBones { first: String, second: String },

    #[error("No root bone found (at least one bone must have parent=None)")]
    NoRootBone,

    #[error("Bone '{name}' has invalid length {length} (must be >= 0)")]
    InvalidBoneLength { name: String, length: f32 },

    #[error("Bone '{name}' has invalid scale ({x}, {y}) (must be > 0)")]
    InvalidBoneScale { name: String, x: f32, y: f32 },

    // Animation validation errors
    #[error("Animation has no timelines")]
    EmptyTimelines,

    #[error("Animation duration {duration} is invalid (must be > 0)")]
    InvalidDuration { duration: f32 },

    #[error("Animation sample rate {sample_rate} is invalid (must be > 0)")]
    InvalidSampleRate { sample_rate: f32 },

    #[error("Timeline references non-existent bone index {bone_index}")]
    TimelineTargetsInvalidBone { bone_index: usize },

    #[error("Keyframe at index {index} has invalid time {time} (must be >= 0 and <= duration {duration})")]
    KeyframeTimeOutOfRange { index: usize, time: f32, duration: f32 },

    #[error("Keyframes are not sorted by time: keyframe {index} has time {time}, but previous has time {prev_time}")]
    KeyframesNotSorted { index: usize, time: f32, prev_time: f32 },

    #[error("Timeline for bone {bone_index} has no rotation keyframes (at least one required)")]
    EmptyRotationKeyframes { bone_index: usize },

    #[error("Root motion deltas count {deltas_count} doesn't match expected frame count {expected_count}")]
    RootMotionDeltasMismatch { deltas_count: usize, expected_count: usize },

    // Compatibility errors
    #[error("Animation targets skeleton '{animation_skeleton}' but was paired with skeleton '{actual_skeleton}'")]
    SkeletonNameMismatch { animation_skeleton: String, actual_skeleton: String },

    #[error("Animation expects {expected} bones but skeleton has {actual} bones")]
    BoneCountMismatch { expected: usize, actual: usize },

    #[error("Animation timeline targets bone index {index} which exceeds skeleton bone count {bone_count}")]
    TimelineBoneIndexOutOfRange { index: usize, bone_count: usize },
}

#[derive(Error, Debug)]
pub enum Mixamo2dError {
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("Failed to parse GLTF file: {0}")]
    GltfParse(#[from] gltf::Error),

    #[error("Failed to parse FBX file: {0}")]
    FbxParse(String),

    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to serialize/deserialize RON: {0}")]
    Ron(#[from] ron::error::SpannedError),

    #[error("No skeleton found in file")]
    NoSkeleton,

    #[error("No animation found in file")]
    NoAnimation,

    #[error("Bone '{0}' not found in skeleton")]
    BoneNotFound(String),

    #[error("Invalid bone mapping configuration: {0}")]
    InvalidMapping(String),
}

pub type Result<T> = std::result::Result<T, Mixamo2dError>;
