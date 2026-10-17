//! The block entities of the world (`ChestRenderer`, `ShulkerBoxRenderer`, `BellRenderer`): each
//! one's model, in the texture of its kind, turned to face, its lid as far open or its clapper
//! as far swung as the server's block events have it.

use photon_core::direction::Direction;
use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block_entity::banner;
use photon_render::block_entity::bell;
use photon_render::block_entity::chest::{ChestKind, ChestMaterial, chest_pose, eased_open};
use photon_render::block_entity::shulker_box;
use photon_render::block_entity::skull::{SkullKind, ground_transformation, wall_transformation};
use photon_render::entity::mob_model::{mob_vertices_posed, mob_vertices_tinted};
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::scene::Vertex;
use photon_world::{BannerView, DYES, ShulkerBoxView, SkullBlockView};

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

/// How far from the camera, in chunks, block entities are drawn.
const RADIUS_CHUNKS: i32 = 8;

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
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    for (x, y, z) in world.chest_positions_near(centre, RADIUS_CHUNKS) {
        let Some(chest) = world.chest_view(x, y, z) else {
            other_block_entity(session, models, textures, view, (x, y, z), vertices);
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
}

/// A block entity that is no chest: a shulker box, a banner, a head, a bell or a book.
fn other_block_entity(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let at = (x, y, z);
    if let Some(shulker) = world.shulker_box_view(x, y, z) {
        vertices.extend(shulker_box_vertices(
            session, models, textures, view, at, shulker,
        ));
    } else if let Some(banner) = world.banner_view(at) {
        banner_vertices(session, models, textures, view, at, &banner, vertices);
    } else if let Some(skull) = world.skull_block_view(at, partial) {
        vertices.extend(skull_vertices(session, models, textures, view, at, skull));
    } else {
        bell_or_book(session, models, textures, view, at, vertices);
    }
}

/// A bell, an enchanting table's book or a lectern's.
fn bell_or_book(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    at: (i32, i32, i32),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    if let Some(shake) = world.bell_shake(at, partial) {
        vertices.extend(bell_vertices(session, models, textures, view, at, shake));
    } else if let Some(table) = world.enchant_table_view(at, partial) {
        vertices.extend(book_vertices(session, models, textures, view, at, table));
    } else if let Some(turn) = world.lectern_book_turn(at) {
        vertices.extend(lectern_vertices(session, models, textures, view, at, turn));
    } else if let Some(pot) = world.pot_view(at, partial) {
        vertices.extend(pot_vertices(session, models, textures, view, at, &pot));
    }
}

/// One enchanting table's book, floating over it, open to the player near it.
fn book_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    table: photon_world::EnchantTableView,
) -> Vec<Vertex> {
    use photon_render::block_entity::enchant_table;
    let (Some(rect), Some(model)) = (
        textures.rect(enchant_table::TEXTURE),
        models.model(enchant_table::LAYER),
    ) else {
        return Vec::new();
    };
    let light = packed_light(
        session.world(),
        Vec3::new(f64::from(x), f64::from(y), f64::from(z)),
    );
    let book = enchant_table::book_pose(table.time, table.flip, table.open);
    model.setup_book(book.openness, book.pages);
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let pose = enchant_table::table_pose(&base, table.time, table.y_rot);
    mob_vertices_posed(model, &pose, rect, (light, false))
}

/// The book on one lectern, held open on the slope, turned to face along it.
fn lectern_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    turn: f32,
) -> Vec<Vertex> {
    use photon_render::block_entity::{enchant_table, lectern};
    let (Some(rect), Some(model)) = (
        textures.rect(enchant_table::TEXTURE),
        models.model(enchant_table::LAYER),
    ) else {
        return Vec::new();
    };
    let light = packed_light(
        session.world(),
        Vec3::new(f64::from(x), f64::from(y), f64::from(z)),
    );
    let book = lectern::lectern_book();
    model.setup_book(book.openness, book.pages);
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let pose = lectern::lectern_pose(&base, turn);
    mob_vertices_posed(model, &pose, rect, (light, false))
}

/// One decorated pot: its neck and planes, and each side in the picture of the sherd set in it,
/// turned to its facing and wobbling as far as its event has it.
fn pot_vertices(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    (x, y, z): (i32, i32, i32),
    pot: &photon_world::PotView,
) -> Vec<Vertex> {
    use photon_render::block_entity::decorated_pot as decorated;
    let light = packed_light(
        session.world(),
        Vec3::new(f64::from(x), f64::from(y), f64::from(z)),
    );
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let pose = decorated::pot_pose(&base, pot.facing, pot.wobble);
    let mut vertices = Vec::new();
    // The order the reference draws them in; the sherds are held back, left, right, front.
    let parts = decorated::BASE_PARTS
        .into_iter()
        .map(|part| {
            (
                decorated::BASE_LAYER,
                part,
                decorated::BASE_TEXTURE.to_owned(),
            )
        })
        .chain(decorated::SIDE_PARTS.into_iter().map(|part| {
            let sherd = match part {
                "back" => &pot.sherds[0],
                "left" => &pot.sherds[1],
                "right" => &pot.sherds[2],
                _ => &pot.sherds[3],
            };
            (
                decorated::SIDES_LAYER,
                part,
                decorated::side_texture(sherd.as_deref()),
            )
        }));
    for (layer, part, texture) in parts {
        let (Some(rect), Some(model)) = (textures.rect(&texture), models.model(layer)) else {
            continue;
        };
        let all = if layer == decorated::BASE_LAYER {
            &decorated::BASE_PARTS[..]
        } else {
            &decorated::SIDE_PARTS[..]
        };
        model.show_only(all, part);
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
    let pose = photon_render::block_entity::skull::block_pose(&base, &transformation);
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
    vertices: &mut Vec<Vertex>,
) {
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
    let pose = photon_render::block_entity::skull::block_pose(&base, &transformation);
    let phase = banner::flag_phase((x, y, z), world.game_time() as i64, view.partial as f32);
    if let (Some(rect), Some(pole)) = (textures.rect(banner::BASE_TEXTURE), models.model(main)) {
        pole.setup_banner_flag(phase);
        vertices.extend(mob_vertices_posed(pole, &pose, rect, (light, false)));
    }
    let Some(cloth) = models.model(flag) else {
        return;
    };
    cloth.setup_banner_flag(phase);
    if let Some(rect) = textures.rect(banner::BASE_TEXTURE) {
        vertices.extend(mob_vertices_posed(cloth, &pose, rect, (light, false)));
    }
    let dye = |name: &str| {
        DYES.iter()
            .position(|candidate| *candidate == name)
            .and_then(photon_data::entity_type::dye_texture_colour)
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
}
