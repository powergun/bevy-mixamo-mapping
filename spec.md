# Coding Plan: Mixamo 3D to 2D Skeletal Animation Converter for Bevy

## Overview

This document specifies the implementation of a Rust CLI application and library crate that converts Mixamo 3D animations (in GLB format) to 2D skeletal animations suitable for profile-view side-scrolling games built with Bevy 0.17+.

### Project Name
`bevy_mixamo_2d`

### Key Deliverables
1. **CLI Application** (`mixamo2d`): Converts GLB files to 2D skeleton and animation RON files
2. **Library Crate** (`bevy_mixamo_2d`): Loads and plays 2D skeletal animations in Bevy games
3. **Documentation**: README.md and technical documentation

---

## Architectural Principle: Data-First Design

**The `Skeleton2D` and `Animation2D` data structures form the core foundation of this library.** All other modules (parsing, projection, serialization, Bevy plugin) are built around these central types.

### Why Data-First?

1. **Single Source of Truth**: All downstream consumers (CLI, Bevy plugin, serialization) work with the same validated data structures
2. **Validation at Construction**: Invalid data is rejected early, preventing runtime errors
3. **Testability**: Core logic can be tested in isolation without I/O or rendering dependencies
4. **Interoperability**: The RON serialization format is a direct representation of these types

### Dependency Graph

```
                    ┌─────────────────────────────────────┐
                    │     CORE DATA LAYER (Phase 1)       │
                    │  Skeleton2D, Animation2D, Bone2D    │
                    │  + Validation + Invariant Checks    │
                    └─────────────────────────────────────┘
                                    │
          ┌─────────────────────────┼─────────────────────────┐
          │                         │                         │
          ▼                         ▼                         ▼
   ┌─────────────┐          ┌─────────────┐          ┌─────────────┐
   │   Parsing   │          │ Serialization│          │ Bevy Plugin │
   │  (Phase 2)  │          │  (Phase 3)   │          │  (Phase 5)  │
   └─────────────┘          └─────────────┘          └─────────────┘
          │
          ▼
   ┌─────────────┐
   │ Projection  │
   │  (Phase 2)  │
   └─────────────┘
          │
          ▼
   ┌─────────────┐
   │    CLI      │
   │  (Phase 4)  │
   └─────────────┘
```

---

## Technical Requirements

### Environment
- **Rust Edition**: 2024
- **Minimum Rust Version**: 1.85.0+
- **Bevy Version**: 0.17.3+
- **Target Platforms**: Linux (x86_64, aarch64), Windows (x86_64), macOS (aarch64)

### Core Dependencies

```toml
[package]
name = "bevy_mixamo_2d"
version = "0.1.0"
edition = "2024"
rust-version = "1.85.0"
license = "MIT OR Apache-2.0"
description = "Convert Mixamo 3D animations to 2D skeletal animations for Bevy"
repository = "https://github.com/<user>/bevy_mixamo_2d"
keywords = ["bevy", "animation", "mixamo", "2d", "skeleton"]
categories = ["game-development", "graphics"]

[lib]
name = "bevy_mixamo_2d"
path = "src/lib.rs"

[[bin]]
name = "mixamo2d"
path = "src/bin/cli.rs"
required-features = ["cli"]

[features]
default = ["visualization"]
cli = ["dep:clap"]
visualization = ["dep:bevy"]

[dependencies]
gltf = { version = "1.4", features = ["names"] }
glam = { version = "0.30", features = ["serde"] }
ron = "0.12"
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"

# CLI dependencies (optional)
clap = { version = "4.5", features = ["derive"], optional = true }

# Bevy dependencies (optional, for visualization and runtime)
bevy = { version = "0.17", default-features = false, features = [
    "bevy_asset",
    "bevy_render",
    "bevy_sprite",
    "bevy_gizmos",
    "bevy_winit",
    "bevy_core_pipeline",
    "multi_threaded",
    "x11",
], optional = true }

[dev-dependencies]
bevy = { version = "0.17", features = ["bevy_dev_tools"] }
```

---

## Architecture

### Project Structure

```
bevy_mixamo_2d/
├── Cargo.toml
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── docs/
│   ├── technical-design.md
│   ├── bone-mapping.md
│   └── file-formats.md
├── src/
│   ├── lib.rs                    # Library entry point
│   ├── error.rs                  # Error types
│   ├── bin/
│   │   └── cli.rs                # CLI entry point
│   ├── core/                     # ★ CORE DATA LAYER ★
│   │   ├── mod.rs
│   │   ├── bone.rs               # Bone2D, Transform2D, PrimitiveShape
│   │   ├── skeleton.rs           # Skeleton2D with validation
│   │   ├── animation.rs          # Animation2D with validation
│   │   ├── keyframe.rs           # Keyframe<T>, Interpolation
│   │   ├── root_motion.rs        # RootMotion data
│   │   ├── mapping.rs            # BoneMapping configuration
│   │   ├── validation.rs         # Validation traits and implementations
│   │   └── compatibility.rs      # Skeleton-Animation compatibility checks
│   ├── gltf_parser/
│   │   ├── mod.rs
│   │   ├── skeleton.rs           # 3D skeleton extraction
│   │   └── animation.rs          # Animation channel extraction
│   ├── projection/
│   │   ├── mod.rs
│   │   ├── transform.rs          # 3D to 2D projection math
│   │   ├── root_motion.rs        # Root motion extraction/stripping
│   │   └── foreshortening.rs     # Foreshortening scale computation
│   ├── export/
│   │   ├── mod.rs
│   │   └── ron.rs                # RON serialization
│   └── bevy_plugin/
│       ├── mod.rs
│       ├── assets.rs             # Custom asset loaders
│       ├── components.rs         # ECS components
│       ├── systems.rs            # Animation playback systems
│       └── primitives.rs         # Debug visualization with primitives
├── assets/
│   └── bone_mappings/
│       └── mixamo_default.ron    # Default Mixamo bone mapping
├── examples/
│   ├── convert_single.rs         # Single file conversion example
│   ├── batch_convert.rs          # Batch conversion example
│   └── play_animation.rs         # Animation playback example
└── tests/
    ├── core/                     # ★ CORE DATA LAYER TESTS ★
    │   ├── mod.rs
    │   ├── skeleton_tests.rs     # Skeleton2D validation tests
    │   ├── animation_tests.rs    # Animation2D validation tests
    │   ├── compatibility_tests.rs # Skeleton-Animation compatibility
    │   └── edge_cases.rs         # Edge case coverage
    ├── integration/
    │   ├── parsing_tests.rs
    │   ├── projection_tests.rs
    │   └── serialization_tests.rs
    └── fixtures/
        ├── sample.glb            # Test GLB file
        ├── valid_skeleton.ron    # Valid skeleton fixture
        ├── valid_animation.ron   # Valid animation fixture
        ├── invalid_skeleton_duplicate_names.ron
        ├── invalid_skeleton_circular_parent.ron
        ├── invalid_animation_missing_bone.ron
        └── incompatible_animation.ron
```

---

## Phase 1: Core Data Layer (FOUNDATION)

**This phase MUST be completed and fully tested before proceeding to other phases.**

### Design Principles

1. **Immutable After Validation**: Once constructed via `::new()` or `::try_from()`, data structures are valid
2. **Fail Fast**: Invalid data is rejected at construction time with descriptive errors
3. **No Orphan Data**: Every bone reference must resolve, every timeline must target an existing bone
4. **Deterministic Ordering**: Bones are stored in topological order (parents before children)

---

### Task 1.1: Error Types
**File**: `src/error.rs`

```rust
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

    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to serialize/deserialize RON: {0}")]
    Ron(#[from] ron::error::SpannedError),

    #[error("No skeleton found in GLTF file")]
    NoSkeleton,

    #[error("No animation found in GLTF file")]
    NoAnimation,

    #[error("Bone '{0}' not found in skeleton")]
    BoneNotFound(String),

    #[error("Invalid bone mapping configuration: {0}")]
    InvalidMapping(String),
}

pub type Result<T> = std::result::Result<T, Mixamo2dError>;
```

---

### Task 1.2: Core Data Structures with Validation
**File**: `src/core/bone.rs`

```rust
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
```

**File**: `src/core/skeleton.rs`

```rust
use super::bone::Bone2D;
use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A 2D skeleton definition, saved as `*.skeleton2d.ron`
///
/// # Invariants
/// - `bones` is non-empty
/// - All bone names are unique
/// - Exactly one bone has `parent = None` (the root)
/// - Parent indices are always < child indices (topological order)
/// - All parent indices are valid (< bones.len())
/// - No circular parent references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skeleton2D {
    /// Unique name for this skeleton
    pub name: String,
    /// Ordered list of bones (index corresponds to bone ID)
    /// Stored in topological order: parents always come before children
    pub bones: Vec<Bone2D>,
}

impl Skeleton2D {
    /// Create a new Skeleton2D with full validation
    ///
    /// # Errors
    /// Returns `ValidationError` if any invariant is violated
    pub fn new(name: impl Into<String>, bones: Vec<Bone2D>) -> Result<Self, ValidationError> {
        let skeleton = Self {
            name: name.into(),
            bones,
        };
        skeleton.validate()?;
        Ok(skeleton)
    }

    /// Validate all skeleton invariants
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_non_empty()?;
        self.validate_unique_names()?;
        self.validate_single_root()?;
        self.validate_parent_indices()?;
        self.validate_topological_order()?;
        self.validate_no_cycles()?;
        self.validate_bone_properties()?;
        Ok(())
    }

    /// Get bone by index
    pub fn get_bone(&self, index: usize) -> Option<&Bone2D> {
        self.bones.get(index)
    }

    /// Get bone by name
    pub fn get_bone_by_name(&self, name: &str) -> Option<(usize, &Bone2D)> {
        self.bones.iter().enumerate().find(|(_, b)| b.name == name)
    }

    /// Get the root bone (the one with parent = None)
    pub fn root_bone(&self) -> &Bone2D {
        // Safe: validate() ensures exactly one root exists
        self.bones.iter().find(|b| b.is_root()).unwrap()
    }

    /// Get root bone index
    pub fn root_bone_index(&self) -> usize {
        self.bones.iter().position(|b| b.is_root()).unwrap()
    }

    /// Get children of a bone
    pub fn children_of(&self, parent_index: usize) -> Vec<usize> {
        self.bones
            .iter()
            .enumerate()
            .filter(|(_, b)| b.parent == Some(parent_index))
            .map(|(i, _)| i)
            .collect()
    }

    /// Build a map from bone name to index
    pub fn name_to_index_map(&self) -> HashMap<&str, usize> {
        self.bones
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name.as_str(), i))
            .collect()
    }

    /// Get bone count
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    // ─────────────────────────────────────────────────────────────
    // Private validation methods
    // ─────────────────────────────────────────────────────────────

    fn validate_non_empty(&self) -> Result<(), ValidationError> {
        if self.bones.is_empty() {
            return Err(ValidationError::EmptyBoneList);
        }
        Ok(())
    }

    fn validate_unique_names(&self) -> Result<(), ValidationError> {
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for (i, bone) in self.bones.iter().enumerate() {
            if let Some(&first_idx) = seen.get(bone.name.as_str()) {
                return Err(ValidationError::DuplicateBoneName {
                    name: bone.name.clone(),
                    first: first_idx,
                    second: i,
                });
            }
            seen.insert(&bone.name, i);
        }
        Ok(())
    }

    fn validate_single_root(&self) -> Result<(), ValidationError> {
        let roots: Vec<_> = self.bones.iter().filter(|b| b.is_root()).collect();
        match roots.len() {
            0 => Err(ValidationError::NoRootBone),
            1 => Ok(()),
            _ => Err(ValidationError::MultipleRootBones {
                first: roots[0].name.clone(),
                second: roots[1].name.clone(),
            }),
        }
    }

    fn validate_parent_indices(&self) -> Result<(), ValidationError> {
        for (i, bone) in self.bones.iter().enumerate() {
            if let Some(parent_idx) = bone.parent {
                if parent_idx >= self.bones.len() {
                    return Err(ValidationError::InvalidParentIndex {
                        name: bone.name.clone(),
                        index: i,
                        parent_index: parent_idx,
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_topological_order(&self) -> Result<(), ValidationError> {
        for (i, bone) in self.bones.iter().enumerate() {
            if let Some(parent_idx) = bone.parent {
                if parent_idx >= i {
                    return Err(ValidationError::ParentAfterChild {
                        name: bone.name.clone(),
                        index: i,
                        parent_index: parent_idx,
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_no_cycles(&self) -> Result<(), ValidationError> {
        for (start_idx, bone) in self.bones.iter().enumerate() {
            let mut visited = HashSet::new();
            let mut current = bone.parent;

            while let Some(idx) = current {
                if !visited.insert(idx) {
                    return Err(ValidationError::CircularParentReference {
                        name: bone.name.clone(),
                    });
                }
                if idx == start_idx {
                    return Err(ValidationError::CircularParentReference {
                        name: bone.name.clone(),
                    });
                }
                current = self.bones.get(idx).and_then(|b| b.parent);
            }
        }
        Ok(())
    }

    fn validate_bone_properties(&self) -> Result<(), ValidationError> {
        for bone in &self.bones {
            if bone.length < 0.0 {
                return Err(ValidationError::InvalidBoneLength {
                    name: bone.name.clone(),
                    length: bone.length,
                });
            }
            if !bone.setup.is_scale_valid() {
                return Err(ValidationError::InvalidBoneScale {
                    name: bone.name.clone(),
                    x: bone.setup.scale.x,
                    y: bone.setup.scale.y,
                });
            }
        }
        Ok(())
    }
}
```

**File**: `src/core/keyframe.rs`

```rust
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Interpolation method between keyframes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Interpolation {
    #[default]
    Linear,
    Step,
    CubicSpline,
}

/// A single keyframe with interpolation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    /// Time in seconds from animation start
    pub time: f32,
    /// Value at this keyframe
    pub value: T,
    /// How to interpolate to the next keyframe
    pub interpolation: Interpolation,
}

impl<T> Keyframe<T> {
    pub fn new(time: f32, value: T, interpolation: Interpolation) -> Self {
        Self { time, value, interpolation }
    }

    pub fn linear(time: f32, value: T) -> Self {
        Self::new(time, value, Interpolation::Linear)
    }
}

/// Animation timeline for a single bone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoneTimeline {
    /// Rotation keyframes (time -> radians) - REQUIRED
    pub rotations: Vec<Keyframe<f32>>,
    /// Translation keyframes (time -> Vec2) - optional, typically only for root
    pub translations: Option<Vec<Keyframe<Vec2>>>,
    /// Scale keyframes (time -> Vec2) - optional, for foreshortening
    pub scales: Option<Vec<Keyframe<Vec2>>>,
}

impl BoneTimeline {
    /// Create a new BoneTimeline with just rotation keyframes
    pub fn new(rotations: Vec<Keyframe<f32>>) -> Self {
        Self {
            rotations,
            translations: None,
            scales: None,
        }
    }

    /// Builder method to add translations
    pub fn with_translations(mut self, translations: Vec<Keyframe<Vec2>>) -> Self {
        self.translations = Some(translations);
        self
    }

    /// Builder method to add scales
    pub fn with_scales(mut self, scales: Vec<Keyframe<Vec2>>) -> Self {
        self.scales = Some(scales);
        self
    }

    /// Check if keyframes are sorted by time
    pub fn are_keyframes_sorted(&self) -> bool {
        Self::check_sorted(&self.rotations)
            && self.translations.as_ref().map_or(true, |t| Self::check_sorted(t))
            && self.scales.as_ref().map_or(true, |s| Self::check_sorted(s))
    }

    fn check_sorted<T>(keyframes: &[Keyframe<T>]) -> bool {
        keyframes.windows(2).all(|w| w[0].time <= w[1].time)
    }
}
```

**File**: `src/core/animation.rs`

```rust
use super::keyframe::{BoneTimeline, Keyframe};
use super::skeleton::Skeleton2D;
use crate::error::ValidationError;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root motion data extracted from the hip/root bone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootMotion {
    /// Total displacement over one animation cycle
    pub total_displacement: Vec2,
    /// Average velocity (units per second)
    pub velocity: Vec2,
    /// Per-frame displacement deltas (for precise motion matching)
    pub deltas: Vec<Vec2>,
}

/// A 2D animation clip, saved as `*.anim2d.ron`
///
/// # Invariants
/// - `duration` > 0
/// - `sample_rate` > 0
/// - `timelines` is non-empty
/// - All timeline bone indices are valid (< target skeleton bone count)
/// - All keyframes have times in range [0, duration]
/// - All keyframes within a timeline are sorted by time
/// - Each timeline has at least one rotation keyframe
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animation2D {
    /// Animation name
    pub name: String,
    /// Reference to the skeleton this animation targets
    pub skeleton_name: String,
    /// Total duration in seconds
    pub duration: f32,
    /// Sample rate (frames per second)
    pub sample_rate: f32,
    /// Whether this animation should loop
    pub looping: bool,
    /// Extracted root motion data
    pub root_motion: Option<RootMotion>,
    /// Per-bone animation timelines, keyed by bone index
    pub timelines: HashMap<usize, BoneTimeline>,
}

impl Animation2D {
    /// Create a new Animation2D with full validation
    ///
    /// # Arguments
    /// * `bone_count` - Number of bones in the target skeleton (for validation)
    ///
    /// # Errors
    /// Returns `ValidationError` if any invariant is violated
    pub fn new(
        name: impl Into<String>,
        skeleton_name: impl Into<String>,
        duration: f32,
        sample_rate: f32,
        looping: bool,
        root_motion: Option<RootMotion>,
        timelines: HashMap<usize, BoneTimeline>,
        bone_count: usize,
    ) -> Result<Self, ValidationError> {
        let animation = Self {
            name: name.into(),
            skeleton_name: skeleton_name.into(),
            duration,
            sample_rate,
            looping,
            root_motion,
            timelines,
        };
        animation.validate(bone_count)?;
        Ok(animation)
    }

    /// Validate animation against a known bone count
    pub fn validate(&self, bone_count: usize) -> Result<(), ValidationError> {
        self.validate_duration()?;
        self.validate_sample_rate()?;
        self.validate_non_empty_timelines()?;
        self.validate_timeline_bone_indices(bone_count)?;
        self.validate_keyframe_times()?;
        self.validate_keyframe_sorting()?;
        self.validate_rotation_keyframes()?;
        self.validate_root_motion()?;
        Ok(())
    }

    /// Validate animation is compatible with a specific skeleton
    pub fn validate_compatibility(&self, skeleton: &Skeleton2D) -> Result<(), ValidationError> {
        // Check skeleton name matches
        if self.skeleton_name != skeleton.name {
            return Err(ValidationError::SkeletonNameMismatch {
                animation_skeleton: self.skeleton_name.clone(),
                actual_skeleton: skeleton.name.clone(),
            });
        }

        // Check all timeline bone indices are valid
        for &bone_idx in self.timelines.keys() {
            if bone_idx >= skeleton.bone_count() {
                return Err(ValidationError::TimelineBoneIndexOutOfRange {
                    index: bone_idx,
                    bone_count: skeleton.bone_count(),
                });
            }
        }

        Ok(())
    }

    /// Get expected frame count based on duration and sample rate
    pub fn expected_frame_count(&self) -> usize {
        (self.duration * self.sample_rate).ceil() as usize
    }

    /// Sample the animation at a given time for a specific bone
    pub fn sample_bone(&self, bone_index: usize, time: f32) -> Option<SampledPose> {
        let timeline = self.timelines.get(&bone_index)?;
        let clamped_time = time.clamp(0.0, self.duration);

        let rotation = Self::sample_keyframes(&timeline.rotations, clamped_time);
        let translation = timeline.translations.as_ref()
            .map(|t| Self::sample_keyframes_vec2(t, clamped_time));
        let scale = timeline.scales.as_ref()
            .map(|s| Self::sample_keyframes_vec2(s, clamped_time));

        Some(SampledPose { rotation, translation, scale })
    }

    fn sample_keyframes(keyframes: &[Keyframe<f32>], time: f32) -> f32 {
        if keyframes.is_empty() {
            return 0.0;
        }
        if keyframes.len() == 1 || time <= keyframes[0].time {
            return keyframes[0].value;
        }
        if time >= keyframes.last().unwrap().time {
            return keyframes.last().unwrap().value;
        }

        // Binary search for surrounding keyframes
        let idx = keyframes.partition_point(|k| k.time <= time);
        let prev = &keyframes[idx.saturating_sub(1)];
        let next = &keyframes[idx.min(keyframes.len() - 1)];

        if prev.time == next.time {
            return prev.value;
        }

        let t = (time - prev.time) / (next.time - prev.time);

        match prev.interpolation {
            super::keyframe::Interpolation::Step => prev.value,
            super::keyframe::Interpolation::Linear => prev.value + (next.value - prev.value) * t,
            super::keyframe::Interpolation::CubicSpline => {
                // Simplified: fall back to linear for now
                prev.value + (next.value - prev.value) * t
            }
        }
    }

    fn sample_keyframes_vec2(keyframes: &[Keyframe<Vec2>], time: f32) -> Vec2 {
        if keyframes.is_empty() {
            return Vec2::ZERO;
        }
        if keyframes.len() == 1 || time <= keyframes[0].time {
            return keyframes[0].value;
        }
        if time >= keyframes.last().unwrap().time {
            return keyframes.last().unwrap().value;
        }

        let idx = keyframes.partition_point(|k| k.time <= time);
        let prev = &keyframes[idx.saturating_sub(1)];
        let next = &keyframes[idx.min(keyframes.len() - 1)];

        if prev.time == next.time {
            return prev.value;
        }

        let t = (time - prev.time) / (next.time - prev.time);

        match prev.interpolation {
            super::keyframe::Interpolation::Step => prev.value,
            super::keyframe::Interpolation::Linear => prev.value.lerp(next.value, t),
            super::keyframe::Interpolation::CubicSpline => prev.value.lerp(next.value, t),
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Private validation methods
    // ─────────────────────────────────────────────────────────────

    fn validate_duration(&self) -> Result<(), ValidationError> {
        if self.duration <= 0.0 {
            return Err(ValidationError::InvalidDuration { duration: self.duration });
        }
        Ok(())
    }

    fn validate_sample_rate(&self) -> Result<(), ValidationError> {
        if self.sample_rate <= 0.0 {
            return Err(ValidationError::InvalidSampleRate { sample_rate: self.sample_rate });
        }
        Ok(())
    }

    fn validate_non_empty_timelines(&self) -> Result<(), ValidationError> {
        if self.timelines.is_empty() {
            return Err(ValidationError::EmptyTimelines);
        }
        Ok(())
    }

    fn validate_timeline_bone_indices(&self, bone_count: usize) -> Result<(), ValidationError> {
        for &bone_idx in self.timelines.keys() {
            if bone_idx >= bone_count {
                return Err(ValidationError::TimelineTargetsInvalidBone { bone_index: bone_idx });
            }
        }
        Ok(())
    }

    fn validate_keyframe_times(&self) -> Result<(), ValidationError> {
        for (&bone_idx, timeline) in &self.timelines {
            Self::check_keyframe_times(&timeline.rotations, self.duration, bone_idx)?;
            if let Some(ref translations) = timeline.translations {
                Self::check_keyframe_times_vec2(translations, self.duration, bone_idx)?;
            }
            if let Some(ref scales) = timeline.scales {
                Self::check_keyframe_times_vec2(scales, self.duration, bone_idx)?;
            }
        }
        Ok(())
    }

    fn check_keyframe_times(
        keyframes: &[Keyframe<f32>],
        duration: f32,
        _bone_idx: usize,
    ) -> Result<(), ValidationError> {
        for (i, kf) in keyframes.iter().enumerate() {
            if kf.time < 0.0 || kf.time > duration {
                return Err(ValidationError::KeyframeTimeOutOfRange {
                    index: i,
                    time: kf.time,
                    duration,
                });
            }
        }
        Ok(())
    }

    fn check_keyframe_times_vec2(
        keyframes: &[Keyframe<Vec2>],
        duration: f32,
        _bone_idx: usize,
    ) -> Result<(), ValidationError> {
        for (i, kf) in keyframes.iter().enumerate() {
            if kf.time < 0.0 || kf.time > duration {
                return Err(ValidationError::KeyframeTimeOutOfRange {
                    index: i,
                    time: kf.time,
                    duration,
                });
            }
        }
        Ok(())
    }

    fn validate_keyframe_sorting(&self) -> Result<(), ValidationError> {
        for timeline in self.timelines.values() {
            Self::check_sorted(&timeline.rotations)?;
            if let Some(ref translations) = timeline.translations {
                Self::check_sorted_vec2(translations)?;
            }
            if let Some(ref scales) = timeline.scales {
                Self::check_sorted_vec2(scales)?;
            }
        }
        Ok(())
    }

    fn check_sorted(keyframes: &[Keyframe<f32>]) -> Result<(), ValidationError> {
        for i in 1..keyframes.len() {
            if keyframes[i].time < keyframes[i - 1].time {
                return Err(ValidationError::KeyframesNotSorted {
                    index: i,
                    time: keyframes[i].time,
                    prev_time: keyframes[i - 1].time,
                });
            }
        }
        Ok(())
    }

    fn check_sorted_vec2(keyframes: &[Keyframe<Vec2>]) -> Result<(), ValidationError> {
        for i in 1..keyframes.len() {
            if keyframes[i].time < keyframes[i - 1].time {
                return Err(ValidationError::KeyframesNotSorted {
                    index: i,
                    time: keyframes[i].time,
                    prev_time: keyframes[i - 1].time,
                });
            }
        }
        Ok(())
    }

    fn validate_rotation_keyframes(&self) -> Result<(), ValidationError> {
        for (&bone_idx, timeline) in &self.timelines {
            if timeline.rotations.is_empty() {
                return Err(ValidationError::EmptyRotationKeyframes { bone_index: bone_idx });
            }
        }
        Ok(())
    }

    fn validate_root_motion(&self) -> Result<(), ValidationError> {
        if let Some(ref root_motion) = self.root_motion {
            let expected = self.expected_frame_count();
            if root_motion.deltas.len() != expected && !root_motion.deltas.is_empty() {
                return Err(ValidationError::RootMotionDeltasMismatch {
                    deltas_count: root_motion.deltas.len(),
                    expected_count: expected,
                });
            }
        }
        Ok(())
    }
}

/// Result of sampling animation at a specific time
#[derive(Debug, Clone, Copy)]
pub struct SampledPose {
    pub rotation: f32,
    pub translation: Option<Vec2>,
    pub scale: Option<Vec2>,
}
```

**File**: `src/core/compatibility.rs`

```rust
use super::animation::Animation2D;
use super::skeleton::Skeleton2D;
use crate::error::ValidationError;

/// Check if an animation is compatible with a skeleton
pub fn check_compatibility(
    skeleton: &Skeleton2D,
    animation: &Animation2D,
) -> Result<(), ValidationError> {
    // 1. Check skeleton name matches
    if animation.skeleton_name != skeleton.name {
        return Err(ValidationError::SkeletonNameMismatch {
            animation_skeleton: animation.skeleton_name.clone(),
            actual_skeleton: skeleton.name.clone(),
        });
    }

    // 2. Check all timeline bone indices are within skeleton bounds
    for &bone_idx in animation.timelines.keys() {
        if bone_idx >= skeleton.bone_count() {
            return Err(ValidationError::TimelineBoneIndexOutOfRange {
                index: bone_idx,
                bone_count: skeleton.bone_count(),
            });
        }
    }

    Ok(())
}

/// Strict compatibility check - animation must have timelines for ALL skeleton bones
pub fn check_strict_compatibility(
    skeleton: &Skeleton2D,
    animation: &Animation2D,
) -> Result<(), ValidationError> {
    // First, do basic compatibility check
    check_compatibility(skeleton, animation)?;

    // Then verify bone count matches
    if animation.timelines.len() != skeleton.bone_count() {
        return Err(ValidationError::BoneCountMismatch {
            expected: skeleton.bone_count(),
            actual: animation.timelines.len(),
        });
    }

    Ok(())
}
```

---

### Task 1.3: Unit Tests for Core Data Layer

**File**: `tests/core/skeleton_tests.rs`

```rust
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
```

**File**: `tests/core/animation_tests.rs`

```rust
use bevy_mixamo_2d::core::*;
use bevy_mixamo_2d::error::ValidationError;
use glam::Vec2;
use std::collections::HashMap;

fn make_simple_skeleton() -> Skeleton2D {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("spine", Some(0), 0.3, Transform2D::default(), PrimitiveShape::None),
    ];
    Skeleton2D::new("test_skeleton", bones).unwrap()
}

fn make_keyframes(times: &[f32]) -> Vec<Keyframe<f32>> {
    times.iter().map(|&t| Keyframe::linear(t, 0.0)).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// VALID ANIMATION CONSTRUCTION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_valid_minimal_animation() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let animation = Animation2D::new(
        "walk",
        "test_skeleton",
        1.0,
        30.0,
        true,
        None,
        timelines,
        2, // bone_count
    );
    assert!(animation.is_ok());
}

#[test]
fn test_valid_animation_with_all_bones() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(0.5, 1.0),
        Keyframe::linear(1.0, 0.0),
    ]));
    timelines.insert(1, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 0.5),
    ]));

    let animation = Animation2D::new(
        "run",
        "test_skeleton",
        1.0,
        60.0,
        true,
        None,
        timelines,
        2,
    );
    assert!(animation.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID DURATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_zero_duration_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", 0.0, 30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidDuration { duration: 0.0 })));
}

#[test]
fn test_negative_duration_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", -1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidDuration { .. })));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID SAMPLE RATE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_zero_sample_rate_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", 1.0, 0.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidSampleRate { sample_rate: 0.0 })));
}

#[test]
fn test_negative_sample_rate_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let result = Animation2D::new("bad", "skel", 1.0, -30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::InvalidSampleRate { .. })));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: EMPTY TIMELINES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_timelines_rejected() {
    let timelines = HashMap::new();

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(result, Err(ValidationError::EmptyTimelines)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: INVALID BONE INDEX IN TIMELINE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_timeline_targets_nonexistent_bone_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));
    timelines.insert(99, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)])); // bone 99 doesn't exist

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 2);
    assert!(matches!(
        result,
        Err(ValidationError::TimelineTargetsInvalidBone { bone_index: 99 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: KEYFRAME TIME OUT OF RANGE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframe_time_exceeds_duration_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(2.0, 1.0), // duration is 1.0, this is 2.0
    ]));

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::KeyframeTimeOutOfRange { time: 2.0, duration: 1.0, .. })
    ));
}

#[test]
fn test_negative_keyframe_time_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(-0.5, 0.0), // negative time
    ]));

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::KeyframeTimeOutOfRange { time, .. }) if time == -0.5
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: UNSORTED KEYFRAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unsorted_keyframes_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.5, 0.0),
        Keyframe::linear(0.2, 1.0), // 0.2 < 0.5, out of order
    ]));

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::KeyframesNotSorted { index: 1, time: 0.2, prev_time: 0.5 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: EMPTY ROTATION KEYFRAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_rotation_keyframes_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![])); // empty rotations

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::EmptyRotationKeyframes { bone_index: 0 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASE: ROOT MOTION DELTAS MISMATCH
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_root_motion_deltas_mismatch_rejected() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let root_motion = RootMotion {
        total_displacement: Vec2::ZERO,
        velocity: Vec2::ZERO,
        deltas: vec![Vec2::ZERO, Vec2::ZERO], // 2 deltas, but duration=1.0, rate=30 means 30 frames
    };

    let result = Animation2D::new("bad", "skel", 1.0, 30.0, true, Some(root_motion), timelines, 1);
    assert!(matches!(
        result,
        Err(ValidationError::RootMotionDeltasMismatch { deltas_count: 2, expected_count: 30 })
    ));
}

#[test]
fn test_empty_root_motion_deltas_allowed() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    let root_motion = RootMotion {
        total_displacement: Vec2::new(1.0, 0.0),
        velocity: Vec2::new(1.0, 0.0),
        deltas: vec![], // empty is OK - means "not computed"
    };

    let result = Animation2D::new("good", "skel", 1.0, 30.0, true, Some(root_motion), timelines, 1);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SAMPLING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_sample_bone_linear_interpolation() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 2.0),
    ]));

    let animation = Animation2D::new("test", "skel", 1.0, 30.0, true, None, timelines, 1).unwrap();

    let sample = animation.sample_bone(0, 0.5).unwrap();
    assert!((sample.rotation - 1.0).abs() < 0.001); // midpoint = 1.0
}

#[test]
fn test_sample_bone_step_interpolation() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::new(0.0, 0.0, Interpolation::Step),
        Keyframe::new(1.0, 2.0, Interpolation::Step),
    ]));

    let animation = Animation2D::new("test", "skel", 1.0, 30.0, true, None, timelines, 1).unwrap();

    let sample = animation.sample_bone(0, 0.5).unwrap();
    assert!((sample.rotation - 0.0).abs() < 0.001); // step holds previous value
}

#[test]
fn test_sample_bone_clamps_time() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 1.0),
        Keyframe::linear(1.0, 2.0),
    ]));

    let animation = Animation2D::new("test", "skel", 1.0, 30.0, true, None, timelines, 1).unwrap();

    // Before start
    let sample = animation.sample_bone(0, -1.0).unwrap();
    assert!((sample.rotation - 1.0).abs() < 0.001);

    // After end
    let sample = animation.sample_bone(0, 5.0).unwrap();
    assert!((sample.rotation - 2.0).abs() < 0.001);
}
```

**File**: `tests/core/compatibility_tests.rs`

```rust
use bevy_mixamo_2d::core::*;
use bevy_mixamo_2d::core::compatibility::*;
use bevy_mixamo_2d::error::ValidationError;
use std::collections::HashMap;

fn make_skeleton(name: &str, bone_count: usize) -> Skeleton2D {
    let mut bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    for i in 1..bone_count {
        bones.push(Bone2D::new(
            format!("bone_{}", i),
            Some(0),
            1.0,
            Transform2D::default(),
            PrimitiveShape::None,
        ));
    }
    Skeleton2D::new(name, bones).unwrap()
}

fn make_animation(skeleton_name: &str, bone_indices: &[usize], bone_count: usize) -> Animation2D {
    let mut timelines = HashMap::new();
    for &idx in bone_indices {
        timelines.insert(idx, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));
    }
    Animation2D::new(
        "test_anim",
        skeleton_name,
        1.0,
        30.0,
        true,
        None,
        timelines,
        bone_count,
    ).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON NAME MISMATCH
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_name_mismatch_rejected() {
    let skeleton = make_skeleton("skeleton_a", 3);
    let animation = make_animation("skeleton_b", &[0, 1, 2], 3); // different name

    let result = check_compatibility(&skeleton, &animation);
    assert!(matches!(
        result,
        Err(ValidationError::SkeletonNameMismatch {
            animation_skeleton,
            actual_skeleton
        }) if animation_skeleton == "skeleton_b" && actual_skeleton == "skeleton_a"
    ));
}

#[test]
fn test_skeleton_name_match_accepted() {
    let skeleton = make_skeleton("my_skeleton", 3);
    let animation = make_animation("my_skeleton", &[0, 1], 3);

    let result = check_compatibility(&skeleton, &animation);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// BONE INDEX OUT OF RANGE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_timeline_bone_index_exceeds_skeleton_rejected() {
    let skeleton = make_skeleton("test", 2); // only indices 0 and 1 valid

    // Animation was created with bone_count=5 (for its internal validation)
    // but skeleton only has 2 bones
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));
    timelines.insert(4, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)])); // index 4 invalid for skeleton

    let animation = Animation2D::new("anim", "test", 1.0, 30.0, true, None, timelines, 5).unwrap();

    let result = check_compatibility(&skeleton, &animation);
    assert!(matches!(
        result,
        Err(ValidationError::TimelineBoneIndexOutOfRange { index: 4, bone_count: 2 })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// STRICT COMPATIBILITY (BONE COUNT MATCH)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_strict_compatibility_bone_count_mismatch() {
    let skeleton = make_skeleton("test", 5);
    let animation = make_animation("test", &[0, 1, 2], 5); // only 3 timelines, but 5 bones

    // Basic compatibility should pass
    assert!(check_compatibility(&skeleton, &animation).is_ok());

    // Strict compatibility should fail
    let result = check_strict_compatibility(&skeleton, &animation);
    assert!(matches!(
        result,
        Err(ValidationError::BoneCountMismatch { expected: 5, actual: 3 })
    ));
}

#[test]
fn test_strict_compatibility_all_bones_covered() {
    let skeleton = make_skeleton("test", 3);
    let animation = make_animation("test", &[0, 1, 2], 3); // all 3 bones covered

    let result = check_strict_compatibility(&skeleton, &animation);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTEGRATION: validate_compatibility METHOD
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_validate_compatibility_method() {
    let skeleton = make_skeleton("my_skel", 3);
    let animation = make_animation("my_skel", &[0, 1], 3);

    // Using the method directly on Animation2D
    assert!(animation.validate_compatibility(&skeleton).is_ok());
}

#[test]
fn test_animation_validate_compatibility_fails_on_mismatch() {
    let skeleton = make_skeleton("skeleton_x", 3);
    let animation = make_animation("skeleton_y", &[0], 3);

    assert!(animation.validate_compatibility(&skeleton).is_err());
}
```

**File**: `tests/core/edge_cases.rs`

```rust
//! Additional edge case tests for comprehensive coverage

use bevy_mixamo_2d::core::*;
use bevy_mixamo_2d::error::ValidationError;
use glam::Vec2;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSFORM2D EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_transform2d_rotation_normalization() {
    // Test that rotations are normalized to [-π, π]
    let t1 = Transform2D::new(Vec2::ZERO, 7.0, Vec2::ONE); // > 2π
    assert!(t1.rotation >= -std::f32::consts::PI && t1.rotation <= std::f32::consts::PI);

    let t2 = Transform2D::new(Vec2::ZERO, -7.0, Vec2::ONE); // < -2π
    assert!(t2.rotation >= -std::f32::consts::PI && t2.rotation <= std::f32::consts::PI);
}

#[test]
fn test_primitive_shape_validation() {
    assert!(PrimitiveShape::Circle { radius: 1.0 }.is_valid());
    assert!(!PrimitiveShape::Circle { radius: 0.0 }.is_valid());
    assert!(!PrimitiveShape::Circle { radius: -1.0 }.is_valid());

    assert!(PrimitiveShape::Rectangle { width: 1.0, height: 1.0 }.is_valid());
    assert!(!PrimitiveShape::Rectangle { width: 0.0, height: 1.0 }.is_valid());
    assert!(!PrimitiveShape::Rectangle { width: 1.0, height: -1.0 }.is_valid());

    assert!(PrimitiveShape::None.is_valid());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON DEEP HIERARCHY
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_deep_bone_hierarchy() {
    // Create a chain of 100 bones
    let mut bones = vec![
        Bone2D::new("bone_0", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    for i in 1..100 {
        bones.push(Bone2D::new(
            format!("bone_{}", i),
            Some(i - 1), // chain: each bone's parent is the previous one
            1.0,
            Transform2D::default(),
            PrimitiveShape::None,
        ));
    }

    let skeleton = Skeleton2D::new("deep_chain", bones);
    assert!(skeleton.is_ok());
    assert_eq!(skeleton.unwrap().bone_count(), 100);
}

#[test]
fn test_wide_bone_hierarchy() {
    // Root with 50 direct children
    let mut bones = vec![
        Bone2D::new("root", None, 0.0, Transform2D::default(), PrimitiveShape::None),
    ];
    for i in 1..=50 {
        bones.push(Bone2D::new(
            format!("child_{}", i),
            Some(0), // all children of root
            1.0,
            Transform2D::default(),
            PrimitiveShape::None,
        ));
    }

    let skeleton = Skeleton2D::new("wide", bones);
    assert!(skeleton.is_ok());
    let s = skeleton.unwrap();
    assert_eq!(s.children_of(0).len(), 50);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION BOUNDARY CONDITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_keyframe_at_exact_duration() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 1.0), // exactly at duration
    ]));

    let animation = Animation2D::new("exact", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(animation.is_ok());
}

#[test]
fn test_animation_single_keyframe() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.5, 1.0), // single keyframe in the middle
    ]));

    let animation = Animation2D::new("single", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(animation.is_ok());

    let anim = animation.unwrap();
    // Sampling should return the single keyframe value regardless of time
    assert_eq!(anim.sample_bone(0, 0.0).unwrap().rotation, 1.0);
    assert_eq!(anim.sample_bone(0, 0.5).unwrap().rotation, 1.0);
    assert_eq!(anim.sample_bone(0, 1.0).unwrap().rotation, 1.0);
}

#[test]
fn test_animation_many_keyframes() {
    let mut timelines = HashMap::new();
    let keyframes: Vec<_> = (0..1000)
        .map(|i| Keyframe::linear(i as f32 * 0.001, i as f32))
        .collect();
    timelines.insert(0, BoneTimeline::new(keyframes));

    let animation = Animation2D::new("many", "skel", 1.0, 30.0, true, None, timelines, 1);
    assert!(animation.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZATION ROUNDTRIP
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_skeleton_ron_roundtrip() {
    let bones = vec![
        Bone2D::new("hips", None, 0.0, Transform2D::default(), PrimitiveShape::Circle { radius: 0.1 }),
        Bone2D::new("spine", Some(0), 0.3, Transform2D::new(Vec2::new(0.0, 0.3), 1.57, Vec2::ONE), PrimitiveShape::Rectangle { width: 0.2, height: 0.3 }),
    ];
    let skeleton = Skeleton2D::new("roundtrip", bones).unwrap();

    let ron_str = ron::to_string(&skeleton).unwrap();
    let parsed: Skeleton2D = ron::from_str(&ron_str).unwrap();

    assert_eq!(skeleton, parsed);
}

#[test]
fn test_animation_ron_roundtrip() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![
        Keyframe::linear(0.0, 0.0),
        Keyframe::linear(1.0, 3.14),
    ]).with_translations(vec![
        Keyframe::linear(0.0, Vec2::ZERO),
        Keyframe::linear(1.0, Vec2::new(1.0, 0.0)),
    ]));

    let animation = Animation2D::new(
        "roundtrip",
        "test_skel",
        1.0,
        30.0,
        true,
        Some(RootMotion {
            total_displacement: Vec2::new(1.0, 0.0),
            velocity: Vec2::new(1.0, 0.0),
            deltas: vec![],
        }),
        timelines,
        1,
    ).unwrap();

    let ron_str = ron::to_string(&animation).unwrap();
    let parsed: Animation2D = ron::from_str(&ron_str).unwrap();

    assert_eq!(animation.name, parsed.name);
    assert_eq!(animation.duration, parsed.duration);
    // Note: Full equality check depends on PartialEq implementation
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNICODE AND SPECIAL CHARACTERS IN NAMES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unicode_bone_names() {
    let bones = vec![
        Bone2D::new("根骨", None, 0.0, Transform2D::default(), PrimitiveShape::None), // Chinese
        Bone2D::new("κεφαλή", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None), // Greek
        Bone2D::new("кость", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None), // Russian
    ];
    let skeleton = Skeleton2D::new("unicode_test", bones);
    assert!(skeleton.is_ok());
}

#[test]
fn test_special_characters_in_names() {
    let bones = vec![
        Bone2D::new("bone-with-dashes", None, 0.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("bone_with_underscores", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("bone.with.dots", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
        Bone2D::new("bone:with:colons", Some(0), 1.0, Transform2D::default(), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("special_chars", bones);
    assert!(skeleton.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// FLOATING POINT EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_very_small_duration() {
    let mut timelines = HashMap::new();
    timelines.insert(0, BoneTimeline::new(vec![Keyframe::linear(0.0, 0.0)]));

    // Very small but positive duration should be valid
    let animation = Animation2D::new("tiny", "skel", 0.001, 1000.0, true, None, timelines, 1);
    assert!(animation.is_ok());
}

#[test]
fn test_very_large_values() {
    let bones = vec![
        Bone2D::new("root", None, 1000000.0, Transform2D::new(
            Vec2::new(1e6, 1e6),
            0.0,
            Vec2::ONE,
        ), PrimitiveShape::None),
    ];
    let skeleton = Skeleton2D::new("large", bones);
    assert!(skeleton.is_ok());
}
```

---

## Phase 2: GLB/GLTF Parsing and Projection

(Continues as before, but now builds on the validated core data layer)

### Task 2.1: Skeleton Extraction
[Previous content remains, but output is validated Skeleton2D]

### Task 2.2: Animation Extraction
[Previous content remains, but output is validated Animation2D]

### Task 2.3: 3D to 2D Projection
[Previous content remains]

---

## Phase 3: RON Serialization

### Task 3.1: Serialization with Validation on Load

```rust
// src/export/ron.rs

use crate::core::{Animation2D, Skeleton2D};
use crate::error::{Mixamo2dError, Result};
use ron::ser::PrettyConfig;

pub fn save_skeleton(skeleton: &Skeleton2D, path: &std::path::Path) -> Result<()> {
    // Skeleton is already validated (invariant of the type)
    let config = PrettyConfig::new()
        .struct_names(true)
        .separate_tuple_members(true)
        .enumerate_arrays(false)
        .decimal_floats(true);

    let ron_string = ron::ser::to_string_pretty(skeleton, config)?;
    std::fs::write(path, ron_string)?;
    Ok(())
}

pub fn load_skeleton(path: &std::path::Path) -> Result<Skeleton2D> {
    let contents = std::fs::read_to_string(path)?;
    let skeleton: Skeleton2D = ron::from_str(&contents)?;
    // Re-validate after deserialization (defense in depth)
    skeleton.validate()?;
    Ok(skeleton)
}

pub fn save_animation(animation: &Animation2D, path: &std::path::Path) -> Result<()> {
    let config = PrettyConfig::new()
        .struct_names(true)
        .separate_tuple_members(true)
        .enumerate_arrays(false)
        .decimal_floats(true);

    let ron_string = ron::ser::to_string_pretty(animation, config)?;
    std::fs::write(path, ron_string)?;
    Ok(())
}

/// Load animation and validate against expected bone count
pub fn load_animation(path: &std::path::Path, bone_count: usize) -> Result<Animation2D> {
    let contents = std::fs::read_to_string(path)?;
    let animation: Animation2D = ron::from_str(&contents)?;
    // Re-validate after deserialization
    animation.validate(bone_count)?;
    Ok(animation)
}

/// Load animation and validate against a specific skeleton
pub fn load_animation_for_skeleton(
    path: &std::path::Path,
    skeleton: &Skeleton2D,
) -> Result<Animation2D> {
    let animation = load_animation(path, skeleton.bone_count())?;
    animation.validate_compatibility(skeleton)?;
    Ok(animation)
}
```

---

## Phase 4: CLI Application

[Previous content remains]

---

## Phase 5: Bevy Plugin

[Previous content remains, but asset loaders call validation on load]

---

## Implementation Order (Revised)

Execute tasks in this order:

1. **Phase 1**: Core data layer with full validation and tests
   - Task 1.1: Error types
   - Task 1.2: Core data structures with validation
   - Task 1.3: Unit tests (must pass before proceeding)
2. **Phase 2**: GLB/GLTF parsing and projection
3. **Phase 3**: RON serialization (with validation on load)
4. **Phase 4**: CLI application
5. **Phase 5**: Bevy plugin

**CRITICAL**: Phase 1 must be 100% complete with all tests passing before starting Phase 2.

---

## Test Coverage Requirements

### Core Data Layer (Phase 1)
- **Skeleton2D**: 100% validation path coverage
- **Animation2D**: 100% validation path coverage
- **Compatibility checks**: 100% coverage

### Edge Cases That MUST Be Tested

| Category | Test Case | Expected Result |
|----------|-----------|-----------------|
| Empty data | Empty bone list | `EmptyBoneList` error |
| Empty data | Empty timelines | `EmptyTimelines` error |
| Duplicates | Duplicate bone names | `DuplicateBoneName` error |
| Invalid references | Parent index out of bounds | `InvalidParentIndex` error |
| Invalid references | Timeline targets non-existent bone | `TimelineTargetsInvalidBone` error |
| Ordering | Parent after child (topological) | `ParentAfterChild` error |
| Ordering | Unsorted keyframes | `KeyframesNotSorted` error |
| Bounds | Keyframe time > duration | `KeyframeTimeOutOfRange` error |
| Bounds | Negative keyframe time | `KeyframeTimeOutOfRange` error |
| Invalid values | Zero/negative duration | `InvalidDuration` error |
| Invalid values | Zero/negative sample rate | `InvalidSampleRate` error |
| Invalid values | Negative bone length | `InvalidBoneLength` error |
| Invalid values | Zero/negative scale | `InvalidBoneScale` error |
| Hierarchy | Multiple root bones | `MultipleRootBones` error |
| Hierarchy | No root bone | `NoRootBone` error |
| Compatibility | Skeleton name mismatch | `SkeletonNameMismatch` error |
| Compatibility | Bone count mismatch (strict) | `BoneCountMismatch` error |
| Root motion | Deltas count mismatch | `RootMotionDeltasMismatch` error |

### Running Tests

```bash
# Run all core data layer tests
cargo test --lib core::

# Run with verbose output
cargo test --lib core:: -- --nocapture

# Run specific test
cargo test --lib test_duplicate_bone_names_rejected
```

---

## Acceptance Criteria (Revised)

### Core Data Layer
- [ ] `Skeleton2D::new()` rejects all invalid configurations with specific errors
- [ ] `Animation2D::new()` rejects all invalid configurations with specific errors
- [ ] All validation errors include context (bone name, index, values)
- [ ] Compatibility checks detect mismatched skeleton/animation pairs
- [ ] RON serialization round-trips preserve all data exactly
- [ ] 100% test coverage for validation logic
- [ ] All edge case tests pass

### CLI Application
- [ ] Validates output before writing files
- [ ] Reports validation errors with file/line context
- [ ] Rejects incompatible skeleton/animation combinations

### Library Crate
- [ ] Asset loaders validate on load
- [ ] Invalid assets produce clear error messages
- [ ] Runtime animation playback handles edge cases gracefully

---

## Notes for Coding Agent

### Critical Implementation Details

1. **Validation First**: Always validate data at construction time. Never create invalid `Skeleton2D` or `Animation2D` instances.

2. **Error Context**: Include relevant context in all errors (bone names, indices, actual values) to aid debugging.

3. **Defense in Depth**: Re-validate after deserialization from RON files. External files may be hand-edited or corrupted.

4. **Test-Driven Development**: Write tests for each validation rule BEFORE implementing the validation.

5. **Topological Order Invariant**: The bones array MUST maintain parents-before-children order. This simplifies forward kinematics and eliminates cycle detection during animation playback.

6. **Single Root Invariant**: Exactly one bone may have `parent = None`. This ensures a proper tree structure.

7. **Unique Names Invariant**: Bone names must be unique to allow lookup by name and unambiguous bone mapping.
