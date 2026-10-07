//! The paintings (`PaintingRenderer`): a slab per block of the painting's size on the wall it
//! hangs on, its picture on the front, each block lit by the block it is in front of.

use voxel_core::direction::Direction;
use voxel_core::math::{Vec3, floor_f64};
use voxel_net::connection::PlaySession;
use voxel_render::block::light::light_uv;
use voxel_render::entity::textures::{EntityTextures, TextureRect};
use voxel_render::joml::Pose;
use voxel_render::objects::painting::{
    PaintingVertex, SpriteUv, painting_center, painting_pose, painting_vertices,
};
use voxel_render::scene::Vertex;

use crate::overlays::{FrameView, packed_light};

/// The texture of a painting's back and edges (`PaintingRenderer.BACK_SPRITE_LOCATION`).
const BACK: &str = "painting/back";

/// The paths (under `textures/`) of the textures the paintings wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    voxel_data::entity_type::painting_variants()
        .map(|variant| variant.texture)
        .chain([BACK])
}

/// A texture's place in the packed image as the sprite coordinates the painting's renderer reads.
fn sprite(rect: TextureRect) -> SpriteUv {
    SpriteUv {
        u0: rect[0],
        u1: rect[2],
        v0: rect[1],
        v1: rect[3],
    }
}

/// The light of each block of the painting, row by row from the bottom
/// (`PaintingRenderer.extractRenderState`): that of the block in front of each.
fn block_lights(
    world: &voxel_world::ClientWorld,
    center: (f64, f64, f64),
    direction: Direction,
    (width, height): (i32, i32),
) -> Vec<i32> {
    let offset_x = -width as f32 / 2.0;
    let offset_y = -height as f32 / 2.0;
    let mut lights = Vec::new();
    for segment_y in 0..height {
        for segment_x in 0..width {
            let along = f64::from(segment_x as f32 + offset_x + 0.5);
            let up = f64::from(segment_y as f32 + offset_y + 0.5);
            let (mut x, y, mut z) = (
                floor_f64(center.0),
                floor_f64(center.1 + up),
                floor_f64(center.2),
            );
            match direction {
                Direction::North => x = floor_f64(center.0 + along),
                Direction::West => z = floor_f64(center.2 - along),
                Direction::South => x = floor_f64(center.0 - along),
                Direction::East => z = floor_f64(center.2 + along),
                Direction::Down | Direction::Up => {}
            }
            lights.push(packed_light(
                world,
                Vec3::new(f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5),
            ));
        }
    }
    lights
}

/// The paintings the server added.
pub fn remote_paintings(
    session: &PlaySession,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let names = session.registry_names("minecraft:painting_variant");
    let Some(back) = textures.rect(BACK) else {
        return Vec::new();
    };
    let mut vertices = Vec::new();
    for entity in world.entities() {
        if voxel_data::entity_type::name(entity.type_id) != Some("minecraft:painting") {
            continue;
        }
        // `ClientboundAddEntityPacket.getData`: the way it faces; the block is its position.
        let direction = Direction::from_3d_data_value(entity.data);
        let Some(variant) = entity
            .variant()
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| names.get(id))
            .and_then(|name| voxel_data::entity_type::painting_variant(name))
        else {
            continue;
        };
        if matches!(direction, Direction::Down | Direction::Up) {
            continue;
        }
        let Some(front) = textures.rect(variant.texture) else {
            continue;
        };
        let (x, y, z) = entity.position();
        let size = (variant.width, variant.height);
        let center = painting_center((floor_f64(x), floor_f64(y), floor_f64(z)), direction, size);
        let mut base = Pose::default();
        base.translate(
            (center.0 - f64::from(view.origin[0])) as f32,
            (center.1 - f64::from(view.origin[1])) as f32,
            (center.2 - f64::from(view.origin[2])) as f32,
        );
        let pose = painting_pose(&base, direction.data2d());
        let lights = block_lights(world, center, direction, size);
        for quad in
            painting_vertices(&pose, size, &lights, (sprite(front), sprite(back))).chunks_exact(4)
        {
            vertices.extend([0, 1, 2, 0, 2, 3].map(|index| vertex(&quad[index])));
        }
    }
    vertices
}

/// A painting's vertex for the entity shader: lit by the world and by its normal.
fn vertex(vertex: &PaintingVertex) -> Vertex {
    Vertex {
        position: vertex.position,
        uv: vertex.uv,
        shade: 0.0,
        tint: [vertex.normal[0], vertex.normal[1], vertex.normal[2], 0.0],
        light: light_uv(vertex.light),
    }
}
