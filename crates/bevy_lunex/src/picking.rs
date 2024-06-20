use bevy::prelude::*;
use bevy_mod_picking::{backend::PointerHits, prelude::*};
use lunex_engine::YInvert;
use std::cmp::Ordering;
use bevy::window::PrimaryWindow;
use bevy_mod_picking::backend::prelude::*;

use crate::{Dimension, Element};


/// Adds picking support for [`bevy_lunex`].
#[derive(Clone)]
pub struct LunexBackend;
impl Plugin for LunexBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, lunex_picking.in_set(PickSet::Backend));
    }
}


/// Checks if any sprite entities are under each pointer
pub fn _lunex_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    node_query: Query<
        (
            Entity,
            &Dimension,
            Option<&Element>,
            &GlobalTransform,
            Option<&Pickable>,
            &ViewVisibility,
        )
    >,
    mut output: EventWriter<PointerHits>,
    mut gizmos: Gizmos,
) {
    let mut sorted_nodes: Vec<_> = node_query.iter().collect();
    sorted_nodes.sort_by(|a, b| {
        (b.3.translation().z).partial_cmp(&a.3.translation().z).unwrap_or(Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform, cam_ortho)) = cameras
            .iter().filter(|(_, camera, _, _)| camera.is_active).find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.get_single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
                    .unwrap()
                    == location.target
            })
        else { continue; };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position) else { continue; };

        let picks: Vec<(Entity, HitData)> = sorted_nodes
            .iter()
            .copied()
            .filter(|(.., visibility)| visibility.get())
            .filter_map(
                |(entity, dimension, element, sprite_transform, pickable, ..)| {
                    if blocked {
                        return None;
                    }

                    //info!("IT RUN");

                    // Hit box in sprite coordinate system
                    /* let (extents, anchor) = if let Some((sprite, atlas)) = sprite.zip(atlas) {
                        let extents = sprite.custom_size.or_else(|| {
                            texture_atlas_layout
                                .get(&atlas.layout)
                                .map(|f| f.textures[atlas.index].size().as_vec2())
                        })?;
                        let anchor = sprite.anchor.as_vec();
                        (extents, anchor)
                    } else if let Some((sprite, image)) = sprite.zip(image) {
                        let extents = sprite
                            .custom_size
                            .or_else(|| images.get(image).map(|f| f.size().as_vec2()))?;
                        let anchor = sprite.anchor.as_vec();
                        (extents, anchor)
                    } else {
                        return None;
                    }; */

                    let anchor = if element.is_none() { Vec2::ZERO } else { Vec2::splat(0.5) };
                    let center = -anchor * dimension.size.invert_y();
                    let rect = Rect::from_center_half_size(center, dimension.size / 2.0);

                    let s = rect.max - rect.min;
                    let p = rect.min + s/2.0;
                    gizmos.rect(p.extend(500.0), Quat::from_rotation_y(0.0), s, Color::linear_rgb(0.0, 0.0, 1.0));

                    //let pos = if !element.is_some() {
                    //    dimension.size.invert_y() / 2.0
                    //} else {
                    //    Vec2::new(0.0, 0.0)
                    //};

                    //let rect = Rect::from_center_size(pos, dimension.size);

                    // Transform cursor pos to sprite coordinate system
                    let cursor_pos_sprite = sprite_transform
                        .affine()
                        .inverse()
                        .transform_point3((cursor_pos_world, 0.0).into());

                    let is_cursor_in_sprite = rect.contains(cursor_pos_sprite.truncate());
                    blocked = is_cursor_in_sprite && pickable.map(|p| p.should_block_lower) != Some(false);

                    // HitData requires a depth as calculated from the camera's near clipping plane
                    let depth = -cam_ortho.near - sprite_transform.translation().z;

                    is_cursor_in_sprite.then_some((entity, HitData::new(cam_entity, depth, None, None)))
                },
            )
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order));
    }
}

/// Checks if any Dimension entities are under each pointer
pub fn lunex_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    node_query: Query<
        (
            Entity,
            &Dimension,
            Option<&Element>,
            &GlobalTransform,
            Option<&Pickable>,
            &ViewVisibility,
        )
    >,
    mut output: EventWriter<PointerHits>,
    mut gizmos: Gizmos,
) {
    let mut sorted_nodes: Vec<_> = node_query.iter().collect();
    sorted_nodes.sort_by(|a, b| { (b.3.translation().z).partial_cmp(&a.3.translation().z).unwrap_or(Ordering::Equal) });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| { pointer_location.location().map(|loc| (pointer, loc)) }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform, cam_ortho)) = cameras.iter().filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.get_single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
                    .unwrap()
                    == location.target
            })
        else { continue; };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position) else { continue; };

        let picks: Vec<(Entity, HitData)> = sorted_nodes
            .iter()
            .copied()
            .filter(|(.., visibility)| visibility.get())
            .filter_map(
                |(entity, dimension, element, sprite_transform, pickable, ..)| {
                    if blocked {
                        return None;
                    }

                    let pos = if !element.is_some() {
                        dimension.size.invert_y() / 2.0
                    } else {
                        let _ = Vec2::ZERO;
                        dimension.size.invert_y() / 2.0
                    };

                    let rect = Rect::from_center_size(pos, dimension.size);

                    let s = rect.max - rect.min;
                    let p = (rect.min + s/2.0).extend(0.0) + sprite_transform.translation();
                    gizmos.rect(p, Quat::from_rotation_y(0.0), s, Color::linear_rgb(0.0, 0.0, 1.0));

                    // Transform cursor pos to sprite coordinate system
                    let cursor_pos_sprite = sprite_transform
                        .affine()
                        .inverse()
                        .transform_point3((cursor_pos_world, 0.0).into());

                    let is_cursor_in_sprite = rect.contains(cursor_pos_sprite.truncate());
                    blocked = is_cursor_in_sprite && pickable.map(|p| p.should_block_lower) != Some(false);

                    if is_cursor_in_sprite { info!("Cursor in {entity} - Is not blocked: {blocked} {:?}", pickable); }

                    // HitData requires a depth as calculated from the camera's near clipping plane
                    let depth = -cam_ortho.near - sprite_transform.translation().z;

                    is_cursor_in_sprite.then_some((entity, HitData::new(cam_entity, depth, None, None)))
                },
            )
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order));
    }
}