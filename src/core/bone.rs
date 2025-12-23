use glam::Vec2;
use serde::{Deserialize, Serialize};

/// 2D transform for a bone
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// Position relative to parent
    pub translation: Vec2,
    /// Rotation in radians (around Z-axis), normalized to [-π, π]
    pub rotation: f32,
    /// Scale factors (x for foreshortening, y typically 1.0)
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    /// Create a new Transform2D, normalizing rotation to [-π, π]
    pub fn new(translation: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            translation,
            rotation: normalize_angle(rotation),
            scale,
        }
    }

    /// Check if scale values are valid (both > 0)
    pub fn is_scale_valid(&self) -> bool {
        self.scale.x > 0.0 && self.scale.y > 0.0
    }
}

/// Normalize angle to [-π, π]
fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle % std::f32::consts::TAU;
    if a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    } else if a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

/// Primitive shape for bone visualization and collision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Triangle { base: f32, height: f32 },
    None,
}

impl PrimitiveShape {
    /// Check if shape dimensions are valid
    pub fn is_valid(&self) -> bool {
        match self {
            PrimitiveShape::Circle { radius } => *radius > 0.0,
            PrimitiveShape::Rectangle { width, height } => *width > 0.0 && *height > 0.0,
            PrimitiveShape::Triangle { base, height } => *base > 0.0 && *height > 0.0,
            PrimitiveShape::None => true,
        }
    }
}

/// A single bone in the 2D skeleton
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bone2D {
    /// Bone name (must be unique within skeleton)
    pub name: String,
    /// Parent bone index (None for root bone)
    pub parent: Option<usize>,
    /// Rest-pose length in 2D space (must be >= 0)
    pub length: f32,
    /// Setup pose transform
    pub setup: Transform2D,
    /// Primitive shape for visualization and collision
    pub shape: PrimitiveShape,
}

impl Bone2D {
    /// Create a new bone with validation
    pub fn new(
        name: impl Into<String>,
        parent: Option<usize>,
        length: f32,
        setup: Transform2D,
        shape: PrimitiveShape,
    ) -> Self {
        Self {
            name: name.into(),
            parent,
            length,
            setup,
            shape,
        }
    }

    /// Check if this bone is the root (has no parent)
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}
