//! A raycasting backend for [`Aabb`](bevy::Aabb).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy::render::primitives::Aabb;
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
        app.add_system(aabb_picking.in_set(PickSet::Backend));
    }
}

/// Checks if any sprite entities are under each pointer
pub fn aabb_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    windows: Query<(Entity, &Window)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    aabb_query: Query<(
        Entity,
        &Aabb,
        &GlobalTransform,
        &ComputedVisibility,
        Option<&FocusPolicy>,
    )>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    let mut sorted_aabbs: Vec<_> = aabb_query.iter().collect();
    sorted_aabbs.sort_by(|a, b| {
        (b.2.translation().z)
            .partial_cmp(&a.2.translation().z)
            .unwrap_or(Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let cursor_position = location.position;
        let mut blocked = false;
        let (camera_entity, camera, camera_transform) = cameras
            .iter()
            .find(|(_entity, camera, global_transform)| {
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
            .and_then(|ray| Some(ray.get_point(0f32))) else {
                continue;
            };
        let over_list = sorted_aabbs
            .iter()
            .copied()
            .filter_map(|(entity, aabb, global_transform, visibility, focus)| {
                if blocked || !visibility.is_visible() {
                    return None;
                }

                let global_position = global_transform.translation();

                let half_extents = aabb.half_extents.truncate() / 2f32;

                let position = global_position.truncate() + aabb.center.truncate();
                let center = aabb.center.truncate();

                let min = position - half_extents + center;
                let max = position + half_extents + center;

                let contains_cursor = (min.x..max.x).contains(&cursor_position.x)
                    && (min.y..max.y).contains(&cursor_position.y);

                blocked = contains_cursor && focus != Some(&FocusPolicy::Pass);
                contains_cursor.then_some((
                    entity,
                    PickData {
                        camera: camera_entity,
                        depth: global_position.z,
                        position: None,
                        normal: None,
                    },
                ))
            })
            .collect::<Vec<_>>();

        output.send(EntitiesUnderPointer {
            pointer: *pointer,
            picks: over_list,
            order: 0,
        })
    }
}
