# Bevy Integration Reference

This document describes the Bevy plugin for `bevy_mixamo_2d`, which enables runtime loading and playback of 2D skeletal animations in Bevy games.

## Overview

The Bevy plugin bridges the gap between the core data layer (Phase 1) and runtime game usage. It provides:

- **Asset Loaders**: Load `.skeleton2d.ron` and `.anim2d.ron` files as Bevy assets
- **ECS Components**: Manage skeleton instances and animation playback
- **Animation Systems**: Update poses and transforms each frame
- **Debug Visualization**: Draw skeletons using gizmos for development

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Bevy Plugin Layer                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐   ┌──────────────────┐   ┌─────────────────────────┐   │
│  │   Assets    │   │    Components    │   │        Systems          │   │
│  ├─────────────┤   ├──────────────────┤   ├─────────────────────────┤   │
│  │ Skeleton2D  │   │ SkeletonInstance │   │ advance_animation_time  │   │
│  │   Asset     │──▶│ AnimationPlayer  │──▶│ initialize_hierarchy    │   │
│  │             │   │     Pose2D       │   │ sample_animations       │   │
│  │ Animation2D │   │   BoneEntity     │   │ apply_poses_to_transform│   │
│  │   Asset     │   │ SkeletonBundle   │   │ draw_skeleton_debug     │   │
│  └─────────────┘   └──────────────────┘   └─────────────────────────┘   │
│         │                   │                        │                  │
└─────────┼───────────────────┼────────────────────────┼──────────────────┘
          │                   │                        │
          ▼                   ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Core Data Layer                                 │
│   Skeleton2D, Animation2D, Bone2D, Transform2D, BoneTimeline, etc.      │
└─────────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/bevy_plugin/
├── mod.rs          # Plugin registration, public API
├── assets.rs       # Asset types and loaders
├── components.rs   # ECS components and bundles
├── systems.rs      # Animation playback systems
└── primitives.rs   # Debug visualization
```

All code is gated behind `#[cfg(feature = "visualization")]`.

## Feature Flag

The plugin requires the `visualization` feature (enabled by default):

```toml
[dependencies]
bevy_mixamo_2d = "0.1"  # visualization feature is default

# Or explicitly:
bevy_mixamo_2d = { version = "0.1", features = ["visualization"] }
```

---

## Asset Types

### Skeleton2DAsset

Wrapper around the core `Skeleton2D` type for Bevy's asset system.

```rust
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Skeleton2DAsset(pub Skeleton2D);
```

**Loading:**
```rust
let handle: Handle<Skeleton2DAsset> = asset_server.load("player.skeleton2d.ron");
```

**File extension:** `.skeleton2d.ron`

**Validation:** The loader validates the skeleton on load, rejecting invalid files with descriptive errors.

### Animation2DAsset

Wrapper around the core `Animation2D` type.

```rust
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Animation2DAsset(pub Animation2D);
```

**Loading:**
```rust
let handle: Handle<Animation2DAsset> = asset_server.load("walk.anim2d.ron");
```

**File extension:** `.anim2d.ron`

**Partial Validation:** The loader validates what it can without knowing the target skeleton (duration, sample rate, keyframe ordering). Full compatibility validation happens at runtime when paired with a skeleton.

---

## ECS Components

### SkeletonInstance

Links an entity to a skeleton asset and manages bone entity hierarchy.

```rust
#[derive(Component)]
pub struct SkeletonInstance {
    pub skeleton: Handle<Skeleton2DAsset>,
    pub initialized: bool,
}
```

When the skeleton asset loads, the system automatically spawns child entities for each bone.

**Usage:**
```rust
commands.spawn(SkeletonInstance::new(skeleton_handle));
```

### AnimationPlayer2D

Controls animation playback on a skeleton.

```rust
#[derive(Component)]
pub struct AnimationPlayer2D {
    pub current_animation: Option<Handle<Animation2DAsset>>,
    pub time: f32,
    pub speed: f32,
    pub state: PlaybackState,
    // ... internal fields
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}
```

**API:**

| Method | Description |
|--------|-------------|
| `play(handle)` | Start animation from beginning |
| `pause()` | Pause at current position |
| `resume()` | Resume from paused position |
| `stop()` | Stop and reset to beginning |
| `set_speed(speed)` | Set playback speed (1.0 = normal) |
| `seek(time)` | Jump to specific time |
| `elapsed()` | Get current playback time |
| `duration()` | Get animation duration |
| `is_playing()` | Check if currently playing |
| `is_finished()` | Check if non-looping animation completed |

**Example:**
```rust
fn control_animation(mut query: Query<&mut AnimationPlayer2D>) {
    for mut player in query.iter_mut() {
        player.play(animation_handle.clone());
        player.set_speed(0.5);  // Half speed
    }
}
```

### Pose2D

Current pose state for a bone, updated each frame by the animation system.

```rust
#[derive(Component)]
pub struct Pose2D {
    pub rotation: f32,      // Radians
    pub translation: Vec2,  // Local translation
    pub scale: Vec2,        // Local scale
}
```

This component is attached to bone entities (children of the skeleton entity).

### BoneEntity

Marker component for bone entities with metadata.

```rust
#[derive(Component)]
pub struct BoneEntity {
    pub bone_index: usize,
    pub name: String,
    pub length: f32,
}
```

### SkeletonDebugConfig

Per-skeleton configuration for debug visualization.

```rust
#[derive(Component)]
pub struct SkeletonDebugConfig {
    pub draw_bones: bool,
    pub draw_shapes: bool,
    pub bone_color: Color,
    pub shape_color: Color,
    pub bone_thickness: f32,
}
```

**Default:**
- `draw_bones: true`
- `draw_shapes: true`
- `bone_color: green`
- `shape_color: yellow`
- `bone_thickness: 2.0`

---

## Bundles

### SkeletonBundle

Convenience bundle for spawning animated skeleton entities.

```rust
#[derive(Bundle, Default)]
pub struct SkeletonBundle {
    pub skeleton_instance: SkeletonInstance,
    pub animation_player: AnimationPlayer2D,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
```

**Usage:**
```rust
commands.spawn((
    SkeletonBundle {
        skeleton_instance: SkeletonInstance::new(skeleton_handle),
        transform: Transform::from_xyz(100.0, 200.0, 0.0),
        ..default()
    },
    SkeletonDebugConfig::default(),  // Optional: enable debug drawing
));
```

---

## Systems

The plugin registers systems in the following schedule:

### PreUpdate

**`advance_animation_time`**
- Advances `AnimationPlayer2D.time` based on delta time and speed
- Handles looping (wraps time) and non-looping (stops at end)
- Updates cached duration/looping from animation asset

### Update (chained)

**1. `initialize_skeleton_hierarchy`**
- Triggered when `SkeletonInstance` changes
- Waits for skeleton asset to load
- Spawns child bone entities with:
  - `BoneEntity` marker
  - `Pose2D` initialized to setup pose
  - `Transform` from setup pose
  - Proper parent-child hierarchy

**2. `sample_animations`**
- Gets skeleton and animation assets
- Validates compatibility (skeleton names must match)
- Samples animation at current time for each bone
- Updates `Pose2D` components
- Falls back to setup pose for bones without animation timelines

**3. `apply_poses_to_transforms`**
- Converts `Pose2D` to `Transform` components
- Only runs when `Pose2D` changes (change detection)

### PostUpdate

**`draw_skeleton_debug`**
- Draws bone lines using `gizmos.line_2d()`
- Draws primitive shapes (Circle, Rectangle, Triangle)
- Respects `SkeletonDebugEnabled` resource and per-skeleton `SkeletonDebugConfig`

---

## Debug Visualization

### Global Toggle

```rust
#[derive(Resource, Default)]
pub struct SkeletonDebugEnabled(pub bool);
```

**Usage:**
```rust
// Enable in app setup
app.insert_resource(SkeletonDebugEnabled(true));

// Toggle at runtime
fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<SkeletonDebugEnabled>) {
    if keys.just_pressed(KeyCode::F3) {
        debug.0 = !debug.0;
    }
}
```

### Per-Skeleton Configuration

Add `SkeletonDebugConfig` to skeleton entities for custom visualization:

```rust
commands.spawn((
    SkeletonBundle { ... },
    SkeletonDebugConfig {
        draw_bones: true,
        draw_shapes: false,  // Only show bones
        bone_color: Color::srgb(1.0, 0.0, 0.0),  // Red bones
        ..default()
    },
));
```

---

## Plugin Registration

### Mixamo2DPlugin

The main plugin that ties everything together.

```rust
pub struct Mixamo2DPlugin;

impl Plugin for Mixamo2DPlugin {
    fn build(&self, app: &mut App) {
        // Register asset types
        app.init_asset::<Skeleton2DAsset>()
            .init_asset::<Animation2DAsset>();

        // Register asset loaders
        app.register_asset_loader(Skeleton2DAssetLoader)
            .register_asset_loader(Animation2DAssetLoader);

        // Register components for reflection
        app.register_type::<SkeletonInstance>()
            .register_type::<AnimationPlayer2D>()
            // ... etc

        // Initialize resources
        app.init_resource::<SkeletonDebugEnabled>();

        // Add systems
        app.add_systems(PreUpdate, advance_animation_time);
        app.add_systems(Update, (
            initialize_skeleton_hierarchy,
            sample_animations,
            apply_poses_to_transforms,
        ).chain());
        app.add_systems(PostUpdate, draw_skeleton_debug);
    }
}
```

**Usage:**
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Mixamo2DPlugin)
        .run();
}
```

---

## Entity Hierarchy

When a skeleton is initialized, the plugin creates this entity structure:

```
Skeleton Entity (user-spawned)
├── SkeletonInstance
├── AnimationPlayer2D
├── Transform
└── Children:
    └── Bone Entity (hips - root bone)
        ├── BoneEntity { bone_index: 0, name: "hips" }
        ├── Pose2D
        ├── Transform
        └── Children:
            ├── Bone Entity (spine)
            │   ├── BoneEntity { bone_index: 1, name: "spine" }
            │   └── Children:
            │       └── Bone Entity (head)
            │           └── ...
            ├── Bone Entity (leg_left)
            │   └── ...
            └── Bone Entity (leg_right)
                └── ...
```

The hierarchy matches the skeleton's bone parent-child relationships, enabling Bevy's transform propagation to work correctly.

---

## Error Handling

### Asset Loading Errors

Asset loaders return typed errors:

```rust
pub enum Skeleton2DAssetLoaderError {
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
    Validation(ValidationError),
}

pub enum Animation2DAssetLoaderError {
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
    Validation(ValidationError),
}
```

Failed loads will appear in Bevy's asset error logs.

### Runtime Compatibility

The animation system silently skips incompatible skeleton/animation pairs:
- Skeleton name mismatch: animation's `skeleton_name` != skeleton's `name`
- Missing bone timelines: falls back to setup pose

---

## Best Practices

1. **Load Assets Early**: Load skeleton and animation assets in `Startup` or preload systems
2. **Use SkeletonBundle**: Includes all required components with sensible defaults
3. **Add Debug Config**: Include `SkeletonDebugConfig` during development
4. **Check Asset Loading**: Use Bevy's asset events to know when assets are ready
5. **Scale Appropriately**: Mixamo skeletons are large; apply scale in `Transform`

---

## Dependencies

The plugin uses these Bevy features:

```toml
bevy = { version = "0.17", features = [
    "bevy_asset",      # Asset loading
    "bevy_render",     # Rendering
    "bevy_sprite",     # 2D sprites
    "bevy_gizmos",     # Debug visualization
    "bevy_winit",      # Windowing
    "bevy_core_pipeline",
    "multi_threaded",
] }
```
