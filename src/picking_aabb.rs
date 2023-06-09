//! A raycasting backend for [`Aabb`](bevy::Aabb).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::ui::FocusPolicy;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_picking_core::backend::prelude::*;

/*
/// Commonly used imports for the [`bevy_picking_sprite`](crate) crate.
pub mod prelude {
    pub use crate::AabbBackend;
}
*/

/// Adds picking support for [`Aabb`](bevy::Aabb)
#[derive(Clone)]
pub struct AabbBackend;

impl PickingBackend for AabbBackend {}

impl Plugin for AabbBackend {
    fn build(&self, app: &mut App) {
        app.add_system(half_extents_picking.in_set(PickSet::Backend));
    }
}

#[derive(Component)]
pub struct HalfExtents(pub Vec2);

/// Checks if any sprite entities are under each pointer
pub fn half_extents_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    data_query: Query<(
        Entity,
        &HalfExtents,
        &GlobalTransform,
        &ComputedVisibility,
        Option<&FocusPolicy>,
    )>,
    mut output: EventWriter<PointerHits>,
) {
    /*
    If
    let mut sorted_data: Vec<_> = data_query.iter().collect();
    sorted_data.sort_by(|a, b| {
        (b.2.translation().z)
            .partial_cmp(&a.2.translation().z)
            .unwrap_or(Ordering::Equal)
    });
    */

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let cursor_position = location.position;
        let mut blocked = false;
        let (camera_entity, camera, camera_transform) = cameras
            .iter()
            .find(|(_entity, camera, _global_transform)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            })
            .map(|(entity, camera, global_transform)| (entity, camera, global_transform))
            .unwrap_or_else(|| panic!("No camera found associated with pointer {:?}.", pointer));

        let Some(cursor_position) = camera
            .viewport_to_world(camera_transform, cursor_position)
            .map(|ray| ray.get_point(0f32)) else {
                continue;
            };
        let over_list = data_query
            .iter()
            .filter_map(|(entity, aabb, global_transform, visibility, focus)| {
                if blocked || !visibility.is_visible() {
                    return None;
                }

                let global_position = global_transform.translation();

                let half_extents = aabb.0;

                let position = global_position.truncate();

                let min = position - half_extents;
                let max = position + half_extents;

                let contains_cursor = (min.x..max.x).contains(&cursor_position.x)
                    && (min.y..max.y).contains(&cursor_position.y);

                blocked = contains_cursor && focus != Some(&FocusPolicy::Pass);
                contains_cursor.then_some((
                    entity,
                    HitData {
                        camera: camera_entity,
                        depth: global_position.z,
                        position: None,
                        normal: None,
                    },
                ))
            })
            .collect::<Vec<_>>();

        output.send(PointerHits {
            pointer: *pointer,
            picks: over_list,
            order: 0,
        })
    }
}
