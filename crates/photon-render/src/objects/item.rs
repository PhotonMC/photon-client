//! A dropped item: how many copies of its stack are drawn, and the pose of
//! each (`ItemEntityRenderer`): the stack bobs and spins, and the copies
//! sit beside one another at offsets its seed gives.

use photon_core::ids::ItemId;
use photon_core::math::mth_sin_f64;
use photon_core::random::RandomSource;

use crate::block::light::light_uv;
use crate::block::world_mesh::ItemLayer;
use crate::joml::{Pose, Quaternion, normalize};
use crate::scene::Vertex;

/// `ItemClusterRenderState.getRenderedAmount`: how many copies show for a
/// stack of this size.
pub fn rendered_amount(stack_count: i32) -> i32 {
    match stack_count {
        ..=1 => 1,
        2..=16 => 2,
        17..=32 => 3,
        33..=48 => 4,
        _ => 5,
    }
}

/// `ItemClusterRenderState.getSeedForItemStack`: what the copies' offsets
/// are drawn from.
pub fn seed_for_stack(item_registry_id: ItemId, damage: i32) -> i32 {
    item_registry_id.0 + damage
}

/// A model's extent as the renderer reads it: its lowest and highest points, and its
/// depth (its size along z), all after the item's display transform.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelExtent {
    pub min_y: f64,
    pub max_y: f64,
    pub depth: f64,
}

/// `ItemEntity.getSpin`.
fn spin(age_in_ticks: f32, bob_offset: f32) -> f32 {
    age_in_ticks / 20.0 + bob_offset
}

/// `ItemEntityRenderer.submit` and `submitMultipleFromCount`: the pose of
/// each copy of the stack, from `start` (the pose of the item entity's
/// feet), in the order they are submitted.
pub fn item_entity_poses(
    start: Pose,
    age_in_ticks: f32,
    bob_offset: f32,
    count: i32,
    seed: i32,
    extent: ModelExtent,
) -> Vec<Pose> {
    let mut base = start;
    let min_offset_y = -(extent.min_y as f32) + 0.0625;
    let bob = mth_sin_f64(f64::from(age_in_ticks / 10.0 + bob_offset)) * 0.1 + 0.1;
    base.translate(0.0, bob + min_offset_y, 0.0);
    base.rotate(&Quaternion::rotation_y(spin(age_in_ticks, bob_offset)));
    cluster_poses(base, count, seed, extent)
}

/// `ItemEntityRenderer.submitMultipleFromCount`: the pose of each copy of a stack of `count`
/// items, from `base`, in the order they are submitted. The offsets are drawn from `seed`.
pub fn cluster_poses(mut base: Pose, count: i32, seed: i32, extent: ModelExtent) -> Vec<Pose> {
    let mut poses = Vec::new();
    if count == 0 {
        return poses;
    }
    let mut random = RandomSource::create(i64::from(seed));
    let depth = extent.depth as f32;
    if depth > 0.0625 {
        poses.push(base);
        for _ in 1..count {
            let mut pose = base;
            let xo = (random.next_float() * 2.0 - 1.0) * 0.15;
            let yo = (random.next_float() * 2.0 - 1.0) * 0.15;
            let zo = (random.next_float() * 2.0 - 1.0) * 0.15;
            pose.translate(xo, yo, zo);
            poses.push(pose);
        }
    } else {
        let offset_z = depth * 1.5;
        base.translate(0.0, 0.0, -(offset_z * (count - 1) as f32 / 2.0));
        poses.push(base);
        base.translate(0.0, 0.0, offset_z);
        for _ in 1..count {
            let mut pose = base;
            let xo = (random.next_float() * 2.0 - 1.0) * 0.15 * 0.5;
            let yo = (random.next_float() * 2.0 - 1.0) * 0.15 * 0.5;
            pose.translate(xo, yo, 0.0);
            poses.push(pose);
            base.translate(0.0, 0.0, offset_z);
        }
    }
    poses
}

/// `ItemStackRenderState.getModelBoundingBox`: the extent of the layers'
/// vertices after each layer's display transform.
pub fn model_extent(layers: &[ItemLayer]) -> ModelExtent {
    let (mut min_y, mut max_y, mut min_z, mut max_z) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for layer in layers {
        let mut pose = Pose::default();
        apply_display_transform(&mut pose, layer, false);
        for vertex in &layer.mesh.vertices {
            let [x, y, z] = vertex.position;
            let [_, y, z] = pose.pose.transform_position(x, y, z);
            let _ = x;
            min_y = min_y.min(f64::from(y));
            max_y = max_y.max(f64::from(y));
            min_z = min_z.min(f64::from(z));
            max_z = max_z.max(f64::from(z));
        }
    }
    if min_y == f64::MAX {
        return ModelExtent {
            min_y: 0.0,
            max_y: 0.0,
            depth: 0.0,
        };
    }
    ModelExtent {
        min_y,
        max_y,
        depth: max_z - min_z,
    }
}

/// `ItemTransform.apply`: the display transform of the layer's context,
/// mirrored for the left hand, and then the half block that centres the
/// model (which a model with no transform gets too).
pub fn apply_display_transform(pose: &mut Pose, layer: &ItemLayer, left_hand: bool) {
    let Some(transform) = layer.transform else {
        pose.translate(-0.5, -0.5, -0.5);
        return;
    };
    let [rx, mut ry, mut rz] = transform.rotation;
    let mut tx = transform.translation[0];
    if left_hand {
        tx = -tx;
        ry = -ry;
        rz = -rz;
    }
    pose.translate(tx, transform.translation[1], transform.translation[2]);
    let degrees = (std::f64::consts::PI / 180.0) as f32;
    pose.rotate(&Quaternion::rotation_xyz(
        rx * degrees,
        ry * degrees,
        rz * degrees,
    ));
    let [sx, sy, sz] = transform.scale;
    pose.scale(sx, sy, sz);
    pose.translate(-0.5, -0.5, -0.5);
}

/// `minecraft_mix_light` with the level's two lights, which the entity
/// shader applies to a face's normal.
fn entity_light(normal: [f32; 3]) -> f32 {
    let light = |direction: [f32; 3]| {
        let direction = normalize(direction);
        (direction[0] * normal[0] + direction[1] * normal[1] + direction[2] * normal[2]).max(0.0)
    };
    let [first, second] = crate::diffuse::level_directions();
    ((light(first) + light(second)) * 0.6 + 0.4).min(1.0)
}

/// A stack's dropped item, drawn: each copy (`poses`) with each layer's
/// display transform, lit by the block and sky light at the item.
pub fn item_entity_vertices(layers: &[ItemLayer], poses: &[Pose], light: i32) -> Vec<Vertex> {
    item_layer_vertices(layers, poses, light, false)
}

/// The layers of a stack at each of `poses`, each with its display transform
/// (mirrored for a left hand), lit by `light`.
pub fn item_layer_vertices(
    layers: &[ItemLayer],
    poses: &[Pose],
    light: i32,
    left_hand: bool,
) -> Vec<Vertex> {
    let light = light_uv(light);
    let mut vertices = Vec::new();
    for copy in poses {
        for layer in layers {
            let mut pose = *copy;
            apply_display_transform(&mut pose, layer, left_hand);
            push_layer(&mut vertices, layer, &pose, light);
        }
    }
    vertices
}

/// A block model drawn on an entity (`BlockModelRenderState.submit`): the
/// model from `pose`, which the layer that places it has already moved to the
/// block's corner, with no display transform. `overlay` is the overlay's code
/// (`mob_model::overlay_code`, zero for none), which rides above the shade.
pub fn entity_block_vertices(
    layer: &ItemLayer,
    pose: &Pose,
    (light, overlay): (i32, f32),
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    push_layer(&mut vertices, layer, pose, light_uv(light));
    // The shade is below two and the held shader reads what is above as the
    // overlay.
    for vertex in &mut vertices {
        vertex.shade += 2.0 * overlay;
    }
    vertices
}

fn push_layer(vertices: &mut Vec<Vertex>, layer: &ItemLayer, pose: &Pose, light: [f32; 2]) {
    for (quad, info) in layer.mesh.quads.iter().enumerate() {
        let (dx, dy, dz) = info.direction.step();
        let normal = pose.transform_normal([dx as f32, dy as f32, dz as f32]);
        let shade = entity_light(normal);
        for index in [0, 1, 2, 0, 2, 3] {
            let source = &layer.mesh.vertices[quad * 4 + index];
            let [x, y, z] = source.position;
            vertices.push(Vertex {
                position: pose.pose.transform_position(x, y, z),
                uv: source.uv,
                shade,
                tint: source.colour(),
                light,
            });
        }
    }
}

/// `ItemFeatureRenderer.prepareFoilSubmit` for a standard foil: the same triangles again, their
/// texture coordinate (of the block atlas, in 0 to 1) moved by the glint's matrix into the glint
/// texture, which sits at `glint` (`[u0, v0, u1, v1]`) in the packed image and repeats.
pub fn item_glint_vertices(
    triangles: &[Vertex],
    matrix: &[f32; 16],
    glint: [f32; 4],
) -> Vec<Vertex> {
    let size = [glint[2] - glint[0], glint[3] - glint[1]];
    triangles
        .iter()
        .map(|vertex| {
            let moved = crate::render_types::glint_uv(matrix, vertex.uv);
            Vertex {
                position: vertex.position,
                uv: [glint[0] + moved[0] * size[0], glint[1] + moved[1] * size[1]],
                shade: 0.0,
                tint: [glint[0], glint[1], size[0], size[1]],
                light: [0.0; 2],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copies_shown_follow_the_size_of_the_stack() {
        let shown: Vec<i32> = [0, 1, 2, 16, 17, 32, 33, 48, 49, 64]
            .into_iter()
            .map(rendered_amount)
            .collect();
        assert_eq!(shown, [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn a_glint_moves_the_atlas_coordinate_into_the_glint_texture() {
        let vertex = Vertex {
            position: [1.0, 2.0, 3.0],
            uv: [0.25, 0.5],
            shade: 1.0,
            tint: [1.0; 4],
            light: [240.0; 2],
        };
        let matrix = crate::render_types::glint_texture_matrix(0, 0.5, 8.0);
        let glint = item_glint_vertices(&[vertex], &matrix, [0.5, 0.25, 0.625, 0.375]);
        let moved = crate::render_types::glint_uv(&matrix, [0.25, 0.5]);
        assert_eq!(glint[0].position, vertex.position);
        assert_eq!(
            glint[0].uv,
            [0.5 + moved[0] * 0.125, 0.25 + moved[1] * 0.125]
        );
        assert_eq!(glint[0].tint, [0.5, 0.25, 0.125, 0.125]);
    }
}
