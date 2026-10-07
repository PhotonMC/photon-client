use super::*;

impl MobModel {
    /// `CopperGolemModel.setupAnim`: the head follows the look; the walk (the
    /// one with arms raised while an item is held, which keeps the arms
    /// raised at least that far), then the idle spin and the chest animations
    /// each if started.
    pub fn setup_copper_golem(&mut self, state: &CopperGolemModelState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self
            .root
            .child_mut("body")
            .and_then(|body| body.child_mut("head"))
        {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        }
        let walk = if state.holds_item {
            "CopperGolem.COPPER_GOLEM_WALK_ITEM"
        } else {
            "CopperGolem.COPPER_GOLEM_WALK"
        };
        if let Some(definition) = voxel_data::animation::definition(walk) {
            // `applyWalk(pos, speed, 2.0, 2.5)`.
            let millis = (state.quadruped.walk_position * 50.0 * 2.0) as i64;
            let scale = (state.quadruped.walk_speed * 2.5).min(1.0);
            self.animate(definition, millis, scale);
        }
        if state.holds_item
            && let Some(body) = self.root.child_mut("body")
        {
            // `poseHeldItemArmsIfStill`.
            pub(super) const RAISE: f32 = -0.87266463;
            const TURN: f32 = 0.1134464;
            const ROLL: f32 = 0.064577185;
            if let Some(arm) = body.child_mut("right_arm") {
                arm.x_rot = arm.x_rot.min(RAISE);
                arm.y_rot = arm.y_rot.min(-TURN);
                arm.z_rot = arm.z_rot.min(-ROLL);
            }
            if let Some(arm) = body.child_mut("left_arm") {
                arm.x_rot = arm.x_rot.min(RAISE);
                arm.y_rot = arm.y_rot.max(TURN);
                arm.z_rot = arm.z_rot.max(ROLL);
            }
        }
        let animations = &state.animations;
        for (name, running) in [
            ("COPPER_GOLEM_IDLE", animations.idle),
            (
                "COPPER_GOLEM_CHEST_INTERACTION_NOITEM_GET",
                animations.get_item,
            ),
            (
                "COPPER_GOLEM_CHEST_INTERACTION_NOITEM_NOGET",
                animations.get_no_item,
            ),
            (
                "COPPER_GOLEM_CHEST_INTERACTION_ITEM_DROP",
                animations.drop_item,
            ),
            (
                "COPPER_GOLEM_CHEST_INTERACTION_ITEM_NODROP",
                animations.drop_no_item,
            ),
        ] {
            if !running.is_started() {
                continue;
            }
            if let Some(definition) =
                voxel_data::animation::definition(&format!("CopperGolem.{name}"))
            {
                self.animate(definition, running.time_in_millis(state.age_in_ticks), 1.0);
            }
        }
    }

    /// `SquidModel.setupAnim`: every tentacle leans by the beat's angle.
    pub fn setup_squid(&mut self, tentacle_angle: f32) {
        self.root.reset();
        for index in 0..8 {
            if let Some(tentacle) = self.root.child_mut(&format!("tentacle{index}")) {
                tentacle.x_rot = tentacle_angle;
            }
        }
    }

    /// `ShulkerModel.setupAnim`: the lid rises and, past a third open, twists;
    /// with the lid more than fully open it also bobs; the head turns from the
    /// body to where the entity looks.
    pub fn setup_shulker(&mut self, state: &ShulkerRenderState) {
        self.root.reset();
        let pi = std::f32::consts::PI;
        let bs = (0.5 + state.peek_amount) * pi;
        let sine = mth_sin_f64(f64::from(bs));
        let q = -1.0 + sine;
        let mut extra = 0.0;
        if bs > pi {
            extra = mth_sin_f64(f64::from(state.age_in_ticks * 0.1)) * 0.7;
        }
        if let Some(lid) = self.root.child_mut("lid") {
            lid.x = 0.0;
            lid.y = 16.0 + sine * 8.0 + extra;
            lid.z = 0.0;
            lid.y_rot = if state.peek_amount > 0.3 {
                q * q * q * q * pi * 0.125
            } else {
                0.0
            };
        }
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = (state.y_head_rot - 180.0 - state.y_body_rot) * degrees;
        }
    }

    /// `MagmaCubeModel.setupAnim`: the eight slices lift apart by the squish
    /// (none while it is squashed), the lowest staying where it is.
    pub fn setup_magma_cube(&mut self, squish: f32) {
        self.root.reset();
        let squish = squish.max(0.0);
        for index in 0..8_i32 {
            if let Some(slice) = self.root.child_mut(&format!("cube{index}")) {
                slice.y = -((4 - index) as f32) * squish * 1.7;
            }
        }
    }

    /// `AdultAxolotlModel.setupAnim`: the body turns with the look, then the
    /// poses of swimming, hovering, crawling, lying still and playing dead
    /// each blended in by its factor, and the right legs mirroring the left.
    pub fn setup_axolotl_adult(&mut self, state: &AxolotlRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let (playing_dead, in_water, on_ground, moving) = (
            state.playing_dead_factor,
            state.in_water_factor,
            state.on_ground_factor,
            state.moving_factor,
        );
        let not_moving = 1.0 - moving;
        let mirrored = 1.0 - on_ground.min(moving);
        let root = &mut self.root;
        at(root, "body").y_rot += state.quadruped.head_y_rot * degrees;
        axolotl_swimming(
            root,
            state.age_in_ticks,
            state.quadruped.x_rot,
            moving.min(in_water),
        );
        axolotl_hovering(root, state.age_in_ticks, not_moving.min(in_water));
        axolotl_crawling(root, state.age_in_ticks, moving.min(on_ground));
        axolotl_lying_still(root, state.age_in_ticks, not_moving.min(on_ground));
        axolotl_playing_dead(root, playing_dead);
        axolotl_mirror_legs(root, mirrored);
    }

    /// `BabyAxolotlModel.setupAnim`: the walking animation by the walk, then
    /// each animation whose state has started. (The underwater walk runs on the
    /// walking state, as the reference has it.)
    pub fn setup_axolotl_baby(&mut self, state: &AxolotlRenderState) {
        self.root.reset();
        if state.walk.is_started()
            && let Some(definition) =
                voxel_data::animation::definition("BabyAxolotl.AXOLOTL_WALK_FLOOR")
        {
            // `applyWalk(pos, speed, 15.0, 30.0)`.
            let millis = (state.quadruped.walk_position * 50.0 * 15.0) as i64;
            let scale = (state.quadruped.walk_speed * 30.0).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("BabyAxolotl.BABY_AXOLOTL_SWIM", state.swim),
            ("BabyAxolotl.WALK_FLOOR_UNDERWATER", state.walk),
            ("BabyAxolotl.BABY_AXOLOTL_IDLE_FLOOR", state.idle_on_ground),
            ("BabyAxolotl.IDLE_UNDERWATER", state.idle_under_water),
            (
                "BabyAxolotl.IDLE_FLOOR_UNDERWATER",
                state.idle_under_water_on_ground,
            ),
            ("BabyAxolotl.BABY_AXOLOTL_PLAY_DEAD", state.play_dead),
        ] {
            if !running.is_started() {
                continue;
            }
            if let Some(definition) = voxel_data::animation::definition(name) {
                let millis = (running.time_in_millis(state.age_in_ticks) as f32 * 1.0) as i64;
                self.animate(definition, millis, 1.0);
            }
        }
    }

    /// `FrogModel.setupAnim`: the jump, croak and tongue animations each if
    /// started, then the walk (a swim while in water) by the stride, then the
    /// idle float; the croaking throat shows while the croak runs.
    pub fn setup_frog(&mut self, state: &FrogRenderState) {
        self.root.reset();
        for (name, running) in [
            ("Frog.FROG_JUMP", state.jump),
            ("Frog.FROG_CROAK", state.croak),
            ("Frog.FROG_TONGUE", state.tongue),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
        let (name, speed_factor) = if state.is_swimming {
            ("Frog.FROG_SWIM", 1.0_f32)
        } else {
            ("Frog.FROG_WALK", 1.5)
        };
        if let Some(definition) = voxel_data::animation::definition(name) {
            // `applyWalk(pos, speed, speedFactor, 2.5)`.
            let millis = (state.quadruped.walk_position * 50.0 * speed_factor) as i64;
            let scale = (state.quadruped.walk_speed * 2.5).min(1.0);
            self.animate(definition, millis, scale);
        }
        self.animate_state("Frog.FROG_IDLE_WATER", state.swim_idle, state.age_in_ticks);
        if let Some(croaking_body) = self
            .root
            .child_mut("root")
            .and_then(|root| root.child_mut("body"))
            .and_then(|body| body.child_mut("croaking_body"))
        {
            croaking_body.visible = state.croak.is_started();
        }
    }

    /// `KeyframeAnimation.apply(animationState, ageInTicks)`: the animation
    /// runs if its state has started.
    pub(super) fn animate_state(
        &mut self,
        name: &str,
        running: voxel_world::animation_state::AnimationState,
        age_in_ticks: f32,
    ) {
        if !running.is_started() {
            return;
        }
        if let Some(definition) = voxel_data::animation::definition(name) {
            let millis = (running.time_in_millis(age_in_ticks) as f32 * 1.0) as i64;
            self.animate(definition, millis, 1.0);
        }
    }

    /// `ParrotModel.setupAnim`: the pose's own resting turns first (the reference's
    /// `prepare`), the head takes the look, then by pose: a standing parrot's
    /// legs walk and it beats its wings like a flying one (the cases run on
    /// into each other), a sitting one does neither, a party parrot sways.
    pub fn setup_parrot(&mut self, state: &ParrotRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let flap = state.flap_angle;
        self.parrot_prepare(state.pose);
        let mut adjust = |name: &str, apply: &dyn Fn(&mut Part)| {
            if let Some(part) = self.root.child_mut(name) {
                apply(part);
            }
        };
        adjust("head", &|head| {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        });
        if state.pose == ParrotPose::Standing {
            adjust("left_leg", &|leg| {
                leg.x_rot += cos(position * 0.6662) * 1.4 * speed;
            });
            adjust("right_leg", &|leg| {
                leg.x_rot += cos(position * 0.6662 + pi) * 1.4 * speed;
            });
        }
        match state.pose {
            ParrotPose::Standing | ParrotPose::Flying | ParrotPose::OnShoulder => {
                let bob = flap * 0.3;
                adjust("head", &|part| part.y += bob);
                adjust("tail", &|part| {
                    part.x_rot += cos(position * 0.6662) * 0.3 * speed;
                    part.y += bob;
                });
                adjust("body", &|part| part.y += bob);
                adjust("left_wing", &|part| {
                    part.z_rot = -0.0873 - flap;
                    part.y += bob;
                });
                adjust("right_wing", &|part| {
                    part.z_rot = 0.0873 + flap;
                    part.y += bob;
                });
                adjust("left_leg", &|leg| leg.y += bob);
                adjust("right_leg", &|leg| leg.y += bob);
            }
            ParrotPose::Sitting => {}
            ParrotPose::Party => {
                let (x, y) = (cos(state.age_in_ticks), sin(state.age_in_ticks));
                adjust("head", &|head| {
                    head.x += x;
                    head.y += y;
                    head.x_rot = 0.0;
                    head.y_rot = 0.0;
                    head.z_rot = sin(state.age_in_ticks) * 0.4;
                });
                adjust("body", &|part| {
                    part.x += x;
                    part.y += y;
                });
                adjust("left_wing", &|part| {
                    part.z_rot = -0.0873 - flap;
                    part.x += x;
                    part.y += y;
                });
                adjust("right_wing", &|part| {
                    part.z_rot = 0.0873 + flap;
                    part.x += x;
                    part.y += y;
                });
                adjust("tail", &|part| {
                    part.x += x;
                    part.y += y;
                });
            }
        }
    }

    /// `BreezeModel.setupAnim`: the six animations, each if its state has
    /// started (the body's, the wind's and the eyes' layers share the parts'
    /// names, so the same call poses all three).
    pub fn setup_breeze(&mut self, state: &BreezeRenderState) {
        self.root.reset();
        for (name, running) in [
            ("Breeze.IDLE", state.idle),
            ("Breeze.SHOOT", state.shoot),
            ("Breeze.SLIDE", state.slide),
            ("Breeze.SLIDE_BACK", state.slide_back),
            ("Breeze.INHALE", state.inhale),
            ("Breeze.JUMP", state.long_jump),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `WardenModel.setupAnim`: the head takes the look, then the walk, the
    /// idle sway and the tendrils' pulse move the parts, then each animation
    /// that has started runs over them. The glowing layers share the parts'
    /// names (the cut parts are empty), so the same call poses all five.
    pub fn setup_warden(&mut self, state: &WardenRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let half_pi = (std::f64::consts::PI / 2.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        // `animateWalk`.
        let speed = (3.0 * state.quadruped.walk_speed).min(0.5);
        let position = state.quadruped.walk_position * 0.8662;
        let (position_cos, position_sin) = (cos(position), sin(position));
        let speed_with_min = speed.min(0.35);
        // `animateIdlePose`.
        let scaled_age = state.age_in_ticks * 0.1;
        let (wobble_cos, wobble_sin) = (cos(scaled_age), sin(scaled_age));
        // `animateTendrils`: the cosine is a double there.
        let tendril_x_rot = state.tendril_animation
            * ((f64::from(state.age_in_ticks) * 2.25).cos()
                * std::f64::consts::PI
                * f64::from(0.1_f32)) as f32;
        let Some(bone) = self.root.child_mut("bone") else {
            return;
        };
        if let Some(body) = bone.child_mut("body") {
            body.z_rot = 0.1 * position_sin * speed;
            body.x_rot = 1.0 * position_cos * speed_with_min;
            body.z_rot += 0.025 * wobble_sin;
            body.x_rot += 0.025 * wobble_cos;
            if let Some(head) = body.child_mut("head") {
                head.x_rot = state.quadruped.x_rot * degrees;
                head.y_rot = state.quadruped.head_y_rot * degrees;
                head.z_rot += 0.3 * position_sin * speed;
                head.x_rot += 1.2 * cos(position + half_pi) * speed_with_min;
                head.z_rot += 0.06 * wobble_cos;
                head.x_rot += 0.06 * wobble_sin;
                if let Some(left) = head.child_mut("left_tendril") {
                    left.x_rot = tendril_x_rot;
                }
                if let Some(right) = head.child_mut("right_tendril") {
                    right.x_rot = -tendril_x_rot;
                }
            }
            // `resetArmPoses`: the arms stand where the model puts them, as the walk
            // does not move them there.
            for (name, swing, side) in [
                ("left_arm", -(0.8 * position_cos * speed), 13.0),
                ("right_arm", -(0.8 * position_sin * speed), -13.0),
            ] {
                if let Some(arm) = body.child_mut(name) {
                    arm.x_rot = swing;
                    arm.z_rot = 0.0;
                    arm.y_rot = 0.0;
                    arm.z = 1.0;
                    arm.x = side;
                    arm.y = -13.0;
                }
            }
        }
        if let Some(leg) = bone.child_mut("left_leg") {
            leg.x_rot = 1.0 * position_cos * speed;
        }
        if let Some(leg) = bone.child_mut("right_leg") {
            leg.x_rot = 1.0 * cos(position + pi) * speed;
        }
        for (name, running) in [
            ("Warden.WARDEN_ATTACK", state.attack),
            ("Warden.WARDEN_SONIC_BOOM", state.sonic_boom),
            ("Warden.WARDEN_DIG", state.digging),
            ("Warden.WARDEN_EMERGE", state.emerge),
            ("Warden.WARDEN_ROAR", state.roar),
            ("Warden.WARDEN_SNIFF", state.sniff),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }
}
