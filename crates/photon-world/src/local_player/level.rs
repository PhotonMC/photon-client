use super::*;
use photon_core::ids::BlockStateId;

impl Level<'_> {
    pub(crate) fn state_id(&self, pos: BlockPos) -> Option<BlockStateId> {
        self.world.block_state_at(pos.x, pos.y, pos.z)
    }

    pub(crate) fn info(&self, pos: BlockPos) -> StateInfo {
        StateInfo::of(self.state_id(pos), self.world.block_state_registry.tags())
    }

    /// `Entity.checkSupportingBlock`: which block an entity on the ground
    /// stands on, from its box, position and fall distance.
    pub(crate) fn check_supporting_block(
        &self,
        (main_supporting_block, on_ground_no_blocks): (&mut Option<BlockPos>, &mut bool),
        (bounding_box, position, fall_distance): (Aabb, Vec3, f64),
        on_ground: bool,
        movement: Option<Vec3>,
    ) -> Result<(), Unsupported> {
        if on_ground {
            let bb = bounding_box;
            let test_area = Aabb::new(
                bb.min_x,
                bb.min_y - 1.0e-6,
                bb.min_z,
                bb.max_x,
                bb.min_y,
                bb.max_z,
            );
            let mut supporting = self.find_supporting_block(test_area, position, fall_distance)?;
            if supporting.is_some() || *on_ground_no_blocks {
                *main_supporting_block = supporting;
            } else if let Some(movement) = movement {
                supporting = self.find_supporting_block(
                    test_area.move_by(-movement.x, 0.0, -movement.z),
                    position,
                    fall_distance,
                )?;
                *main_supporting_block = supporting;
            }
            *on_ground_no_blocks = supporting.is_none();
        } else {
            *on_ground_no_blocks = false;
            *main_supporting_block = None;
        }
        Ok(())
    }

    pub(super) fn find_supporting_block(
        &self,
        area: Aabb,
        position: Vec3,
        fall_distance: f64,
    ) -> Result<Option<BlockPos>, Unsupported> {
        let mut main: Option<BlockPos> = None;
        let mut main_distance = f64::MAX;
        for (pos, _) in self.block_collisions(area, fall_distance)? {
            let dx = f64::from(pos.x) + 0.5 - position.x;
            let dy = f64::from(pos.y) + 0.5 - position.y;
            let dz = f64::from(pos.z) + 0.5 - position.z;
            let distance = dx * dx + dy * dy + dz * dz;
            let later =
                |current: BlockPos| (current.y, current.z, current.x) < (pos.y, pos.z, pos.x);
            if distance < main_distance || distance == main_distance && main.is_none_or(later) {
                main = Some(pos);
                main_distance = distance;
            }
        }
        Ok(main)
    }

    /// `Entity.getOnPos(offset)`.
    pub(crate) fn on_pos(
        &self,
        main_supporting_block: Option<BlockPos>,
        position: Vec3,
        offset: f32,
    ) -> BlockPos {
        let y = floor_f64(position.y - f64::from(offset));
        if let Some(pos) = main_supporting_block {
            if !(offset > 1.0e-5) {
                return pos;
            }
            let below = self.info(pos);
            let keeps_support = f64::from(offset) <= 0.5 && below.flags & flag::FENCE != 0
                || below.flags & flag::WALL != 0
                || below.flags & flag::FENCE_GATE != 0;
            if keeps_support {
                pos
            } else {
                BlockPos::new(pos.x, y, pos.z)
            }
        } else {
            BlockPos::new(floor_f64(position.x), y, floor_f64(position.z))
        }
    }

    /// `Entity.getBlockSpeedFactor`, for the block the entity is in and the
    /// one that affects its movement below.
    pub(crate) fn block_speed_factor(&self, here: BlockPos, below: BlockPos) -> f32 {
        let here = self.info(here);
        if matches!(here.name, "minecraft:water" | "minecraft:bubble_column")
            || here.speed_factor != 1.0
        {
            here.speed_factor
        } else {
            self.info(below).speed_factor
        }
    }

    /// `ClientLevel.hasChunk`, which `Level.hasChunkAt` asks: the client level answers yes for
    /// every chunk, loaded or not.
    pub(super) fn has_chunk_at(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `LiquidBlock.getCollisionShape` for an entity that can stand on lava: a source block of lava
    /// has the half-high column of `Strider.getLiquidCollisionShape` where the entity's feet are
    /// above it and no lava lies on top of it.
    pub(super) fn lava_surface_shape(
        &self,
        state: BlockStateId,
        at: [i32; 3],
        bottom: f64,
    ) -> Option<VoxelShape> {
        let info = StateInfo::of(Some(state), self.world.block_state_registry.tags());
        if info.fluid != Fluid::Lava {
            return None;
        }
        let above = self.info(BlockPos::new(at[0], at[1] + 1, at[2]));
        if above.is_lava() || bottom <= f64::from(at[1]) + 0.5 - f64::from(1.0e-5_f32) {
            return None;
        }
        Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 0.5, 1.0).ok()
    }

    pub(crate) fn collision_shape(&self, pos: BlockPos) -> VoxelShape {
        match self.state_id(pos) {
            Some(state) => self
                .world
                .block_state_registry
                .collision_shape_at(state, [pos.x, pos.y, pos.z]),
            None => Shapes::empty(),
        }
    }

    /// `getBlockCollisions` for the player, who has fallen `fall_distance`:
    /// powder snow holds it only once it falls far enough. Scaffolding is
    /// refused until its shape is ported.
    pub(crate) fn block_collisions(
        &self,
        area: Aabb,
        fall_distance: f64,
    ) -> Result<Vec<(BlockPos, VoxelShape)>, Unsupported> {
        let registry = &self.world.block_state_registry;
        let collisions = self.world.collect_block_collisions_with(area, |state, at| {
            Some(if is_block(state, "minecraft:powder_snow") {
                powder_snow_collision_shape(fall_distance)
            } else if let Some(bottom) = self.lava_stand
                && let Some(shape) = self.lava_surface_shape(state, at, bottom)
            {
                shape
            } else {
                registry.collision_shape_at(state, at)
            })
        });
        let mut out = Vec::with_capacity(collisions.len());
        for collision in collisions {
            let info = self.info(collision.position);
            if info.name == "minecraft:scaffolding" {
                return unsupported(format!("entity-dependent collision shape of {}", info.name));
            }
            out.push((collision.position, collision.shape));
        }
        Ok(out)
    }

    /// `Entity.collectColliders`: the world border's shape if `source` stands close inside it,
    /// then the blocks'.
    pub(crate) fn colliders(
        &self,
        area: Aabb,
        fall_distance: f64,
        source: Vec3,
    ) -> Result<Vec<VoxelShape>, Unsupported> {
        let mut out: Vec<VoxelShape> = self
            .world
            .border_collider(source, area)
            .into_iter()
            .collect();
        out.extend(
            self.block_collisions(area, fall_distance)?
                .into_iter()
                .map(|(_, shape)| shape),
        );
        Ok(out)
    }

    /// `CollisionGetter.noCollision(source, area)`: no block, and not the world border.
    pub(crate) fn no_collision(
        &self,
        area: Aabb,
        fall_distance: f64,
        source: Vec3,
    ) -> Result<bool, Unsupported> {
        Ok(self.block_collisions(area, fall_distance)?.is_empty()
            && self.world.no_border_collision(
                Some(crate::collision::ClientWorldCollisionSource { position: source }),
                area,
            ))
    }

    pub(super) fn collides_with_suffocating_block(&self, area: Aabb) -> bool {
        let registry = &self.world.block_state_registry;
        !self
            .world
            .collect_block_collisions_with(area, |state, at| {
                crate::block::state_or_air(state)
                    .suffocating
                    .then(|| registry.collision_shape_at(state, at))
            })
            .is_empty()
    }

    /// `FluidState.getHeight`: a full block under the same fluid.
    pub(crate) fn fluid_height(&self, pos: BlockPos, info: &StateInfo) -> f32 {
        let above = self.info(BlockPos::new(pos.x, pos.y + 1, pos.z));
        let same_above =
            (info.is_water() && above.is_water()) || (info.is_lava() && above.is_lava());
        if same_above { 1.0 } else { info.own_height }
    }

    /// `FlowingFluid.isSolidFace` for a horizontal direction: not for the
    /// same fluid, never for ice, otherwise whether the block's face on that
    /// side (the side named by the direction itself) is fully sturdy.
    pub(super) fn is_solid_face(
        &self,
        pos: BlockPos,
        info: &StateInfo,
        direction: Direction,
    ) -> bool {
        let there = self.info(pos);
        let same_fluid =
            (info.is_water() && there.is_water()) || (info.is_lava() && there.is_lava());
        if same_fluid {
            return false;
        }
        let state = crate::block::state_or_air(
            self.state_id(pos)
                .unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID),
        );
        !state.block().class.is(BlockClass::IceBlock) && state.is_face_sturdy(direction)
    }

    /// `FlowingFluid.getFlow`.
    pub(crate) fn fluid_flow(&self, pos: BlockPos, info: &StateInfo) -> Result<Vec3, Unsupported> {
        let same = |other: &StateInfo| {
            other.fluid == Fluid::Empty
                || (info.is_water() && other.is_water())
                || (info.is_lava() && other.is_lava())
        };
        let mut flow_x = 0.0_f64;
        let mut flow_z = 0.0_f64;
        // Direction.Plane.HORIZONTAL: north, east, south, west.
        for (step_x, step_z) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let neighbour_pos = BlockPos::new(pos.x + step_x, pos.y, pos.z + step_z);
            let neighbour = self.info(neighbour_pos);
            if !same(&neighbour) {
                continue;
            }
            let mut neighbour_height = neighbour.own_height;
            let mut distance = 0.0_f32;
            if neighbour_height == 0.0 {
                if !self.blocks_motion(neighbour_pos, &neighbour)? {
                    let below_pos =
                        BlockPos::new(neighbour_pos.x, neighbour_pos.y - 1, neighbour_pos.z);
                    let below = self.info(below_pos);
                    if same(&below) {
                        neighbour_height = below.own_height;
                        if neighbour_height > 0.0 {
                            distance = info.own_height - (neighbour_height - 0.888_888_9_f32);
                        }
                    }
                }
            } else if neighbour_height > 0.0 {
                distance = info.own_height - neighbour_height;
            }
            if distance != 0.0 {
                flow_x += f64::from(step_x) * f64::from(distance);
                flow_z += f64::from(step_z) * f64::from(distance);
            }
        }
        let mut flow = Vec3::new(flow_x, 0.0, flow_z);
        // A falling fluid beside a solid face is pulled down hard.
        if info.falling {
            // North, east, south, west.
            for (step_x, step_z, direction) in [
                (0, -1, Direction::North),
                (1, 0, Direction::East),
                (0, 1, Direction::South),
                (-1, 0, Direction::West),
            ] {
                let side = BlockPos::new(pos.x + step_x, pos.y, pos.z + step_z);
                let above = BlockPos::new(side.x, side.y + 1, side.z);
                if self.is_solid_face(side, info, direction)
                    || self.is_solid_face(above, info, direction)
                {
                    let normal = flow.normalized();
                    flow = Vec3::new(normal.x, normal.y - 6.0, normal.z);
                    break;
                }
            }
        }
        Ok(flow.normalized())
    }

    /// Whether `clip` with `ClipContext.Block.FALLDAMAGE_RESETTING` and
    /// `ClipContext.Fluid.WATER` hits anything between the two points.
    pub(crate) fn clip_hits_fall_reset(&self, from: Vec3, to: Vec3) -> bool {
        if from.x == to.x && from.y == to.y && from.z == to.z {
            return false;
        }
        let lerp = |delta: f64, start: f64, end: f64| start + delta * (end - start);
        let to_x = lerp(-1.0e-7, to.x, from.x);
        let to_y = lerp(-1.0e-7, to.y, from.y);
        let to_z = lerp(-1.0e-7, to.z, from.z);
        let from_x = lerp(-1.0e-7, from.x, to.x);
        let from_y = lerp(-1.0e-7, from.y, to.y);
        let from_z = lerp(-1.0e-7, from.z, to.z);
        let mut block_x = floor_f64(from_x);
        let mut block_y = floor_f64(from_y);
        let mut block_z = floor_f64(from_z);
        if self.fall_reset_hit(BlockPos::new(block_x, block_y, block_z), from, to) {
            return true;
        }
        let dx = to_x - from_x;
        let dy = to_y - from_y;
        let dz = to_z - from_z;
        let sign = |value: f64| {
            if value > 0.0 {
                1
            } else if value < 0.0 {
                -1
            } else {
                0
            }
        };
        let frac = |value: f64| value - photon_core::math::lfloor_f64(value) as f64;
        let (sign_x, sign_y, sign_z) = (sign(dx), sign(dy), sign(dz));
        let t_delta_x = if sign_x == 0 {
            f64::MAX
        } else {
            f64::from(sign_x) / dx
        };
        let t_delta_y = if sign_y == 0 {
            f64::MAX
        } else {
            f64::from(sign_y) / dy
        };
        let t_delta_z = if sign_z == 0 {
            f64::MAX
        } else {
            f64::from(sign_z) / dz
        };
        let mut t_x = t_delta_x
            * if sign_x > 0 {
                1.0 - frac(from_x)
            } else {
                frac(from_x)
            };
        let mut t_y = t_delta_y
            * if sign_y > 0 {
                1.0 - frac(from_y)
            } else {
                frac(from_y)
            };
        let mut t_z = t_delta_z
            * if sign_z > 0 {
                1.0 - frac(from_z)
            } else {
                frac(from_z)
            };
        while t_x <= 1.0 || t_y <= 1.0 || t_z <= 1.0 {
            if t_x < t_y {
                if t_x < t_z {
                    block_x += sign_x;
                    t_x += t_delta_x;
                } else {
                    block_z += sign_z;
                    t_z += t_delta_z;
                }
            } else if t_y < t_z {
                block_y += sign_y;
                t_y += t_delta_y;
            } else {
                block_z += sign_z;
                t_z += t_delta_z;
            }
            if self.fall_reset_hit(BlockPos::new(block_x, block_y, block_z), from, to) {
                return true;
            }
        }
        false
    }

    pub(super) fn fall_reset_hit(&self, pos: BlockPos, from: Vec3, to: Vec3) -> bool {
        let info = self.info(pos);
        let resets = info.flags & flag::CLIMBABLE != 0
            || matches!(info.name, "minecraft:sweet_berry_bush" | "minecraft:cobweb");
        if resets && Shapes::block().clip(from, to, pos).is_some() {
            return true;
        }
        if info.is_water() {
            let height = f64::from(self.fluid_height(pos, &info));
            let shape = Shapes::box_shape(0.0, 0.0, 0.0, 1.0, height, 1.0)
                .expect("a fluid's shape is a normalized box");
            return shape.clip(from, to, pos).is_some();
        }
        false
    }

    /// `BlockState.blocksMotion`.
    pub(super) fn blocks_motion(
        &self,
        pos: BlockPos,
        _info: &StateInfo,
    ) -> Result<bool, Unsupported> {
        Ok(self
            .state_id(pos)
            .is_some_and(|state| crate::block::state_or_air(state).blocks_motion))
    }
}

/// The level as the tick reads it.
#[derive(Clone, Copy)]
pub(crate) struct Level<'a> {
    pub(crate) world: &'a ClientWorld,
    /// The height of the feet of an entity that can stand on lava (`EntityCollisionContext` of a
    /// strider): the lava's source blocks below it then hold it up.
    pub(crate) lava_stand: Option<f64>,
}

impl<'a> Level<'a> {
    pub(crate) fn new(world: &'a ClientWorld) -> Self {
        Self {
            world,
            lava_stand: None,
        }
    }

    /// The level as seen by an entity that can stand on lava, with its feet at `bottom`.
    pub(crate) fn standing_on_lava_at(self, bottom: f64) -> Self {
        Self {
            lava_stand: Some(bottom),
            ..self
        }
    }
}
