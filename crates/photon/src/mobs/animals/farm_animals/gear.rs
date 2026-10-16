//! What the horse family and the llamas wear over their models: the saddle and the horse armor
//! (`SimpleEquipmentLayer`, `EquineSaddleModel`) and the llama's carpet or the trader's blanket
//! (`LlamaDecorLayer`).

use photon_render::entity::mob_model::{
    EquineKind, EquineState, MobPlacement, QuadrupedState, mob_vertices_coloured,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::ClientEntity;
use photon_world::EquipmentSlot;

use super::Animal;
use crate::equipment::armor::Armor;

/// The saddle's layer type and model of each equine, by the horse (`None`) or the index of
/// [`super::OTHER_EQUINES`].
const SADDLES: [(&str, &str); 5] = [
    ("horse_saddle", "minecraft:horse#saddle"),
    ("donkey_saddle", "minecraft:donkey#saddle"),
    ("mule_saddle", "minecraft:mule#saddle"),
    ("skeleton_horse_saddle", "minecraft:skeleton_horse#saddle"),
    ("zombie_horse_saddle", "minecraft:zombie_horse#saddle"),
];

/// What the entity, its look and the model kind pose an equine's model from.
pub(super) fn equine_state(
    entity: &ClientEntity,
    kind: EquineKind,
    (look, partial): (QuadrupedState, f32),
) -> EquineState {
    let anim = entity.horse();
    EquineState {
        kind,
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        eat_animation: anim.map_or(0.0, |horse| horse.eat_animation(partial)),
        stand_animation: anim.map_or(0.0, |horse| horse.stand_animation(partial)),
        feeding_animation: anim.map_or(0.0, |horse| horse.mouth_animation(partial)),
        animate_tail: anim.is_some_and(|horse| horse.tail_counter() > 0),
        // Whether the animal is in water is not tracked for mobs yet.
        in_water: false,
        age_scale: if entity.is_baby() { 0.5 } else { 1.0 },
        has_chest: entity.horse_has_chest(),
    }
}

/// The layers an animal wears, appended to `vertices`; none for another animal. A baby equine
/// wears neither saddle nor armor.
pub(super) fn layers(
    armor: &mut Armor,
    textures: &EntityTextures,
    (animal, entity): (Animal, &ClientEntity),
    (look, partial): (QuadrupedState, f32),
    placed: &MobPlacement,
    vertices: &mut Vec<Vertex>,
) {
    let place = |model: &_, rect, colour| mob_vertices_coloured(model, placed, rect, colour);
    match animal {
        Animal::Horse | Animal::Equine(_) if !entity.is_baby() => {
            // The saddle and armor models carry no chests.
            let state = EquineState {
                has_chest: false,
                ..equine_state(entity, EquineKind::Horse, (look, partial))
            };
            let (saddle, index) = match animal {
                Animal::Equine(index) => (SADDLES[index + 1], Some(index)),
                _ => (SADDLES[0], None),
            };
            // The donkey and the mule carry no armor.
            if matches!(index, None | Some(2 | 3))
                && let Some(stack) = entity.equipment(EquipmentSlot::Body)
            {
                let model = if index.is_none() {
                    "minecraft:horse_armor#main"
                } else {
                    "minecraft:undead_horse_armor#main"
                };
                vertices.extend(armor.simple_layer(
                    stack,
                    ("body", "horse_body", model),
                    textures,
                    &|model| model.setup_equine(&state),
                    &place,
                ));
            }
            if let Some(stack) = entity.equipment(EquipmentSlot::Saddle) {
                let ridden = !entity.passenger_ids().is_empty();
                vertices.extend(armor.simple_layer(
                    stack,
                    ("saddle", saddle.0, saddle.1),
                    textures,
                    &|model| model.setup_equine_saddle(&state, ridden),
                    &place,
                ));
            }
        }
        Animal::Llama(trader) => {
            let baby = entity.is_baby();
            let model = if baby {
                "minecraft:llama_baby#decor"
            } else {
                "minecraft:llama#decor"
            };
            let has_chest = !baby && entity.horse_has_chest();
            let pose = |model: &mut _| {
                photon_render::entity::mob_model::MobModel::setup_llama(model, &look, has_chest);
            };
            let carpet = entity
                .equipment(EquipmentSlot::Body)
                .filter(|_| !baby)
                .map(|stack| {
                    armor.simple_layer(
                        stack,
                        ("body", "llama_body", model),
                        textures,
                        &pose,
                        &place,
                    )
                })
                .filter(|drawn| !drawn.is_empty());
            if let Some(drawn) = carpet {
                vertices.extend(drawn);
            } else if trader {
                let asset = if baby {
                    "trader_llama_baby"
                } else {
                    "trader_llama"
                };
                vertices.extend(armor.named_layer(
                    asset,
                    ("llama_body", model),
                    textures,
                    &pose,
                    &place,
                ));
            }
        }
        _ => {}
    }
}
