use crate::particles::PickedUp;
use crate::*;
use photon_core::random::RandomSource;

impl ClientWorld {
    /// `ClientPacketListener.handleTakeItemEntity`'s part of the level: the pickup sound where the
    /// entity is (pitched by the listener's random), what it looked like, and then the stack
    /// shrunk by what was taken and the entity gone when nothing is left of it (an orb stays for
    /// the server to remove). A silent entity makes no sound in 26.3. Nothing for an entity that is
    /// not here.
    pub fn take_item_entity(
        &mut self,
        item_id: EntityId,
        amount: i32,
        random: &mut RandomSource,
    ) -> Option<(PickedUp, (f64, f64, f64))> {
        let entity = self.entities.at(item_id)?;
        let position = entity.position();
        let is_orb =
            photon_data::entity_type::id("minecraft:experience_orb") == Some(entity.type_id);
        let is_item = photon_data::entity_type::id("minecraft:item") == Some(entity.type_id);
        // 26.3 skips the sound, and the draws that pitch it, for a silent entity.
        let silent = entity.is_silent()
            && photon_core::versions::rules()
                .world
                .silent_pickups_make_no_sound;
        let (event, volume, pitch) = match (silent, is_orb) {
            (true, _) => ("", 0.0, 0.0),
            (false, true) => {
                let pitch = (random.next_float() - random.next_float()) * 0.35 + 0.9;
                ("minecraft:entity.experience_orb.pickup", 0.1, pitch)
            }
            (false, false) => {
                let pitch = (random.next_float() - random.next_float()) * 1.4 + 2.0;
                ("minecraft:entity.item.pickup", 0.2, pitch)
            }
        };
        let picked = picked_up(&entity, (is_orb, is_item));
        if !silent {
            self.push_sound(local_player::LocalSound {
                relative: false,
                delay_ticks: 0,
                follows_player: false,
                event,
                source: local_player::LocalSoundSource::Players,
                volume,
                pitch,
                position,
            });
        }
        let gone = if is_item {
            let entity = self.entities.at_mut(item_id)?;
            entity.item.stack.as_mut().is_none_or(|stack| {
                if stack.item_count() > 0 {
                    stack.shrink(amount);
                }
                stack.item_count() <= 0
            })
        } else {
            !is_orb
        };
        if gone {
            self.remove_entity(item_id);
        }
        Some((picked, position))
    }
}

/// `extractEntity(from, 1.0F)`: what the taken entity looked like, as it is at the end of the tick.
fn picked_up(entity: &crate::entity::EntityView<'_>, (is_orb, is_item): (bool, bool)) -> PickedUp {
    let age_in_ticks = entity.tick_count() as f32 + 1.0;
    match (is_orb, is_item, entity.item_stack()) {
        (true, _, _) => PickedUp::Orb {
            value: entity.orb_value().unwrap_or(0),
            age_in_ticks,
        },
        (_, true, Some(stack)) => PickedUp::Item {
            stack: stack.clone(),
            age_in_ticks,
            bob_offset: entity.bob_offset(),
        },
        _ => PickedUp::Other,
    }
}
