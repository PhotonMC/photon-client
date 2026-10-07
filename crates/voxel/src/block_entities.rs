//! The block entities of the world (`ChestRenderer`, `ShulkerBoxRenderer`, `BellRenderer`): each
//! one's model, in the texture of its kind, turned to face, its lid as far open or its clapper
//! as far swung as the server's block events have it.

use voxel_core::direction::Direction;
use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::banner;
use voxel_render::bell;
use voxel_render::chest::{ChestKind, ChestMaterial, chest_pose, eased_open};
use voxel_render::entity_textures::EntityTextures;
use voxel_render::joml::Pose;
use voxel_render::mob_model::{mob_vertices_posed, mob_vertices_tinted};
use voxel_render::scene::Vertex;
use voxel_render::shulker_box;
use voxel_render::skull::{SkullKind, ground_transformation, wall_transformation};
use voxel_world::{BannerView, DYES, ShulkerBoxView, SkullBlockView};

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

/// How far from the camera, in chunks, block entities are drawn.
const RADIUS_CHUNKS: i32 = 8;

/// The paths (under `textures/`) of every chest and shulker box texture.
const TEXTURES: [&str; 37] = [
    "entity/chest/normal",
    "entity/chest/normal_left",
    "entity/chest/normal_right",
    "entity/chest/trapped",
    "entity/chest/trapped_left",
    "entity/chest/trapped_right",
    "entity/chest/copper",
    "entity/chest/copper_left",
    "entity/chest/copper_right",
    "entity/chest/copper_exposed",
    "entity/chest/copper_exposed_left",
    "entity/chest/copper_exposed_right",
    "entity/chest/copper_weathered",
    "entity/chest/copper_weathered_left",
    "entity/chest/copper_weathered_right",
    "entity/chest/copper_oxidized",
    "entity/chest/copper_oxidized_left",
    "entity/chest/copper_oxidized_right",
    "entity/chest/ender",
    "entity/bell/bell_body",
    "entity/shulker/shulker",
    "entity/shulker/shulker_white",
    "entity/shulker/shulker_orange",
    "entity/shulker/shulker_magenta",
    "entity/shulker/shulker_light_blue",
    "entity/shulker/shulker_yellow",
    "entity/shulker/shulker_lime",
    "entity/shulker/shulker_pink",
    "entity/shulker/shulker_gray",
    "entity/shulker/shulker_light_gray",
    "entity/shulker/shulker_cyan",
    "entity/shulker/shulker_purple",
    "entity/shulker/shulker_blue",
    "entity/shulker/shulker_brown",
    "entity/shulker/shulker_green",
    "entity/shulker/shulker_red",
    "entity/shulker/shulker_black",
];

/// The banner textures: the cloth and each pattern's mask.
const BANNER_TEXTURES: [&str; 44] = [
    "entity/banner/banner_base",
    "entity/banner/base",
    "entity/banner/border",
    "entity/banner/bricks",
    "entity/banner/circle",
    "entity/banner/creeper",
    "entity/banner/cross",
    "entity/banner/curly_border",
    "entity/banner/diagonal_left",
    "entity/banner/diagonal_right",
    "entity/banner/diagonal_up_left",
    "entity/banner/diagonal_up_right",
    "entity/banner/flow",
    "entity/banner/flower",
    "entity/banner/globe",
    "entity/banner/gradient",
    "entity/banner/gradient_up",
    "entity/banner/guster",
    "entity/banner/half_horizontal",
    "entity/banner/half_horizontal_bottom",
    "entity/banner/half_vertical",
    "entity/banner/half_vertical_right",
    "entity/banner/mojang",
    "entity/banner/piglin",
    "entity/banner/rhombus",
    "entity/banner/skull",
    "entity/banner/small_stripes",
    "entity/banner/square_bottom_left",
    "entity/banner/square_bottom_right",
    "entity/banner/square_top_left",
    "entity/banner/square_top_right",
    "entity/banner/straight_cross",
    "entity/banner/stripe_bottom",
    "entity/banner/stripe_center",
    "entity/banner/stripe_downleft",
    "entity/banner/stripe_downright",
    "entity/banner/stripe_left",
    "entity/banner/stripe_middle",
    "entity/banner/stripe_right",
    "entity/banner/stripe_top",
    "entity/banner/triangle_bottom",
    "entity/banner/triangle_top",
    "entity/banner/triangles_bottom",
    "entity/banner/triangles_top",
];

pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES
        .into_iter()
        .chain(SkullKind::ALL.into_iter().map(SkullKind::texture))
        .chain(BANNER_TEXTURES)
}

/// `ChestRenderer.getChestMaterial` from the block's name.
fn material_of(name: &str) -> ChestMaterial {
    match name {
        "minecraft:ender_chest" => ChestMaterial::Ender,
        "minecraft:trapped_chest" => ChestMaterial::Trapped,
        "minecraft:copper_chest" | "minecraft:waxed_copper_chest" => {
            ChestMaterial::CopperUnaffected
        }
        "minecraft:exposed_copper_chest" | "minecraft:waxed_exposed_copper_chest" => {
            ChestMaterial::CopperExposed
        }
        "minecraft:weathered_copper_chest" | "minecraft:waxed_weathered_copper_chest" => {
            ChestMaterial::CopperWeathered
        }
        "minecraft:oxidized_copper_chest" | "minecraft:waxed_oxidized_copper_chest" => {
            ChestMaterial::CopperOxidized
        }
        _ => ChestMaterial::Regular,
    }
}

/// `LightCoordsUtil.max`: each of the block and sky light the greater.
fn max_light(first: i32, second: i32) -> i32 {
    let block = |packed: i32| packed >> 4 & 15;
    let sky = |packed: i32| packed >> 20 & 15;
    block(first).max(block(second)) << 4 | sky(first).max(sky(second)) << 20
}

/// The block entities near the camera.
pub fn block_entities(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    let mut vertices = Vec::new();
    for (x, y, z) in world.chest_positions_near(centre, RADIUS_CHUNKS) {
        let Some(chest) = world.chest_view(x, y, z) else {
            if let Some(shulker) = world.shulker_box_view(x, y, z) {
                vertices.extend(shulker_box_vertices(
                    session,
                    models,
                    textures,
                    view,
                    (x, y, z),
                    shulker,
                ));
            } else if let Some(banner) = world.banner_view((x, y, z)) {
                vertices.extend(banner_vertices(
                    session,
                    models,
                    textures,
                    view,
                    (x, y, z),
                    &banner,
                ));
            } else if let Some(skull) = world.skull_block_view((x, y, z), partial) {
                vertices.extend(skull_vertices(
                    session,
                    models,
                    textures,
                    view,
                    (x, y, z),
                    skull,
                ));
            } else if let Some(shake) = world.bell_shake((x, y, z), partial) {
                vertices.extend(bell_vertices(
                    session,
                    models,
                    textures,
                    view,
                    (x, y, z),
                    shake,
                ));
            }
            continue;
        };
        let kind = match chest.kind {
            1 => ChestKind::Left,
            2 => ChestKind::Right,
            _ => ChestKind::Single,
        };
        // The ender chest has one half only, whatever its state says.
        let material = material_of(chest.name);
        let kind = if material == ChestMaterial::Ender {
            ChestKind::Single
        } else {
            kind
        };
        let (Some(rect), Some(model)) = (
            textures.rect(&material.texture(kind)),
            models.model(kind.layer()),
        ) else {
            continue;
        };
        let at =
            |pos: (i32, i32, i32)| Vec3::new(f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        let mut light = packed_light(world, at((x, y, z)));
        if let Some(other) = chest.partner {
            light = max_light(light, packed_light(world, at(other)));
        }
        let open = world.chest_openness((x, y, z), chest.partner, partial);
        model.setup_chest(eased_open(open));
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let facing: Direction = chest.facing;
        let pose = chest_pose(&base, facing);
        vertices.extend(mob_vertices_posed(model, &pose, rect, (light, false)));
    }
    vertices
}

/// One shulker box, its lid as far up and turned as its progress says.
fn shulker_box_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    shulker: ShulkerBoxView,
) -> Vec<Vertex> {
    let world = session.world();
    let (Some(rect), Some(model)) = (
        textures.rect(&shulker_box::texture(shulker.color)),
        models.model(shulker_box::LAYER),
    ) else {
        return Vec::new();
    };
    let light = packed_light(world, Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
    model.setup_shulker_box(world.shulker_box_progress((x, y, z), view.partial as f32));
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let pose = shulker_box::box_pose(&base, shulker.facing);
    mob_vertices_posed(model, &pose, rect, (light, false))
}

/// One bell, swung as far as its ticks since the blow say.
fn bell_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    (ticks, shake): (f32, Option<Direction>),
) -> Vec<Vertex> {
    let (Some(rect), Some(model)) = (textures.rect(bell::TEXTURE), models.model(bell::LAYER))
    else {
        return Vec::new();
    };
    let light = packed_light(
        session.world(),
        Vec3::new(f64::from(x), f64::from(y), f64::from(z)),
    );
    model.setup_bell(ticks, shake);
    let mut pose = Pose::default();
    pose.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    mob_vertices_posed(model, &pose, rect, (light, false))
}

/// One skull or head block, turned to its wall or its segment, its jaw or ears at the animation.
fn skull_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    skull: SkullBlockView,
) -> Vec<Vertex> {
    // A wall head is named for its wall (`minecraft:zombie_wall_head`); the item's name is not.
    let Some(kind) = SkullKind::of_item(&skull.name.replace("_wall_", "_")) else {
        return Vec::new();
    };
    let (Some(rect), Some(model)) = (textures.rect(kind.texture()), models.model(kind.layer()))
    else {
        return Vec::new();
    };
    let light = packed_light(
        session.world(),
        Vec3::new(f64::from(x), f64::from(y), f64::from(z)),
    );
    model.setup_skull(kind, skull.animation);
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let transformation = match skull.wall {
        Some(facing) => wall_transformation(facing),
        None => ground_transformation(skull.rotation),
    };
    let pose = voxel_render::skull::block_pose(&base, &transformation);
    mob_vertices_posed(model, &pose, rect, (light, false))
}

/// One banner: its pole and bar, its flag and the layers of the flag, each tinted by its dye and
/// blended over the one before.
fn banner_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    banner: &BannerView,
) -> Vec<Vertex> {
    let world = session.world();
    let (main, flag) = banner::layers(banner.wall.is_some());
    let light = packed_light(world, Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let transformation = match banner.wall {
        Some(facing) => banner::wall_transformation(facing),
        None => banner::ground_transformation(banner.rotation),
    };
    let pose = voxel_render::skull::block_pose(&base, &transformation);
    eprintln!(
        "BANNERDEBUG {:?} {:?} {:?}",
        banner.color,
        banner.patterns,
        world
            .block_entity_at(x, y, z)
            .map(|e| (e.type_id, e.tag.clone()))
    );
    let phase = banner::flag_phase((x, y, z), world.game_time() as i64, view.partial as f32);
    let mut vertices = Vec::new();
    if let (Some(rect), Some(pole)) = (textures.rect(banner::BASE_TEXTURE), models.model(main)) {
        pole.setup_banner_flag(phase);
        vertices.extend(mob_vertices_posed(pole, &pose, rect, (light, false)));
    }
    let Some(cloth) = models.model(flag) else {
        return vertices;
    };
    cloth.setup_banner_flag(phase);
    if let Some(rect) = textures.rect(banner::BASE_TEXTURE) {
        vertices.extend(mob_vertices_posed(cloth, &pose, rect, (light, false)));
    }
    let dye = |name: &str| {
        DYES.iter()
            .position(|candidate| *candidate == name)
            .and_then(voxel_data::entity_type::dye_texture_colour)
            .map(|colour| colour & 0x00FF_FFFF)
    };
    let layers = std::iter::once((banner::PATTERN_BASE_TEXTURE.to_owned(), dye(banner.color)))
        .chain(
            banner
                .patterns
                .iter()
                .take(banner::MAX_PATTERNS)
                .map(|(pattern, color)| (banner::pattern_texture(pattern), dye(color))),
        );
    for (texture, colour) in layers {
        let (Some(rect), Some(colour)) = (textures.rect(&texture), colour) else {
            continue;
        };
        vertices.extend(mob_vertices_tinted(cloth, &pose, rect, light, colour));
    }
    vertices
}
