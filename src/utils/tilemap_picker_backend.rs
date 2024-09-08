use backend::{HitData, PointerHits};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::prelude::*;
use bevy_mod_picking::prelude::*;
use picking_core::PickSet;

pub struct TilemapPickerBackend;

impl Plugin for TilemapPickerBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, tile_picking.in_set(PickSet::Backend));
    }
}

fn tile_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &GlobalTransform,
        &ViewVisibility,
    )>,
    tile_q: Query<(&TileVisible, Option<&Pickable>)>,
    mut output: EventWriter<PointerHits>,
) {
    for (p_id, p_loc) in pointers
        .iter()
        .filter_map(|(p_id, p_loc)| p_loc.location().map(|l| (p_id, l)))
    {
        let mut blocked = false;

        let Some((cam_entity, camera, cam_transform, cam_ortho)) = cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.get_single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
                    .unwrap()
                    == p_loc.target
            })
        else {
            continue;
        };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, p_loc.position)
        else {
            continue;
        };

        let picks = tilemap_q
            .iter()
            .filter(|(.., vis)| vis.get())
            .filter_map(|(t_s, tgs, tty, t_store, gt, ..)| {
                if blocked {
                    return None;
                }
                let in_map_pos: Vec2 = {
                    let pos = Vec4::from((cursor_pos_world + Vec2::new(0., -4.), 0.0, 1.0));
                    let in_map_pos = gt.compute_matrix().inverse() * pos;
                    in_map_pos.xy()
                };
                let picked: Entity = TilePos::from_world_pos(&in_map_pos, t_s, tgs, tty)
                    .and_then(|tile_pos| t_store.get(&tile_pos))?;
                let (vis, pck) = tile_q.get(picked).ok()?;
                if !vis.0 {
                    return None;
                }
                blocked = pck.is_some_and(|p| p.should_block_lower);
                let depth = -cam_ortho.near - gt.translation().z;
                Some((picked, HitData::new(cam_entity, depth, None, None)))
            })
            .collect();

        // f32 required by PointerHits
        #[allow(clippy::cast_precision_loss)]
        let order = camera.order as f32;
        output.send(PointerHits::new(*p_id, picks, order));
    }
}
