# Architecture Overview

This document provides a high-level architecture guide for `bevy_mixamo_2d`, a Rust library that converts Mixamo 3D animations to 2D skeletal animations for Bevy game engine.

**Target audience:** Coding agents and developers who need to understand the project structure quickly.

## Project Purpose

Convert Mixamo animations (FBX/GLB) → 2D skeleton + animation data (RON) → Runtime playback in Bevy.

## Module Structure

```
src/
├── core/           ← Foundation: validated data structures
├── fbx_parser/     ← Input: FBX file parsing (preferred)
├── gltf_parser/    ← Input: GLB/GLTF file parsing (fallback)
├── projection/     ← Transform: 3D → 2D conversion
├── export/         ← Output: RON file serialization
├── bevy_plugin/    ← Runtime: Bevy integration (optional feature)
├── error.rs        ← Error types
└── bin/cli.rs      ← CLI tool (optional feature)
```

### Dependency Flow

```
                     ┌─────────────────────────┐
                     │      Core Data Layer    │
                     │  Skeleton2D, Animation2D│
                     └───────────┬─────────────┘
                                 │
       ┌──────────┬──────────────┼──────────────┬──────────────┐
       ▼          ▼              ▼              ▼              ▼
   fbx_parser  gltf_parser   projection     export      bevy_plugin
   (input)      (input)      (transform)   (output)     (runtime)
```

All modules depend on `core/`. The core layer has no external dependencies beyond glam and serde.

## Core Design Principle: Data-First with Validation

The `Skeleton2D` and `Animation2D` types are the foundation. They enforce invariants at construction time via `::new()` methods that return `Result<T, ValidationError>`.

**Key invariants enforced:**
- Non-empty bone/timeline lists
- Unique bone names
- Single root bone (parent = None)
- Topological order (parent index < child index)
- Keyframes sorted by time, within duration bounds
- Positive duration, sample rate, bone length, scale

See: [`walkthrough/data-models.md`](walkthrough/data-models.md) for complete type documentation.

## Data Flow

### Conversion Pipeline (CLI)

```
FBX/GLB file
    │
    ▼
fbx_parser::extract_skeleton() / gltf_parser::extract_skeleton()
    │  → Vec<GltfNode3D> (3D bone hierarchy)
    ▼
projection::project_skeleton()
    │  → Skeleton2D (validated)
    ▼
export::save_skeleton()
    │  → *.skeleton2d.ron
    ▼
fbx_parser::extract_animations() / gltf_parser::extract_animations()
    │  → Vec<GltfAnimation3D>
    ▼
projection::project_animation()
    │  → Animation2D (validated)
    ▼
export::save_animation()
        → *.anim2d.ron
```

### Runtime Pipeline (Bevy Plugin)

```
*.skeleton2d.ron / *.anim2d.ron
    │
    ▼
Skeleton2DAssetLoader / Animation2DAssetLoader
    │  → Handle<Skeleton2DAsset> / Handle<Animation2DAsset>
    ▼
SkeletonInstance + AnimationPlayer2D (components)
    │
    ▼
animation_system (samples keyframes → Pose2D)
    │
    ▼
skeleton_render_system (transforms bone entities)
```

See: [`walkthrough/bevy_integration.md`](walkthrough/bevy_integration.md) for plugin usage.

## Key Source Files

| Purpose | File | Key Types/Functions |
|---------|------|---------------------|
| 2D skeleton | `src/core/skeleton.rs` | `Skeleton2D`, `Bone2D` |
| 2D animation | `src/core/animation.rs` | `Animation2D`, `RootMotion`, `SampledPose` |
| Keyframes | `src/core/keyframe.rs` | `Keyframe<T>`, `BoneTimeline`, `Interpolation` |
| Transforms | `src/core/bone.rs` | `Transform2D`, `PrimitiveShape` |
| Compatibility | `src/core/compatibility.rs` | `check_compatibility()`, `check_strict_compatibility()` |
| FBX input | `src/fbx_parser/skeleton.rs` | `extract_skeleton()`, `extract_animations()` |
| FBX traversal | `src/fbx_parser/traversal.rs` | `traverse_skeleton_hierarchy()`, `SkeletonHierarchy`, `FormattedBone` |
| Bone filtering | `src/fbx_parser/filter.rs` | `BoneFilter`, `filter_nodes()` |
| GLB input | `src/gltf_parser/skeleton.rs` | `extract_skeleton()`, `extract_animations()` |
| 3D→2D projection | `src/projection/transform.rs` | `project_skeleton()`, `project_animation()`, `ProjectionConfig` |
| Root motion | `src/projection/root_motion.rs` | `extract_root_motion()`, `strip_root_motion_horizontal()` |
| Serialization | `src/export/ron.rs` | `save_skeleton()`, `load_skeleton()`, `save_animation()`, `load_animation()` |
| Error types | `src/error.rs` | `Mixamo2dError`, `ValidationError` |
| Bevy plugin | `src/bevy_plugin/mod.rs` | `Mixamo2DPlugin` |
| Bevy components | `src/bevy_plugin/components.rs` | `SkeletonInstance`, `AnimationPlayer2D`, `Pose2D` |
| Bevy systems | `src/bevy_plugin/systems.rs` | `animation_system`, `skeleton_render_system` |

## Feature Flags

```toml
[features]
default = ["visualization"]
visualization = ["dep:bevy"]   # Bevy plugin
cli = ["dep:clap"]              # CLI tool
```

## FBX vs GLB: When to Use

Use FBX as much as we can because it is a standard format in the DCC pipelines.
Consider GLB a fallback (and a last resort) when FBX is not available.
We may remove the GLB data source support in the future.

When develop new functionality and writing test, prioritize FBX as the golden
data source, but also ensure the existing GLB tests (unit tests and integration tests)
can pass.

| Format | Pros | Cons |
|--------|------|------|
| **FBX** (recommended) | Clean hierarchy (65 bones), full animation data, native Mixamo format | Requires ufbx dependency |
| **GLB** (fallback) | Standard glTF format | May have duplicate bones (117), animation data often lost in conversion |

Bone naming conventions:
- FBX: `mixamorig:Hips` (with colon)
- GLB: `mixamorigHips` (colon stripped during conversion)

## Testing Strategy

Tests are organized in `tests/` directory:

```
tests/
├── core/                          # Unit tests for core data layer
│   ├── skeleton_tests.rs          # Skeleton validation (17 tests)
│   ├── animation_tests.rs         # Animation validation (18 tests)
│   ├── compatibility_tests.rs     # Skeleton-animation pairing (7 tests)
│   └── edge_cases.rs              # Boundary conditions (11 tests)
├── fbx_integration_test.rs        # FBX parsing integration (10 tests)
├── skeleton_traversal_test.rs     # Skeleton hierarchy traversal (11 tests)
├── bone_filter_integration_test.rs # Bone filtering with real FBX (13 tests)
├── gltf_integration_test.rs       # GLB parsing integration (9 tests)
├── bevy_plugin_tests.rs           # Bevy plugin tests
└── debug_*.rs                     # Debugging utilities
```

**Total: 124+ tests** (including 27 bone filter unit tests)

### Running Tests

```bash
# All tests
cargo test

# Core data layer only
cargo test --test core_tests

# FBX integration
cargo test --test fbx_integration_test

# GLB integration
cargo test --test gltf_integration_test

# Bevy plugin (requires visualization feature)
cargo test --test bevy_plugin_tests --features visualization
```

### Test Data

Test fixtures are in `testdata/`:
- `testdata/simple_standing.fbx` - Mixamo standing animation (FBX)
- `testdata/simple_standing.glb` - Same animation converted to GLB

## Development Guidelines

1. **Validate at construction**: All `::new()` methods must validate inputs and return `Result`. Never create invalid `Skeleton2D` or `Animation2D`.

2. **Fail fast with context**: `ValidationError` variants include relevant data (bone names, indices, values) for debugging.

3. **Defense in depth**: Re-validate after deserialization from RON files (files may be hand-edited).

4. **Topological order invariant**: Bones array must have parents before children. This enables single-pass forward kinematics.

5. **Single root invariant**: Exactly one bone has `parent = None`. This ensures proper tree structure.

6. **Prefer FBX**: When both formats are available, use FBX for cleaner data.

## Visual Conventions

### Left/Right Color Coding

When visually distinguishing left-side elements from right-side elements (e.g., bones, limbs, debug markers), use the following color scheme consistently across the project:

| Side | Color | RGB Value | Usage |
|------|-------|-----------|-------|
| **Left** | Red | `(1.0, 0.2, 0.2)` | Elements with "Left" or "left" in their name |
| **Right** | Blue | `(0.2, 0.2, 1.0)` | Elements with "Right" or "right" in their name |
| **Center/Other** | Green | `(0.2, 0.8, 0.2)` | Elements that are neither left nor right (spine, head, hips) |

**Note:** Left/right refers to the character's perspective, not the viewer's. When viewing a character from the front:
- Character's left arm (red) appears on the viewer's right
- Character's right arm (blue) appears on the viewer's left

**Implementation:** Use `BoneColorMode::left_right()` in `SkeletonDebugConfig` for bone visualization:

```rust
SkeletonDebugConfig {
    bone_color: Color::srgb(0.2, 0.8, 0.2),  // Green default
    bone_color_mode: BoneColorMode::left_right(
        Color::srgb(1.0, 0.2, 0.2),  // Red for left
        Color::srgb(0.2, 0.2, 1.0),  // Blue for right
    ),
    ..default()
}
```

## Data Organization

We organize the data files in this project following this convention:

| Directory | Purpose | Tracked in Git | Dependencies Allowed |
|-----------|---------|----------------|---------------------|
| `assets/` | Reusable, immutable assets only | Yes | Source code may reference |
| `ws/` | Generated data across sessions | No (.gitignore) | No source/test dependencies |
| `.tmp/` | Temporary/scratch data | No (.gitignore) | None (can be trashed) |
| `testdata/` | Immutable test fixtures | Yes | Integration tests only |

### Detailed Guidelines

**`assets/`** - Reusable, immutable assets only; **not for generated data**.
- Example: reference character models, shared textures
- Files here are part of the project distribution

**`ws/`** (workspace) - Generated data that may live across multiple sessions.
- Example: converted RON files, debug outputs
- Should not be depended on by source code or test code
- Useful for manual inspection and debugging

**`.tmp/`** - Temporary data that can be trashed at any time.
- There should be **absolutely no dependencies** on files here
- Exception: a single test case may create and read a file here for verification
- May contain git-cloned repositories for short-lived reference during coding sessions
- Prefix with `.` to hide from casual directory listings

**`testdata/`** - Immutable, shared data sources for integration tests.
- Use minimalism to keep these files small
- Files in this directory should be treated as project files
- Organized by format: `testdata/fbx/`, `testdata/*.fbx` for FBX files

## Further Reading

| Document | Content |
|----------|---------|
| [`spec.md`](../spec.md) | Original project specification |
| [`walkthrough/data-models.md`](walkthrough/data-models.md) | Complete type reference |
| [`walkthrough/bevy_integration.md`](walkthrough/bevy_integration.md) | Bevy plugin usage |
| [`walkthrough/bevy_examples.md`](walkthrough/bevy_examples.md) | Example code |
| [`walkthrough/cli.md`](walkthrough/cli.md) | CLI tool documentation |
| [`walkthrough/serialization.md`](walkthrough/serialization.md) | RON file format |
