use super::*;

/// The part of a convex polygon on one side of `u = 1` (the lower side, or the
/// upper), cut where its edges cross it.
pub(super) fn clip_u(polygon: &[([f32; 3], [f32; 2])], below: bool) -> Vec<([f32; 3], [f32; 2])> {
    let inside = |u: f32| if below { u <= 1.0 } else { u >= 1.0 };
    let mut out = Vec::new();
    for (index, &(position, coordinate)) in polygon.iter().enumerate() {
        let (next_position, next_coordinate) = polygon[(index + 1) % polygon.len()];
        if inside(coordinate[0]) {
            out.push((position, coordinate));
        }
        if inside(coordinate[0]) != inside(next_coordinate[0]) {
            let t = (1.0 - coordinate[0]) / (next_coordinate[0] - coordinate[0]);
            out.push((
                std::array::from_fn(|axis| {
                    position[axis] + (next_position[axis] - position[axis]) * t
                }),
                [
                    1.0,
                    coordinate[1] + (next_coordinate[1] - coordinate[1]) * t,
                ],
            ));
        }
    }
    out
}

/// [`mob_vertices`] for a model the renderer has already placed: lit by the
/// packed light, with the hurt overlay or not.
pub fn mob_vertices_posed(
    model: &MobModel,
    pose: &Pose,
    texture: TextureRect,
    (packed_light, red_overlay): (i32, bool),
) -> Vec<Vertex> {
    posed_vertices(
        model,
        pose,
        texture,
        (packed_light, overlay_code(red_overlay, 0.0)),
    )
}

/// [`mob_vertices_posed`] for a layer the pipeline blends and a colour (`0xRRGGBB`) multiplies
/// (`RenderTypes.bannerPattern` with the layer's dye): each texel's own alpha decides how much of
/// it covers what the layer lies on.
pub fn mob_vertices_tinted(
    model: &MobModel,
    pose: &Pose,
    texture: TextureRect,
    packed_light: i32,
    colour: u32,
) -> Vec<Vertex> {
    let mut vertices = posed_vertices(model, pose, texture, (packed_light, BLENDED_SHADE + 255.0));
    let colour = crate::scene::entity_colour(colour);
    for vertex in &mut vertices {
        vertex.tint[3] = colour;
    }
    vertices
}

/// [`mob_vertices_posed`] for a model blended by a colour of this alpha (a whole 255th, the
/// colour's own byte), sorted far to near as the reference's translucent pipeline sorts it.
pub fn mob_vertices_blended(
    model: &MobModel,
    pose: &Pose,
    texture: TextureRect,
    (packed_light, alpha): (i32, u8),
    camera: [f32; 3],
) -> Vec<Vertex> {
    let mut vertices = posed_vertices(
        model,
        pose,
        texture,
        (packed_light, BLENDED_SHADE + f32::from(alpha)),
    );
    sort_translucent_quads(&mut vertices, camera);
    vertices
}

pub(super) fn posed_vertices(
    model: &MobModel,
    pose: &Pose,
    texture: TextureRect,
    (packed_light, shade): (i32, f32),
) -> Vec<Vertex> {
    let light = crate::block_light::light_uv(packed_light);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade,
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 0.0],
                light,
            });
        }
    }
    vertices
}

/// [`mob_vertices_posed`] for an entity that dissolves (`entityCutoutDissolve`): no hurt
/// overlay, and each vertex's shade carries the alpha (a whole 255th, as
/// `ARGB.white` makes it) the entity's mask is held against.
pub fn mob_vertices_dissolving(
    model: &MobModel,
    pose: &Pose,
    texture: TextureRect,
    (packed_light, alpha): (i32, f32),
) -> Vec<Vertex> {
    let alpha = (alpha * 255.0).floor() / 255.0;
    let mut vertices = mob_vertices_posed(model, pose, texture, (packed_light, false));
    for vertex in &mut vertices {
        vertex.shade = alpha;
    }
    vertices
}

/// [`mob_vertices`], each texel multiplied by a colour (`0xRRGGBB`): a
/// sheep's wool.
pub fn mob_vertices_coloured(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    colour: Option<u32>,
) -> Vec<Vertex> {
    let colour = colour.map_or(0.0, crate::scene::entity_colour);
    let pose = placement_pose(placement);
    let light = crate::block_light::light_uv(placement.light);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: overlay_code(placement.red_overlay, placement.white_overlay),
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], colour],
                light,
            });
        }
    }
    vertices
}
