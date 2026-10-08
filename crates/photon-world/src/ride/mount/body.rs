use super::*;

impl MountBody {
    /// `LivingEntity.checkFallDamage` on a client: the fluids are looked at again, and a fall adds
    /// up until the ground ends it.
    pub(super) fn check_fall_damage(
        &mut self,
        level: &Level<'_>,
        ya: f64,
    ) -> Result<(), Unsupported> {
        // `Strider.checkFallDamage`: in lava a fall is forgiven.
        if self.kind == MountKind::Strider && self.is_in_lava() {
            self.state.fall_distance = 0.0;
            return Ok(());
        }
        if !self.is_in_water() {
            self.update_fluid_interaction(level)?;
        }
        if !self.is_in_water() && ya < 0.0 {
            self.state.fall_distance -= f64::from(ya as f32);
        }
        if self.on_ground {
            self.state.fall_distance = 0.0;
        }
        Ok(())
    }

    /// `Entity.applyEffectsFromBlocks`.
    pub(super) fn apply_effects_from_blocks(&mut self, level: &Level<'_>) {
        let mut movements = std::mem::take(&mut self.movements);
        match movements.last() {
            None => movements.push(Movement {
                from: self.old_position,
                to: self.position,
                original: None,
            }),
            Some(last)
                if last.to.distance_to_sqr(self.position) > f64::from(9.999_999_4e-11_f32) =>
            {
                let from = last.to;
                movements.push(Movement {
                    from,
                    to: self.position,
                    original: None,
                });
            }
            Some(_) => {}
        }
        if self.on_ground {
            let effect_pos = self.get_on_pos(level, 0.2);
            if level.info(effect_pos).name == "minecraft:slime_block" {
                let abs_delta_y = self.delta.y.abs();
                if abs_delta_y < 0.1 {
                    let scale = 0.4 + abs_delta_y * 0.2;
                    self.delta = self.delta.multiply_components(scale, 1.0, scale);
                }
            }
        }
        let _ = walk_inside_blocks(self, level, &movements);
    }

    pub(crate) fn new(
        kind: MountKind,
        position: Vec3,
        old_position: Vec3,
        dimensions: ClientEntityDimensions,
        attributes: Attributes,
        state: MountState,
    ) -> Self {
        Self {
            kind,
            position,
            old_position,
            delta: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            on_ground: false,
            supporting_block: None,
            on_ground_no_blocks: false,
            dimensions,
            attributes,
            state,
            standing: false,
            allow_stand_sliding: false,
            floats_when_ridden: false,
            controller: Controller::default(),
            camel: CamelRide::default(),
            water: FluidTracker::default(),
            lava: FluidTracker::default(),
            movements: Vec::new(),
        }
    }

    pub(super) fn bounding_box(&self) -> Aabb {
        self.dimensions.make_bounding_box(self.position)
    }

    pub(super) fn set_pos(&mut self, position: Vec3) {
        self.position = position;
    }

    pub(super) fn is_in_water(&self) -> bool {
        self.water.height > 0.0
    }

    pub(super) fn is_in_lava(&self) -> bool {
        self.lava.height > 0.0
    }

    /// `LivingEntity.getEffectiveGravity`: the attribute, none without gravity.
    pub(super) fn gravity(&self) -> f64 {
        self.attributes.gravity
    }

    /// `LivingEntity.tick` for a saddled horse the local player rides: the fluids around it, then
    /// `aiStep`, which hands the mount to its rider's keys (`travelRidden`) and applies what the
    /// blocks it passed through do to it.
    pub(crate) fn tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.update_fluid_interaction(level)?;
        self.state.lava_height = self.lava.height;
        self.state.in_liquid = self.is_in_water() || self.is_in_lava();
        // `LivingEntity.aiStep`: a small motion is none, for a mob on each axis.
        self.state.no_jump_delay = (self.state.no_jump_delay - 1).max(0);
        let delta = self.delta;
        self.delta = Vec3::new(
            if delta.x.abs() < 0.003 { 0.0 } else { delta.x },
            if delta.y.abs() < 0.003 { 0.0 } else { delta.y },
            if delta.z.abs() < 0.003 { 0.0 } else { delta.z },
        );
        // Nothing but the rider asks it to jump, so the jump wait stays spent.
        self.state.no_jump_delay = 0;
        self.travel_ridden(level)?;
        self.apply_effects_from_blocks(level);
        if self.kind == MountKind::Strider {
            self.float_strider(level);
        }
        Ok(())
    }

    /// The level as this mount's collisions see it: a strider's feet stand on lava.
    pub(super) fn seen_by<'a>(&self, level: &Level<'a>) -> Level<'a> {
        if self.kind == MountKind::Strider {
            level.standing_on_lava_at(self.position.y)
        } else {
            *level
        }
    }

    pub(super) fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }

    pub(super) fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.supporting_block, self.position, offset)
    }
}

/// The mount as one tick reads and changes it.
#[derive(Debug, Clone)]
pub(crate) struct MountBody {
    pub(crate) kind: MountKind,
    pub(crate) position: Vec3,
    pub(crate) old_position: Vec3,
    pub(crate) delta: Vec3,
    pub(crate) y_rot: f32,
    pub(crate) x_rot: f32,
    pub(crate) on_ground: bool,
    pub(crate) supporting_block: Option<BlockPos>,
    pub(crate) on_ground_no_blocks: bool,
    pub(crate) dimensions: ClientEntityDimensions,
    pub(crate) attributes: Attributes,
    pub(crate) state: MountState,
    /// Standing (reared) and eating, which hold a saddled horse where it is.
    pub(crate) standing: bool,
    /// `AbstractHorse.allowStandSliding`.
    pub(crate) allow_stand_sliding: bool,
    /// Whether the type floats while ridden (`EntityTypeTags.CAN_FLOAT_WHILE_RIDDEN`).
    pub(crate) floats_when_ridden: bool,
    pub(crate) controller: Controller,
    /// What a camel's pose and dash make of this tick.
    pub(crate) camel: CamelRide,
    pub(super) water: FluidTracker,
    pub(super) lava: FluidTracker,
    pub(super) movements: Vec<Movement>,
}

impl InsideBody for MountBody {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> photon_core::shape::VoxelShape {
        crate::local_player::powder_snow_collision_shape(self.state.fall_distance)
    }

    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let (sx, sy, sz) = self.state.stuck_speed_multiplier;
        let mut stuck = Vec3::new(sx, sy, sz);
        let block_pos = self.block_position();
        living_inside_effect(
            level,
            visit,
            LivingEffects {
                delta: &mut self.delta,
                fall_distance: &mut self.state.fall_distance,
                stuck: &mut stuck,
                position: self.position,
                on_ground: self.on_ground,
                width: self.dimensions.width(),
                block_pos,
            },
        );
        self.state.stuck_speed_multiplier = (stuck.x, stuck.y, stuck.z);
    }
}
