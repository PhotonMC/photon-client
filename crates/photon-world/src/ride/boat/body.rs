use super::*;

impl BoatBody {
    pub(crate) fn new(
        position: Vec3,
        old_position: Vec3,
        dimensions: ClientEntityDimensions,
        ride: RideState,
    ) -> Self {
        Self {
            position,
            old_position,
            delta: Vec3::ZERO,
            y_rot: 0.0,
            delta_rotation: 0.0,
            on_ground: false,
            supporting_block: None,
            on_ground_no_blocks: false,
            dimensions,
            no_gravity: false,
            input: BoatInput::EMPTY,
            player_aboard: false,
            has_passengers: false,
            paddles: (false, false),
            ride,
            movements: Vec::new(),
        }
    }

    pub(crate) fn bounding_box(&self) -> Aabb {
        self.dimensions.make_bounding_box(self.position)
    }

    pub(super) fn height(&self) -> f32 {
        self.dimensions.height()
    }

    /// `Entity.setPos`.
    pub(super) fn set_pos(&mut self, position: Vec3) {
        self.position = position;
    }

    /// `Entity.getGravity`.
    pub(super) fn gravity(&self) -> f64 {
        if self.no_gravity { 0.0 } else { GRAVITY }
    }

    /// `AbstractBoat.tick` for a boat the local client is the authority of. Returns the paddles
    /// the client tells the server about (`ServerboundPaddleBoatPacket`).
    pub(crate) fn tick(&mut self, level: &Level<'_>) -> Result<(bool, bool), Unsupported> {
        self.ride.old_status = self.ride.status;
        let status = self.status(level);
        self.ride.status = Some(status);
        self.base_tick(level)?;
        if !self.player_aboard {
            self.paddles = (false, false);
        }
        self.float_boat(level)?;
        self.control_boat();
        let paddles = self.paddles;
        self.move_self(level)?;
        // `applyEffectsFromBlocks`, which the boat asks for twice: the second finds no movement
        // left to replay and takes the tick's whole way as one.
        self.apply_effects_from_blocks(level);
        self.apply_effects_from_blocks(level);
        Ok(paddles)
    }
}

/// The boat as one tick reads and changes it.
#[derive(Debug, Clone)]
pub(crate) struct BoatBody {
    pub(crate) position: Vec3,
    /// Where the boat was at the start of the tick (`Entity.oldPosition`).
    pub(crate) old_position: Vec3,
    pub(crate) delta: Vec3,
    pub(crate) y_rot: f32,
    pub(crate) delta_rotation: f32,
    pub(crate) on_ground: bool,
    pub(crate) supporting_block: Option<BlockPos>,
    pub(crate) on_ground_no_blocks: bool,
    pub(crate) dimensions: ClientEntityDimensions,
    pub(crate) no_gravity: bool,
    pub(crate) input: BoatInput,
    /// Whether the first passenger is a player, and whether there is a passenger at all.
    pub(crate) player_aboard: bool,
    pub(crate) has_passengers: bool,
    pub(crate) paddles: (bool, bool),
    pub(crate) ride: RideState,
    pub(super) movements: Vec<Movement>,
}

impl InsideBody for BoatBody {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> VoxelShape {
        crate::local_player::powder_snow_collision_shape(self.ride.fall_distance)
    }

    /// What a block the boat is inside does to it: honey slides it, a cobweb or powder snow
    /// holds it (a boat is not a living entity), and a bubble column carries it when it is not
    /// at the column's top.
    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let pos = visit.pos;
        match visit.info.name {
            "minecraft:honey_block" if self.is_sliding_down_honey(pos) => {
                let old_delta_y = |delta_y: f64| delta_y / f64::from(0.98_f32) + 0.08;
                let new_delta_y = |delta_y: f64| (delta_y - 0.08) * f64::from(0.98_f32);
                let delta = self.delta;
                self.delta = if old_delta_y(delta.y) < -0.13 {
                    let factor = -0.05 / old_delta_y(delta.y);
                    Vec3::new(delta.x * factor, new_delta_y(-0.05), delta.z * factor)
                } else {
                    Vec3::new(delta.x, new_delta_y(-0.05), delta.z)
                };
                self.ride.fall_distance = 0.0;
            }
            "minecraft:cobweb" => {
                self.make_stuck_in_block(Vec3::new(0.25, f64::from(0.05_f32), 0.25));
            }
            "minecraft:powder_snow" => {
                self.make_stuck_in_block(Vec3::new(f64::from(0.9_f32), 1.5, f64::from(0.9_f32)));
            }
            "minecraft:bubble_column" if visit.precise => {
                let above = level.info(BlockPos::new(pos.x, pos.y + 1, pos.z));
                let nothing_above = visit.state.is_none() || {
                    let above_state = level.world.block_state_at(pos.x, pos.y + 1, pos.z);
                    above_state.is_none_or(|state| {
                        level
                            .world
                            .block_state_registry
                            .collision_shape(state)
                            .is_empty()
                            && above.fluid == Fluid::Empty
                    })
                };
                // `AbstractBoat.onAboveBubbleColumn` only makes sound and spray on a client.
                if !nothing_above {
                    let drag_down = visit.state.is_some_and(|state| {
                        crate::local_player::property(Some(state), "drag") == Some("true")
                    });
                    let y = if drag_down {
                        (self.delta.y - 0.03).max(-0.3)
                    } else {
                        (self.delta.y + 0.06).min(0.7)
                    };
                    self.delta = Vec3::new(self.delta.x, y, self.delta.z);
                    self.ride.fall_distance = 0.0;
                }
            }
            _ => {}
        }
    }
}

impl BoatBody {
    /// `HoneyBlock.isSlidingDown`.
    pub(super) fn is_sliding_down_honey(&self, pos: BlockPos) -> bool {
        if self.on_ground || self.position.y > f64::from(pos.y) + 0.9375 - 1.0e-7 {
            return false;
        }
        if self.delta.y / f64::from(0.98_f32) + 0.08 >= -0.08 {
            return false;
        }
        let dx = (f64::from(pos.x) + 0.5 - self.position.x).abs();
        let dz = (f64::from(pos.z) + 0.5 - self.position.z).abs();
        let overlap = 0.4375 + f64::from(self.dimensions.width() / 2.0);
        dx + 1.0e-7 > overlap || dz + 1.0e-7 > overlap
    }
}
