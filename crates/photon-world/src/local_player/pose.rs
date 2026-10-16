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

    /// Whether the eyes are inside water, as the last fluid update found.
    pub fn is_eye_in_water(&self) -> bool {
        self.water.eyes_inside
    }

    /// `LocalPlayer.isShiftKeyDown`: the input, not the synced flag.
    pub fn is_shift_key_down(&self) -> bool {
        self.input.key_presses.shift
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    pub fn is_flying(&self) -> bool {
        self.abilities.flying
    }

    /// `Entity.isColliding`: whether a block's collision shape at `pos`
    /// overlaps the player.
    pub fn is_colliding(&self, world: &ClientWorld, pos: BlockPos, state: i32) -> bool {
        let shape = world
            .block_state_registry()
            .collision_shape_at(state, [pos.x, pos.y, pos.z])
            .move_by_block_pos(pos);
        let Ok(player) = Shapes::create_from_aabb(self.bounding_box) else {
            return false;
        };
        Shapes::join_is_not_empty(&shape, &player, BooleanOp::And)
    }

    pub(super) fn update_player_pose(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if !self.can_fit(level, ClientEntityPose::Swimming)? {
            return Ok(());
        }
        let desired = if self.is_swimming() {
            ClientEntityPose::Swimming
        } else if self.is_shift_key_down() && !self.abilities.flying {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Standing
        };
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

    pub(super) fn is_spectator(&self) -> bool {
        self.game_mode.is_spectator()
    }

    pub(super) fn is_in_water(&self) -> bool {
        self.was_touching_water
    }

    pub(super) fn is_in_lava(&self) -> bool {
        !self.first_tick && self.lava.height > 0.0
    }

    pub fn is_under_water(&self) -> bool {
        self.was_eye_in_water && self.is_in_water()
    }

    pub(super) fn is_affected_by_fluids(&self) -> bool {
        !self.abilities.flying
    }

    pub(super) fn is_moving_slowly(&self) -> bool {
        self.crouching || self.pose == ClientEntityPose::Swimming && !self.is_in_water()
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

pub(super) fn is_block(state_id: i32, name: &str) -> bool {
    registry::block_for_state(BlockStateId(state_id)).is_some_and(|(block, _)| block.name == name)
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
