//! Export functionality for skeleton and animation data

pub mod ron;

pub use ron::{load_animation, load_animation_for_skeleton, load_skeleton, save_animation, save_skeleton};
