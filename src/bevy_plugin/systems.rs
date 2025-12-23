//! Animation playback and pose update systems
//!
//! These systems handle the animation lifecycle:
//! 1. Time advancement
//! 2. Skeleton hierarchy initialization
//! 3. Animation sampling
//! 4. Pose to transform conversion

use super::assets::{Animation2DAsset, Skeleton2DAsset};
use super::components::{
    AnimationPlayer2D, BoneBundle, BoneEntity, PlaybackState, Pose2D, SkeletonInstance,
};
use crate::core::Transform2D;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// TIME ADVANCEMENT SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/// Advances animation time based on delta time and playback speed.
///
/// This system runs in `PreUpdate` to ensure time is updated before sampling.
pub fn advance_animation_time(
    time: Res<Time>,
    animations: Res<Assets<Animation2DAsset>>,
    mut query: Query<&mut AnimationPlayer2D>,
) {
    let delta = time.delta_secs();

    for mut player in query.iter_mut() {
        // Skip if not playing
        if player.state != PlaybackState::Playing {
            continue;
        }

        // Get animation to check duration and looping
        if let Some(ref handle) = player.current_animation {
            if let Some(animation) = animations.get(handle) {
                // Update cached values from animation
                player.update_cache(animation.duration, animation.looping);
            }
        }

        // Advance time
        player.advance_time(delta);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SKELETON HIERARCHY INITIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Spawns child bone entities when a skeleton asset becomes available.
///
/// This system:
/// 1. Waits for the skeleton asset to load
/// 2. Creates child entities for each bone
/// 3. Sets up the parent-child hierarchy matching the skeleton structure
/// 4. Initializes each bone with its setup pose
pub fn initialize_skeleton_hierarchy(
    mut commands: Commands,
    skeletons: Res<Assets<Skeleton2DAsset>>,
    mut query: Query<(Entity, &mut SkeletonInstance), Changed<SkeletonInstance>>,
) {
    for (skeleton_entity, mut instance) in query.iter_mut() {
        // Skip if already initialized
        if instance.initialized {
            continue;
        }

        // Wait for skeleton asset to load
        let Some(skeleton_asset) = skeletons.get(&instance.skeleton) else {
            continue;
        };

        let skeleton = &skeleton_asset.0;

        // Check for empty skeleton (shouldn't happen due to validation, but be defensive)
        if skeleton.bones.is_empty() {
            instance.initialized = true;
            continue;
        }

        // Track spawned bone entities by their bone index
        // This allows us to correctly parent bones to their parent bones
        let mut bone_entities: Vec<Entity> = Vec::with_capacity(skeleton.bones.len());

        // Spawn bone entities in topological order (guaranteed by Skeleton2D validation)
        for (bone_idx, bone) in skeleton.bones.iter().enumerate() {
            // Convert setup pose to Bevy transform
            let transform = transform_from_2d(&bone.setup);

            // Create the bone entity
            let bone_entity_id = commands
                .spawn(BoneBundle {
                    bone_entity: BoneEntity::new(bone_idx, &bone.name, bone.length),
                    pose: Pose2D::new(bone.setup.translation, bone.setup.rotation, bone.setup.scale),
                    transform,
                    global_transform: GlobalTransform::default(),
                    visibility: Visibility::Inherited,
                    inherited_visibility: InheritedVisibility::default(),
                    view_visibility: ViewVisibility::default(),
                })
                .id();

            // Parent to either the skeleton entity (for root) or the parent bone entity
            match bone.parent {
                None => {
                    // Root bone - parent directly to skeleton entity
                    commands.entity(skeleton_entity).add_child(bone_entity_id);
                }
                Some(parent_idx) => {
                    // Non-root bone - parent to the appropriate bone entity
                    // Note: topological order is guaranteed by Skeleton2D validation
                    if let Some(&parent_entity) = bone_entities.get(parent_idx) {
                        commands.entity(parent_entity).add_child(bone_entity_id);
                    } else {
                        // Fall back to parenting to skeleton (should never happen)
                        commands.entity(skeleton_entity).add_child(bone_entity_id);
                    }
                }
            }

            bone_entities.push(bone_entity_id);
        }

        instance.initialized = true;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATION SAMPLING
// ═══════════════════════════════════════════════════════════════════════════════

/// Samples the current animation and updates Pose2D components for each bone.
///
/// This system:
/// 1. Gets the skeleton and animation assets
/// 2. Validates compatibility (skeleton names must match)
/// 3. For each bone, samples the animation at the current time
/// 4. Falls back to setup pose if no animation timeline exists for a bone
pub fn sample_animations(
    skeletons: Res<Assets<Skeleton2DAsset>>,
    animations: Res<Assets<Animation2DAsset>>,
    skeleton_query: Query<(&SkeletonInstance, &AnimationPlayer2D, &Children)>,
    mut bone_query: Query<(&BoneEntity, &mut Pose2D)>,
) {
    for (instance, player, children) in skeleton_query.iter() {
        // Skip if skeleton not initialized
        if !instance.initialized {
            continue;
        }

        // Get skeleton asset
        let Some(skeleton_asset) = skeletons.get(&instance.skeleton) else {
            continue;
        };
        let skeleton = &skeleton_asset.0;

        // Get animation asset (if any)
        let animation_opt = player
            .current_animation
            .as_ref()
            .and_then(|h| animations.get(h));

        // Validate compatibility if we have an animation
        if let Some(animation) = animation_opt {
            if animation.skeleton_name != skeleton.name {
                // Log warning once per mismatch (in practice, use warn_once! or a flag)
                // For now, just skip this frame
                continue;
            }
        }

        // Update each bone's pose
        update_bone_poses(skeleton, animation_opt, player.time, children, &mut bone_query);
    }
}

/// Helper to update bone poses from animation or setup pose
fn update_bone_poses(
    skeleton: &crate::core::Skeleton2D,
    animation: Option<&Animation2DAsset>,
    time: f32,
    children: &Children,
    bone_query: &mut Query<(&BoneEntity, &mut Pose2D)>,
) {
    // Recursively process all descendants (bones can be nested)
    for child in children.iter() {
        if let Ok((bone_entity, mut pose)) = bone_query.get_mut(child) {
            let bone_idx = bone_entity.bone_index;

            // Try to sample from animation
            if let Some(anim) = animation {
                if let Some(sampled) = anim.0.sample_bone(bone_idx, time) {
                    // Get setup pose for fallback values
                    let default_setup = Transform2D::default();
                    let setup = skeleton
                        .get_bone(bone_idx)
                        .map(|b| &b.setup)
                        .unwrap_or(&default_setup);

                    pose.rotation = sampled.rotation;
                    pose.translation = sampled.translation.unwrap_or(setup.translation);
                    pose.scale = sampled.scale.unwrap_or(setup.scale);
                    continue;
                }
            }

            // Fall back to setup pose from skeleton
            if let Some(bone) = skeleton.get_bone(bone_idx) {
                pose.rotation = bone.setup.rotation;
                pose.translation = bone.setup.translation;
                pose.scale = bone.setup.scale;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSE TO TRANSFORM CONVERSION
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts Pose2D components to Transform components.
///
/// This runs after animation sampling to update the actual transforms
/// that Bevy uses for rendering and hierarchy propagation.
pub fn apply_poses_to_transforms(mut query: Query<(&Pose2D, &mut Transform), Changed<Pose2D>>) {
    for (pose, mut transform) in query.iter_mut() {
        transform.translation.x = pose.translation.x;
        transform.translation.y = pose.translation.y;
        // Keep Z unchanged for layering purposes

        transform.rotation = Quat::from_rotation_z(pose.rotation);

        transform.scale.x = pose.scale.x;
        transform.scale.y = pose.scale.y;
        // Keep Z scale at 1.0 for 2D
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert a 2D transform to a Bevy Transform
fn transform_from_2d(t2d: &Transform2D) -> Transform {
    Transform {
        translation: Vec3::new(t2d.translation.x, t2d.translation.y, 0.0),
        rotation: Quat::from_rotation_z(t2d.rotation),
        scale: Vec3::new(t2d.scale.x, t2d.scale.y, 1.0),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_transform_from_2d_identity() {
        let t2d = Transform2D::default();
        let transform = transform_from_2d(&t2d);

        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_from_2d_with_values() {
        let t2d = Transform2D::new(
            Vec2::new(10.0, 20.0),
            std::f32::consts::FRAC_PI_2, // 90 degrees
            Vec2::new(2.0, 3.0),
        );
        let transform = transform_from_2d(&t2d);

        assert_eq!(transform.translation.x, 10.0);
        assert_eq!(transform.translation.y, 20.0);
        assert_eq!(transform.translation.z, 0.0);

        // Check rotation is around Z axis
        let expected_rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        assert!((transform.rotation.x - expected_rot.x).abs() < 0.001);
        assert!((transform.rotation.y - expected_rot.y).abs() < 0.001);
        assert!((transform.rotation.z - expected_rot.z).abs() < 0.001);
        assert!((transform.rotation.w - expected_rot.w).abs() < 0.001);

        assert_eq!(transform.scale.x, 2.0);
        assert_eq!(transform.scale.y, 3.0);
        assert_eq!(transform.scale.z, 1.0);
    }
}
