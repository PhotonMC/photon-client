//! The sounds an entity event plays on the client with `Level.playLocalSound`: an armadillo
//! peeking, an armor stand hit, a zombie villager cured and evoker fangs biting. The events that
//! play through `Entity.playSound` (a golem's attack, a hoglin's) are sent by the server, because
//! the client level plays a sound with no player to leave out for no one.

use crate::local_player::{LocalSound, LocalSoundSource};
use crate::*;

const ARMADILLO: EntityTypeId = photon_data::entity_type::vanilla("armadillo");
const ARMOR_STAND: EntityTypeId = photon_data::entity_type::vanilla("armor_stand");
const ZOMBIE_VILLAGER: EntityTypeId = photon_data::entity_type::vanilla("zombie_villager");
const EVOKER_FANGS: EntityTypeId = photon_data::entity_type::vanilla("evoker_fangs");

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
