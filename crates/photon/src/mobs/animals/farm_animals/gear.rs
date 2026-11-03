//! What the horse family and the llamas wear over their models: the saddle and the horse armor
//! (`SimpleEquipmentLayer`, `EquineSaddleModel`) and the llama's carpet or the trader's blanket
//! (`LlamaDecorLayer`).

use photon_render::entity::mob_model::{
    CamelState, EquineKind, EquineState, MobPlacement, QuadrupedState,
};
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_render::scene::Vertex;
use photon_world::EntityView;
use photon_world::EquipmentSlot;
use photon_world::animation_state::AnimationState;

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
    entity: EntityView<'_>,
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

/// What poses a camel's model, and its saddle's (`CamelModel.setupAnim`).
pub(super) fn camel_state(
    entity: EntityView<'_>,
    (look, partial): (QuadrupedState, f32),
) -> CamelState {
    let camel = entity.horse().map(|horse| &horse.camel);
    let state = |pick: fn(&photon_world::mobs::horse::CamelState) -> AnimationState| {
        camel.map(pick).unwrap_or_default()
    };
    CamelState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        // The cooldown is only counted for a camel its own player rides.
        jump_cooldown: 0.0,
        sit: state(|camel| camel.sit),
        sit_pose: state(|camel| camel.sit_pose),
        sit_up: state(|camel| camel.sit_up),
        idle: state(|camel| camel.idle),
        dash: state(|camel| camel.dash),
        baby: entity.is_baby(),
    }
}

/// What draws a layer of a mob in a colour, which joins the outline when the mob glows.
fn outlined<'a>(
    entity: EntityView<'a>,
    placed: &'a MobPlacement,
) -> impl Fn(&photon_render::entity::mob_model::MobModel, TextureRect, Option<u32>) -> Vec<Vertex> + 'a
{
    move |model, rect, colour| crate::mobs::outline::coloured(entity, model, placed, (rect, colour))
}

/// What the layers of one animal are drawn with.
struct Dress<'a> {
    armor: &'a mut Armor,
    textures: &'a EntityTextures,
    entity: EntityView<'a>,
    look: (QuadrupedState, f32),
    placed: &'a MobPlacement,
}

/// The layers an animal wears, appended to `vertices`; none for another animal. A baby equine
/// wears neither saddle nor armor.
pub(super) fn layers(
    armor: &mut Armor,
    textures: &EntityTextures,
    (animal, entity): (Animal, EntityView<'_>),
    look: (QuadrupedState, f32),
    placed: &MobPlacement,
    vertices: &mut Vec<Vertex>,
) {
    let mut dress = Dress {
        armor,
        textures,
        entity,
        look,
        placed,
    };
    match animal {
        Animal::Horse | Animal::Equine(_) if !entity.is_baby() => {
            dress.equine(animal, vertices);
        }
        Animal::Llama(trader) => dress.llama(trader, vertices),
        // `CamelRenderer`'s saddle: a baby wears none.
        Animal::Camel(husk) if !entity.is_baby() => dress.camel_saddle(husk, vertices),
        _ => {}
    }
}

impl Dress<'_> {
    fn equine(&mut self, animal: Animal, vertices: &mut Vec<Vertex>) {
        let (entity, (look, partial)) = (self.entity, self.look);
        let place = outlined(entity, self.placed);
        let (armor, textures) = (&mut *self.armor, self.textures);
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

    fn llama(&mut self, trader: bool, vertices: &mut Vec<Vertex>) {
        let (entity, (look, _)) = (self.entity, self.look);
        let place = outlined(entity, self.placed);
        let (armor, textures) = (&mut *self.armor, self.textures);
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

    fn camel_saddle(&mut self, husk: bool, vertices: &mut Vec<Vertex>) {
        let Some(stack) = self.entity.equipment(EquipmentSlot::Saddle) else {
            return;
        };
        let (layer, model) = if husk {
            (
                "camel_husk_saddle",
                photon_data::entity_model::ids::CAMEL_HUSK_SADDLE,
            )
        } else {
            ("camel_saddle", photon_data::entity_model::ids::CAMEL_SADDLE)
        };
        let state = camel_state(self.entity, self.look);
        vertices.extend(self.armor.simple_layer(
            stack,
            ("saddle", layer, model),
            self.textures,
            &|model| model.setup_camel(&state),
            &outlined(self.entity, self.placed),
        ));
    }
}
