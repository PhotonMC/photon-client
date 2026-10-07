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

    /// `Entity.baseTick`'s part a boat has: the fluids around it push it and wash off a fall.
    pub(super) fn base_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let bounding_box = self.bounding_box();
        let found = fluid::scan(
            level,
            (bounding_box, self.position, self.dimensions.eye_height()),
            false,
        )?;
        if found.water.height > 0.0 {
            self.ride.fall_distance = 0.0;
            self.delta =
                fluid::apply_current(found.water, self.delta, WATER_FLOW_SCALE, Pushed::Other);
        }
        if found.lava.height > 0.0 {
            self.delta =
                fluid::apply_current(found.lava, self.delta, LAVA_FLOW_SCALE, Pushed::Other);
            self.ride.fall_distance *= 0.5;
        }
        Ok(())
    }

    /// `AbstractBoat.getStatus`.
    pub(super) fn status(&mut self, level: &Level<'_>) -> Status {
        if let Some(under) = self.is_underwater(level) {
            self.ride.water_level = self.bounding_box().max_y;
            under
        } else if self.check_in_water(level) {
            Status::InWater
        } else {
            let friction = self.ground_friction(level);
            if friction > 0.0 {
                self.ride.land_friction = friction;
                Status::OnLand
            } else {
                Status::InAir
            }
        }
    }

    /// `AbstractBoat.isUnderwater` (the private status check): whether the water reaches a
    /// thousandth above the box, and whether that water flows.
    pub(super) fn is_underwater(&self, level: &Level<'_>) -> Option<Status> {
        let bb = self.bounding_box();
        let max_y = bb.max_y + 0.001;
        let mut under_water = false;
        for x in floor_f64(bb.min_x)..ceil_f64(bb.max_x) {
            for y in floor_f64(bb.max_y)..ceil_f64(max_y) {
                for z in floor_f64(bb.min_z)..ceil_f64(bb.max_z) {
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.is_water()
                        && max_y < f64::from(y as f32 + level.fluid_height(pos, &info))
                    {
                        if info.fluid != Fluid::Water {
                            return Some(Status::UnderFlowingWater);
                        }
                        under_water = true;
                    }
                }
            }
        }
        under_water.then_some(Status::UnderWater)
    }

    /// `AbstractBoat.checkInWater`: whether the water reaches above the bottom of the box, and
    /// how high it stands.
    pub(super) fn check_in_water(&mut self, level: &Level<'_>) -> bool {
        let bb = self.bounding_box();
        let mut in_water = false;
        self.ride.water_level = -f64::MAX;
        for x in floor_f64(bb.min_x)..ceil_f64(bb.max_x) {
            for y in floor_f64(bb.min_y)..ceil_f64(bb.min_y + 0.001) {
                for z in floor_f64(bb.min_z)..ceil_f64(bb.max_z) {
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.is_water() {
                        let height = y as f32 + level.fluid_height(pos, &info);
                        self.ride.water_level =
                            java_math::max(f64::from(height), self.ride.water_level);
                        in_water |= bb.min_y < f64::from(height);
                    }
                }
            }
        }
        in_water
    }

    /// `AbstractBoat.getGroundFriction`: the mean friction of the blocks under the boat (not
    /// lily pads), and not a number when it rests on none.
    pub(super) fn ground_friction(&self, level: &Level<'_>) -> f32 {
        let bb = self.bounding_box();
        let probe = Aabb::new(
            bb.min_x,
            bb.min_y - 0.001,
            bb.min_z,
            bb.max_x,
            bb.min_y,
            bb.max_z,
        );
        let (x0, x1) = (floor_f64(probe.min_x) - 1, ceil_f64(probe.max_x) + 1);
        let (y0, y1) = (floor_f64(probe.min_y) - 1, ceil_f64(probe.max_y) + 1);
        let (z0, z1) = (floor_f64(probe.min_z) - 1, ceil_f64(probe.max_z) + 1);
        let boat_shape = Shapes::create_from_aabb(probe).unwrap_or_else(|_| Shapes::empty());
        let mut friction = 0.0_f32;
        let mut count = 0_i32;
        for x in x0..x1 {
            for z in z0..z1 {
                let edges = i32::from(x == x0 || x == x1 - 1) + i32::from(z == z0 || z == z1 - 1);
                if edges == 2 {
                    continue;
                }
                for y in y0..y1 {
                    if edges > 0 && y != y0 && y != y1 - 1 {
                        continue;
                    }
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.name != "minecraft:lily_pad"
                        && Shapes::join_is_not_empty(
                            &level.collision_shape(pos).move_by_block_pos(pos),
                            &boat_shape,
                            BooleanOp::And,
                        )
                    {
                        friction += info.friction;
                        count += 1;
                    }
                }
            }
        }
        friction / count as f32
    }

    /// `AbstractBoat.getWaterLevelAbove`: the height of the water's surface above the boat.
    pub(super) fn water_level_above(&self, level: &Level<'_>) -> f32 {
        let bb = self.bounding_box();
        let (min_x, max_x) = (floor_f64(bb.min_x), ceil_f64(bb.max_x));
        let (min_y, max_y) = (floor_f64(bb.max_y), ceil_f64(bb.max_y - self.ride.last_yd));
        let (min_z, max_z) = (floor_f64(bb.min_z), ceil_f64(bb.max_z));
        for y in min_y..max_y {
            let mut block_height = 0.0_f32;
            let mut full = false;
            'columns: for x in min_x..max_x {
                for z in min_z..max_z {
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.is_water() {
                        block_height = block_height.max(level.fluid_height(pos, &info));
                    }
                    if block_height >= 1.0 {
                        full = true;
                        break 'columns;
                    }
                }
            }
            if !full {
                return y as f32 + block_height;
            }
        }
        (max_y + 1) as f32
    }

    /// `AbstractBoat.floatBoat`: the pull of gravity and of the water, and how much the medium
    /// slows the boat, by its status.
    pub(super) fn float_boat(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let status = self.ride.status.unwrap_or(Status::InAir);
        if self.ride.old_status == Some(Status::InAir)
            && status != Status::InAir
            && status != Status::OnLand
        {
            // Coming down onto the water: set on its surface, if there is room.
            self.ride.water_level = self.position.y + f64::from(self.height());
            let target_y = f64::from(self.water_level_above(level) - self.height()) + 0.101;
            let moved = self
                .bounding_box()
                .move_by(0.0, target_y - self.position.y, 0.0);
            if level.no_collision(moved, self.ride.fall_distance, self.position)? {
                self.set_pos(Vec3::new(self.position.x, target_y, self.position.z));
                self.delta = self.delta.multiply_components(1.0, 0.0, 1.0);
                self.ride.last_yd = 0.0;
            }
            self.ride.status = Some(Status::InWater);
            return Ok(());
        }
        let gravity = self.gravity();
        let (vspeed, buoyancy, inv_friction) = match status {
            Status::InWater => (
                -gravity,
                (self.ride.water_level - self.position.y) / f64::from(self.height()),
                0.9_f32,
            ),
            Status::UnderFlowingWater => (-7.0e-4, 0.0, 0.9),
            Status::UnderWater => (-gravity, f64::from(0.01_f32), 0.45),
            Status::InAir => (-gravity, 0.0, 0.9),
            Status::OnLand => {
                let friction = self.ride.land_friction;
                if self.player_aboard {
                    self.ride.land_friction /= 2.0;
                }
                (-gravity, 0.0, friction)
            }
        };
        let friction = f64::from(inv_friction);
        self.delta = Vec3::new(
            self.delta.x * friction,
            self.delta.y + vspeed,
            self.delta.z * friction,
        );
        self.delta_rotation *= inv_friction;
        if buoyancy > 0.0 {
            self.delta = Vec3::new(
                self.delta.x,
                (self.delta.y + buoyancy * (GRAVITY / 0.65)) * 0.75,
                self.delta.z,
            );
        }
        Ok(())
    }

    /// `AbstractBoat.controlBoat`: the rider's keys turn the boat and push it along.
    pub(super) fn control_boat(&mut self) {
        if !self.has_passengers {
            return;
        }
        let input = self.input;
        let mut acceleration = 0.0_f32;
        if input.left {
            self.delta_rotation -= 1.0;
        }
        if input.right {
            self.delta_rotation += 1.0;
        }
        if input.right != input.left && !input.forward && !input.backward {
            acceleration += 0.005;
        }
        self.y_rot += self.delta_rotation;
        if input.forward {
            acceleration += 0.04;
        }
        if input.backward {
            acceleration -= 0.005;
        }
        let x = mth_sin_f64(f64::from(-self.y_rot * DEG_TO_RAD)) * acceleration;
        let z = mth_cos_f64(f64::from(self.y_rot * DEG_TO_RAD)) * acceleration;
        self.delta = self.delta.add_components(f64::from(x), 0.0, f64::from(z));
        self.paddles = (
            input.right && !input.left || input.forward,
            input.left && !input.right || input.forward,
        );
    }

    /// `Entity.move(SELF, ...)` for a boat that is its own authority: it collides with the
    /// blocks (it cannot step up), is stopped or slowed by what it meets, and the block it ends
    /// on sets its speed.
    pub(super) fn move_self(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let mut delta = self.delta;
        let (sx, sy, sz) = self.ride.stuck_speed_multiplier;
        let stuck = Vec3::new(sx, sy, sz);
        if stuck.length_squared() > 1.0e-7 {
            delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            self.ride.stuck_speed_multiplier = (0.0, 0.0, 0.0);
            self.delta = Vec3::ZERO;
        }
        let bb = self.bounding_box();
        let movement = if delta.length_squared() == 0.0 {
            delta
        } else {
            let colliders = level.colliders(
                bb.expand_towards_vec3(delta),
                self.ride.fall_distance,
                self.position,
            )?;
            collide_entity_movement(delta, bb, &colliders)
        };
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.ride.fall_distance != 0.0 && movement_length >= 1.0 {
                // A long fall that crosses something that forgives it (water, a cobweb).
                let distance = movement.length().min(8.0);
                let to = self
                    .position
                    .add_vec3(movement.normalized().scale(distance));
                if level.clip_hits_fall_reset(self.position, to) {
                    self.ride.fall_distance = 0.0;
                }
            }
            let from = self.position;
            let to = from.add_vec3(movement);
            self.movements.push(Movement {
                from,
                to,
                original: Some(delta),
            });
            self.set_pos(to);
        }

        let x_collision = !voxel_core::math::equal_f64(delta.x, movement.x);
        let z_collision = !voxel_core::math::equal_f64(delta.z, movement.z);
        self.ride.horizontal_collision = x_collision || z_collision;
        let moved_vertically = delta.y.abs() > 0.0;
        self.ride.vertical_collision = delta.y != movement.y;
        self.ride.vertical_collision_below = self.ride.vertical_collision && delta.y < 0.0;
        self.on_ground = self.ride.vertical_collision_below;
        let moved_box = self.bounding_box();
        level.check_supporting_block(
            (&mut self.supporting_block, &mut self.on_ground_no_blocks),
            (moved_box, self.position, self.ride.fall_distance),
            self.on_ground,
            Some(movement),
        )?;

        let effect_pos = level.on_pos(self.supporting_block, self.position, 0.2);
        let effect_info = level.info(effect_pos);
        self.check_fall_damage(level, movement.y);

        if moved_vertically && self.ride.vertical_collision || self.ride.horizontal_collision {
            self.restitute_movement_after_collisions(
                effect_info.flags & flag::SUPPRESSES_BOUNCE != 0,
                effect_info.bounce,
                (x_collision, z_collision),
                movement,
            );
        }

        let here = BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        );
        let below = level.on_pos(self.supporting_block, self.position, 0.500_001);
        let speed_factor = f64::from(level.block_speed_factor(here, below));
        self.delta = self
            .delta
            .multiply_components(speed_factor, 1.0, speed_factor);
        Ok(())
    }

    /// `AbstractBoat.checkFallDamage`: the boat remembers how fast it was falling, and, with no
    /// rider's weight on a fall, counts the distance fallen unless it is over water.
    pub(super) fn check_fall_damage(&mut self, level: &Level<'_>, ya: f64) {
        self.ride.last_yd = self.delta.y;
        // The boat is not itself a passenger.
        if self.on_ground {
            self.ride.fall_distance = 0.0;
        } else {
            let below = BlockPos::new(
                floor_f64(self.position.x),
                floor_f64(self.position.y) - 1,
                floor_f64(self.position.z),
            );
            if !level.info(below).is_water() && ya < 0.0 {
                self.ride.fall_distance -= f64::from(ya as f32);
            }
        }
    }
}
