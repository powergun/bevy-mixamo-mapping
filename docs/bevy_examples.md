# Bevy Examples Guide

This document explains the example applications that demonstrate how to use `bevy_mixamo_2d` in Bevy games.

## Prerequisites

Before running examples, you need animation assets in RON format. Convert Mixamo FBX files using the CLI:

```bash
# Convert a single animation
cargo run --features cli -- convert path/to/animation.fbx -o assets/

# Convert all animations in a directory
cargo run --features cli -- batch path/to/animations/ -o assets/ -r
```

This generates:
- `{name}.skeleton2d.ron` - Skeleton definition
- `{animation_name}.anim2d.ron` - Animation clip(s)

---

## Example: play_animation

**Location:** `examples/play_animation.rs`

An interactive example demonstrating skeleton loading, animation playback, and debug visualization.

### Running the Example

```bash
# First, generate test assets (if not already done)
cargo run --features cli -- convert testdata/Sprint.fbx -o assets/

# Run the example
cargo run --example play_animation
```

### Controls

| Key | Action |
|-----|--------|
| Space | Play/Pause animation |
| R | Reset animation to beginning |
| Left Arrow | Seek backward 0.1 seconds |
| Right Arrow | Seek forward 0.1 seconds |
| Up Arrow | Increase speed by 0.25x |
| Down Arrow | Decrease speed by 0.25x |
| F3 | Toggle debug visualization |

### Code Walkthrough

#### 1. Plugin Setup

```rust
use bevy::prelude::*;
use bevy_mixamo_2d::bevy_plugin::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Mixamo2DPlugin)          // Add the animation plugin
        .insert_resource(SkeletonDebugEnabled(true))  // Enable debug drawing
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input, update_ui))
        .run();
}
```

The `Mixamo2DPlugin` registers:
- Asset loaders for `.skeleton2d.ron` and `.anim2d.ron`
- Animation playback systems
- Debug visualization system

#### 2. Loading Assets

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Load skeleton and animation assets
    let skeleton_handle: Handle<Skeleton2DAsset> =
        asset_server.load("Sprint.skeleton2d.ron");
    let animation_handle: Handle<Animation2DAsset> =
        asset_server.load("mixamo_com.anim2d.ron");

    // Spawn skeleton entity
    commands.spawn((
        SkeletonBundle {
            skeleton_instance: SkeletonInstance::new(skeleton_handle),
            transform: Transform::from_xyz(0.0, -200.0, 0.0)
                .with_scale(Vec3::splat(0.02)),  // Scale down (Mixamo skeletons are large)
            ..default()
        },
        SkeletonDebugConfig::default(),  // Enable per-skeleton debug drawing
        AnimationToLoad(animation_handle),  // Custom component to track animation to play
    ));
}
```

Key points:
- Assets are loaded asynchronously via `AssetServer`
- `SkeletonBundle` includes all required components
- Scale is applied to fit the skeleton on screen
- `SkeletonDebugConfig` enables bone/shape visualization

#### 3. Starting Animation Playback

```rust
#[derive(Component)]
struct AnimationToLoad(Handle<Animation2DAsset>);

fn keyboard_input(
    mut skeleton_query: Query<(&mut AnimationPlayer2D, Option<&AnimationToLoad>)>,
    animations: Res<Assets<Animation2DAsset>>,
    // ...
) {
    for (mut player, animation_to_load) in skeleton_query.iter_mut() {
        // Start animation when asset is ready
        if player.current_animation.is_none() {
            if let Some(anim_to_load) = animation_to_load {
                if animations.get(&anim_to_load.0).is_some() {
                    player.play(anim_to_load.0.clone());
                }
            }
        }
        // ... handle keyboard input
    }
}
```

Animation playback starts when:
1. The animation asset has finished loading
2. No animation is currently playing
3. We call `player.play(handle)`

#### 4. Playback Control

```rust
// Play/Pause toggle
if keys.just_pressed(KeyCode::Space) {
    if player.is_playing() {
        player.pause();
    } else if player.is_paused() {
        player.resume();
    } else {
        // Restart if stopped
        let current = player.current_animation.clone();
        if let Some(handle) = current {
            player.play(handle);
        }
    }
}

// Speed control
if keys.just_pressed(KeyCode::ArrowUp) {
    let new_speed = (player.speed + 0.25).min(3.0);
    player.set_speed(new_speed);
}

// Seeking
if keys.just_pressed(KeyCode::ArrowRight) {
    let new_time = player.elapsed() + 0.1;
    player.seek(new_time);
}
```

#### 5. Debug Visualization Toggle

```rust
fn keyboard_input(
    mut debug_enabled: ResMut<SkeletonDebugEnabled>,
    // ...
) {
    if keys.just_pressed(KeyCode::F3) {
        debug_enabled.0 = !debug_enabled.0;
    }
}
```

#### 6. UI Status Display

```rust
fn update_ui(
    skeleton_query: Query<(&SkeletonInstance, &AnimationPlayer2D)>,
    skeletons: Res<Assets<Skeleton2DAsset>>,
    animations: Res<Assets<Animation2DAsset>>,
    // ...
) {
    // Display asset loading status
    let skeleton_status = if skeletons.get(&instance.skeleton).is_some() {
        "Loaded"
    } else {
        "Loading..."
    };

    // Display playback state
    let state = match player.state {
        PlaybackState::Playing => "Playing",
        PlaybackState::Paused => "Paused",
        PlaybackState::Stopped => "Stopped",
    };

    // Display timing info
    format!("Time: {:.2}s / {:.2}s", player.elapsed(), player.duration());
}
```

---

## Common Patterns

### Pattern 1: Multiple Animations

Load and switch between multiple animations:

```rust
#[derive(Resource)]
struct AnimationLibrary {
    walk: Handle<Animation2DAsset>,
    run: Handle<Animation2DAsset>,
    jump: Handle<Animation2DAsset>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AnimationLibrary {
        walk: asset_server.load("walk.anim2d.ron"),
        run: asset_server.load("run.anim2d.ron"),
        jump: asset_server.load("jump.anim2d.ron"),
    });
}

fn switch_animation(
    keys: Res<ButtonInput<KeyCode>>,
    library: Res<AnimationLibrary>,
    mut query: Query<&mut AnimationPlayer2D>,
) {
    for mut player in query.iter_mut() {
        if keys.just_pressed(KeyCode::Digit1) {
            player.play(library.walk.clone());
        } else if keys.just_pressed(KeyCode::Digit2) {
            player.play(library.run.clone());
        } else if keys.just_pressed(KeyCode::Digit3) {
            player.play(library.jump.clone());
        }
    }
}
```

### Pattern 2: Animation Events

React to animation completion:

```rust
fn check_animation_finished(
    mut query: Query<(&AnimationPlayer2D, &mut AnimationState)>,
) {
    for (player, mut state) in query.iter_mut() {
        if player.is_finished() {
            // Non-looping animation completed
            *state = AnimationState::Idle;
        }
    }
}
```

### Pattern 3: Character Controller Integration

Combine with movement:

```rust
fn update_character(
    keys: Res<ButtonInput<KeyCode>>,
    library: Res<AnimationLibrary>,
    mut query: Query<(&mut AnimationPlayer2D, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        let mut moving = false;

        if keys.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 100.0 * time.delta_secs();
            moving = true;
        }

        // Switch to walk animation when moving
        if moving && !player.is_playing() {
            player.play(library.walk.clone());
        } else if !moving && player.is_playing() {
            player.stop();
        }
    }
}
```

### Pattern 4: Scaling for Screen

Mixamo skeletons use centimeter units and are typically 15,000-20,000 units tall. Scale appropriately:

```rust
// Option 1: Scale in transform
commands.spawn(SkeletonBundle {
    transform: Transform::from_scale(Vec3::splat(0.01)),  // 1/100 scale
    ..default()
});

// Option 2: Use projection config when converting
// cargo run --features cli -- convert anim.fbx --scale 1.0  // Instead of default 100.0
```

### Pattern 5: Multiple Skeletons

Spawn multiple characters with the same skeleton:

```rust
fn spawn_characters(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let skeleton = asset_server.load("character.skeleton2d.ron");
    let walk = asset_server.load("walk.anim2d.ron");

    for i in 0..5 {
        let mut player = AnimationPlayer2D::default();
        // Stagger animation start times
        player.play(walk.clone());
        player.seek(i as f32 * 0.2);

        commands.spawn((
            SkeletonBundle {
                skeleton_instance: SkeletonInstance::new(skeleton.clone()),
                animation_player: player,
                transform: Transform::from_xyz(i as f32 * 100.0, 0.0, 0.0),
                ..default()
            },
        ));
    }
}
```

---

## Debugging Tips

### 1. Enable Debug Visualization

```rust
// Global toggle
app.insert_resource(SkeletonDebugEnabled(true));

// Per-skeleton config
commands.spawn((
    SkeletonBundle { ... },
    SkeletonDebugConfig {
        draw_bones: true,
        draw_shapes: true,
        bone_color: Color::srgb(0.0, 1.0, 0.0),
        shape_color: Color::srgb(1.0, 1.0, 0.0),
        bone_thickness: 2.0,
    },
));
```

### 2. Check Asset Loading Status

```rust
fn debug_assets(
    skeletons: Res<Assets<Skeleton2DAsset>>,
    animations: Res<Assets<Animation2DAsset>>,
    query: Query<(&SkeletonInstance, &AnimationPlayer2D)>,
) {
    for (instance, player) in query.iter() {
        if skeletons.get(&instance.skeleton).is_none() {
            println!("Skeleton still loading...");
        }
        if let Some(ref handle) = player.current_animation {
            if animations.get(handle).is_none() {
                println!("Animation still loading...");
            }
        }
    }
}
```

### 3. Inspect Bone Hierarchy

```rust
fn print_bone_hierarchy(
    query: Query<(&SkeletonInstance, &Children)>,
    bone_query: Query<&BoneEntity>,
    children_query: Query<&Children>,
) {
    for (instance, children) in query.iter() {
        if !instance.initialized {
            continue;
        }
        println!("Skeleton bones:");
        print_bones(children, &bone_query, &children_query, 0);
    }
}

fn print_bones(
    children: &Children,
    bone_query: &Query<&BoneEntity>,
    children_query: &Query<&Children>,
    depth: usize,
) {
    for child in children.iter() {
        if let Ok(bone) = bone_query.get(child) {
            println!("{:indent$}[{}] {}", "", bone.bone_index, bone.name, indent = depth * 2);
            if let Ok(grandchildren) = children_query.get(child) {
                print_bones(grandchildren, bone_query, children_query, depth + 1);
            }
        }
    }
}
```

---

## Performance Considerations

1. **Asset Loading**: Load assets in advance; avoid loading during gameplay
2. **Animation Complexity**: Animations with many keyframes use more memory
3. **Debug Visualization**: Disable `SkeletonDebugEnabled` in release builds
4. **Many Skeletons**: The animation system uses change detection to minimize work

---

## Troubleshooting

### Skeleton Not Visible

- Check transform scale (Mixamo skeletons are large)
- Ensure camera is positioned correctly for 2D view
- Enable debug visualization to see bones

### Animation Not Playing

- Verify animation asset loaded (check asset events)
- Check skeleton/animation name compatibility
- Call `player.play(handle)` to start playback

### Bones in Wrong Position

- Verify skeleton and animation are compatible
- Check that animation targets the correct skeleton name
- Ensure transform hierarchy is correct

### Performance Issues

- Reduce keyframe density during conversion (lower sample rate)
- Disable debug visualization in production
- Profile to identify bottlenecks
