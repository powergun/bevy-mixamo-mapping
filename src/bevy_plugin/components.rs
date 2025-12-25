//! ECS components for 2D skeletal animation
//!
//! Provides components for skeleton instances, animation playback, and bone poses.

use super::assets::{Animation2DAsset, Skeleton2DAsset};
use bevy::prelude::*;
use glam::Vec2;

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Links an entity to a [`Skeleton2DAsset`] and manages the bone entity hierarchy.
///
/// When the skeleton asset loads, child entities are spawned for each bone.
///
/// # Example
/// ```ignore
/// commands.spawn(SkeletonBundle {
///     skeleton_instance: SkeletonInstance {
///         skeleton: asset_server.load("player.skeleton2d.ron"),
///         ..default()
///     },
///     ..default()
/// });
/// ```
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct SkeletonInstance {
    /// Handle to the skeleton asset
    pub skeleton: Handle<Skeleton2DAsset>,
    /// Whether the bone hierarchy has been initialized
    pub initialized: bool,
}

impl SkeletonInstance {
    /// Create a new skeleton instance with the given asset handle
    pub fn new(skeleton: Handle<Skeleton2DAsset>) -> Self {
        Self {
            skeleton,
            initialized: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Playback state for an animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum PlaybackState {
    /// Animation is stopped and reset to the beginning
    #[default]
    Stopped,
    /// Animation is actively playing
    Playing,
    /// Animation is paused at the current time
    Paused,
}

/// Controls animation playback on a skeleton instance.
///
/// Attach this component alongside a [`SkeletonInstance`] to enable animation.
///
/// # Example
/// ```ignore
/// // Start playing an animation
/// player.play(animation_handle.clone());
///
/// // Control playback
/// player.pause();
/// player.resume();
/// player.set_speed(0.5); // Half speed
/// player.seek(0.5);      // Jump to 0.5 seconds
/// ```
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct AnimationPlayer2D {
    /// Currently playing animation (if any)
    pub current_animation: Option<Handle<Animation2DAsset>>,
    /// Playback time in seconds (wraps for looping animations)
    pub time: f32,
    /// Playback speed multiplier (1.0 = normal, 0.5 = half speed, -1.0 = reverse)
    pub speed: f32,
    /// Current playback state
    pub state: PlaybackState,
    /// Cached animation duration (set when animation loads)
    cached_duration: f32,
    /// Cached looping flag
    cached_looping: bool,
}

impl Default for AnimationPlayer2D {
    fn default() -> Self {
        Self {
            current_animation: None,
            time: 0.0,
            speed: 1.0,
            state: PlaybackState::Stopped,
            cached_duration: 0.0,
            cached_looping: false,
        }
    }
}

impl AnimationPlayer2D {
    /// Start playing an animation from the beginning
    pub fn play(&mut self, animation: Handle<Animation2DAsset>) -> &mut Self {
        self.current_animation = Some(animation);
        self.time = 0.0;
        self.state = PlaybackState::Playing;
        // Duration will be cached when animation asset is available
        self.cached_duration = 0.0;
        self
    }

    /// Pause playback at the current position
    pub fn pause(&mut self) -> &mut Self {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
        self
    }

    /// Resume playback from the current position
    pub fn resume(&mut self) -> &mut Self {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing;
        }
        self
    }

    /// Stop playback and reset to the beginning
    pub fn stop(&mut self) -> &mut Self {
        self.state = PlaybackState::Stopped;
        self.time = 0.0;
        self
    }

    /// Set playback speed (1.0 = normal, 0.5 = half, 2.0 = double, -1.0 = reverse)
    pub fn set_speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    /// Seek to a specific time in the animation
    pub fn seek(&mut self, time: f32) -> &mut Self {
        self.time = time.max(0.0);
        if self.cached_duration > 0.0 {
            if self.cached_looping {
                self.time = self.time % self.cached_duration;
            } else {
                self.time = self.time.min(self.cached_duration);
            }
        }
        self
    }

    /// Check if the animation is currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Check if the animation is paused
    pub fn is_paused(&self) -> bool {
        self.state == PlaybackState::Paused
    }

    /// Check if the animation is stopped
    pub fn is_stopped(&self) -> bool {
        self.state == PlaybackState::Stopped
    }

    /// Check if a non-looping animation has finished
    pub fn is_finished(&self) -> bool {
        !self.cached_looping && self.cached_duration > 0.0 && self.time >= self.cached_duration
    }

    /// Get the current playback time
    pub fn elapsed(&self) -> f32 {
        self.time
    }

    /// Get the animation duration (0 if not loaded)
    pub fn duration(&self) -> f32 {
        self.cached_duration
    }

    /// Update cached duration and looping from animation data
    ///
    /// This is called internally by the animation system when an animation is loaded.
    pub fn update_cache(&mut self, duration: f32, looping: bool) {
        self.cached_duration = duration;
        self.cached_looping = looping;
    }

    /// Advance time by delta, handling looping
    ///
    /// This is called internally by the animation system each frame.
    pub fn advance_time(&mut self, delta: f32) {
        if self.state != PlaybackState::Playing {
            return;
        }

        self.time += delta * self.speed;

        if self.cached_duration > 0.0 {
            if self.cached_looping {
                // Handle looping (including reverse playback)
                while self.time >= self.cached_duration {
                    self.time -= self.cached_duration;
                }
                while self.time < 0.0 {
                    self.time += self.cached_duration;
                }
            } else {
                // Clamp to bounds and stop at end
                if self.time >= self.cached_duration {
                    self.time = self.cached_duration;
                    self.state = PlaybackState::Stopped;
                } else if self.time < 0.0 {
                    self.time = 0.0;
                    self.state = PlaybackState::Stopped;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSE COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Current pose state for a bone, updated each frame during animation.
///
/// This component is attached to bone entities (children of the skeleton entity).
/// The animation system updates this based on sampled keyframes.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct Pose2D {
    /// Local rotation in radians
    pub rotation: f32,
    /// Local translation (typically only for root bone)
    pub translation: Vec2,
    /// Local scale
    pub scale: Vec2,
}

impl Pose2D {
    /// Create a new pose with identity transform
    pub fn identity() -> Self {
        Self {
            rotation: 0.0,
            translation: Vec2::ZERO,
            scale: Vec2::ONE,
        }
    }

    /// Create a pose from individual components
    pub fn new(translation: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            rotation,
            translation,
            scale,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BONE COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Marker component for bone entities within a skeleton hierarchy.
///
/// Contains metadata about the bone for lookups and debugging.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct BoneEntity {
    /// Index in the skeleton's bone array
    pub bone_index: usize,
    /// Bone name (cached for convenience and debugging)
    pub name: String,
    /// Bone length from the skeleton definition
    pub length: f32,
}

impl BoneEntity {
    /// Create a new bone entity marker
    pub fn new(bone_index: usize, name: impl Into<String>, length: f32) -> Self {
        Self {
            bone_index,
            name: name.into(),
            length,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEBUG VISUALIZATION COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Mode for determining bone colors during debug visualization.
///
/// This allows bones to be colored based on their names or positions.
#[derive(Clone, Debug, Default, Reflect)]
pub enum BoneColorMode {
    /// All bones use the same color
    #[default]
    Uniform,
    /// Color bones based on left/right keywords in their names.
    /// - Bones with "Left" or "left" in their name use `left_color`
    /// - Bones with "Right" or "right" in their name use `right_color`
    /// - Other bones use the default `bone_color` from config
    LeftRightSplit {
        /// Color for bones with "Left" or "left" in their name
        left_color: Color,
        /// Color for bones with "Right" or "right" in their name
        right_color: Color,
    },
}

impl BoneColorMode {
    /// Create a LeftRightSplit mode with the given colors
    pub fn left_right(left: Color, right: Color) -> Self {
        Self::LeftRightSplit {
            left_color: left,
            right_color: right,
        }
    }

    /// Get the color for a bone based on its name and the default color
    pub fn color_for_bone(&self, bone_name: &str, default_color: Color) -> Color {
        match self {
            BoneColorMode::Uniform => default_color,
            BoneColorMode::LeftRightSplit {
                left_color,
                right_color,
            } => {
                if bone_name.contains("Left") || bone_name.contains("left") {
                    *left_color
                } else if bone_name.contains("Right") || bone_name.contains("right") {
                    *right_color
                } else {
                    default_color
                }
            }
        }
    }
}

/// Configuration for skeleton debug visualization.
///
/// Add this component to a skeleton entity to enable debug drawing.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct SkeletonDebugConfig {
    /// Whether to draw bone lines
    pub draw_bones: bool,
    /// Whether to draw primitive shapes attached to bones
    pub draw_shapes: bool,
    /// Color for bone lines (used as default or for uniform coloring)
    pub bone_color: Color,
    /// Color for shape outlines
    pub shape_color: Color,
    /// Line thickness for bones
    pub bone_thickness: f32,
    /// Mode for determining per-bone colors
    pub bone_color_mode: BoneColorMode,
}

impl Default for SkeletonDebugConfig {
    fn default() -> Self {
        Self {
            draw_bones: true,
            draw_shapes: true,
            bone_color: Color::srgb(0.0, 1.0, 0.0), // Green
            shape_color: Color::srgb(1.0, 1.0, 0.0), // Yellow
            bone_thickness: 2.0,
            bone_color_mode: BoneColorMode::Uniform,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BUNDLES
// ═══════════════════════════════════════════════════════════════════════════════

/// Bundle for spawning an animated skeleton entity.
///
/// # Example
/// ```ignore
/// commands.spawn(SkeletonBundle {
///     skeleton_instance: SkeletonInstance::new(skeleton_handle),
///     transform: Transform::from_xyz(100.0, 200.0, 0.0),
///     ..default()
/// });
/// ```
#[derive(Bundle, Default)]
pub struct SkeletonBundle {
    /// Links to the skeleton asset
    pub skeleton_instance: SkeletonInstance,
    /// Animation playback controller
    pub animation_player: AnimationPlayer2D,
    /// Transform in 2D space
    pub transform: Transform,
    /// Global transform (computed by Bevy)
    pub global_transform: GlobalTransform,
    /// Visibility control
    pub visibility: Visibility,
    /// Inherited visibility (computed by Bevy)
    pub inherited_visibility: InheritedVisibility,
    /// View visibility (computed by Bevy)
    pub view_visibility: ViewVisibility,
}

/// Bundle for bone child entities (spawned internally by the system).
#[derive(Bundle)]
pub(crate) struct BoneBundle {
    pub bone_entity: BoneEntity,
    pub pose: Pose2D,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_player_default() {
        let player = AnimationPlayer2D::default();
        assert!(player.current_animation.is_none());
        assert_eq!(player.time, 0.0);
        assert_eq!(player.speed, 1.0);
        assert_eq!(player.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_animation_player_play() {
        let mut player = AnimationPlayer2D::default();
        player.time = 0.5; // Simulate previous state
        player.state = PlaybackState::Paused;

        let handle = Handle::default();
        player.play(handle);

        assert_eq!(player.time, 0.0); // Reset to beginning
        assert_eq!(player.state, PlaybackState::Playing);
    }

    #[test]
    fn test_animation_player_pause_resume() {
        let mut player = AnimationPlayer2D::default();

        // Pause when stopped does nothing
        player.pause();
        assert_eq!(player.state, PlaybackState::Stopped);

        // Start playing
        player.state = PlaybackState::Playing;
        player.pause();
        assert_eq!(player.state, PlaybackState::Paused);

        player.resume();
        assert_eq!(player.state, PlaybackState::Playing);
    }

    #[test]
    fn test_animation_player_stop() {
        let mut player = AnimationPlayer2D::default();
        player.time = 0.5;
        player.state = PlaybackState::Playing;

        player.stop();

        assert_eq!(player.time, 0.0);
        assert_eq!(player.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_animation_player_advance_time_looping() {
        let mut player = AnimationPlayer2D::default();
        player.state = PlaybackState::Playing;
        player.update_cache(1.0, true); // 1 second, looping

        player.advance_time(0.5);
        assert_eq!(player.time, 0.5);

        player.advance_time(0.7);
        assert!((player.time - 0.2).abs() < 0.001); // Wrapped: 1.2 - 1.0 = 0.2
        assert_eq!(player.state, PlaybackState::Playing);
    }

    #[test]
    fn test_animation_player_advance_time_non_looping() {
        let mut player = AnimationPlayer2D::default();
        player.state = PlaybackState::Playing;
        player.update_cache(1.0, false); // 1 second, non-looping

        player.advance_time(1.5);

        assert_eq!(player.time, 1.0); // Clamped to duration
        assert_eq!(player.state, PlaybackState::Stopped); // Stopped at end
    }

    #[test]
    fn test_animation_player_reverse_playback() {
        let mut player = AnimationPlayer2D::default();
        player.state = PlaybackState::Playing;
        player.speed = -1.0;
        player.time = 0.5;
        player.update_cache(1.0, true);

        player.advance_time(0.3);
        assert!((player.time - 0.2).abs() < 0.001); // 0.5 - 0.3 = 0.2
    }

    #[test]
    fn test_animation_player_is_finished() {
        let mut player = AnimationPlayer2D::default();
        player.update_cache(1.0, false);

        assert!(!player.is_finished()); // At beginning

        player.time = 1.0;
        assert!(player.is_finished()); // At end, non-looping

        player.update_cache(1.0, true); // Switch to looping
        assert!(!player.is_finished()); // Looping never "finishes"
    }

    #[test]
    fn test_pose2d_identity() {
        let pose = Pose2D::identity();
        assert_eq!(pose.rotation, 0.0);
        assert_eq!(pose.translation, Vec2::ZERO);
        assert_eq!(pose.scale, Vec2::ONE);
    }

    #[test]
    fn test_bone_entity_new() {
        let bone = BoneEntity::new(5, "test_bone", 1.5);
        assert_eq!(bone.bone_index, 5);
        assert_eq!(bone.name, "test_bone");
        assert_eq!(bone.length, 1.5);
    }

    #[test]
    fn test_skeleton_debug_config_default() {
        let config = SkeletonDebugConfig::default();
        assert!(config.draw_bones);
        assert!(config.draw_shapes);
        assert_eq!(config.bone_thickness, 2.0);
    }
}
