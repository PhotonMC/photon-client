use crate::*;

impl ClientWorld {
    /// `ClientLevel.shouldTickDeath`: a dying entity falls over only within the
    /// server's simulation distance of the local player's chunk.
    pub(crate) fn should_tick_death(&self, entity: &ClientEntity) -> bool {
        let Some((player_x, player_z)) = self.local_chunk else {
            return true;
        };
        let (x, _, z) = entity.position();
        let chunk = |value: f64| (value.floor() as i32) >> 4;
        voxel_core::math::chessboard_distance(chunk(x), chunk(z), player_x, player_z)
            <= self.server_simulation_distance
    }

    /// `handleDamageEvent`: the entity flinches, and its limbs swing at once.
    pub fn apply_entity_damage_event(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.walk.set_speed(1.5);
        entity.living.hurt_time = 10;
        true
    }

    /// `handleHurtAnimation` (`LivingEntity.animateHurt`).
    pub fn apply_entity_hurt_animation(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.living.hurt_time = 10;
        true
    }

    /// `LivingEntity.handleEntityEvent(3)` for an entity that is not a
    /// player: it dies (health spent), the sound aside.
    /// `handleAnimate`: an arm swings (actions 0 and 3). The others (waking
    /// from bed, the critical-hit particles) are not carried out.
    pub fn apply_entity_animate(&mut self, entity_id: i32, action: u8) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.living_kind().is_none() || !matches!(action, 0 | 3) {
            return false;
        }
        let durations = entity.swing_durations();
        entity.living.swing.swing(action == 3, durations);
        true
    }

    /// `handleEntityEvent` for the events that are not a death: a sheep is
    /// told to eat.
    pub fn apply_entity_event(&mut self, entity_id: i32, event: i8) -> bool {
        let game_time = self.game_time;
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if event == 2 && entity.living_kind().is_some() {
            entity.on_kinetic_hit(game_time);
            return true;
        }
        if event == 19 && entity.is_squid() {
            entity
                .kind
                .squid_or_insert_with(|| Box::new(mobs::squid::SquidState::new()))
                .restart_beat();
            return true;
        }
        if event == mobs::animal::SHEEP_EAT_EVENT
            && voxel_data::entity_type::name(entity.type_id) == Some("minecraft:sheep")
        {
            entity.living.animal.eat_tick = mobs::animal::SHEEP_EAT_TICKS;
            return true;
        }
        match voxel_data::entity_type::name(entity.type_id) {
            Some("minecraft:iron_golem") => return entity.living.animal.golem_event(event),
            Some("minecraft:goat") => return entity.living.animal.goat_event(event),
            Some("minecraft:ravager") => {
                return entity
                    .kind
                    .ravager_or_insert_with(Default::default)
                    .event(event);
            }
            Some("minecraft:hoglin" | "minecraft:zoglin") => {
                return entity.living.animal.hoglin_event(event);
            }
            Some("minecraft:tnt_minecart") if event == objects::minecart::PRIME_FUSE_EVENT => {
                entity.minecart_mut().prime_fuse();
                return true;
            }
            Some("minecraft:evoker_fangs") if event == objects::evoker_fangs::ATTACK_EVENT => {
                entity.evoker_fangs_mut().start_attack();
                return true;
            }
            Some("minecraft:armadillo") if event == mobs::armadillo::PEEK_EVENT => {
                return entity.armadillo_mut().event(event);
            }
            Some("minecraft:creaking")
                if matches!(
                    event,
                    mobs::creaking::ATTACK_EVENT | mobs::creaking::INVULNERABLE_EVENT
                ) =>
            {
                return entity.creaking_mut().event(event);
            }
            Some("minecraft:warden")
                if matches!(
                    event,
                    mobs::warden::ATTACK_EVENT
                        | mobs::warden::TENDRIL_EVENT
                        | mobs::warden::SONIC_BOOM_EVENT
                ) =>
            {
                let tick_count = entity.tick_count;
                return entity.warden_mut().event(event, tick_count);
            }
            Some("minecraft:wolf") if event == mobs::wolf::SHAKE_EVENT => {
                entity.living.wolf.start_shake();
                return true;
            }
            Some("minecraft:wolf") if event == mobs::wolf::CANCEL_SHAKE_EVENT => {
                entity.living.wolf.cancel_shake();
                return true;
            }
            Some("minecraft:rabbit") if event == 1 => {
                entity
                    .kind
                    .rabbit_or_insert_with(|| Box::new(mobs::rabbit::RabbitState::new()))
                    .start_hop();
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn apply_entity_death_event(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.is_player() {
            return true;
        }
        entity.health_bits = Some(0.0_f32.to_bits());
        entity.alive = false;
        // `EnderDragon.handleKillingBlow`: unless it sits, it is not dead yet but at
        // one health, in the dying phase, till the server says otherwise.
        if entity.is_ender_dragon() && !entity.ender_dragon_mut().phase().is_sitting() {
            entity.health_bits = Some(1.0_f32.to_bits());
            entity.alive = true;
            entity
                .ender_dragon_mut()
                .set_phase(mobs::ender_dragon::DragonPhase::Dying);
        }
        // `Wolf.die` stops a shake.
        entity.living.wolf.cancel_shake();
        true
    }

    /// `handleEntityLinkPacket`: the entity is tied to the entity with this id, or let go when
    /// it is zero. Only what can be leashed takes it; the holder is looked up when the leash is
    /// drawn, as it may arrive later than the lead.
    pub fn set_leash_holder(&mut self, entity_id: i32, holder_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if voxel_data::leash::leash_of(entity.type_id).is_none() {
            return false;
        }
        entity.leash_holder_id = holder_id;
        true
    }
}
