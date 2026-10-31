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
    let light = crate::block::light::light_uv(packed_light);
    let mut vertices = Vec::new();
    for corners in model.quads(pose) {
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
    let light = crate::block::light::light_uv(placement.light);
    let shade = overlay_code(placement.red_overlay, placement.white_overlay);
    let mut vertices = Vec::with_capacity(QUADS_TO_EXPECT * 6);
    model.each_quad(&pose, &mut |corners| {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade,
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], colour],
                light,
            });
        }
    });
    vertices
}

/// How many faces most mobs have, so a model's vertices rarely grow their vector.
const QUADS_TO_EXPECT: usize = 48;

/// The posed model's triangles, placed, each vertex carrying its normal in
/// its tint for the entity shader, with texture coordinates in the packed
/// entity textures.
pub fn mob_vertices(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    mob_vertices_coloured(model, placement, texture, None)
}

/// `RenderTypes.entityTranslucent` sorts the quads of what it draws far to near
/// (`VertexSorting.DISTANCE_TO_ORIGIN` over each quad's centre, the midpoint of
/// its first and third corner), so a blended model shows its far faces
/// through its near ones. `vertices` are the six triangle corners of each quad
/// [`mob_vertices`] makes, and `camera` is where the eye is among them.
pub fn sort_translucent_quads(vertices: &mut [Vertex], camera: [f32; 3]) {
    let key = |quad: &[Vertex]| -> f32 {
        let centre =
            |axis: usize| (quad[0].position[axis] + quad[2].position[axis]) / 2.0 - camera[axis];
        let (x, y, z) = (centre(0), centre(1), centre(2));
        x * x + y * y + z * z
    };
    let mut quads: Vec<(f32, [Vertex; 6])> = vertices
        .as_chunks::<6>()
        .0
        .iter()
        .map(|quad| {
            (
                key(quad),
                [quad[0], quad[1], quad[2], quad[3], quad[4], quad[5]],
            )
        })
        .collect();
    // A stable sort, as the reference's merge sort is.
    quads.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (slot, (_, quad)) in vertices.as_chunks_mut::<6>().0.iter_mut().zip(quads) {
        slot.copy_from_slice(&quad);
    }
}

/// A glowing layer's triangles (`EyesLayer`, `RenderTypes.eyes`): the same
/// posed model, drawn unlit and without the hurt overlay, each vertex's tint
/// the white it is multiplied by.
pub fn mob_vertices_glow(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    mob_vertices_glow_posed(model, &placement_pose(placement), texture)
}

/// [`mob_vertices_glow`] for a model the renderer has already placed.
pub fn mob_vertices_glow_posed(model: &MobModel, pose: &Pose, texture: TextureRect) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for corners in model.quads(pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: 0.0,
                tint: [1.0; 4],
                light: [0.0; 2],
            });
        }
    }
    vertices
}

/// An energy swirl's triangles (`EnergySwirlLayer`, `RenderTypes.energySwirl`):
/// the same posed model, each vertex carrying where its texture is in the
/// packed image (the corner and the size, in its tint), the texture offset
/// (in its light) and the colour (`0xRRGGBB`, in its shade) that the shader
/// moves, wraps and multiplies the texture by.
pub fn mob_vertices_swirl(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    offset: [f32; 2],
    colour: u32,
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let rect = [
        texture[0],
        texture[1],
        texture[2] - texture[0],
        texture[3] - texture[1],
    ];
    let mut vertices = Vec::new();
    for corners in model.quads(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: (colour & 0x00FF_FFFF) as f32,
                tint: rect,
                light: offset,
            });
        }
    }
    vertices
}

/// A glowing layer drawn lit by faces but not by the world (`EMISSIVE` with
/// per-face lighting, `RenderTypes.entityTranslucentEmissive`): the same posed
/// model, no overlay, its light the full bright the entity shader reads as
/// negative.
pub fn mob_vertices_emissive(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    emissive_vertices(model, placement, texture, (0.0, [-1.0; 2]))
}

/// A layer drawn unlit but shaded by face and blended (`RenderTypes.entityTranslucentEmissive`)
/// by `LivingEntityEmissiveLayer`: the same posed model, its colour white with
/// this alpha (a whole 255th, as `ARGB.white` makes it), and the hurt overlay
/// of the mob it covers.
pub fn mob_vertices_emissive_faded(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    alpha: f32,
) -> Vec<Vertex> {
    let alpha = (alpha * 255.0).floor() / 255.0;
    emissive_vertices(
        model,
        placement,
        texture,
        (overlay_code(placement.red_overlay, 0.0), [-1.0, alpha]),
    )
}

/// An emissive layer's triangles: each vertex carries its face's normal for
/// the shader, and its light the negative full bright; the second value of
/// the light, when not negative, is the layer's alpha.
pub(super) fn emissive_vertices(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    (shade, light): (f32, [f32; 2]),
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let mut vertices = Vec::new();
    for corners in model.quads(&pose) {
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

/// A layer whose texture scrolls along `u` and repeats (`RenderPipelines.BREEZE_WIND`):
/// lit by the world but not by faces, no overlay. The packed image does not
/// repeat, so each quad the scroll carries across the texture's edge is cut
/// there, and the part beyond is drawn from the start of the texture.
pub fn mob_vertices_scrolled(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    u_offset: f32,
) -> Vec<Vertex> {
    mob_vertices_scrolled_posed(
        model,
        &placement_pose(placement),
        placement.light,
        (texture, u_offset),
    )
}

/// [`mob_vertices_scrolled`] for a model the renderer has already placed (a wind charge's).
pub fn mob_vertices_scrolled_posed(
    model: &MobModel,
    pose: &Pose,
    packed_light: i32,
    (texture, u_offset): (TextureRect, f32),
) -> Vec<Vertex> {
    let light = crate::block::light::light_uv(packed_light);
    let mut vertices = Vec::new();
    for corners in model.quads(pose) {
        let scrolled: Vec<([f32; 3], [f32; 2])> = corners
            .iter()
            .map(|corner| (corner.position, [corner.uv[0] + u_offset, corner.uv[1]]))
            .collect();
        for (polygon, shift) in [
            (clip_u(&scrolled, true), 0.0),
            (clip_u(&scrolled, false), 1.0),
        ] {
            // A fan of triangles from the first corner.
            for index in 1..polygon.len().saturating_sub(1) {
                for (position, coordinate) in [polygon[0], polygon[index], polygon[index + 1]] {
                    vertices.push(Vertex {
                        position,
                        uv: uv(texture, [coordinate[0] - shift, coordinate[1]]),
                        shade: 0.0,
                        tint: [0.0; 4],
                        light,
                    });
                }
            }
        }
    }
    vertices
}
