//! The sounds an entity event plays on the client with `Level.playLocalSound`: an armadillo
//! peeking, an armor stand hit, a zombie villager cured and evoker fangs biting. The events that
//! play through `Entity.playSound` (a golem's attack, a hoglin's) are sent by the server, because
//! the client level plays a sound with no player to leave out for no one.

use crate::local_player::{LocalSound, LocalSoundSource};
use crate::*;
use photon_data::entity_type::ids::{ARMADILLO, ARMOR_STAND, EVOKER_FANGS, ZOMBIE_VILLAGER};

/// `Armadillo.handleEntityEvent`.
const PEEK_EVENT: i8 = 64;
/// `ArmorStand.handleEntityEvent`.
const HIT_EVENT: i8 = 32;
/// `ZombieVillager.handleEntityEvent`.
const CURE_EVENT: i8 = 16;
/// `EvokerFangs.handleEntityEvent`.
const BITE_EVENT: i8 = 4;

impl ClientWorld {
    /// The sound `event` makes on this entity, queued for the sound engine.
    pub fn play_entity_event_sound(&mut self, id: EntityId, event: i8) {
        if let Some(slot) = break_slot(event) {
            self.play_break_sound(id, slot);
            return;
        }
        let Some(entity) = self.entities.at(id) else {
            return;
        };
        let (x, y, z) = entity.position();
        let eye_y = y + f64::from(entity.eye_height());
        let silent = entity.is_silent();
        let sound = match (entity.type_id, event) {
            (ARMADILLO, PEEK_EVENT) => LocalSound::at(
                photon_data::sound_event::vanilla("entity.armadillo.peek"),
                LocalSoundSource::Neutral,
                (1.0, 1.0),
                (x, y, z),
            ),
            (ARMOR_STAND, HIT_EVENT) => LocalSound::at(
                photon_data::sound_event::vanilla("entity.armor_stand.hit"),
                LocalSoundSource::Neutral,
                (0.3, 1.0),
                (x, y, z),
            ),
            (ZOMBIE_VILLAGER, CURE_EVENT) if !silent => {
                let random = self.entity_sound_random.source();
                let volume = 1.0 + random.next_float();
                let pitch = random.next_float() * 0.7 + 0.3;
                LocalSound::at(
                    photon_data::sound_event::vanilla("entity.zombie_villager.cure"),
                    LocalSoundSource::Hostile,
                    (volume, pitch),
                    (x, eye_y, z),
                )
            }
            (EVOKER_FANGS, BITE_EVENT) if !silent => {
                let pitch = self.entity_sound_random.source().next_float() * 0.2 + 0.85;
                LocalSound::at(
                    photon_data::sound_event::vanilla("entity.evoker_fangs.attack"),
                    LocalSoundSource::Neutral,
                    (1.0, pitch),
                    (x, y, z),
                )
            }
            _ => return,
        };
        self.sounds.0.push(sound);
    }
}

/// The slot whose item `LivingEntity.handleEntityEvent` breaks on the event.
fn break_slot(event: i8) -> Option<EquipmentSlot> {
    Some(match event {
        47 => EquipmentSlot::MainHand,
        48 => EquipmentSlot::OffHand,
        49 => EquipmentSlot::Head,
        50 => EquipmentSlot::Chest,
        51 => EquipmentSlot::Legs,
        52 => EquipmentSlot::Feet,
        65 => EquipmentSlot::Body,
        68 => EquipmentSlot::Saddle,
        _ => return None,
    })
}

impl ClientWorld {
    /// `LivingEntity.breakItem`: the broken item's sound, at the entity, unless it is silent.
    fn play_break_sound(&mut self, id: EntityId, slot: EquipmentSlot) {
        let Some(entity) = self.entities.at(id) else {
            return;
        };
        let Some(stack) = entity.equipment(slot).cloned() else {
            return;
        };
        let source = if entity.is_player() {
            LocalSoundSource::Players
        } else if photon_data::entity_type::pick(entity.type_id).is_some_and(|pick| pick.monster) {
            LocalSoundSource::Hostile
        } else {
            LocalSoundSource::Neutral
        };
        if !entity.is_silent() {
            self.play_item_break(&stack, source, entity.position());
        }
    }

    /// The slot whose item an entity event breaks (`LivingEntity.handleEntityEvent` 47 to 52, 65
    /// and 68).
    pub fn broken_slot(event: i8) -> Option<EquipmentSlot> {
        break_slot(event)
    }

    /// The sound of a broken stack (`LivingEntity.breakItem`), whose wearer is not silent.
    pub fn play_item_break(
        &mut self,
        stack: &ClientItemStackState,
        source: LocalSoundSource,
        at: (f64, f64, f64),
    ) {
        let Some(event) = stack.break_sound() else {
            return;
        };
        let pitch = 0.8 + self.entity_sound_random.source().next_float() * 0.4;
        self.sounds
            .0
            .push(LocalSound::at(event, source, (0.8, pitch), at));
    }
}
