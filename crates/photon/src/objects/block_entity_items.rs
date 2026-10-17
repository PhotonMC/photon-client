//! The items block entities show: the item a suspicious block holds, rising out of its dust
//! (`BrushableBlockRenderer`), and those on a shelf (`ShelfRenderer`).

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::block_entity::{brushable, shelf};
use photon_render::joml::Pose;
use photon_render::objects::item::{item_layer_vertices, model_extent};
use photon_render::scene::Vertex;
use photon_world::ClientItemStackState;

use crate::equipment::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// How far from the camera, in chunks, block entities are drawn.
const RADIUS_CHUNKS: i32 = 8;

/// The triangles of the items suspicious blocks hold, once they have been brushed, in `fixed`'s
/// display context.
pub fn brushable_items(
    session: &PlaySession,
    fixed: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    for (x, y, z) in world.chest_positions_near(centre, RADIUS_CHUNKS) {
        let Some(brushable) = world.brushable_view((x, y, z)) else {
            continue;
        };
        if brushable.dusted == 0 {
            continue;
        }
        // The light is the one of the face that was brushed.
        let (dx, dy, dz) = brushable.hit_direction.step();
        let light = packed_light(
            world,
            Vec3::new(f64::from(x + dx), f64::from(y + dy), f64::from(z + dz)),
        );
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let pose = brushable::item_pose(&base, brushable.hit_direction, brushable.dusted);
        let stack = ClientItemStackState::new(brushable.item.1, brushable.item.0);
        vertices.extend(item_layer_vertices(
            fixed.get(&stack, renderer, tints),
            &[pose],
            light,
            false,
        ));
    }
}

/// The triangles of the items on shelves, in `on_shelf`'s display context.
pub fn shelf_items(
    session: &PlaySession,
    on_shelf: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    for (x, y, z) in world.chest_positions_near(centre, RADIUS_CHUNKS) {
        let Some(shelf) = world.shelf_view((x, y, z)) else {
            continue;
        };
        let light = packed_light(world, Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let turn = (shelf::item_y_rot(shelf.facing), shelf.align_to_bottom);
        for (slot, item) in shelf.items.iter().enumerate() {
            let Some((item, count)) = item else {
                continue;
            };
            let stack = ClientItemStackState::new(*count, *item);
            let layers = on_shelf.get(&stack, renderer, tints);
            let pose = shelf::item_pose(&base, slot, turn, model_extent(layers));
            vertices.extend(item_layer_vertices(layers, &[pose], light, false));
        }
    }
}
