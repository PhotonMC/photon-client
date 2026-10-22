//! The happy ghast (`HappyGhastRenderer`): its model grown or baby, squeezed
//! while it wears a harness, and the harness itself with goggles that sit on
//! its brow until it is ridden. The ropes of a leashed one are not drawn.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::mob_vertices_coloured;
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EquipmentSlot;

use crate::equipment::armor::Armor;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const TEXTURE: &str = "entity/ghast/happy_ghast";
const BABY_TEXTURE: &str = "entity/ghast/happy_ghast_baby";

/// The paths (under `textures/`) of the textures a happy ghast wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [TEXTURE, BABY_TEXTURE].into_iter()
}

/// `HappyGhastRenderer` for the happy ghasts the server added.
pub fn remote_happy_ghasts(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(ghast) = photon_data::entity_type::id("minecraft:happy_ghast") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs().filter(|entity| entity.type_id == ghast) {
        let baby = entity.is_baby();
        let (layer, texture, harness) = if baby {
            (
                "minecraft:happy_ghast_baby#main",
                BABY_TEXTURE,
                "minecraft:happy_ghast_baby_harness#main",
            )
        } else {
            (
                "minecraft:happy_ghast#main",
                TEXTURE,
                "minecraft:happy_ghast_harness#main",
            )
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let age_in_ticks = entity.tick_count() as f32 + partial;
        let body_item = entity.equipment(EquipmentSlot::Body);
        model.setup_happy_ghast(age_in_ticks, body_item.is_some());
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
        if let Some(stack) = body_item {
            let ridden = entity.is_vehicle();
            vertices.extend(armor.simple_layer(
                stack,
                ("body", "happy_ghast_body", harness),
                textures,
                &|model| model.setup_happy_ghast_harness(ridden),
                &|model, rect, colour| mob_vertices_coloured(model, &placed, rect, colour),
            ));
        }
    }
}
