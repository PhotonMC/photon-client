use super::*;

impl ClientEntity {
    /// A minecart of the new behaviour: the steps the server sent along the track.
    pub fn minecart_track(&self) -> Option<&crate::minecart_track::TrackLerp> {
        self.kind
            .minecart_slot()
            .and_then(|slot| slot.track.as_ref())
    }

    pub(crate) fn is_end_crystal(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:end_crystal")
    }

    /// An end crystal's age and data, if it has any.
    pub fn end_crystal(&self) -> Option<&crate::end_crystal::EndCrystalState> {
        self.kind.end_crystal()
    }

    pub(crate) fn end_crystal_mut(&mut self) -> &mut crate::end_crystal::EndCrystalState {
        let seed = i64::from(self.id);
        self.kind
            .end_crystal_or_insert_with(|| Box::new(crate::end_crystal::EndCrystalState::new(seed)))
    }

    pub(crate) fn is_ender_dragon(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:ender_dragon")
    }

    /// An ender dragon's flight history, wing beat and phase, if it has any.
    pub fn ender_dragon(&self) -> Option<&crate::ender_dragon::EnderDragonState> {
        self.kind.ender_dragon()
    }

    pub(crate) fn ender_dragon_mut(&mut self) -> &mut crate::ender_dragon::EnderDragonState {
        let seed = i64::from(self.id);
        self.kind.ender_dragon_or_insert_with(|| {
            Box::new(crate::ender_dragon::EnderDragonState::new(seed))
        })
    }

    /// `EnderDragon.aiStep` on the client, which replaces `LivingEntity.aiStep`: the wing
    /// beat and the flight history, then the position eases to the last packet's. Nothing
    /// fades its motion and its head does not turn after anything; while it dies, none of it.
    pub(crate) fn tick_ender_dragon(&mut self, targets: &EntityTargets) {
        self.ender_dragon_mut().begin_ai_step();
        if self.is_dead_or_dying() {
            return;
        }
        let tick_count = self.tick_count;
        let heals = self
            .ender_dragon_mut()
            .check_crystals(targets.dragon, tick_count);
        if heals
            && let (Some(health), Some(max)) = (self.health(), self.max_health())
            && health < max
        {
            self.health_bits = Some((health + 1.0).to_bits());
        }
        let y_rot = voxel_core::math::wrap_degrees_f32(self.y_rot.get());
        self.y_rot = JavaF32::new(y_rot);
        let (motion, no_ai, y) = (self.motion(), self.is_no_ai(), self.position().1);
        self.ender_dragon_mut().fly(motion, no_ai, (y, y_rot));
        if !no_ai {
            self.tick_position_interpolation();
            self.y_body_rot = self.y_rot;
            let (at, y_rot) = (self.position(), self.y_rot.get());
            self.ender_dragon_mut().tick_parts(at, y_rot);
        }
    }

    pub(crate) fn is_wither(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:wither")
    }

    /// A wither's head turns, targets and invulnerable ticks, if it has any.
    pub fn wither(&self) -> Option<&crate::wither::WitherState> {
        self.kind.wither()
    }

    pub(crate) fn wither_mut(&mut self) -> &mut crate::wither::WitherState {
        self.kind.wither_or_insert_with(Default::default)
    }

    /// The ids of the entities a wither's side heads look at, zero for none.
    pub(crate) fn wither_targets(&self) -> [i32; 2] {
        self.kind.wither().map_or([0; 2], |wither| wither.targets)
    }

    /// `WitherBoss.aiStep` before `super.aiStep`: the motion's height settles by
    /// a factor of 0.6, and a body moving faster than that turns to face where it goes.
    pub(crate) fn tick_wither_motion(&mut self) {
        if !self.is_wither() {
            return;
        }
        let (x, y, z) = self.motion();
        self.motion.y = JavaF64::new(y * 0.6);
        if x * x + z * z > 0.05 {
            let turn = voxel_core::math::mth_atan2(z, x) as f32
                * (180.0_f32 / std::f64::consts::PI as f32)
                - 90.0;
            self.y_rot = JavaF32::new(turn);
        }
    }

    /// `WitherBoss.aiStep` after `super.aiStep`: the side heads turn.
    pub(crate) fn tick_wither_heads(
        &mut self,
        targets: [Option<crate::guardian::GuardianTarget>; 2],
    ) {
        if !self.is_wither() {
            return;
        }
        let place = crate::wither::WitherPlace {
            position: self.position(),
            body_rot: self.y_body_rot.get(),
            scale: self.scale(),
        };
        self.wither_mut().tick_heads(place, targets);
    }

    pub(crate) fn is_armadillo(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:armadillo")
    }

    /// An armadillo's state and animation states, if it has any.
    pub fn armadillo(&self) -> Option<&crate::armadillo::ArmadilloState> {
        self.kind.armadillo()
    }

    pub(crate) fn armadillo_mut(&mut self) -> &mut crate::armadillo::ArmadilloState {
        self.kind.armadillo_or_insert_with(Default::default)
    }

    /// `Armadillo.tick`'s client half, after the rest of the tick: the animation
    /// states follow the state, a scared armadillo's head is held to its body
    /// (`Mob.clampHeadRotationToBody`), and the time in the state counts on.
    pub(crate) fn tick_armadillo(&mut self) {
        if !self.is_armadillo() {
            return;
        }
        let tick_count = self.tick_count;
        let state = self.armadillo_mut();
        state.animate(tick_count);
        let scared = state.is_scared();
        if scared {
            let (head, body) = (self.y_head_rot.get(), self.y_body_rot.get());
            let limit = 75.0_f32;
            let delta = voxel_core::math::wrap_degrees_f32(body - head);
            let target = voxel_core::math::wrap_degrees_f32(body - head).clamp(-limit, limit);
            self.y_head_rot = JavaF32::new(head + delta - target);
        }
        self.armadillo_mut().finish_tick();
    }

    pub(crate) fn is_creaking(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:creaking")
    }

    /// A creaking's animation states and flags, if it has any.
    pub fn creaking(&self) -> Option<&crate::creaking::CreakingState> {
        self.kind.creaking()
    }

    /// The creaking's state, made if it has none yet.
    pub(crate) fn creaking_mut(&mut self) -> &mut crate::creaking::CreakingState {
        let seed = i64::from(self.id);
        self.kind
            .creaking_or_insert_with(|| Box::new(crate::creaking::CreakingState::new(seed)))
    }

    /// `Creaking.tick`'s client half, after the rest of the tick.
    pub(crate) fn tick_creaking(&mut self) {
        if self.is_creaking() {
            let (tick_count, death_time) = (self.tick_count, self.living.death_time);
            self.creaking_mut().tick(tick_count, death_time);
        }
    }

    pub(crate) fn is_warden(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:warden")
    }

    /// A warden's animation states and pulses, if it has any.
    pub fn warden(&self) -> Option<&crate::warden::WardenState> {
        self.kind.warden()
    }

    /// The warden's state, made if it has none yet.
    pub(crate) fn warden_mut(&mut self) -> &mut crate::warden::WardenState {
        self.kind.warden_or_insert_with(Default::default)
    }

    /// `Warden.tick`'s client half, after the rest of the tick.
    pub(crate) fn tick_warden(&mut self) {
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.warden_mut().tick(tick_count);
        }
    }

    pub(crate) fn is_breeze(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:breeze")
    }

    /// A breeze's animation states, if it has any.
    pub fn breeze(&self) -> Option<&crate::breeze::BreezeState> {
        self.kind.breeze()
    }

    /// `Breeze.tick`'s client half, before the rest of the tick.
    pub(crate) fn tick_breeze(&mut self) {
        if !self.is_breeze() {
            return;
        }
        let (pose, tick_count) = (self.pose, self.tick_count);
        self.kind
            .breeze_or_insert_with(Default::default)
            .tick(pose, tick_count);
    }

    /// `Frog.tick`'s client half, which runs before the rest of the tick and
    /// so sees where the last one left the frog.
    pub(crate) fn tick_frog(&mut self) {
        if !self.is_frog() {
            return;
        }
        let (in_water, moving, tick_count) =
            (self.living.in_water, self.walk.is_moving(), self.tick_count);
        self.kind
            .frog_or_insert_with(Default::default)
            .tick(in_water, moving, tick_count);
    }

    /// An axolotl's animators and animation states, if it has ticked.
    pub fn axolotl(&self) -> Option<&crate::axolotl::AxolotlState> {
        self.kind.axolotl()
    }

    /// `Axolotl.getVariant`: the index of the variant (an id no variant has is the first).
    pub fn axolotl_variant(&self) -> usize {
        usize::try_from(self.living.axolotl_variant)
            .ok()
            .filter(|id| *id < 5)
            .unwrap_or(0)
    }

    /// `Axolotl.baseTick` on the client: the animators and animation states
    /// follow where the axolotl is and whether it walks. The rotation test of
    /// the reference never holds here: `LivingEntity.baseTick` has set the old
    /// turn to the new before it.
    pub(crate) fn tick_axolotl(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:axolotl") {
            return;
        }
        let input = crate::axolotl::AxolotlInput {
            baby: self.is_baby(),
            playing_dead: self.living.axolotl_playing_dead,
            in_water: self.living.in_water,
            on_ground: self.on_ground,
            moving: self.walk.is_moving(),
            tick_count: self.tick_count,
        };
        self.kind
            .axolotl_or_insert_with(|| Box::new(crate::axolotl::AxolotlState::new()))
            .tick(input);
    }

    /// `Turtle.hasEgg`.
    pub fn turtle_has_egg(&self) -> bool {
        self.living.turtle_has_egg
    }

    /// `Turtle.isLayingEgg`.
    pub fn turtle_laying_egg(&self) -> bool {
        self.living.turtle_laying_egg
    }

    /// What `DolphinRenderer.extractRenderState` calls moving: the motion the
    /// server gave it covers more than a ten-millionth of a block squared in a tick.
    pub fn is_moving_horizontally(&self) -> bool {
        let (x, _, z) = self.motion();
        x * x + z * z > 1.0E-7
    }

    /// Whether this is one of the fish `AbstractFish` makes.
    pub fn is_fish(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some(
                "minecraft:cod"
                    | "minecraft:salmon"
                    | "minecraft:tropical_fish"
                    | "minecraft:pufferfish"
            )
        )
    }

    /// `Salmon.getVariant`: medium until the data says.
    pub fn salmon_size(&self) -> crate::fish::SalmonSize {
        crate::fish::SalmonSize::from_id(self.living.fish_variant.unwrap_or(1))
    }

    /// `TropicalFish.getPackedVariant`: a white kob until the data says.
    pub fn tropical_fish_variant(&self) -> i32 {
        self.living.fish_variant.unwrap_or(0)
    }

    /// `Pufferfish.getPuffState`.
    pub fn puff_state(&self) -> i32 {
        self.living.fish_variant.unwrap_or(0)
    }

    /// What `getDefaultDimensions` scales a fish's box by, from the size the
    /// type's table holds (that of a new one: a medium salmon, an unpuffed
    /// pufferfish, whose half is undone by doubling): a salmon by its size, a
    /// pufferfish by how puffed it is.
    pub(crate) fn fish_dimension_scale(&self) -> f32 {
        match voxel_data::entity_type::name(self.type_id) {
            Some("minecraft:salmon") => self.salmon_size().bounding_box_scale(),
            Some("minecraft:pufferfish") => {
                crate::fish::pufferfish_bounding_box_scale(self.puff_state()) * 2.0
            }
            _ => 1.0,
        }
    }

    /// `Entity.isInWater` for a mob that tracks it.
    pub fn is_in_water(&self) -> bool {
        self.living.in_water
    }

    /// Whether a position interpolation is still under way
    /// (`LivingEntity.isInterpolating`).
    pub(crate) fn is_interpolating(&self) -> bool {
        self.interpolation
            .as_ref()
            .is_some_and(|interpolation| interpolation.steps > 0)
    }

    /// What `LivingEntity.aiStep` does to the motion of a mob the client does not move:
    /// it decays by a fiftieth less while no position interpolation runs, and motion
    /// under three thousandths a tick is dropped.
    pub(crate) fn tick_mob_motion(&mut self, was_interpolating: bool) {
        let (mut x, mut y, mut z) = self.motion();
        if !was_interpolating {
            x *= 0.98;
            y *= 0.98;
            z *= 0.98;
        }
        if x.abs() < 0.003 {
            x = 0.0;
        }
        if z.abs() < 0.003 {
            z = 0.0;
        }
        if y.abs() < 0.003 {
            y = 0.0;
        }
        self.motion.x = JavaF64::new(x);
        self.motion.y = JavaF64::new(y);
        self.motion.z = JavaF64::new(z);
    }

    /// `Entity.baseTick`'s fluid pass, then `LivingEntity.aiStep`: a living entity
    /// eases to the position and turn its last packet gave and its head to the head
    /// turn, and a mob's motion fades; then what a species does of its own there.
    pub(crate) fn tick_ai_step(
        &mut self,
        living: Option<LivingKind>,
        in_water: Option<bool>,
        targets: EntityTargets,
    ) {
        if self.is_ender_dragon() {
            self.tick_ender_dragon(&targets);
            return;
        }
        if self.is_minecart() {
            self.tick_minecart();
            return;
        }
        if let Some(display) = self.kind.display_mut() {
            // `Display.tick`'s client half: the notes the data left, then the position's easing.
            display.tick(self.tick_count);
            self.tick_position_interpolation();
            return;
        }
        self.tick_frog();
        self.tick_breeze();
        self.tick_wither_motion();
        if let Some(in_water) = in_water {
            self.living.in_water = in_water;
        }
        self.tick_axolotl();
        self.tick_guardian(targets.guardian);
        let was_interpolating = self.is_interpolating();
        if living.is_some() {
            self.tick_position_interpolation();
        }
        self.tick_head_rotation();
        if living == Some(LivingKind::Mob) {
            self.tick_mob_motion(was_interpolating);
            self.tick_squid();
            self.tick_cube_mob();
            self.tick_shulker();
        }
        self.tick_wither_heads(targets.wither);
    }

    /// `Squid.aiStep`, which turns the body (and with it the entity) in water.
    pub(crate) fn tick_squid(&mut self) {
        if !self.is_squid() {
            return;
        }
        let (in_water, motion, body) = (self.living.in_water, self.motion(), self.y_body_rot.get());
        let turned = self
            .kind
            .squid_or_insert_with(|| Box::new(crate::squid::SquidState::new()))
            .ai_step(in_water, motion, body);
        if let Some(body) = turned {
            self.y_body_rot = JavaF32::new(body);
            self.y_rot = JavaF32::new(body);
        }
        // `GlowSquid.aiStep`: the dark ticks a hurt glow squid was given run out.
        if self.living.glow_dark_ticks > 0 {
            self.living.glow_dark_ticks -= 1;
        }
    }

    pub(crate) fn is_shulker(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:shulker")
    }

    /// `Shulker.getAttachFace`'s data value (down 0, up 1, north 2, south 3, west 4, east 5).
    pub fn shulker_attach_face(&self) -> i32 {
        self.living.shulker_face
    }

    /// `Shulker.getColor`'s dye id, none for the default.
    pub fn shulker_color(&self) -> Option<u8> {
        self.living.shulker_color
    }

    /// A shulker's lid between two ticks.
    pub fn shulker(&self) -> &crate::shulker::ShulkerState {
        &self.living.shulker
    }
}
