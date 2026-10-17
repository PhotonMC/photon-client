//! The items block entities show: the item a suspicious block holds, rising out of its dust
//! (`BrushableBlockRenderer`), and those on a shelf (`ShelfRenderer`).

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::block::world_mesh::ConduitMeshes;
use photon_render::block_entity::{brushable, campfire, conduit, shelf, vault};
use photon_render::entity::name_tag::camera_orientation;
use photon_render::joml::Pose;
use photon_render::objects::item::{entity_block_vertices, item_layer_vertices, model_extent};
use photon_render::scene::Vertex;
use photon_world::ClientItemStackState;

use crate::equipment::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// How far from the camera, in chunks, block entities are drawn.
const RADIUS_CHUNKS: i32 = 8;

/// `BlockEntityRenderer.getViewDistance`'s default: how far from the camera, in blocks, a block
/// entity is drawn.
const VIEW_DISTANCE: f64 = 64.0;

/// Whether the centre of a block is closer to the camera than the view distance (`Vec3.closerThan`).
fn within_view((x, y, z): (i32, i32, i32), view: FrameView) -> bool {
    let centre = [x, y, z].map(|axis| f64::from(axis) + 0.5);
    let distance: f64 = (0..3)
        .map(|axis| {
            let camera = f64::from(view.origin[axis]) + f64::from(view.camera[axis]);
            (centre[axis] - camera).powi(2)
        })
        .sum();
    distance < VIEW_DISTANCE * VIEW_DISTANCE
}

/// The block entities the renderers of items draw: those in the chunks around the camera that are
/// within the view distance.
fn in_view(world: &photon_world::ClientWorld, view: FrameView) -> Vec<(i32, i32, i32)> {
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    world
        .chest_positions_near(centre, RADIUS_CHUNKS)
        .into_iter()
        .filter(|pos| within_view(*pos, view))
        .collect()
}

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
    for (x, y, z) in in_view(world, view) {
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
    for (x, y, z) in in_view(world, view) {
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

/// The triangles of the conduits' shells, from the sprites of the block atlas.
pub fn conduits(
    session: &PlaySession,
    meshes: &ConduitMeshes,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let orientation = camera_orientation(view.rotation.0, view.rotation.1);
    for held in world.conduit_views() {
        let (x, y, z) = held.position;
        if !within_view(held.position, view) {
            continue;
        }
        let light = packed_light(world, Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let frame = conduit::ConduitFrame::of(
            held.active,
            held.hunting,
            (held.tick_count, held.rotation),
            partial,
        );
        for (piece, pose) in conduit::pieces(&base, &frame, &orientation) {
            if let Some(layer) = meshes.get(piece) {
                vertices.extend(entity_block_vertices(layer, &pose, (light, 0.0)));
            }
        }
    }
}

/// The triangles of the food cooking on campfires, in `fixed`'s display context.
pub fn campfire_items(
    session: &PlaySession,
    fixed: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    for (x, y, z) in in_view(world, view) {
        let Some(fire) = world.campfire_view((x, y, z)) else {
            continue;
        };
        let light = packed_light(world, Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        for (slot, item) in fire.items.iter().enumerate() {
            let Some((item, count)) = item else {
                continue;
            };
            let stack = ClientItemStackState::new(*count, *item);
            let pose = campfire::item_pose(&base, fire.facing, slot);
            vertices.extend(item_layer_vertices(
                fixed.get(&stack, renderer, tints),
                &[pose],
                light,
                false,
            ));
        }
    }
}

/// The triangles of the items vaults show, in `ground`'s display context, turning as far as the
/// last two ticks have them at the frame's partial tick.
pub fn vault_items(
    session: &PlaySession,
    ground: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    for (x, y, z) in in_view(world, view) {
        let Some(shown) = world.vault_view((x, y, z)) else {
            continue;
        };
        let light = packed_light(world, Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let (item, count) = shown.display_item;
        let stack = ClientItemStackState::new(count, item);
        let layers = ground.get(&stack, renderer, tints);
        let spin = vault::spin(view.partial as f32, shown.spin);
        let poses = vault::item_poses(&base, spin, shown.display_item, model_extent(layers));
        vertices.extend(item_layer_vertices(layers, &poses, light, false));
    }
}
