use super::*;
use photon_core::ids::BlockStateId;

impl LocalPlayer {
    /// Whether the player only watches.
    pub fn spectating(&self) -> bool {
        self.is_spectator()
    }

    /// Whether the player stands in water.
    pub fn in_water(&self) -> bool {
        self.is_in_water()
    }

    /// What the player model is posed from, between the last two ticks.
    pub fn model_pose(&self, partial_tick: f32) -> ModelPose {
        let lerp = |old: f32, new: f32| old + partial_tick * (new - old);
        ModelPose {
            body_rot: rot_lerp(partial_tick, self.y_body_rot_old, self.y_body_rot),
            head_rot: rot_lerp(partial_tick, self.y_head_rot_old, self.y_head_rot),
            x_rot: self.x_rot,
            walk_position: self.walk.position_at(partial_tick),
            walk_speed: self.walk.speed_at(partial_tick),
            swim_amount: lerp(self.swim_amount_old, self.swim_amount),
            crouching: self.pose == ClientEntityPose::Crouching,
            pose: self.pose,
        }
    }

    pub(super) fn is_passenger(&self) -> bool {
        self.riding.is_some()
    }

    pub fn eye_height(&self) -> f32 {
        self.dimensions.eye_height()
    }

    /// `Entity.getBbWidth`.
    pub fn bb_width(&self) -> f32 {
        self.dimensions.width()
    }

    /// `Entity.getBbHeight`.
    pub fn bb_height(&self) -> f32 {
        self.dimensions.height()
    }

    /// `LocalPlayer.aiStep`'s count of how long the eyes have been under water: up by one a tick
    /// (ten for a spectator) while they are, down by ten while they are not.
    pub(super) fn update_water_vision(&mut self) {
        if self.water.eyes_inside {
            let speed = if self.is_spectator() { 10 } else { 1 };
            self.eyes.water_vision_time = (self.eyes.water_vision_time + speed).clamp(0, 600);
        } else if self.eyes.water_vision_time > 0 {
            self.eyes.water_vision_time = (self.eyes.water_vision_time - 10).clamp(0, 600);
        }
    }

    /// `LocalPlayer.getWaterVision`: how well the player sees under water, from none to all.
    pub fn water_vision(&self) -> f32 {
        water_vision(self.eyes.water_vision_time, self.water.eyes_inside)
    }

    /// Whether the eyes are inside water, as the last fluid update found.
    pub fn is_eye_in_water(&self) -> bool {
        self.water.eyes_inside
    }

    /// `LocalPlayer.isShiftKeyDown`: the input, not the synced flag.
    pub fn is_shift_key_down(&self) -> bool {
        self.input.key_presses.shift
    }

    /// `LocalPlayer.jumpableVehicle` and `getJumpRidingScale`: how far the jump of the mount that
    /// can jump is charged, if the player rides one.
    pub fn jumpable_vehicle_scale(&self) -> Option<f32> {
        self.riding
            .is_some_and(|riding| riding.can_jump)
            .then_some(self.jump_riding_scale)
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    pub fn is_flying(&self) -> bool {
        self.abilities.flying
    }

    /// `Entity.isColliding`: whether a block's collision shape at `pos`
    /// overlaps the player.
    pub fn is_colliding(&self, world: &ClientWorld, pos: BlockPos, state: BlockStateId) -> bool {
        let shape = world
            .block_state_registry()
            .collision_shape_at(state, [pos.x, pos.y, pos.z])
            .move_by_block_pos(pos);
        let Ok(player) = Shapes::create_from_aabb(self.bounding_box) else {
            return false;
        };
        Shapes::join_is_not_empty(&shape, &player, BooleanOp::And)
    }

    /// `Player.getDesiredPose`.
    fn desired_pose(&self) -> ClientEntityPose {
        if self.is_sleeping() {
            ClientEntityPose::Sleeping
        } else if self.is_swimming() {
            ClientEntityPose::Swimming
        } else if self.elytra.fall_flying {
            ClientEntityPose::FallFlying
        } else if self.is_shift_key_down() && !self.abilities.flying {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Standing
        }
    }

    pub(super) fn update_player_pose(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if !self.can_fit(level, ClientEntityPose::Swimming)? {
            return Ok(());
        }
        let desired = self.desired_pose();
        let actual =
            if self.is_spectator() || self.is_passenger() || self.can_fit(level, desired)? {
                desired
            } else if self.can_fit(level, ClientEntityPose::Crouching)? {
                ClientEntityPose::Crouching
            } else {
                ClientEntityPose::Swimming
            };
        self.set_pose(actual);
        Ok(())
    }

    pub(super) fn set_pose(&mut self, pose: ClientEntityPose) {
        if pose != self.pose {
            self.pose = pose;
            self.dimensions = pose.player_dimensions();
            self.bounding_box = self.dimensions.make_bounding_box(self.position);
        }
    }

    pub fn is_spectator(&self) -> bool {
        self.game_mode.is_spectator()
    }

    pub(super) fn is_in_water(&self) -> bool {
        self.was_touching_water
    }

    pub(super) fn is_in_lava(&self) -> bool {
        !self.first_tick && self.lava.height > 0.0
    }

    pub fn is_under_water(&self) -> bool {
        self.eyes.was_in_water && self.is_in_water()
    }

    pub(super) fn is_affected_by_fluids(&self) -> bool {
        !self.abilities.flying
    }

    pub(super) fn is_moving_slowly(&self) -> bool {
        self.crouching || self.is_visually_swimming() && !self.is_in_water()
    }

    /// `LivingEntity.isVisuallySwimming`: the swimming pose, or the glide's pose for the tick
    /// after the flag ended and before the pose follows.
    pub fn is_visually_swimming(&self) -> bool {
        self.pose == ClientEntityPose::Swimming
            || !self.elytra.fall_flying && self.pose == ClientEntityPose::FallFlying
    }
}

/// `HoneyBlock.isSlidingDown`.
pub(super) fn is_sliding_down_honey(body: &LivingEffects<'_>, pos: BlockPos) -> bool {
    if body.on_ground || body.position.y > f64::from(pos.y) + 0.9375 - 1.0e-7 {
        return false;
    }
    if body.delta.y / f64::from(0.98_f32) + 0.08 >= -0.08 {
        return false;
    }
    let dx = (f64::from(pos.x) + 0.5 - body.position.x).abs();
    let dz = (f64::from(pos.z) + 0.5 - body.position.z).abs();
    let overlap = 0.4375 + f64::from(body.width / 2.0);
    dx + 1.0e-7 > overlap || dz + 1.0e-7 > overlap
}

pub(super) fn is_block(state_id: BlockStateId, name: &str) -> bool {
    registry::block_for_state(state_id).is_some_and(|(block, _)| block.name == name)
}

/// What the player model is posed from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPose {
    pub body_rot: f32,
    pub head_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
    pub swim_amount: f32,
    pub crouching: bool,
    pub pose: ClientEntityPose,
}

/// `LocalPlayer.getWaterVision` for a time under water, none if the eyes are not in water now.
pub fn water_vision(water_vision_time: i32, eyes_in_water: bool) -> f32 {
    if !eyes_in_water {
        return 0.0;
    }
    if water_vision_time >= 600 {
        return 1.0;
    }
    let time = water_vision_time as f32;
    let a = (time / 100.0).clamp(0.0, 1.0);
    let b = if water_vision_time < 100 {
        0.0
    } else {
        ((time - 100.0) / 500.0).clamp(0.0, 1.0)
    };
    a * 0.6 + b * 0.399_999_98
}

/// What the eyes did in the ticks before: whether they were in water, and for how long they have
/// been under it (`LocalPlayer.waterVisionTime`, up to 600 ticks).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EyeHistory {
    pub(crate) was_in_water: bool,
    pub(crate) water_vision_time: i32,
}
