//! Debug visualization for skeletons using Bevy gizmos
//!
//! Provides systems for drawing skeleton bones and primitive shapes
//! for debugging and development purposes.

use super::assets::Skeleton2DAsset;
use super::components::{BoneEntity, Pose2D, SkeletonDebugConfig, SkeletonInstance};
use crate::core::PrimitiveShape;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════════════

/// Global resource to enable/disable skeleton debug visualization.
///
/// Set to `true` to draw all skeletons that have [`SkeletonDebugConfig`] components.
///
/// # Example
/// ```ignore
/// // Enable debug visualization
/// commands.insert_resource(SkeletonDebugEnabled(true));
///
/// // Toggle with keyboard
/// fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<SkeletonDebugEnabled>) {
///     if keys.just_pressed(KeyCode::F3) {
///         debug.0 = !debug.0;
///     }
/// }
/// ```
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct SkeletonDebugEnabled(pub bool);

// ═══════════════════════════════════════════════════════════════════════════════
// DEBUG DRAWING SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/// Draws skeleton bones and shapes using gizmos.
///
/// This system runs in `PostUpdate` when [`SkeletonDebugEnabled`] is true.
/// It respects per-skeleton configuration via [`SkeletonDebugConfig`].
pub fn draw_skeleton_debug(
    mut gizmos: Gizmos,
    debug_enabled: Res<SkeletonDebugEnabled>,
    skeletons: Res<Assets<Skeleton2DAsset>>,
    skeleton_query: Query<(
        &SkeletonInstance,
        &GlobalTransform,
        &SkeletonDebugConfig,
        &Children,
    )>,
    bone_query: Query<(&BoneEntity, &GlobalTransform, &Pose2D)>,
    children_query: Query<&Children>,
) {
    // Check global debug flag
    if !debug_enabled.0 {
        return;
    }

    for (instance, _skeleton_transform, config, children) in skeleton_query.iter() {
        // Skip uninitialized skeletons
        if !instance.initialized {
            continue;
        }

        // Get skeleton asset for bone data
        let Some(skeleton_asset) = skeletons.get(&instance.skeleton) else {
            continue;
        };
        let skeleton = &skeleton_asset.0;

        // Draw each bone recursively
        draw_bone_hierarchy(
            &mut gizmos,
            skeleton,
            config,
            children,
            &bone_query,
            &children_query,
        );
    }
}

/// Recursively draw bones in the hierarchy
fn draw_bone_hierarchy(
    gizmos: &mut Gizmos,
    skeleton: &crate::core::Skeleton2D,
    config: &SkeletonDebugConfig,
    children: &Children,
    bone_query: &Query<(&BoneEntity, &GlobalTransform, &Pose2D)>,
    children_query: &Query<&Children>,
) {
    for child in children.iter() {
        if let Ok((bone_entity, global_transform, _pose)) = bone_query.get(child) {
            let bone_idx = bone_entity.bone_index;

            // Get bone data from skeleton
            if let Some(bone) = skeleton.get_bone(bone_idx) {
                let position = global_transform.translation().truncate();

                // Draw bone line if enabled
                if config.draw_bones && bone.length > 0.0 {
                    draw_bone_line(gizmos, global_transform, bone.length, config.bone_color);
                }

                // Draw primitive shape if enabled
                if config.draw_shapes {
                    draw_primitive_shape(gizmos, &bone.shape, position, global_transform, config.shape_color);
                }
            }

            // Recursively draw children
            if let Ok(grandchildren) = children_query.get(child) {
                draw_bone_hierarchy(gizmos, skeleton, config, grandchildren, bone_query, children_query);
            }
        }
    }
}

/// Draw a bone as a line from joint to tip
fn draw_bone_line(gizmos: &mut Gizmos, transform: &GlobalTransform, length: f32, color: Color) {
    // Start position is the bone's origin (joint)
    let start = transform.translation().truncate();

    // End position is along the bone's local Y axis (standard bone orientation)
    // We need to apply the rotation to the direction
    let local_direction = Vec2::Y * length;
    let world_direction = transform
        .affine()
        .transform_vector3(Vec3::new(local_direction.x, local_direction.y, 0.0))
        .truncate();

    let end = start + world_direction;

    gizmos.line_2d(start, end, color);

    // Draw a small circle at the joint
    gizmos.circle_2d(start, 2.0, color);
}

/// Draw a primitive shape at the bone's position
fn draw_primitive_shape(
    gizmos: &mut Gizmos,
    shape: &PrimitiveShape,
    position: Vec2,
    transform: &GlobalTransform,
    color: Color,
) {
    match shape {
        PrimitiveShape::Circle { radius } => {
            // Scale the radius by the transform's average scale
            let scale = transform.affine().matrix3.x_axis.truncate().length();
            let scaled_radius = radius * scale;
            gizmos.circle_2d(position, scaled_radius, color);
        }

        PrimitiveShape::Rectangle { width, height } => {
            // Get rotation from transform
            let (_, rotation, _) = transform.to_scale_rotation_translation();
            let angle = rotation.to_euler(EulerRot::ZYX).0;

            // Get scale
            let scale = transform.affine().matrix3.x_axis.truncate().length();
            let scaled_width = width * scale;
            let scaled_height = height * scale;

            // Draw rectangle using Isometry2d
            gizmos.rect_2d(
                Isometry2d::new(position, Rot2::radians(angle)),
                Vec2::new(scaled_width, scaled_height),
                color,
            );
        }

        PrimitiveShape::Triangle { base, height } => {
            // Get rotation and scale from transform
            let (_, rotation, _) = transform.to_scale_rotation_translation();
            let angle = rotation.to_euler(EulerRot::ZYX).0;
            let scale = transform.affine().matrix3.x_axis.truncate().length();

            let scaled_base = base * scale;
            let scaled_height = height * scale;

            // Triangle vertices (centered at origin, pointing up)
            let half_base = scaled_base / 2.0;
            let vertices = [
                Vec2::new(-half_base, 0.0),
                Vec2::new(half_base, 0.0),
                Vec2::new(0.0, scaled_height),
            ];

            // Rotate and translate vertices
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            let rotated: Vec<Vec2> = vertices
                .iter()
                .map(|v| {
                    Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a) + position
                })
                .collect();

            // Draw triangle as line strip
            gizmos.linestrip_2d(
                [rotated[0], rotated[1], rotated[2], rotated[0]],
                color,
            );
        }

        PrimitiveShape::None => {
            // Draw nothing for None shape
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_debug_enabled_default() {
        let enabled = SkeletonDebugEnabled::default();
        assert!(!enabled.0); // Default is disabled
    }

    #[test]
    fn test_skeleton_debug_enabled_toggle() {
        let mut enabled = SkeletonDebugEnabled(false);
        enabled.0 = !enabled.0;
        assert!(enabled.0);
        enabled.0 = !enabled.0;
        assert!(!enabled.0);
    }
}
