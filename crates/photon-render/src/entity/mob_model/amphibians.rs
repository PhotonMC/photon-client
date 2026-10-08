use super::*;

impl MobModel {
    /// `TadpoleModel.setupAnim`: the tail swings, wider out of water.
    pub fn setup_tadpole(&mut self, state: &TadpoleState) {
        self.root.reset();
        let amplitude = if state.is_in_water { 1.0_f32 } else { 1.5 };
        if let Some(tail) = self.root.child_mut("tail") {
            tail.y_rot = -amplitude * 0.25 * mth_sin_f64(f64::from(0.3 * state.age_in_ticks));
        }
    }

    /// `TurtleModel.setupAnim` (and `AdultTurtleModel`'s belly): the walk of
    /// the quadruped, then the flippers rowing on land (faster when laying)
    /// or paddling in water; the egg shows, and lowers the model a sixteenth.
    pub fn setup_turtle(&mut self, state: &TurtleState) {
        self.setup_quadruped(&state.quadruped);
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let mut set = |name: &str, apply: &dyn Fn(&mut Part)| {
            if let Some(part) = self.root.child_mut(name) {
                apply(part);
            }
        };
        if state.is_on_land {
            let lay_egg = if state.is_laying_egg { 4.0_f32 } else { 1.0 };
            let amplitude = if state.is_laying_egg { 2.0_f32 } else { 1.0 };
            let swing_pos = position * 5.0;
            let front = cos(lay_egg * swing_pos);
            let hind = cos(swing_pos);
            set("right_front_leg", &|leg| {
                leg.y_rot = -front * 8.0 * speed * amplitude;
            });
            set("left_front_leg", &|leg| {
                leg.y_rot = front * 8.0 * speed * amplitude;
            });
            set("right_hind_leg", &|leg| leg.y_rot = -hind * 3.0 * speed);
            set("left_hind_leg", &|leg| leg.y_rot = hind * 3.0 * speed);
        } else {
            let swing = cos(position * 0.6662 * 0.6) * (0.5 * speed);
            set("right_hind_leg", &|leg| leg.x_rot = swing);
            set("left_hind_leg", &|leg| leg.x_rot = -swing);
            set("right_front_leg", &|leg| leg.z_rot = -swing);
            set("left_front_leg", &|leg| leg.z_rot = swing);
        }
        if let Some(belly) = self.root.child_mut("egg_belly") {
            belly.visible = state.has_egg;
            if state.has_egg {
                self.root.y -= 1.0;
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
                photon_data::animation::definition("BabyAxolotl.AXOLOTL_WALK_FLOOR")
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
            if let Some(definition) = photon_data::animation::definition(name) {
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
        if let Some(definition) = photon_data::animation::definition(name) {
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
}

/// `AdultAxolotlModel.setupSwimmingAnimation`.
pub(super) fn axolotl_swimming(root: &mut Part, age_in_ticks: f32, x_rot: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let degrees = (std::f64::consts::PI / 180.0) as f32;
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    let anim_move_speed = age_in_ticks * 0.33;
    let sine_sway = mth_sin_f64(f64::from(anim_move_speed));
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let body_sway = 0.13 * sine_sway;
    at(root, "body").x_rot += (x_rot * degrees + body_sway) * factor;
    at(root, "body/head").x_rot -= body_sway * 1.8 * factor;
    at(root, "body").y -= 0.45 * cosine_sway * factor;
    at(root, "body/head/top_gills").x_rot += (-0.5 * sine_sway - 0.8) * factor;
    let gill_y_rot = (0.3 * sine_sway + 0.9) * factor;
    at(root, "body/head/left_gills").y_rot += gill_y_rot;
    at(root, "body/head/right_gills").y_rot -= gill_y_rot;
    let tail = at(root, "body/tail");
    tail.y_rot += 0.3 * mth_cos_f64(f64::from(anim_move_speed * 0.9)) * factor;
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += 1.884_955_8 * factor;
    hind.y_rot += -0.4 * sine_sway * factor;
    hind.z_rot += half_pi * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += 1.884_955_8 * factor;
    front.y_rot += (-0.2 * cosine_sway - 0.1) * factor;
    front.z_rot += half_pi * factor;
}

/// `AdultAxolotlModel.setupWaterHoveringAnimation`.
pub(super) fn axolotl_hovering(root: &mut Part, age_in_ticks: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let anim_move_speed = age_in_ticks * 0.075;
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let sine_sway = mth_sin_f64(f64::from(anim_move_speed)) * 0.15;
    let body_x_rot = (-0.15 + 0.075 * cosine_sway) * factor;
    let body = at(root, "body");
    body.x_rot += body_x_rot;
    body.y -= sine_sway * factor;
    at(root, "body/head").x_rot -= body_x_rot;
    at(root, "body/head/top_gills").x_rot += 0.2 * cosine_sway * factor;
    let gill_y_rot = (-0.3 * cosine_sway - 0.19) * factor;
    at(root, "body/head/left_gills").y_rot += gill_y_rot;
    at(root, "body/head/right_gills").y_rot -= gill_y_rot;
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += ((std::f64::consts::PI * 3.0 / 4.0) as f32 - cosine_sway * 0.11) * factor;
    hind.y_rot += 0.471_238_94 * factor;
    hind.z_rot += 1.727_876_1 * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += ((std::f64::consts::PI / 4.0) as f32 - cosine_sway * 0.2) * factor;
    front.y_rot += 2.042_035 * factor;
    at(root, "body/tail").y_rot += 0.5 * cosine_sway * factor;
}

/// `AdultAxolotlModel.setupGroundCrawlingAnimation`.
pub(super) fn axolotl_crawling(root: &mut Part, age_in_ticks: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    let anim_move_speed = age_in_ticks * 0.11;
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let hind_leg_y_rot_sway = (cosine_sway * cosine_sway - 2.0 * cosine_sway) / 5.0;
    let front_leg_y_rot_sway = 0.7 * cosine_sway;
    let head_and_tail_y_rot = 0.09 * cosine_sway * factor;
    at(root, "body/head").y_rot += head_and_tail_y_rot;
    at(root, "body/tail").y_rot += head_and_tail_y_rot;
    let sine = mth_sin_f64(f64::from(anim_move_speed));
    let gill_angle = (0.6 - 0.08 * (cosine_sway * cosine_sway + 2.0 * sine)) * factor;
    at(root, "body/head/top_gills").x_rot += gill_angle;
    at(root, "body/head/left_gills").y_rot -= gill_angle;
    at(root, "body/head/right_gills").y_rot += gill_angle;
    let hind_leg_x_rot = 0.942_477_9 * factor;
    let front_leg_x_rot = 1.099_557_4 * factor;
    let left_hind = at(root, "body/left_hind_leg");
    left_hind.x_rot += hind_leg_x_rot;
    left_hind.y_rot += (1.5 - hind_leg_y_rot_sway) * factor;
    left_hind.z_rot += -0.1 * factor;
    let left_front = at(root, "body/left_front_leg");
    left_front.x_rot += front_leg_x_rot;
    left_front.y_rot += (half_pi - front_leg_y_rot_sway) * factor;
    let right_hind = at(root, "body/right_hind_leg");
    right_hind.x_rot += hind_leg_x_rot;
    right_hind.y_rot += (-1.0 - hind_leg_y_rot_sway) * factor;
    let right_front = at(root, "body/right_front_leg");
    right_front.x_rot += front_leg_x_rot;
    right_front.y_rot += ((-std::f64::consts::PI / 2.0) as f32 - front_leg_y_rot_sway) * factor;
}

/// `AdultAxolotlModel.setupLayStillOnGroundAnimation`.
pub(super) fn axolotl_lying_still(root: &mut Part, age_in_ticks: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let anim_move_speed = age_in_ticks * 0.09;
    let sine_sway = mth_sin_f64(f64::from(anim_move_speed));
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let movement = sine_sway * sine_sway - 2.0 * sine_sway;
    let movement2 = cosine_sway * cosine_sway - 3.0 * sine_sway;
    let head = at(root, "body/head");
    head.x_rot += -0.09 * movement * factor;
    head.z_rot += -0.2 * factor;
    at(root, "body/tail").y_rot += (-0.1 + 0.1 * movement) * factor;
    let gill_angle = (0.6 + 0.05 * movement2) * factor;
    at(root, "body/head/top_gills").x_rot += gill_angle;
    at(root, "body/head/left_gills").y_rot -= gill_angle;
    at(root, "body/head/right_gills").y_rot += gill_angle;
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += 1.1 * factor;
    hind.y_rot += 1.0 * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += 0.8 * factor;
    front.y_rot += 2.3 * factor;
    front.z_rot -= 0.5 * factor;
}

/// `AdultAxolotlModel.setupPlayDeadAnimation`.
pub(super) fn axolotl_playing_dead(root: &mut Part, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += 1.413_716_7 * factor;
    hind.y_rot += 1.099_557_4 * factor;
    hind.z_rot += (std::f64::consts::PI / 4.0) as f32 * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += (std::f64::consts::PI / 4.0) as f32 * factor;
    front.y_rot += 2.042_035 * factor;
    let body = at(root, "body");
    body.x_rot += -0.15 * factor;
    body.z_rot += 0.35 * factor;
}

/// `AdultAxolotlModel.applyMirrorLegRotations`: the right legs add the left
/// ones, mirrored.
pub(super) fn axolotl_mirror_legs(root: &mut Part, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    for (left, right) in [
        ("body/left_hind_leg", "body/right_hind_leg"),
        ("body/left_front_leg", "body/right_front_leg"),
    ] {
        let (x, y, z) = {
            let left = at(root, left);
            (left.x_rot, left.y_rot, left.z_rot)
        };
        let right = at(root, right);
        right.x_rot += x * factor;
        right.y_rot += -y * factor;
        right.z_rot += -z * factor;
    }
}

/// What an axolotl's models are posed from (`AxolotlRenderState`): the
/// adult's blend factors and the baby's animation states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxolotlRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub playing_dead_factor: f32,
    pub moving_factor: f32,
    pub in_water_factor: f32,
    pub on_ground_factor: f32,
    pub swim: photon_world::animation_state::AnimationState,
    pub walk: photon_world::animation_state::AnimationState,
    pub idle_under_water: photon_world::animation_state::AnimationState,
    pub idle_under_water_on_ground: photon_world::animation_state::AnimationState,
    pub idle_on_ground: photon_world::animation_state::AnimationState,
    pub play_dead: photon_world::animation_state::AnimationState,
}

/// What a frog's model is posed from (`FrogRenderState`): the animation
/// states its pose and floating run, and whether it swims.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrogRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_swimming: bool,
    pub jump: photon_world::animation_state::AnimationState,
    pub croak: photon_world::animation_state::AnimationState,
    pub tongue: photon_world::animation_state::AnimationState,
    pub swim_idle: photon_world::animation_state::AnimationState,
}

/// What a tadpole's model is posed from (`LivingEntityRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TadpoleState {
    pub age_in_ticks: f32,
    pub is_in_water: bool,
}

/// What a turtle's model is posed from (`TurtleRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TurtleState {
    pub quadruped: QuadrupedState,
    pub is_on_land: bool,
    pub is_laying_egg: bool,
    /// Whether the egg shows on the belly (never on a baby).
    pub has_egg: bool,
}
