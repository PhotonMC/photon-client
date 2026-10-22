use super::*;

/// `Crackiness.byFraction` for the thresholds `(low, medium, high)` of one kind.
pub fn crackiness((low, medium, high): (f32, f32, f32), fraction: f32) -> Crackiness {
    if fraction < high {
        Crackiness::High
    } else if fraction < medium {
        Crackiness::Medium
    } else if fraction < low {
        Crackiness::Low
    } else {
        Crackiness::None
    }
}

/// `Crackiness.GOLEM.byFraction`: how cracked an iron golem with this
/// fraction of its health left looks.
pub fn golem_crackiness(fraction: f32) -> Crackiness {
    crackiness((0.75, 0.5, 0.25), fraction)
}

/// `Crackiness.WOLF_ARMOR.byDamage`: how cracked wolf armor with `damage` of `max_damage` looks;
/// an item that does not wear down is not cracked.
pub fn wolf_armor_crackiness(damage: i32, max_damage: i32) -> Crackiness {
    if max_damage <= 0 {
        return Crackiness::None;
    }
    crackiness(
        (0.95, 0.69, 0.32),
        (max_damage - damage) as f32 / max_damage as f32,
    )
}

/// What `IronGolemRenderer.setupRotations` rolls the pose by while the golem
/// walks, in degrees, or none while it stands.
pub fn iron_golem_roll(walk_position: f32, walk_speed: f32) -> Option<f32> {
    (walk_speed >= 0.01).then(|| 6.5 * triangle_wave(walk_position + 6.0, 13.0))
}

/// What the iron golem's model is posed from (`IronGolemRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IronGolemState {
    pub quadruped: QuadrupedState,
    /// `attackTicksRemaining`: the arms' swing, in ticks with the part of one.
    pub attack_ticks_remaining: f32,
    /// `offerFlowerTick`: how long it has still to hold out its flower.
    pub offer_flower_tick: i32,
}

/// `Crackiness.Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crackiness {
    None,
    Low,
    Medium,
    High,
}

/// What a copper golem's model is posed from (`CopperGolemRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CopperGolemModelState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// Either hand holds something (`rightHandItemState` or `leftHandItemState`).
    pub holds_item: bool,
    pub animations: photon_world::mobs::copper_golem::CopperGolemPlaying,
}

impl MobModel {
    /// `IronGolemModel.setupAnim`: the arms swing as the golem strikes, hold
    /// out a flower, or swing with its stride; the head follows the look and
    /// the legs stride as a pair.
    pub fn setup_iron_golem(&mut self, state: &IronGolemState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let look = &state.quadruped;
        let (position, speed) = (look.walk_position, look.walk_speed);
        let stride = triangle_wave(position, 13.0);
        let (right_arm, left_arm) = if state.attack_ticks_remaining > 0.0 {
            let swing = -2.0 + 1.5 * triangle_wave(state.attack_ticks_remaining, 10.0);
            (swing, swing)
        } else if state.offer_flower_tick > 0 {
            (
                -0.8 + 0.025 * triangle_wave(state.offer_flower_tick as f32, 70.0),
                0.0,
            )
        } else {
            ((-0.2 + 1.5 * stride) * speed, (-0.2 - 1.5 * stride) * speed)
        };
        for (name, x_rot) in [
            ("right_arm", right_arm),
            ("left_arm", left_arm),
            ("right_leg", -1.5 * stride * speed),
            ("left_leg", 1.5 * stride * speed),
        ] {
            if let Some(part) = self.root.child_mut(name) {
                part.x_rot = x_rot;
                if name.ends_with("_leg") {
                    part.y_rot = 0.0;
                }
            }
        }
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = look.head_y_rot * degrees;
            head.x_rot = look.x_rot * degrees;
        }
    }

    /// `SnowGolemModel.setupAnim`: the head follows the look, the upper body
    /// follows a quarter of it, and the arms, held out from the upper body,
    /// turn with it.
    pub fn setup_snow_golem(&mut self, state: &QuadrupedState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = state.head_y_rot * degrees;
            head.x_rot = state.x_rot * degrees;
        }
        let turn = state.head_y_rot * degrees * 0.25;
        let sin = mth_sin_f64(f64::from(turn));
        let cos = mth_cos_f64(f64::from(turn));
        if let Some(body) = self.root.child_mut("upper_body") {
            body.y_rot = turn;
        }
        if let Some(arm) = self.root.child_mut("left_arm") {
            arm.y_rot = turn;
            arm.x = cos * 5.0;
            arm.z = -sin * 5.0;
        }
        if let Some(arm) = self.root.child_mut("right_arm") {
            arm.y_rot = turn + pi;
            arm.x = -cos * 5.0;
            arm.z = sin * 5.0;
        }
    }

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
        if let Some(definition) = photon_data::animation::definition(walk) {
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
                photon_data::animation::definition(&format!("CopperGolem.{name}"))
            {
                self.animate(definition, running.time_in_millis(state.age_in_ticks), 1.0);
            }
        }
    }
}

#[cfg(test)]
mod crackiness_tests {
    use super::*;

    #[test]
    fn wolf_armor_cracks_below_each_fraction_of_its_durability_left() {
        let level = |damage| wolf_armor_crackiness(damage, 100);
        assert_eq!(level(0), Crackiness::None);
        assert_eq!(level(5), Crackiness::None);
        assert_eq!(level(6), Crackiness::Low);
        assert_eq!(level(31), Crackiness::Low);
        assert_eq!(level(32), Crackiness::Medium);
        assert_eq!(level(68), Crackiness::Medium);
        assert_eq!(level(69), Crackiness::High);
        assert_eq!(wolf_armor_crackiness(9, 0), Crackiness::None);
    }
}
