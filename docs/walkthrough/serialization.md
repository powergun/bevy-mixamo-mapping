# Serialization Reference

This document describes the RON serialization system for `bevy_mixamo_2d`. Designed for coding agents to understand the file format and I/O operations.

## Overview

The library uses [RON (Rusty Object Notation)](https://github.com/ron-rs/ron) for serializing skeleton and animation data. RON is human-readable, supports Rust types directly, and integrates well with serde.

```
FBX/GLB Input → Parse → Project → Serialize → RON Files
                                              ↓
RON Files → Deserialize → Validate → Skeleton2D/Animation2D
```

## File Extensions

| Extension | Content | Description |
|-----------|---------|-------------|
| `*.skeleton2d.ron` | `Skeleton2D` | 2D skeleton definition |
| `*.anim2d.ron` | `Animation2D` | 2D animation clip |

## Module Location

```
src/export/
├── mod.rs        # Re-exports public API
└── ron.rs        # RON serialization (58 lines)
```

## Public API

### Skeleton I/O

```rust
use bevy_mixamo_2d::{save_skeleton, load_skeleton, Skeleton2D};
use std::path::Path;

// Save skeleton to RON file
let skeleton: Skeleton2D = /* ... */;
save_skeleton(&skeleton, Path::new("player.skeleton2d.ron"))?;

// Load skeleton from RON file (with validation)
let skeleton = load_skeleton(Path::new("player.skeleton2d.ron"))?;
```

### Animation I/O

```rust
use bevy_mixamo_2d::{save_animation, load_animation, Animation2D, Skeleton2D};
use bevy_mixamo_2d::export::load_animation_for_skeleton;

// Save animation to RON file
let animation: Animation2D = /* ... */;
save_animation(&animation, Path::new("walk.anim2d.ron"))?;

// Load animation with bone count validation
let animation = load_animation(Path::new("walk.anim2d.ron"), bone_count)?;

// Load animation with full skeleton compatibility check
let skeleton = load_skeleton(Path::new("player.skeleton2d.ron"))?;
let animation = load_animation_for_skeleton(Path::new("walk.anim2d.ron"), &skeleton)?;
```

## Defense in Depth

Serialization implements **validation on load** as a defense-in-depth measure:

```rust
// src/export/ron.rs
pub fn load_skeleton(path: &Path) -> Result<Skeleton2D> {
    let contents = std::fs::read_to_string(path)?;
    let skeleton: Skeleton2D = ron::from_str(&contents)?;
    // Re-validate after deserialization
    skeleton.validate()?;
    Ok(skeleton)
}
```

This catches:
- Hand-edited files with invalid data
- Corrupted files
- Files from incompatible versions

## RON Format Examples

### Skeleton2D

```ron
Skeleton2D(
    name: "player",
    bones: [
        Bone2D(
            name: "mixamorig:Hips",
            parent: None,
            length: 9979.194,
            setup: Transform2D(
                translation: Vec2(-0.0006757, 9979.194),
                rotation: 0.00008726,
                scale: Vec2(0.9999951, 0.9999951),
            ),
            shape: None,
        ),
        Bone2D(
            name: "mixamorig:Spine",
            parent: Some(0),
            length: 992.346,
            setup: Transform2D(
                translation: Vec2(0.086, 992.346),
                rotation: 0.00001221,
                scale: Vec2(0.8907, 0.8907),
            ),
            shape: None,
        ),
        // ... more bones
    ],
)
```

### Animation2D

```ron
Animation2D(
    name: "walk",
    skeleton_name: "player",
    duration: 1.0,
    sample_rate: 30.0,
    looping: true,
    root_motion: None,
    timelines: {
        0: BoneTimeline(
            rotations: [
                Keyframe(time: 0.0, value: 0.0, interpolation: Linear),
                Keyframe(time: 0.5, value: 0.15, interpolation: Linear),
                Keyframe(time: 1.0, value: 0.0, interpolation: Linear),
            ],
            translations: Some([
                Keyframe(time: 0.0, value: Vec2(0.0, 100.0), interpolation: Linear),
            ]),
            scales: None,
        ),
        1: BoneTimeline(
            rotations: [
                Keyframe(time: 0.0, value: 0.0, interpolation: Linear),
            ],
            translations: None,
            scales: None,
        ),
        // ... more bone timelines
    },
)
```

## Pretty Config

The serializer uses `PrettyConfig` for readable output:

```rust
// src/export/ron.rs
let config = PrettyConfig::new()
    .struct_names(true)       // Include struct names (Bone2D, etc.)
    .separate_tuple_members(true)
    .enumerate_arrays(false);  // No array indices

let ron_string = ron::ser::to_string_pretty(skeleton, config)?;
```

## Error Handling

All I/O functions return `Result<T, Mixamo2dError>`:

```rust
pub enum Mixamo2dError {
    // File I/O errors
    Io(std::io::Error),

    // RON parse errors
    Ron(ron::error::SpannedError),

    // Validation errors (re-validation on load)
    Validation(ValidationError),

    // ...
}
```

## Implementation Details

### Why RON?

1. **Human-readable**: Easy to inspect and debug
2. **Rust-native**: Direct serde support for Rust types
3. **Type-preserving**: Struct names in output aid debugging
4. **Bevy ecosystem**: Commonly used in Bevy projects

### Serde Derives

All serializable types implement serde traits:

```rust
// src/core/skeleton.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skeleton2D { /* ... */ }

// src/core/animation.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animation2D { /* ... */ }

// src/core/bone.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bone2D { /* ... */ }
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D { /* ... */ }
```

### glam Integration

glam types (`Vec2`) serialize correctly via the `serde` feature:

```toml
# Cargo.toml
glam = { version = "0.30", features = ["serde"] }
```

## Testing

Serialization is tested via roundtrip tests in `tests/core/edge_cases.rs`:

```rust
#[test]
fn test_skeleton_ron_roundtrip() {
    let skeleton = Skeleton2D::new("test", bones).unwrap();
    let ron_str = ron::to_string(&skeleton).unwrap();
    let parsed: Skeleton2D = ron::from_str(&ron_str).unwrap();
    assert_eq!(skeleton, parsed);
}

#[test]
fn test_animation_ron_roundtrip() {
    let animation = Animation2D::new(/* ... */).unwrap();
    let ron_str = ron::to_string(&animation).unwrap();
    let parsed: Animation2D = ron::from_str(&ron_str).unwrap();
    assert_eq!(animation.name, parsed.name);
}
```

## File Size Estimates

For a typical Mixamo humanoid (65 bones, 30 FPS, 1 second):

| File | Approximate Size |
|------|------------------|
| `*.skeleton2d.ron` | ~30 KB |
| `*.anim2d.ron` | ~150-200 KB |

File size scales with:
- Number of bones
- Animation duration
- Sample rate
- Number of animated properties (rotation, translation, scale)

## Best Practices

1. **Always use the library functions** - Don't parse RON manually
2. **Validate after loading** - The library does this automatically
3. **Check compatibility** - Use `load_animation_for_skeleton()` when possible
4. **Handle errors** - All I/O can fail

## Dependencies

```toml
# Cargo.toml
ron = "0.12"
serde = { version = "1.0", features = ["derive"] }
glam = { version = "0.30", features = ["serde"] }
```
