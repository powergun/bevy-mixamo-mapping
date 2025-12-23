//! 3D to 2D projection for side-scrolling games
//!
//! Converts Mixamo 3D skeleton and animation data to 2D format suitable for
//! profile-view side-scrolling games.

pub mod root_motion;
pub mod transform;

pub use root_motion::extract_root_motion;
pub use transform::{project_animation, project_skeleton, ProjectionConfig, ViewAxis};
