//! A mount the local player rides: the saddled horse family, as a client that is its authority
//! ticks it (`LivingEntity.aiStep` into `travelRidden`: the rider's keys steer it, and its own
//! attributes decide how fast and how high it goes). The movement is `LivingEntity`'s; the
//! player's own port of it is `LocalPlayer`'s.

use voxel_core::aabb::Aabb;
use voxel_core::binary::BlockPos;
use voxel_core::java_float::{JavaF32, JavaF64};
use voxel_core::math::{Vec3, ceil_f64, equal_f64, floor_f64, mth_cos_f64, mth_sin_f64};
use voxel_data::block::definition::Fluid;

use crate::ClientWorld;
use crate::collision::{collide_entity_movement, select_step_up_movement};
use crate::entity::{ClientEntityDimensions, Support};
use crate::fluid::{self, FluidTracker, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use crate::java_math;
use crate::local_player::{
    InsideBody, InsideVisit, Level, LivingEffects, Movement, Unsupported,
    compute_modified_friction, flag, living_inside_effect, property, walk_inside_blocks,
};

/// `(float) (Math.PI / 180.0)`.
const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;

/// Which mount it is, for the parts of its tick that differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
    /// A saddled horse, donkey, mule or undead horse.
    Horse,
    /// A saddled pig steered with a carrot on a stick.
    Pig,
    /// A saddled strider steered with a warped fungus on a stick, which walks on lava.
    Strider,
    /// A saddled camel (or camel husk), a horse that dashes and sits.
    Camel,
}

/// What a ridden mount keeps from one tick to the next beyond its place and motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MountState {
    pub fall_distance: f64,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    /// `Entity.stuckSpeedMultiplier`, spent by the next move.
    pub stuck_speed_multiplier: (f64, f64, f64),
    pub no_jump_delay: i32,
    /// `AbstractHorse.playerJumpPendingScale`: how hard the rider asked to jump, until the horse
    /// is on the ground to do it.
    pub jump_pending_scale: f32,
    /// How deep the lava it stands in reached at the last look (`getFluidHeight(LAVA)`), which a
    /// strider's warmth reads the next tick.
    pub lava_height: f64,
    /// Whether it was in water or lava at the last look (`Entity.isInLiquid`).
    pub in_liquid: bool,
}

/// The values are finite, so the comparison is a total one.
impl Eq for MountState {}

impl Default for MountState {
    fn default() -> Self {
        Self {
            fall_distance: 0.0,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            stuck_speed_multiplier: (0.0, 0.0, 0.0),
            no_jump_delay: 0,
            jump_pending_scale: 0.0,
            lava_height: 0.0,
            in_liquid: false,
        }
    }
}

/// The player who steers the mount, as the mount's tick reads it: the keys' effect on the player's
/// own movement input (`xxa`, `zza`) and where the player looks.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controller {
    pub xxa: f32,
    pub zza: f32,
    pub y_rot: f32,
    pub x_rot: f32,
    /// Whether a hand of the player holds a carrot on a stick, which steers a saddled pig.
    pub holds_carrot_on_a_stick: bool,
    /// Likewise for a warped fungus on a stick, which steers a saddled strider.
    pub holds_warped_fungus_on_a_stick: bool,
    /// `Entity.isSprinting`, which a camel's speed reads.
    pub sprinting: bool,
}

/// The values are finite, so the comparison is a total one.
impl Eq for Controller {}

/// The attributes the mount's movement reads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Attributes {
    pub(crate) speed: f32,
    /// The movement speed attribute itself, which a camel's dash reads.
    pub(crate) base_speed: f64,
    pub(crate) gravity: f64,
    pub(crate) step_height: f32,
    pub(crate) jump_strength: f32,
    pub(crate) friction_modifier: f32,
    pub(crate) air_drag_modifier: f32,
    pub(crate) water_movement_efficiency: f32,
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
    water: FluidTracker,
    lava: FluidTracker,
    movements: Vec<Movement>,
}

impl InsideBody for MountBody {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> voxel_core::shape::VoxelShape {
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

/// `LivingEntity.getFluidFallingAdjustedMovement`: gravity's share in the fluid.
fn fluid_falling_adjusted_movement(base_gravity: f64, is_falling: bool, movement: Vec3) -> Vec3 {
    if base_gravity != 0.0 {
        let y = if is_falling
            && (movement.y - 0.005).abs() >= 0.003
            && (movement.y - base_gravity / 16.0).abs() < 0.003
        {
            -0.003
        } else {
            movement.y - base_gravity / 16.0
        };
        Vec3::new(movement.x, y, movement.z)
    } else {
        movement
    }
}

/// `Level.containsAnyLiquid`.
fn contains_any_liquid(level: &Level<'_>, area: Aabb) -> bool {
    for x in floor_f64(area.min_x)..ceil_f64(area.max_x) {
        for y in floor_f64(area.min_y)..ceil_f64(area.max_y) {
            for z in floor_f64(area.min_z)..ceil_f64(area.max_z) {
                if level.info(BlockPos::new(x, y, z)).fluid != Fluid::Empty {
                    return true;
                }
            }
        }
    }
    false
}

/// Whether an entity type is one of the horses a saddle makes rideable (`AbstractHorse` that is
/// not a llama).
pub(crate) fn is_horse_type(type_id: i32) -> bool {
    [
        "minecraft:horse",
        "minecraft:donkey",
        "minecraft:mule",
        "minecraft:skeleton_horse",
        "minecraft:zombie_horse",
        "minecraft:camel",
        "minecraft:camel_husk",
    ]
    .iter()
    .any(|name| voxel_data::entity_type::id(name) == Some(type_id))
}

impl ClientWorld {
    /// Which mount the local player drives this entity as, if it does: a saddled horse it sits
    /// in the first seat of, or a saddled pig it steers with a carrot on a stick.
    pub(crate) fn mount_kind_of(&self, entity: &crate::entity::ClientEntity) -> Option<MountKind> {
        let player = self.local_player.as_ref()?;
        if entity.passenger_ids.first() != Some(&player.id) {
            return None;
        }
        if entity.is_controllable_horse() {
            Some(if entity.is_camel() {
                MountKind::Camel
            } else {
                MountKind::Horse
            })
        } else if entity.is_saddled_pig() && self.rider_controls.holds_carrot_on_a_stick {
            Some(MountKind::Pig)
        } else if entity.is_saddled_strider() && self.rider_controls.holds_warped_fungus_on_a_stick
        {
            Some(MountKind::Strider)
        } else {
            None
        }
    }

    /// What the rider's keys make of the mount's input, as the next tick of the mount reads them.
    pub fn set_rider_controls(&mut self, controls: Controller) {
        self.rider_controls = controls;
    }

    /// `AbstractHorse.onPlayerJump`: the rider let go of the jump key after charging a jump.
    pub fn on_player_jump(&mut self, vehicle_id: i32, jump_amount: i32) {
        let Some(horse) = self
            .entities
            .get_mut(&vehicle_id)
            .filter(|entity| entity.is_controllable_horse())
        else {
            return;
        };
        let camel = horse.is_camel();
        let on_ground = horse.on_ground;
        let state = horse
            .kind
            .horse_or_insert_with(|| Box::new(crate::horse::HorseState::new()));
        // `Camel.onPlayerJump`: only when it has rested from its last dash and stands on the ground.
        if camel && !(state.camel.dash_cooldown() <= 0 && on_ground) {
            return;
        }
        let amount = if jump_amount < 0 {
            0
        } else {
            state.set_allow_stand_sliding(true);
            // A camel does not rear (`Camel.canPerformRearing`).
            if !camel {
                state.stand_if_possible();
            }
            jump_amount
        };
        // `PlayerRideableJumping.getPlayerJumpPendingScale`.
        horse.mount_ride.jump_pending_scale = if amount >= 90 {
            1.0
        } else {
            0.4 + 0.4 * amount as f32 / 90.0
        };
    }

    /// `LivingEntity.tick` for the saddled horse the local player rides.
    pub(crate) fn tick_ridden_mount(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let Some(pick) = voxel_data::entity_type::pick(entity.type_id) else {
            return;
        };
        let (width, height, eye_height) = pick.size;
        // A camel that sits is a good deal lower (`Camel.getDefaultDimensions`).
        let (height, eye_height) =
            if entity.is_camel() && entity.pose() == crate::entity::ClientEntityPose::Sitting {
                (height - 1.43, 0.845)
            } else {
                (height, eye_height)
            };
        let dimensions = ClientEntityDimensions::scalable(width, height)
            .with_eye_height(eye_height)
            .scale(entity.scale(), entity.scale());
        let Some(kind) = self.mount_kind_of(entity) else {
            return;
        };
        if kind == MountKind::Strider {
            // `Strider.tick`: it takes the cold or the warmth before it moves.
            let suffocating = strider_is_suffocating(self, entity);
            if let Some(strider) = self.entities.get_mut(&entity_id) {
                strider.set_strider_suffocating(suffocating);
            }
        }
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let attributes = mount_attributes(entity, kind);
        let (x, y, z) = entity.position();
        let (ox, oy, oz) = entity.previous_position();
        let mut body = MountBody::new(
            kind,
            Vec3::new(x, y, z),
            Vec3::new(ox, oy, oz),
            dimensions,
            attributes,
            entity.mount_ride,
        );
        let (mx, my, mz) = entity.motion();
        body.delta = Vec3::new(mx, my, mz);
        body.y_rot = entity.y_rot.get();
        body.x_rot = entity.x_rot.get();
        body.on_ground = entity.on_ground;
        body.supporting_block = entity.tracking.support.main_block;
        body.on_ground_no_blocks = entity.tracking.support.no_blocks;
        body.standing = entity.horse().is_some_and(|horse| horse.is_standing());
        body.allow_stand_sliding = entity
            .horse()
            .is_some_and(|horse| horse.allow_stand_sliding());
        body.floats_when_ridden = matches!(kind, MountKind::Horse | MountKind::Camel);
        body.controller = self.rider_controls;
        if kind == MountKind::Camel {
            let game_time = self.game_time as i64;
            if let Some(horse) = entity.horse() {
                body.camel = CamelRide {
                    sitting: horse.camel.sitting(),
                    refuses_to_move: horse.camel.refuses_to_move(game_time),
                    in_pose_transition: horse.camel.in_pose_transition(game_time),
                    cooldown: horse.camel.dash_cooldown(),
                    ..CamelRide::default()
                };
            }
        }
        if body.tick(&Level::new(self)).is_err() {
            return;
        }
        let game_time = self.game_time as i64;
        let in_water = body.is_in_water();
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the mount was read above");
        entity.set_position((body.position.x, body.position.y, body.position.z));
        entity.motion.x = JavaF64::new(body.delta.x);
        entity.motion.y = JavaF64::new(body.delta.y);
        entity.motion.z = JavaF64::new(body.delta.z);
        // `tickRidden`: it faces where the rider does, body and head too.
        entity.old_y_rot = JavaF32::new(body.y_rot);
        entity.y_rot = JavaF32::new(body.y_rot);
        entity.x_rot = JavaF32::new(body.x_rot);
        entity.y_body_rot = JavaF32::new(body.y_rot);
        entity.y_head_rot = JavaF32::new(body.y_rot);
        entity.on_ground = body.on_ground;
        entity.tracking.support = Support {
            main_block: body.supporting_block,
            no_blocks: body.on_ground_no_blocks,
        };
        entity.mount_ride = body.state;
        if kind == MountKind::Camel {
            write_back_camel(entity, &body.camel, in_water, game_time);
        }
    }
}

/// The attributes a ridden mount moves by, read from what the server sent.
fn mount_attributes(entity: &crate::entity::ClientEntity, kind: MountKind) -> Attributes {
    let attribute = |name: &str, default: f64| entity.attribute_value(name).unwrap_or(default);
    Attributes {
        // `getRiddenSpeed`: a pig's is its speed times 0.225 (and its boost, which is one here).
        base_speed: attribute("minecraft:movement_speed", 0.7),
        speed: match kind {
            MountKind::Horse => attribute("minecraft:movement_speed", 0.7) as f32,
            // `Camel.getRiddenSpeed`: the attribute, and the sprint's bonus the body adds.
            MountKind::Camel => attribute("minecraft:movement_speed", 0.09) as f32,
            MountKind::Pig => (attribute("minecraft:movement_speed", 0.25) * 0.225 * 1.0) as f32,
            // `Strider.getRiddenSpeed`: slower while it is cold (suffocating).
            MountKind::Strider => {
                let suffocating = entity.is_suffocating();
                (attribute("minecraft:movement_speed", 0.175)
                    * f64::from(if suffocating { 0.35_f32 } else { 0.55_f32 })
                    * 1.0) as f32
            }
        },
        gravity: attribute("minecraft:gravity", 0.08),
        step_height: attribute("minecraft:step_height", 0.6) as f32,
        jump_strength: attribute("minecraft:jump_strength", 0.42) as f32,
        friction_modifier: attribute("minecraft:friction_modifier", 1.0) as f32,
        air_drag_modifier: attribute("minecraft:air_drag_modifier", 1.0) as f32,
        water_movement_efficiency: attribute("minecraft:water_movement_efficiency", 0.0) as f32,
    }
}

/// What a ridden camel's tick leaves in its state: standing up, a dash begun, the water.
fn write_back_camel(
    entity: &mut crate::entity::ClientEntity,
    ride: &CamelRide,
    in_water: bool,
    game_time: i64,
) {
    let horse = entity
        .kind
        .horse_or_insert_with(|| Box::new(crate::horse::HorseState::new()));
    if ride.stand_up {
        horse.camel.stand_up(game_time);
        entity.pose = crate::entity::ClientEntityPose::Standing;
    }
    if let Some(cooldown) = ride.dash_cooldown {
        let horse = entity.kind.horse_mut().expect("made above");
        horse.camel.set_dash_cooldown(cooldown);
        horse.camel.set_dashing(true);
    }
    // `Camel.tick`: a sitting camel in the water stands up at once.
    let horse = entity.kind.horse_mut().expect("made above");
    if horse.camel.sitting() && in_water {
        horse.camel.stand_up_instantly(game_time);
        entity.pose = crate::entity::ClientEntityPose::Standing;
    }
}

/// `Entity.calculateViewVector`: where an entity looks, from its pitch and its yaw.
fn view_vector(x_rot: f32, y_rot: f32) -> Vec3 {
    let real_x_rot = x_rot * DEG_TO_RAD;
    let real_y_rot = -y_rot * DEG_TO_RAD;
    let y_cos = mth_cos_f64(f64::from(real_y_rot));
    let y_sin = mth_sin_f64(f64::from(real_y_rot));
    let x_cos = mth_cos_f64(f64::from(real_x_rot));
    let x_sin = mth_sin_f64(f64::from(real_x_rot));
    Vec3::new(
        f64::from(y_sin * x_cos),
        f64::from(-x_sin),
        f64::from(y_cos * x_cos),
    )
}

/// What a ridden camel's pose and dash are, going into its tick, and what the tick makes of them.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CamelRide {
    pub(crate) sitting: bool,
    pub(crate) refuses_to_move: bool,
    pub(crate) in_pose_transition: bool,
    /// `Camel.dashCooldown` going into the tick.
    pub(crate) cooldown: i32,
    /// The tick asks it to stand up, and the dash's new cooldown.
    pub(crate) stand_up: bool,
    pub(crate) dash_cooldown: Option<i32>,
}

/// `Strider.tick`'s setting of the suffocating flag: cold unless it is in or on a warm block or in
/// lava (the depth it last looked at; a strider that rides a warm one is not a case here).
fn strider_is_suffocating(world: &ClientWorld, strider: &crate::entity::ClientEntity) -> bool {
    let level = Level::new(world);
    let (x, y, z) = strider.position();
    let inside = BlockPos::new(floor_f64(x), floor_f64(y), floor_f64(z));
    let on = level.on_pos(strider.tracking.support.main_block, Vec3::new(x, y, z), 0.2);
    let warm = |pos: BlockPos| {
        let state = crate::block::state_or_air(level.state_id(pos).map_or(0, i64::from));
        world
            .block_state_registry
            .tags()
            .contains(usize::from(state.block), "minecraft:strider_warm_blocks")
    };
    !(warm(inside) || warm(on) || strider.mount_ride.lava_height > 0.0)
}

#[cfg(test)]
mod tests {

    use crate::{ClientWorld, DimensionBounds, EntityInput};
    use voxel_core::binary::Uuid;

    #[test]
    fn a_horse_has_the_attributes_its_riding_reads() {
        let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
        let id = voxel_data::entity_type::id("minecraft:horse").expect("horse");
        assert!(world.apply_entity(EntityInput {
            entity_id: 5,
            uuid: Uuid::new(0, 5),
            type_id: id,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let horse = world.entities().find(|entity| entity.id() == 5).unwrap();
        // The values a saddled horse's riding reads, as the type starts with them.
        for (name, value) in [
            ("minecraft:gravity", 0.08),
            ("minecraft:step_height", 1.0),
            ("minecraft:jump_strength", 0.7),
            ("minecraft:friction_modifier", 1.0),
            ("minecraft:air_drag_modifier", 1.0),
            ("minecraft:water_movement_efficiency", 0.0),
        ] {
            assert_eq!(horse.attribute_value(name), Some(value), "{name}");
        }
        assert_eq!(
            horse
                .attribute_value("minecraft:movement_speed")
                .map(|speed| speed as f32),
            Some(0.225)
        );
    }
}

mod mount_body_jump;
mod mount_body_move_ops;
