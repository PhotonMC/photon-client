//! The layers over the mooshroom and the horse: its mushrooms and the horse's markings.

use super::*;
use photon_render::entity::block::mooshroom_mushroom_poses;
use photon_render::entity::mob_model::{overlay_code, placement_pose};
use photon_world::EntityView;

/// `MushroomCowMushroomLayer`: the mushrooms on a grown, visible mooshroom.
pub(super) fn mushrooms(
    model: &MobModel,
    entity: EntityView<'_>,
    placed: &MobPlacement,
    held: &mut Vec<HeldItem>,
) {
    if entity.is_baby() || entity.is_invisible() {
        return;
    }
    let (Some(state), Some(head)) = (
        photon_data::block::registry::block(MOOSHROOMS[mooshroom_variant(entity)].1)
            .map(photon_data::block::registry::Block::default_state_id),
        model
            .root()
            .children()
            .iter()
            .find(|part| part.name() == "head"),
    ) else {
        return;
    };
    for pose in mooshroom_mushroom_poses(&placement_pose(placed), head) {
        // `MushroomCowMushroomLayer` takes the entity's overlay.
        held.push(HeldItem::block(
            state,
            pose,
            placed.light,
            overlay_code(placed.red_overlay, 0.0),
        ));
    }
}

/// `HorseMarkingLayer`: the markings over the coat, not on an invisible horse.
pub(super) fn horse_markings(
    model: &MobModel,
    (animal, entity): (Animal, EntityView<'_>),
    (placed, textures): (&MobPlacement, &EntityTextures),
) -> Vec<Vertex> {
    if animal != Animal::Horse || entity.is_invisible() {
        return Vec::new();
    }
    usize::try_from(entity.horse_look().1)
        .ok()
        .and_then(|index| MARKINGS[index])
        .and_then(|markings| textures.rect(markings[usize::from(entity.is_baby())]))
        .map(|marking| mob_vertices(model, placed, marking))
        .unwrap_or_default()
}
