use super::*;

impl ClientEntity {}

impl EntityViewMut<'_> {
    /// `ItemStack.getSwingAnimation().duration()` of what each hand holds, the main hand's first.
    pub(crate) fn swing_durations(&self) -> [i32; 2] {
        [EquipmentSlot::MainHand, EquipmentSlot::OffHand].map(|slot| {
            self.equipment(slot)
                .and_then(|stack| crate::block::rules::item(stack.item_registry_id()))
                .and_then(|item| photon_data::spear::spear_of(item.name))
                .map_or(crate::living::DEFAULT_SWING_DURATION, |spear| {
                    spear.swing_ticks
                })
        })
    }
}

impl EntityViewMut<'_> {
    /// What a living entity does each tick besides moving: hurt and death,
    /// its pose, its swing, and what its kind does of its own.
    pub(crate) fn tick_living(&mut self, kind: LivingKind, tick_death: bool, game_time: u64) {
        self.living.using_item_ticks = if self.as_view().is_using_item() {
            self.living.using_item_ticks.saturating_add(1)
        } else {
            0
        };
        let (mx, my, mz) = self.motion();
        self.living.previous_motion = [mx.to_bits(), my.to_bits(), mz.to_bits()];
        self.living.fall_fly_ticks = if self.as_view().is_fall_flying() {
            self.living.fall_fly_ticks.saturating_add(1)
        } else {
            0
        };
        let elytra_input = crate::elytra::ElytraInput {
            fall_flying: self.as_view().is_fall_flying(),
            motion: self.motion(),
            crouching: self.as_view().is_crouching(),
        };
        self.living.elytra.tick(elytra_input);
        self.tick_hurt_and_death(tick_death);
        self.tick_living_pose(kind);
        self.tick_swing(kind);
        self.tick_animal(game_time);
    }
}

impl EntityViewMut<'_> {
    /// What a sheep or a chicken does of its own each tick (`aiStep`).
    pub(crate) fn tick_animal(&mut self, game_time: u64) {
        match photon_data::entity_type::name(self.type_id) {
            Some("minecraft:sheep") => self.living.animal.tick_sheep(),
            Some("minecraft:chicken") => self.living.animal.tick_chicken(self.entity.on_ground),
            Some("minecraft:creeper") => self.living.animal.tick_creeper(self.status.alive),
            Some("minecraft:iron_golem") => self.living.animal.tick_iron_golem(),
            Some("minecraft:goat") => self.living.animal.tick_goat(),
            Some("minecraft:hoglin" | "minecraft:zoglin") => self.living.animal.tick_hoglin(),
            Some("minecraft:ravager") => {
                let alive = self.status.alive;
                self.kind.ravager_state().tick(alive);
            }
            Some("minecraft:bee") => {
                let rolling = self.living.bee_flags & 2 != 0;
                self.living.animal.tick_bee(rolling);
            }
            Some(
                "minecraft:horse"
                | "minecraft:donkey"
                | "minecraft:mule"
                | "minecraft:skeleton_horse"
                | "minecraft:zombie_horse"
                | "minecraft:llama"
                | "minecraft:trader_llama"
                | "minecraft:camel"
                | "minecraft:camel_husk",
            ) => self.tick_horse_like(game_time),
            Some("minecraft:rabbit") => {
                let (tick_count, no_ai) = (self.tick_count, self.as_view().is_no_ai());
                self.kind.rabbit_state().tick(tick_count, no_ai);
            }
            Some("minecraft:copper_golem") => {
                let (action, tick_count, flags) = (
                    self.living.copper_golem_action,
                    self.tick_count,
                    (self.as_view().is_no_ai(), self.status.alive),
                );
                self.kind
                    .copper_golem_state()
                    .tick(action, tick_count, flags);
            }
            Some("minecraft:polar_bear") => self.living.polar_bear.tick(),
            Some("minecraft:fox") => self.living.fox.tick(),
            Some("minecraft:wolf") => {
                let (alive, interested) = (self.status.alive, self.living.wolf_interested);
                self.living.wolf.tick(alive, interested);
            }
            _ => {}
        }
    }
}

impl EntityViewMut<'_> {
    /// What a horse, a llama or a camel does of its own each tick (`aiStep`).
    fn tick_horse_like(&mut self, game_time: u64) {
        let (alive, camel) = (
            self.status.alive,
            matches!(
                photon_data::entity_type::name(self.type_id),
                Some("minecraft:camel" | "minecraft:camel_husk")
            ),
        );
        let (tick_count, on_ground, passenger) = (
            self.tick_count,
            self.on_ground,
            self.links.vehicle_id.is_some(),
        );
        let horse = self.kind.horse_state();
        horse.tick(alive);
        if camel {
            let in_liquid = self.ride.mount_ride.in_liquid;
            horse.tick_camel(
                (tick_count, game_time as i64),
                (on_ground, in_liquid, passenger),
            );
        }
    }
}

impl EntityView<'_> {
    /// A chicken's `flap` and `flapSpeed` between two ticks.
    pub fn chicken_flap(&self, partial: f32) -> (f32, f32) {
        let animal = &self.living.animal;
        (animal.flap_at(partial), animal.flap_speed_at(partial))
    }

    /// The swing of an arm (`LivingEntity.swing`).
    pub fn swing_state(&self) -> &crate::living::SwingState {
        &self.living.swing
    }
}

impl EntityViewMut<'_> {
    /// What a living entity's swing does each tick: the last tick's attack
    /// animation is kept, and the swing runs on for a player and a monster
    /// (`Player.aiStep`, `Monster.aiStep`).
    pub(crate) fn tick_swing(&mut self, kind: LivingKind) {
        self.living.swing.begin_tick();
        let monster = photon_data::entity_type::pick(self.type_id).is_some_and(|pick| pick.monster);
        if kind == LivingKind::Player || monster {
            let durations = self.swing_durations();
            self.living.swing.update(durations);
        }
    }

    /// What a living entity's drawn pose follows from its tick: the limbs'
    /// swing with the distance moved, and the body turning (a player's after
    /// where it walks, a mob's by its body control), and for a player the
    /// swimming pose easing in or out.
    pub(crate) fn tick_living_pose(&mut self, kind: LivingKind) {
        let moved = (
            self.pos.position.x.get() - self.pos.old_position.x.get(),
            self.pos.position.z.get() - self.pos.old_position.z.get(),
        );
        if kind == LivingKind::Player {
            self.anim.old_swim_amount = self.anim.swim_amount;
            self.anim.swim_amount = JavaF32::new(crate::living::ease_swim_amount(
                self.anim.swim_amount.get(),
                self.status.pose == ClientEntityPose::Swimming,
            ));
        }
        let mut body = self.rotation.y_body_rot.get();
        let mut body_old = self.rotation.old_y_body_rot.get();
        let mut head = self.rotation.y_head_rot.get();
        let mut head_old = self.rotation.old_y_head_rot.get();
        if kind == LivingKind::Mob && self.is_shulker() {
            // `Shulker.ShulkerBodyRotationControl`: the body does not follow.
            crate::living::keep_old_angles_near((body, &mut body_old), (head, &mut head_old));
        } else if kind == LivingKind::Mob {
            let mut control = crate::living::MobBodyControl {
                head_stable_time: self.anim.head_stable_time,
                last_stable_y_head_rot: self.anim.last_stable_y_head_rot.get(),
            };
            crate::living::tick_mob_body(
                &mut control,
                (&mut body, &mut head),
                self.rotation.y_rot.get(),
                moved,
                !self.links.passenger_ids.is_empty(),
            );
            self.anim.head_stable_time = control.head_stable_time;
            self.anim.last_stable_y_head_rot = JavaF32::new(control.last_stable_y_head_rot);
            crate::living::keep_old_angles_near((body, &mut body_old), (head, &mut head_old));
            self.rotation.y_head_rot = JavaF32::new(head);
        } else {
            crate::living::turn_body(
                (&mut body, &mut body_old),
                (self.rotation.y_rot.get(), head, &mut head_old),
                moved,
            );
        }
        self.rotation.y_body_rot = JavaF32::new(body);
        self.rotation.old_y_body_rot = JavaF32::new(body_old);
        self.rotation.old_y_head_rot = JavaF32::new(head_old);
        self.tick_walk_animation(moved);
    }

    /// The limbs' swing with the distance moved (`calculateEntityAnimation`).
    fn tick_walk_animation(&mut self, moved: (f64, f64)) {
        // `calculateEntityAnimation`: a rider or a dead entity stands still.
        // The dragon's own `aiStep` does not reach `calculateEntityAnimation`.
        if self.links.vehicle_id.is_none() && self.status.alive && !self.is_ender_dragon() {
            let camel = matches!(
                photon_data::entity_type::name(self.type_id),
                Some("minecraft:camel" | "minecraft:camel_husk")
            );
            if camel {
                self.tick_camel_walk(moved);
            } else if self.is_creaking() {
                // `Creaking.updateWalkAnimation`: a quicker stride, capped at three.
                let target = (moved.0.hypot(moved.1) as f32 * 25.0).min(3.0);
                self.anim.walk.update(target, 0.4, 1.0);
            } else if self.is_frog() {
                let target = self.kind.frog().map_or_else(
                    || ((moved.0.hypot(moved.1)) as f32 * 25.0).min(1.0),
                    |frog| frog.walk_target(moved.0.hypot(moved.1)),
                );
                self.anim
                    .walk
                    .update(target, 0.4, if self.status.baby { 3.0 } else { 1.0 });
            } else {
                self.anim
                    .walk
                    .follow(moved.0.hypot(moved.1), self.status.baby);
            }
        } else {
            self.anim.walk.stop();
        }
    }

    /// What a cat or a panda does of its own after `super.tick`, given what
    /// the world around it holds (a sleeper near a cat, thunder).
    pub(crate) fn tick_species(
        &mut self,
        (sleeper_near_cat, thundering, jukebox_near): (bool, bool, bool),
    ) {
        // `AgeableMob.makeAgeLockedParticle` counts the wait down on the client.
        self.living.age_lock_timer = (self.living.age_lock_timer - 1).max(0);
        self.tick_cat(sleeper_near_cat);
        self.tick_panda(thundering);
        self.tick_bat();
        self.tick_parrot(jukebox_near);
        self.tick_allay();
        self.tick_warden();
        self.tick_creaking();
        self.tick_armadillo();
        if self.is_end_crystal() {
            self.end_crystal_mut().tick();
        }
        if self.is_abstract_arrow() {
            self.arrow_mut().tick();
        }
        if self.is_evoker_fangs() {
            self.evoker_fangs_mut().tick();
        }
        if self.is_primed_tnt() {
            self.primed_tnt_mut().tick();
        }
        if self.is_fishing_bobber() {
            self.tick_position_interpolation();
        }
        if self.is_lightning_bolt() {
            let _seed = i64::from(self.id.0);
            self.lightning_done = self.kind.lightning_state().tick();
        }
    }
}

impl EntityViewMut<'_> {
    /// `Camel.updateWalkAnimation`: only a standing camel that is not dashing walks, and its
    /// stride is quicker to settle.
    fn tick_camel_walk(&mut self, moved: (f64, f64)) {
        let walking = self.status.pose == ClientEntityPose::Standing
            && !self
                .kind
                .horse()
                .is_some_and(|horse| horse.camel.is_dash_animation_started());
        let target = if walking {
            ((moved.0.hypot(moved.1)) as f32 * 6.0).min(1.0)
        } else {
            0.0
        };
        self.anim
            .walk
            .update(target, 0.2, if self.status.baby { 3.0 } else { 1.0 });
    }
}
