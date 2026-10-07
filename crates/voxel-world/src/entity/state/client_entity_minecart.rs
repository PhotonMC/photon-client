use super::*;

impl ClientEntity {
    /// A boat's data: its hit, paddles and bubbles.
    pub(crate) fn set_boat_data(&mut self, data: crate::metadata::BoatData) {
        let boat = self.kind.boat_or_insert_with(Default::default);
        if let Some(hurt) = data.hurt_time {
            boat.hurt_time = hurt;
        }
        if let Some(dir) = data.hurt_dir {
            boat.hurt_dir = dir;
        }
        if let Some(damage) = data.damage {
            boat.damage = damage;
        }
        if let Some(bubbles) = data.bubble_time {
            boat.bubble_time = bubbles;
        }
        if let Some(left) = data.paddle_left {
            self.paddle_left = left;
        }
        if let Some(right) = data.paddle_right {
            self.paddle_right = right;
        }
    }

    /// `Parrot.getVariant`'s id.
    pub fn parrot_variant(&self) -> i32 {
        self.living.parrot_variant
    }

    /// A parrot's wing beats, if it has ticked.
    pub fn parrot(&self) -> Option<&crate::parrot::ParrotState> {
        self.kind.parrot()
    }

    /// `Parrot.calculateFlapping` after `super.aiStep`: the wings beat by
    /// whether the parrot is in the air, and one falling there sinks slowly.
    pub(crate) fn tick_parrot(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:parrot") {
            return;
        }
        let (on_ground, passenger) = (self.on_ground, self.vehicle_id.is_some());
        if !on_ground && self.motion.y.get() < 0.0 {
            self.motion.y = JavaF64::new(self.motion.y.get() * 0.6);
        }
        self.kind
            .parrot_or_insert_with(Default::default)
            .tick(on_ground, passenger);
    }

    /// A bat's animation states, if it has ticked.
    pub fn bat(&self) -> Option<&crate::bat::BatState> {
        self.kind.bat()
    }

    /// An allay's progress, if it has ticked.
    pub fn allay(&self) -> Option<&crate::allay::AllayState> {
        self.kind.allay()
    }

    /// `Bat.tick` after `super.tick`: a hanging bat does not move and hangs
    /// from the block above it; one flying slows its fall and climb.
    pub(crate) fn tick_bat(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:bat") {
            return;
        }
        let resting = self.living.bat_resting;
        if resting {
            self.motion.x = JavaF64::new(0.0_f64);
            self.motion.y = JavaF64::new(0.0_f64);
            self.motion.z = JavaF64::new(0.0_f64);
            let height = voxel_data::entity_type::pick(self.type_id)
                .map_or(0.0, |pick| pick.size.1 * self.scale());
            let y = self.position.y.get();
            self.position.y = JavaF64::new(y.floor() + 1.0 - f64::from(height));
        } else {
            self.motion.y = JavaF64::new(self.motion.y.get() * 0.6);
        }
        let tick_count = self.tick_count;
        self.kind
            .bat_or_insert_with(Default::default)
            .tick(resting, tick_count);
    }

    /// `Allay.tick`'s client half, after `super.tick`.
    pub(crate) fn tick_allay(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:allay") {
            return;
        }
        let holds = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let dancing = self.living.allay_dancing;
        self.kind
            .allay_or_insert_with(Default::default)
            .tick(holds, dancing);
    }

    /// `Ghast.isCharging`: the ghast has its mouth open for a fireball.
    pub fn is_ghast_charging(&self) -> bool {
        self.living.ghast_charging
    }

    /// `Phantom.getPhantomSize`.
    pub fn phantom_size(&self) -> i32 {
        self.living.phantom_size
    }

    /// A guardian's animations, if it has ticked.
    pub fn guardian(&self) -> Option<&crate::guardian::GuardianState> {
        self.kind.guardian()
    }

    pub(crate) fn is_guardian(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:guardian" | "minecraft:elder_guardian")
        )
    }

    /// `Guardian.getActiveAttackTarget`'s id: what the beam aims at, if anything.
    pub fn guardian_attack_target(&self) -> Option<i32> {
        (self.living.guardian_target != 0).then_some(self.living.guardian_target)
    }

    /// `Guardian.isMoving`.
    pub fn is_guardian_moving(&self) -> bool {
        self.living.guardian_moving
    }

    /// `Guardian.onSyncedDataUpdated`: when the data holds the attack target,
    /// the beam starts over.
    pub(crate) fn set_guardian_data(&mut self, moving: Option<bool>, target: Option<i32>) {
        if let Some(moving) = moving {
            self.living.guardian_moving = moving;
        }
        if let Some(target) = target {
            self.living.guardian_target = target;
            self.kind
                .guardian_or_insert_with(|| Box::new(crate::guardian::GuardianState::new()))
                .restart_attack();
        }
    }

    /// `Guardian.aiStep`'s client half, while the guardian lives: the beam's
    /// target (which the world resolves from its id) is where the head turns.
    pub(crate) fn tick_guardian(&mut self, target: Option<crate::guardian::GuardianTarget>) {
        if !self.is_guardian() || !self.alive {
            return;
        }
        let eye_height = self
            .dimensions()
            .scale(self.scale(), self.scale())
            .eye_height();
        let place = crate::guardian::GuardianPlace {
            in_water: self.living.in_water,
            moving: self.living.guardian_moving,
            position: self.position(),
            eye_height,
            target,
        };
        let mut pose = crate::guardian::GuardianPose {
            y_rot: self.y_rot.get(),
            x_rot: self.x_rot.get(),
            y_head_rot: self.y_head_rot.get(),
            motion: self.motion(),
            on_ground: self.on_ground,
        };
        let has_target = self.living.guardian_target != 0;
        let duration = self.guardian_attack_duration();
        self.kind
            .guardian_or_insert_with(|| Box::new(crate::guardian::GuardianState::new()))
            .tick(&place, has_target, duration, &mut pose);
        self.y_rot = JavaF32::new(pose.y_rot);
        self.x_rot = JavaF32::new(pose.x_rot);
        self.y_head_rot = JavaF32::new(pose.y_head_rot);
        self.motion = DVec3::new(pose.motion.0, pose.motion.1, pose.motion.2);
        self.on_ground = pose.on_ground;
    }

    /// `Guardian.getAttackDuration`, shorter for the elder.
    pub fn guardian_attack_duration(&self) -> i32 {
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:elder_guardian") {
            60
        } else {
            80
        }
    }

    /// Whether this is a frog.
    pub(crate) fn is_frog(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:frog")
    }

    /// A frog's animation states, if it has any.
    pub fn frog(&self) -> Option<&crate::frog::FrogState> {
        self.kind.frog()
    }

    /// `Frog.onSyncedDataUpdated(DATA_POSE)`: the pose the data names starts
    /// or stops the frog's animations.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose) {
        self.pose = pose;
        if self.is_frog() {
            let tick_count = self.tick_count;
            self.kind
                .frog_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
        if self.is_breeze() {
            let tick_count = self.tick_count;
            self.kind
                .breeze_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.kind
                .warden_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
    }

    /// A sniffer's state and animation states, if it has any.
    pub fn sniffer(&self) -> Option<&crate::sniffer::SnifferState> {
        self.kind.sniffer()
    }

    /// `Sniffer.onSyncedDataUpdated(DATA_STATE)`.
    pub(crate) fn set_sniffer_mode(&mut self, mode: crate::sniffer::SnifferMode) {
        let tick_count = self.tick_count;
        self.kind
            .sniffer_or_insert_with(Default::default)
            .set_mode(mode, tick_count);
    }

    /// `FireworkRocketEntity.isShotAtAngle`.
    pub fn is_shot_at_angle(&self) -> bool {
        self.shot_at_angle
    }

    /// Whether the type draws as an item facing the camera (`ThrownItemRenderer`, with its
    /// scale and whether it is fully lit), and so has its item among its data.
    pub fn thrown_item_look(&self) -> Option<(f32, bool)> {
        match voxel_data::entity_type::name(self.type_id)? {
            "minecraft:egg"
            | "minecraft:ender_pearl"
            | "minecraft:experience_bottle"
            | "minecraft:lingering_potion"
            | "minecraft:snowball"
            | "minecraft:splash_potion" => Some((1.0, false)),
            "minecraft:eye_of_ender" => Some((1.0, true)),
            "minecraft:fireball" => Some((3.0, true)),
            "minecraft:small_fireball" => Some((0.75, true)),
            _ => None,
        }
    }

    /// Whether this is an arrow, a spectral arrow or a trident (`AbstractArrow`).
    pub fn is_abstract_arrow(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:arrow" | "minecraft:spectral_arrow" | "minecraft:trident")
        )
    }

    /// The block the entity was added in (`FallingBlockEntity.getStartPos`).
    pub fn start_block(&self) -> (i32, i32, i32) {
        self.start_block
    }

    /// The id of the entity this one is leashed to, if it is.
    pub fn leash_holder_id(&self) -> Option<i32> {
        (self.leash_holder_id != 0).then_some(self.leash_holder_id)
    }

    /// Whether this is a lightning bolt (`LightningBolt`).
    pub fn is_lightning_bolt(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:lightning_bolt")
    }

    /// A lightning bolt's shape and flashes, if it has begun to tick.
    pub fn lightning(&self) -> Option<&crate::lightning::LightningState> {
        self.kind.lightning()
    }

    pub(crate) fn lightning_finished(&self) -> bool {
        self.lightning_done
    }

    /// A display entity's data and render state.
    pub fn display(&self) -> Option<&crate::display::DisplayState> {
        self.kind.display()
    }

    /// Whether the entity has an `InterpolationHandler` (`getInterpolation` is not null): the
    /// living, the minecarts and the displays ease to the positions the server sends.
    pub(crate) fn has_interpolation_handler(&self) -> bool {
        self.living_kind().is_some() || self.is_minecart() || self.kind.display().is_some()
    }

    /// The length of the handler's easing (`InterpolationHandler.interpolationSteps`): three, or
    /// a display's own.
    pub(in super::super) fn interpolation_length(&self) -> i32 {
        self.kind
            .display()
            .map_or(i32::from(LIVING_ENTITY_POSITION_LERP_STEPS), |display| {
                display.pos_rot_duration()
            })
    }

    /// Whether this is an item frame or a glow item frame (`ItemFrame`).
    pub fn is_item_frame(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:item_frame" | "minecraft:glow_item_frame")
        )
    }

    /// `ItemFrame.getRotation`.
    pub fn item_frame_rotation(&self) -> i32 {
        self.item_frame_rotation
    }

    /// Whether this is a block of primed tnt (`PrimedTnt`).
    pub fn is_primed_tnt(&self) -> bool {
        self.type_id == VANILLA_PRIMED_TNT_ENTITY_TYPE_ID
    }

    /// A primed tnt's fuse and block, if it has any.
    pub fn primed_tnt(&self) -> Option<&crate::primed_tnt::PrimedTntState> {
        self.kind.primed_tnt()
    }

    pub(crate) fn primed_tnt_mut(&mut self) -> &mut crate::primed_tnt::PrimedTntState {
        self.kind.primed_tnt_or_insert_with(Default::default)
    }

    /// Whether this is a pair of evoker fangs.
    pub fn is_evoker_fangs(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:evoker_fangs")
    }

    /// Evoker fangs' bite, if it has begun to be told of.
    pub fn evoker_fangs(&self) -> Option<&crate::evoker_fangs::FangsState> {
        self.kind.evoker_fangs()
    }

    pub(crate) fn evoker_fangs_mut(&mut self) -> &mut crate::evoker_fangs::FangsState {
        self.kind.evoker_fangs_or_insert_with(Default::default)
    }

    /// An arrow's shake and data, if it has any.
    pub fn arrow(&self) -> Option<&crate::arrow::ArrowState> {
        self.kind.arrow()
    }

    pub(crate) fn arrow_mut(&mut self) -> &mut crate::arrow::ArrowState {
        self.kind.arrow_or_insert_with(Default::default)
    }

    /// Whether this is one of the minecarts (`AbstractMinecart`).
    pub fn is_minecart(&self) -> bool {
        voxel_data::entity_type::name(self.type_id)
            .and_then(crate::minecart::default_display_offset)
            .is_some()
    }

    /// A minecart's hit, carried block and fuel, if it has any.
    pub fn minecart(&self) -> Option<&crate::minecart::MinecartState> {
        self.kind
            .minecart_slot()
            .and_then(|slot| slot.state.as_ref())
    }

    pub(crate) fn minecart_mut(&mut self) -> &mut crate::minecart::MinecartState {
        self.kind
            .minecart_slot_or_insert_with(Default::default)
            .state
            .get_or_insert_with(Default::default)
    }

    /// `AbstractMinecart.getDisplayBlockState`: the block state the server named, else the
    /// type's own (the plain minecart shows none).
    pub fn minecart_display_block(&self) -> Option<u32> {
        let state = self.minecart().copied().unwrap_or_default();
        state.custom_display.or_else(|| {
            voxel_data::entity_type::name(self.type_id)
                .and_then(|name| crate::minecart::default_display_block(name, state.fuel))
        })
    }

    /// `AbstractMinecart.getDisplayOffset`, in sixteenths of a block.
    pub fn minecart_display_offset(&self) -> i32 {
        let default = voxel_data::entity_type::name(self.type_id)
            .and_then(crate::minecart::default_display_offset)
            .unwrap_or(6);
        self.minecart()
            .copied()
            .unwrap_or_default()
            .display_offset(default)
    }

    /// `AbstractMinecart.tick` on the client, with `OldMinecartBehavior.tick`'s client half: the
    /// hit runs down, then the position and turn ease to the last packet's while that lasts,
    /// else the turns are kept under a whole turn.
    pub(crate) fn tick_minecart(&mut self) {
        self.minecart_mut().tick();
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:tnt_minecart") {
            self.minecart_mut().burn_fuse();
        }
        if self.minecart_track().is_some() {
            self.tick_minecart_track();
        } else if self.is_interpolating() {
            self.tick_position_interpolation();
        } else {
            let (y_rot, x_rot) = (self.y_rot.get(), self.x_rot.get());
            self.x_rot = JavaF32::new(x_rot % 360.0);
            self.y_rot = JavaF32::new(y_rot % 360.0);
        }
    }

    /// `NewMinecartBehavior.lerpClientPositionAndRotation`: the steps the server sent are played
    /// back, putting the minecart where they lead.
    pub(in super::super) fn tick_minecart_track(&mut self) {
        let (y_rot, x_rot) = self.body_rotation();
        let now = crate::minecart_track::MinecartStep {
            position: self.position(),
            movement: self.motion(),
            y_rot,
            x_rot,
            weight: 0.0,
        };
        let Some(place) = self
            .kind
            .minecart_slot_mut()
            .and_then(|slot| slot.track.as_mut())
            .and_then(|track| track.tick(now))
        else {
            return;
        };
        self.set_position(place.position);
        self.motion.x = JavaF64::new(place.movement.0);
        self.motion.y = JavaF64::new(place.movement.1);
        self.motion.z = JavaF64::new(place.movement.2);
        // `Entity.setXRot` and `setYRot` discard what is not finite.
        if place.x_rot.is_finite() {
            self.x_rot = JavaF32::new((place.x_rot % 360.0).clamp(-90.0, 90.0));
        }
        if place.y_rot.is_finite() {
            self.y_rot = JavaF32::new(place.y_rot);
        }
    }
}
