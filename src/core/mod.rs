pub mod animation;
pub mod bone;
pub mod compatibility;
pub mod keyframe;
pub mod skeleton;

// Re-export main types for convenience
pub use animation::{Animation2D, RootMotion, SampledPose};
pub use bone::{Bone2D, PrimitiveShape, Transform2D};
pub use compatibility::{check_compatibility, check_strict_compatibility};
pub use keyframe::{BoneTimeline, Interpolation, Keyframe};
pub use skeleton::Skeleton2D;
