use crate::*;

/// Which hand a player holds a rocket in, for where it is drawn.
#[derive(Debug, Clone, Copy, Default)]
pub struct RocketHand {
    pub main_arm_left: bool,
    pub only_off_hand: bool,
}

impl ClientWorld {
    /// The fireworks that fly with an entity, each with its target, in the order the level ticks
    /// them.
    fn attached_rockets(&self) -> Vec<(EntityId, EntityId)> {
        let mut rockets: Vec<(EntityId, EntityId)> = self
            .entities
            .iter()
            .filter_map(|(&id, entity)| Some((id, entity.attached_target?)))
            .collect();
        rockets.sort_by_key(|(id, _)| id.0);
        rockets
    }

    /// The fireworks attached to entities act after the level ticked them (`FireworkRocketEntity.tick`).
    /// One on `player` pushes a glide along and takes its place, the hand's offset added while it
    /// glides; one on another entity takes that entity's place and motion (their push is not ported).
    pub fn boost_from_rockets(
        &mut self,
        player: &mut crate::local_player::LocalPlayer,
        hand: RocketHand,
    ) {
        for (rocket, target) in self.attached_rockets() {
            let (position, motion) = if target == player.entity_id() {
                let gliding = player.boost_from_rocket();
                let offset = if gliding {
                    player.hand_holding_item_angle(hand.main_arm_left, hand.only_off_hand)
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                };
                let (at, delta) = (player.position(), player.delta_movement());
                (
                    (at.x + offset.x, at.y + offset.y, at.z + offset.z),
                    (delta.x, delta.y, delta.z),
                )
            } else if let Some(entity) = self.entities.at(target) {
                (entity.position(), entity.motion())
            } else {
                // `FireworkRocketEntity.tick` throws its trail whether or not it found the
                // entity it is attached to.
                self.throw_rocket_trail(rocket);
                continue;
            };
            self.follow_target(rocket, position, motion);
        }
    }

    /// `FireworkRocketEntity.tick`: an attached rocket takes its target's place and motion.
    fn follow_target(
        &mut self,
        rocket: EntityId,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
    ) {
        let Some(entity) = self.entities.at_mut(rocket) else {
            return;
        };
        entity.pos.position.x = JavaF64::new(position.0);
        entity.pos.position.y = JavaF64::new(position.1);
        entity.pos.position.z = JavaF64::new(position.2);
        entity.vel.motion.x = JavaF64::new(motion.0);
        entity.vel.motion.y = JavaF64::new(motion.1);
        entity.vel.motion.z = JavaF64::new(motion.2);
        self.throw_rocket_trail(rocket);
    }
}
