# Data Models Reference

This document describes the core data structures for `bevy_mixamo_2d`. Designed for coding agents to quickly understand the codebase.

## Architecture: Data-First Design

All modules depend on these core data types. Invalid data is rejected at construction time via `::new()` methods that return `Result<T, ValidationError>`.

```
Core Data Layer (src/core/)
         │
    ┌────┴────┬────────────┬─────────────┬───────────┐
    │         │            │             │           │
 Parsing   Projection  Serialization    CLI      Bevy Plugin
  (done)     (done)       (done)      (done)     (Phase 5)
```

## Core Types

### Skeleton2D (`src/core/skeleton.rs`)

2D skeleton definition. Serialized as `*.skeleton2d.ron`.

```rust
Skeleton2D {
    name: String,           // Unique skeleton identifier
    bones: Vec<Bone2D>,     // Topologically ordered (parents before children)
}
```

**Invariants enforced by `Skeleton2D::new()`:**
- Non-empty bone list
- Unique bone names
- Exactly one root bone (parent = None)
- Valid parent indices (< bones.len())
- Topological order (parent_index < child_index)
- No circular references
- Bone length >= 0, scale > 0

**Key methods:**
- `get_bone(index)` / `get_bone_by_name(name)` - Bone lookup
- `root_bone()` / `root_bone_index()` - Root access
- `children_of(parent_index)` - Get child indices
- `validate()` - Re-validate (used after deserialization)

### Bone2D (`src/core/bone.rs`)

```rust
Bone2D {
    name: String,              // Unique within skeleton
    parent: Option<usize>,     // None = root bone
    length: f32,               // Rest-pose length (>= 0)
    setup: Transform2D,        // Setup pose transform
    shape: PrimitiveShape,     // Visualization/collision shape
}
```

### Transform2D (`src/core/bone.rs`)

```rust
Transform2D {
    translation: Vec2,    // Position relative to parent
    rotation: f32,        // Radians, normalized to [-π, π]
    scale: Vec2,          // Must be > 0 for both components
}
```

### Animation2D (`src/core/animation.rs`)

2D animation clip. Serialized as `*.anim2d.ron`.

```rust
Animation2D {
    name: String,
    skeleton_name: String,              // Target skeleton reference
    duration: f32,                      // Seconds (> 0)
    sample_rate: f32,                   // FPS (> 0)
    looping: bool,
    root_motion: Option<RootMotion>,
    timelines: HashMap<usize, BoneTimeline>,  // bone_index -> timeline
}
```

**Invariants enforced by `Animation2D::new(bone_count)`:**
- Duration > 0, sample_rate > 0
- Non-empty timelines
- All timeline bone indices < bone_count
- Keyframe times in [0, duration]
- Keyframes sorted by time
- Each timeline has >= 1 rotation keyframe
- Root motion deltas count matches expected frame count (or empty)

**Key methods:**
- `sample_bone(bone_index, time)` - Returns `SampledPose` with interpolated values
- `validate(bone_count)` - Validate against bone count
- `validate_compatibility(skeleton)` - Check skeleton match

### BoneTimeline / Keyframe (`src/core/keyframe.rs`)

```rust
BoneTimeline {
    rotations: Vec<Keyframe<f32>>,             // Required
    translations: Option<Vec<Keyframe<Vec2>>>, // Optional (usually root only)
    scales: Option<Vec<Keyframe<Vec2>>>,       // Optional (foreshortening)
}

Keyframe<T> {
    time: f32,                    // Seconds from animation start
    value: T,                     // f32 for rotation, Vec2 for translation/scale
    interpolation: Interpolation, // Linear, Step, CubicSpline
}
```

### RootMotion (`src/core/animation.rs`)

```rust
RootMotion {
    total_displacement: Vec2,  // Total movement over one cycle
    velocity: Vec2,            // Average velocity (units/sec)
    deltas: Vec<Vec2>,         // Per-frame deltas (empty = not computed)
}
```

## Mixamo Bone Naming Convention

**FBX (source of truth):** `mixamorig:Hips` (with colon separator)
- 65 bones for standard humanoid
- Clean hierarchy, no duplicates
- Full animation data preserved

**GLB (converted):** `mixamorigHips` (no colon, colon stripped during conversion)
- May have duplicate bone nodes (117 bones from 65 original)
- Animation data may be lost during FBX-to-GLB conversion
- Parser deduplicates by appending `.2`, `.3` suffixes

**Recommendation:** Use FBX files directly from Mixamo for best results.

## FBX Parsing (`src/fbx_parser/`) - RECOMMENDED

### FbxData (`src/fbx_parser/mod.rs`)

Wrapper for loaded FBX scene using ufbx.

```rust
let data = FbxData::load(path)?;       // From file
let data = FbxData::from_bytes(bytes)?; // From memory
```

### Skeleton Extraction (`src/fbx_parser/skeleton.rs`)

```rust
let nodes: Vec<GltfNode3D> = fbx_parser::extract_skeleton(&data)?;
let name = fbx_parser::get_skeleton_name(&data, "fallback");
```

Finds skeleton root (`mixamorig:Hips`) and extracts all bone nodes in topological order.

### Animation Extraction (`src/fbx_parser/animation.rs`)

```rust
let animations: Vec<GltfAnimation3D> = fbx_parser::extract_animations(&data)?;
```

Uses ufbx's `bake_anim()` to resample animations to 30 FPS linear keyframes.

## GLTF Parsing (`src/gltf_parser/`) - FALLBACK

### GltfData (`src/gltf_parser/mod.rs`)

Wrapper for loaded GLTF document and buffers.

```rust
let data = GltfData::load(path)?;       // From file
let data = GltfData::from_glb_bytes(bytes)?; // From memory
```

### Skeleton Extraction (`src/gltf_parser/skeleton.rs`)

```rust
let nodes: Vec<GltfNode3D> = gltf_parser::extract_skeleton(&data)?;
let name = gltf_parser::get_skeleton_name(&data, "fallback");
```

Finds skeleton root (looks for "mixamorig:Hips", "mixamorigHips", "Hips", etc.) and extracts all descendant bones in topological order.

**Duplicate Bone Names:** FBX-to-GLB conversions often produce duplicate bone names. The extractor automatically makes names unique by appending `.2`, `.3`, etc.

### Animation Extraction (`src/gltf_parser/animation.rs`)

```rust
let animations: Vec<GltfAnimation3D> = gltf_parser::extract_animations(&data)?;
```

Extracts all animations with translation, rotation, and scale channels. Note: GLB files converted from FBX may not contain animation data.

## Projection (`src/projection/`)

### ProjectionConfig (`src/projection/transform.rs`)

```rust
ProjectionConfig {
    view_axis: ViewAxis,       // X, Y, or Z (default: Z for side view)
    scale: f32,                // 3D to 2D scale (default: 100.0)
    enable_foreshortening: bool,
    sample_rate: f32,          // Output FPS (default: 30.0)
}
```

### 3D to 2D Conversion (`src/projection/transform.rs`)

```rust
let skeleton_2d = project_skeleton(&nodes, "name", &config)?;
let animation_2d = project_animation(&anim_3d, &nodes, "skeleton_name", &config)?;
```

Projection for side-scrolling games (ViewAxis::Z):
- 3D X → 2D X (horizontal)
- 3D Y → 2D Y (vertical)
- 3D Z → depth (foreshortening scale)
- 3D rotation → 2D angle (rotation around view axis)

### Root Motion (`src/projection/root_motion.rs`)

```rust
let root_motion = extract_root_motion(&anim_3d, root_node_index, &config);
strip_root_motion_horizontal(&mut anim_3d, root_node_index);
```

## Export (`src/export/`)

RON file I/O with validation:

```rust
save_skeleton(&skeleton, path)?;
save_animation(&animation, path)?;

let skeleton = load_skeleton(path)?;
let animation = load_animation(path, bone_count)?;
let animation = load_animation_for_skeleton(path, &skeleton)?;
```

## Compatibility Checking (`src/core/compatibility.rs`)

```rust
check_compatibility(skeleton, animation)?;        // Basic
check_strict_compatibility(skeleton, animation)?; // All bones must have timelines
```

## CLI Application (`src/bin/cli.rs`)

The `mixamo2d` CLI provides commands for converting Mixamo 3D animations to 2D.

### Installation

```bash
cargo install --features cli --path .
```

### Commands

#### Convert Single File

```bash
# Basic conversion
mixamo2d convert animation.fbx -o output/

# With options
mixamo2d convert animation.fbx -o output/ \
  --name "player" \
  --view-axis z \
  --scale 100 \
  --sample-rate 30 \
  -v
```

#### Batch Conversion

```bash
# Convert all FBX/GLB files in a directory
mixamo2d batch animations/ -o output/

# Recursive search
mixamo2d batch animations/ -o output/ -r
```

#### File Information

```bash
# Show skeleton and animation info
mixamo2d info animation.fbx

# Show only bones
mixamo2d info animation.fbx --bones

# Show only animations
mixamo2d info animation.fbx --animations
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--view-axis` | Camera view axis (x, y, z) | z |
| `--scale` | 3D to 2D scale factor | 100.0 |
| `--foreshortening` | Enable depth-based scaling | true |
| `--sample-rate` | Output animation FPS | 30.0 |
| `-v, --verbose` | Verbose output | false |

## Error Types (`src/error.rs`)

- `ValidationError` - Specific validation failures with context
- `Mixamo2dError` - Wrapper (validation, GLTF, IO, RON errors)

## Test Coverage

73 tests total:

**Core tests** (`tests/core/`, 53 tests):
- `skeleton_tests.rs` - Skeleton validation
- `animation_tests.rs` - Animation validation
- `compatibility_tests.rs` - Skeleton-animation pairing
- `edge_cases.rs` - Boundary conditions, serialization roundtrip

**FBX Integration tests** (`tests/fbx_integration_test.rs`, 10 tests):
- Load Mixamo FBX without panic
- Extract skeleton hierarchy (65 bones)
- Verify topological order
- Validate bone naming convention (`mixamorig:Hips` with colon)
- Extract animations (931 keyframes, 156 channels)
- Verify animation bone coverage (80%+)
- Project to valid 2D skeleton

**GLB Integration tests** (`tests/gltf_integration_test.rs`, 9 tests):
- Load converted GLB without panic
- Extract skeleton with deduplication (117 → 117 unique names)
- Handle missing animations gracefully
- Verify bone naming (`mixamorigHips` without colon)
