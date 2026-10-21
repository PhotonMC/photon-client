//! The fish (`CodRenderer`, `SalmonRenderer`, `TropicalFishRenderer`,
//! `PufferfishRenderer`): their textures, the sizes of a salmon and a
//! pufferfish, a tropical fish's colours and pattern layer, the swing of their
//! tails and fins, and how they lie out of water.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{
    MobPlacement, cod_turns, mob_vertices_coloured, pufferfish_turns, salmon_turns,
    tropical_fish_turns,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::mobs::fish::{
    COD_TEXTURE, PUFFERFISH_TEXTURE, SALMON_TEXTURE, TROPICAL_LARGE_TEXTURE,
    TROPICAL_SMALL_TEXTURE, TropicalPattern, pufferfish_layer, tropical_base_dye,
    tropical_pattern_dye,
};

use crate::equipment::armor::Armor;
use crate::equipment::held_items::HeldItem;
use crate::mobs::aquatic::axolotls;
use crate::mobs::aquatic::dolphins;
use crate::mobs::aquatic::frogs;
use crate::mobs::aquatic::guardians;
use crate::mobs::aquatic::nautilus;
use crate::mobs::aquatic::squids;
use crate::mobs::aquatic::turtles;
use crate::mobs::{MobModels, placement};
use crate::objects::boats;
use crate::overlays::FrameView;

/// The paths (under `textures/`) of the textures the fish wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    let patterns = [false, true].into_iter().flat_map(|large| {
        (0..6).map(move |index| TropicalPattern { large, index }.pattern_texture())
    });
    [
        COD_TEXTURE,
        SALMON_TEXTURE,
        PUFFERFISH_TEXTURE,
        TROPICAL_SMALL_TEXTURE,
        TROPICAL_LARGE_TEXTURE,
    ]
    .into_iter()
    .chain(patterns)
}

/// The animals of the water the server added: squids, fish, dolphins, turtles, axolotls, frogs, tadpoles, guardians and nautiluses.
pub fn remote_water_animals(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    squids::remote_squids(session, models, textures, view, vertices);
    // The boats afloat on the same water.
    boats::remote_boats(session, models, textures, view, vertices);
    remote_fish(session, models, textures, view, vertices);
    dolphins::remote_dolphins(session, models, textures, (view, held), vertices);
    turtles::remote_turtles(session, models, textures, view, vertices);
    axolotls::remote_axolotls(session, models, textures, view, vertices);
    frogs::remote_frogs(session, models, textures, view, vertices);
    guardians::remote_guardians(session, models, textures, view, vertices);
    nautilus::remote_nautiluses(session, (models, armor), textures, view, vertices);
}

/// The fish the server added, each posed from where it is between two ticks.
fn remote_fish(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world
        .entities()
        .drawn_with_invisible()
        .filter(|entity| entity.is_fish())
    {
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let age = entity.tick_count() as f32 + partial;
        let in_water = entity.is_in_water();
        let mut placed = placement(world, &snapshot, view, entity.scale());
        let name = photon_data::entity_type::name(entity.type_id).unwrap_or_default();
        match name {
            "minecraft:cod" => {
                placed.fish = Some(cod_turns(age, in_water));
                if let (Some(rect), Some(model)) = (
                    textures.rect(COD_TEXTURE),
                    models.model("minecraft:cod#main"),
                ) {
                    model.setup_tail_fish("tail_fin", age, in_water);
                    vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
                }
            }
            "minecraft:salmon" => {
                placed.fish = Some(salmon_turns(age, in_water));
                if let (Some(rect), Some(model)) = (
                    textures.rect(SALMON_TEXTURE),
                    models.model(entity.salmon_size().layer()),
                ) {
                    model.setup_salmon(age, in_water);
                    vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
                }
            }
            "minecraft:pufferfish" => {
                placed.fish = Some(pufferfish_turns(age));
                pufferfish(entity, (models, textures), (&placed, age), vertices);
            }
            _ => tropical_fish(entity, models, textures, (&placed, age, in_water), vertices),
        }
    }
}

/// `TropicalFishRenderer` and its `TropicalFishPatternLayer`: the fish tinted
/// by its base dye, then the pattern over it tinted by the pattern's.
fn tropical_fish(
    entity: &photon_world::ClientEntity,
    models: &mut MobModels,
    textures: &EntityTextures,
    (placed, age, in_water): (&MobPlacement, f32, bool),
    vertices: &mut Vec<Vertex>,
) {
    let packed = entity.tropical_fish_variant();
    let pattern = TropicalPattern::from_packed(packed);
    let (main, layer) = pattern.layers();
    let mut placed = *placed;
    placed.fish = Some(tropical_fish_turns(age, in_water));
    let dye =
        |dye| photon_data::entity_type::dye_texture_colour(dye).map(|colour| colour & 0x00FF_FFFF);
    // The base body takes the dye as a tint, which a ghost does not carry yet: it is left out while
    // the fish is invisible.
    if !entity.is_invisible()
        && let (Some(rect), Some(model)) =
            (textures.rect(pattern.base_texture()), models.model(main))
    {
        model.setup_tail_fish("tail", age, in_water);
        vertices.extend(mob_vertices_coloured(
            model,
            &placed,
            rect,
            dye(tropical_base_dye(packed)),
        ));
    }
    if !entity.is_invisible()
        && let (Some(rect), Some(model)) = (
            textures.rect(pattern.pattern_texture()),
            models.model(layer),
        )
    {
        model.setup_tail_fish("tail", age, in_water);
        vertices.extend(mob_vertices_coloured(
            model,
            &placed,
            rect,
            dye(tropical_pattern_dye(packed)),
        ));
    }
}

/// A pufferfish, in the model of its puff state.
fn pufferfish(
    entity: &photon_world::ClientEntity,
    (models, textures): (&mut MobModels, &EntityTextures),
    (placed, age): (&MobPlacement, f32),
    vertices: &mut Vec<Vertex>,
) {
    let puff_state = entity.puff_state();
    let fins = if puff_state == 0 {
        ["right_fin", "left_fin"]
    } else {
        ["right_blue_fin", "left_blue_fin"]
    };
    if let (Some(rect), Some(model)) = (
        textures.rect(PUFFERFISH_TEXTURE),
        models.model(pufferfish_layer(puff_state)),
    ) {
        model.setup_pufferfish(fins, age);
        vertices.extend(crate::mobs::visible_body(entity, model, (placed, rect)));
    }
}
