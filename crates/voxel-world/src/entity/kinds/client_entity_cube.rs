use super::*;

impl ClientEntity {
    /// `Shulker.tick`'s lid.
    pub(crate) fn tick_shulker(&mut self) {
        if self.is_shulker() {
            let raw = self.living.shulker_raw_peek;
            self.living.shulker.tick(raw);
            self.living.shulker.tick_teleport();
        }
    }

    /// `Shulker.makeBoundingBox`: the body and the lid as far open as it is,
    /// on the side opposite the one it clings to.
    pub fn shulker_bounding_box(&self) -> Aabb {
        let physical = crate::shulker::physical_peek(self.living.shulker.current_peek());
        crate::shulker::progress_delta_aabb(
            self.shulker_scale(),
            self.living.shulker_face ^ 1,
            (-1.0, physical),
            self.position(),
        )
    }

    /// `Shulker.sanitizeScale` of `getScale`.
    pub(in super::super) fn shulker_scale(&self) -> f32 {
        self.scale().min(3.0)
    }

    /// `Shulker.getRenderPosition`: where the slide to its block draws it,
    /// relative to the block.
    pub fn shulker_render_offset(&self, partial_tick: f32) -> Option<[f64; 3]> {
        self.living.shulker.render_offset(
            block_of(self.position()),
            self.shulker_scale(),
            partial_tick,
        )
    }

    /// `Shulker.setPos`: unless it rides, it is held to the middle of its
    /// block (`floor(y + 0.5)` vertically); having come to another block after
    /// its first tick it closes its lid, and a shulker that is not riding
    /// slides there from the old one.
    pub(in super::super) fn shulker_set_pos(
        &mut self,
        position: (f64, f64, f64),
        old_block: [i32; 3],
    ) {
        let riding = self.vehicle_id.is_some();
        let (x, y, z) = if riding {
            position
        } else {
            (
                position.0.floor() + 0.5,
                (position.1 + 0.5).floor(),
                position.2.floor() + 0.5,
            )
        };
        self.position.x = JavaF64::new(x);
        self.position.y = JavaF64::new(y);
        self.position.z = JavaF64::new(z);
        let block = block_of((x, y, z));
        if self.tick_count != 0 && block != old_block {
            self.living.shulker_raw_peek = 0;
            if !riding && self.living.shulker.start_teleport(old_block, block) {
                self.old_position.x = JavaF64::new(x);
                self.old_position.y = JavaF64::new(y);
                self.old_position.z = JavaF64::new(z);
            }
        }
    }

    /// `Shulker.startRiding` (client side): it stops sliding and clings to
    /// the floor of its vehicle.
    pub(crate) fn shulker_starts_riding(&mut self) {
        self.living.shulker.cancel_teleport();
        self.living.shulker_face = 0;
    }

    /// `Shulker.stopRiding` (client side): it slides from where it stands, and
    /// its body faces south again.
    pub(crate) fn shulker_stops_riding(&mut self) {
        let block = block_of(self.position());
        self.living.shulker.set_old_attach(block);
        self.y_body_rot = JavaF32::new(0.0_f32);
        self.old_y_body_rot = JavaF32::new(0.0_f32);
    }

    /// Whether this is a slime or a magma cube (`AbstractCubeMob`).
    pub fn is_cube_mob(&self) -> bool {
        crate::metadata::is_cube_mob_type(self.type_id)
    }

    /// `SulfurCube.getFuse`.
    pub fn sulfur_fuse(&self) -> i32 {
        self.living.cube.fuse()
    }

    /// `AbstractCubeMob.getSize`.
    pub fn cube_size(&self) -> i32 {
        self.living.cube_size.unwrap_or(1)
    }

    /// A slime's or a magma cube's squish and what it eases to.
    pub fn cube_mob(&self) -> &crate::cube_mob::CubeMobState {
        &self.living.cube
    }

    /// `AbstractCubeMob.onSyncedDataUpdated` for the size: each time the data
    /// says it (the same size too), the body and the entity turn to where the
    /// head looks.
    pub(crate) fn set_cube_size(&mut self, size: i32) {
        self.y_rot = self.y_head_rot;
        self.y_body_rot = self.y_head_rot;
        self.living.cube_size = Some(size);
    }

    /// `AbstractCubeMob.tick`'s squish (`decreaseSquish`: a magma cube's target
    /// fades slower).
    pub(crate) fn tick_cube_mob(&mut self) {
        if !self.is_cube_mob() {
            return;
        }
        let magma = voxel_data::entity_type::name(self.type_id) == Some("minecraft:magma_cube");
        let on_ground = self.on_ground;
        // `SulfurCube.tick` runs its fuse before `AbstractCubeMob.tick`.
        self.living.cube.tick_fuse();
        self.living
            .cube
            .tick(on_ground, if magma { 0.9 } else { 0.6 });
    }

    /// `GlowSquid.getDarkTicksRemaining`.
    pub fn glow_dark_ticks(&self) -> i32 {
        self.living.glow_dark_ticks
    }

    /// A copper golem's animation states, if it has ticked.
    pub fn copper_golem(&self) -> Option<&crate::copper_golem::CopperGolemAnimations> {
        self.kind.copper_golem()
    }

    /// `CopperGolem.getState`.
    pub fn copper_golem_action(&self) -> crate::copper_golem::GolemAction {
        self.living.copper_golem_action
    }

    /// `CopperGolem.getWeatherState`: unaffected, exposed, weathered, oxidized (0 to 3).
    pub fn copper_golem_weather(&self) -> u8 {
        self.living.copper_golem_weather
    }

    /// `Goat.hasLeftHorn` and `hasRightHorn`.
    pub fn goat_horns(&self) -> (bool, bool) {
        (!self.living.lost_left_horn, !self.living.lost_right_horn)
    }

    /// `Goat.getRammingXHeadRot`: how far the head is lowered, in radians.
    pub fn goat_ramming_x_head_rot(&self) -> f32 {
        let maximum = if self.baby { 52.5 } else { 30.0 };
        self.living.animal.goat_lower_head_tick as f32 / 20.0
            * maximum
            * (std::f64::consts::PI / 180.0) as f32
    }

    /// `Creeper.isPowered`.
    pub fn is_powered(&self) -> bool {
        self.living.powered
    }

    /// `Bee.getRollAmount`: how far the bee has rolled over, between two ticks.
    pub fn bee_roll_amount(&self, partial: f32) -> f32 {
        self.living.animal.bee_roll_at(partial)
    }

    /// `Bee.hasStung`.
    pub fn bee_has_stung(&self) -> bool {
        self.living.bee_flags & 4 != 0
    }

    /// `Bee.hasNectar`.
    pub fn bee_has_nectar(&self) -> bool {
        self.living.bee_flags & 8 != 0
    }

    /// The game time a neutral mob's anger ends (`getPersistentAngerEndTime`).
    pub fn anger_end(&self) -> i64 {
        self.living.anger_end
    }

    /// `NeutralMob.isAngry`: the anger ends later than the game time now.
    pub fn is_angry(&self, game_time: u64) -> bool {
        let end = self.living.anger_end;
        end > 0 && end - game_time as i64 > 0
    }

    /// `TamableAnimal.isInSittingPose`.
    pub fn is_sitting(&self) -> bool {
        self.living.tamable_flags & 1 != 0
    }

    /// `TamableAnimal.isTame`.
    pub fn is_tame(&self) -> bool {
        self.living.tamable_flags & 4 != 0
    }

    /// The dye of a wolf's collar (`Wolf.getCollarColor`).
    pub fn collar_dye(&self) -> i32 {
        // `DyeColor.RED` until the data says otherwise.
        self.living.collar.unwrap_or(14)
    }

    /// Whether this is a cat whose data says it lies down.
    pub(crate) fn is_lying_cat(&self) -> bool {
        self.living.cat_lying
            && voxel_data::entity_type::name(self.type_id) == Some("minecraft:cat")
    }

    /// `Cat.tick` after `super.tick`: the lie-down and relax amounts.
    pub(crate) fn tick_cat(&mut self, sleeper_near: bool) {
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:cat") {
            let (lying, relaxed) = (self.living.cat_lying, self.living.cat_relaxed);
            self.living.cat.tick(lying, relaxed, sleeper_near);
        }
    }

    /// A panda's data and own client state (its counters and how far it sits,
    /// lies on its back and rolls); the starting state until data comes.
    pub fn panda(&self) -> crate::panda::PandaState {
        self.kind.panda().cloned().unwrap_or_default()
    }

    /// Fixes the source a panda's chance to eat is drawn from (see
    /// [`crate::panda::PandaState::fix_seed`]).
    pub fn fix_panda_seed(&mut self, seed: i64) {
        self.kind
            .panda_or_insert_with(Default::default)
            .fix_seed(seed);
    }

    /// A ravager's counters (the starting state until an event comes).
    pub fn ravager(&self) -> crate::ravager::RavagerState {
        self.kind.ravager().cloned().unwrap_or_default()
    }

    /// What a cat or a panda does of its own after `super.tick`, given what
    /// the world around it holds (a sleeper near a cat, thunder).
    pub(crate) fn tick_species(&mut self, (sleeper_near_cat, thundering): (bool, bool)) {
        self.tick_cat(sleeper_near_cat);
        self.tick_panda(thundering);
        self.tick_bat();
        self.tick_parrot();
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
        if self.is_lightning_bolt() {
            let seed = i64::from(self.id);
            self.lightning_done = self
                .kind
                .lightning_or_insert_with(|| Box::new(crate::lightning::LightningState::new(seed)))
                .tick();
        }
    }

    /// `Panda.tick` after `super.tick`; a sitting panda levels its pitch.
    pub(crate) fn tick_panda(&mut self, thundering: bool) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:panda") {
            return;
        }
        let holds_item = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let around = crate::panda::PandaSurroundings {
            thundering,
            holds_item,
        };
        let panda = self.kind.panda_or_insert_with(Default::default);
        if panda.tick(around) {
            self.x_rot = JavaF32::new(0.0_f32);
        }
    }

    /// A fox's flags, variant and own client state (its head roll and crouch).
    pub fn fox(&self) -> &crate::fox::FoxState {
        &self.living.fox
    }

    /// `LivingEntityRenderer.isShaking` with the overrides of the hoglin
    /// (`Hoglin.isConverting`) and the strider (`Strider.isSuffocating`):
    /// frozen through, converting to a zoglin, or out of lava and cold.
    pub fn is_shaking(&self, piglins_zombify: bool) -> bool {
        self.is_fully_frozen()
            || match voxel_data::entity_type::name(self.type_id) {
                Some("minecraft:hoglin") => {
                    !self.living.hoglin_immune && !self.is_no_ai() && piglins_zombify
                }
                Some("minecraft:strider") => self.living.strider_suffocating,
                _ => false,
            }
    }

    /// `Strider.isSuffocating`.
    pub fn is_suffocating(&self) -> bool {
        self.living.strider_suffocating
    }

    /// `Entity.isVehicle`: something rides it.
    pub fn is_vehicle(&self) -> bool {
        !self.passenger_ids.is_empty()
    }

    /// A polar bear's own client state (how far it has stood up).
    pub fn polar_bear(&self) -> &crate::polar_bear::PolarBearState {
        &self.living.polar_bear
    }

    /// A cat's own client state (how far it lies and relaxes).
    pub fn cat(&self) -> &crate::cat::CatState {
        &self.living.cat
    }

    /// `Entity.isCrouching`: the crouching pose.
    pub fn is_crouching(&self) -> bool {
        self.pose == ClientEntityPose::Crouching
    }

    /// A wolf's own client state (its interest and its shake).
    pub fn wolf(&self) -> &crate::wolf::WolfState {
        &self.living.wolf
    }

    /// `Wolf.getTailAngle`: raised while angry, by health while tame.
    pub fn wolf_tail_angle(&self, game_time: u64) -> f32 {
        let pi = std::f64::consts::PI as f32;
        if self.is_angry(game_time) {
            1.539_380_4
        } else if self.is_tame() {
            let max_health = self.max_health().unwrap_or(0.0);
            let health = self.health().unwrap_or(max_health);
            let damage_ratio = (max_health - health) / max_health;
            (0.55 - damage_ratio * 0.4) * pi
        } else {
            (std::f64::consts::PI / 5.0) as f32
        }
    }

    /// `Rabbit.getVariant`'s id: zero is brown (an id the variants lack is brown).
    pub fn rabbit_variant(&self) -> i32 {
        self.living.rabbit_type
    }

    /// `MushroomCow.getVariant`'s id: zero is red, one brown (an id past the
    /// last takes the last).
    pub fn mooshroom_variant(&self) -> i32 {
        self.living.mooshroom_type.clamp(0, 1)
    }

    /// `SnowGolem.hasPumpkin`.
    pub fn has_pumpkin(&self) -> bool {
        !self.living.bare_head
    }

    /// `EnderMan.isCreepy`.
    pub fn is_creepy(&self) -> bool {
        self.living.creepy
    }

    /// `Hoglin.getAttackAnimationRemainingTicks` (a zoglin's too).
    pub fn hoglin_attack_ticks(&self) -> i32 {
        self.living.animal.hoglin_attack_ticks
    }

    /// An iron golem's `getAttackAnimationTick` and `getOfferFlowerTick`.
    pub fn iron_golem_ticks(&self) -> (i32, i32) {
        let animal = &self.living.animal;
        (animal.golem_attack_tick, animal.golem_offer_flower_tick)
    }

    /// `LivingEntity.getAttributeValue`: the attribute's value, from what the
    /// type starts with and what the server has changed (`AttributeInstance.getValue`:
    /// the base, the modifiers in their order, kept in the attribute's range).
    /// Nothing for an attribute the type does not have.
    pub fn attribute_value(&self, attribute: &str) -> Option<f64> {
        let (id, range) = voxel_data::entity_type::attribute(attribute)?;
        let default = voxel_data::entity_type::default_attribute(
            voxel_data::entity_type::name(self.type_id)?,
            attribute,
        )?;
        let (low, high) = range.range.unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
        let changed = self
            .attributes
            .iter()
            .find(|state| usize::try_from(state.attribute_id) == Ok(id));
        Some(changed.map_or_else(
            || ClientAttributeState::new(0, default, Vec::new()).effective_value(low, high),
            |state| state.effective_value(low, high),
        ))
    }

    /// `LivingEntity.getMaxHealth`.
    pub fn max_health(&self) -> Option<f32> {
        self.attribute_value("minecraft:max_health")
            .map(|value| value as f32)
    }

    /// `LivingEntity.getScale`: one for an entity that has no scale attribute.
    pub fn scale(&self) -> f32 {
        self.attribute_value("minecraft:scale")
            .map_or(1.0, |value| value as f32)
    }

    /// `Raider.isCelebrating`.
    pub fn is_celebrating(&self) -> bool {
        self.living.celebrating
    }

    /// `SpellcasterIllager.isCastingSpell`.
    pub fn is_casting_spell(&self) -> bool {
        self.living.casting_spell
    }

    /// `Sheep.getColor`: the dye's id (`DyeColor.getId`).
    pub fn sheep_dye_id(&self) -> u8 {
        self.living.animal.wool & 15
    }

    /// `Sheep.isSheared`.
    pub fn is_sheared(&self) -> bool {
        self.living.animal.wool & 16 != 0
    }

    /// `Sheep.getHeadEatPositionScale` and `getHeadEatAngleScale` at a point
    /// between two ticks, with the pitch the sheep is drawn at, in degrees.
    pub fn sheep_head_eat_scales(&self, partial: f32, pitch: f32) -> (f32, f32) {
        let tick = self.living.animal.eat_tick;
        (
            crate::animal::head_eat_position_scale(tick, partial),
            crate::animal::head_eat_angle_scale(tick, partial, pitch),
        )
    }

    /// `Creeper.getSwelling`: how far the fuse has burned, between two ticks.
    pub fn creeper_swelling(&self, partial: f32) -> f32 {
        self.living.animal.creeper_swelling(partial)
    }
}
