//! Mob models: a layer of the data table baked into parts, posed the way the
//! reference's model classes pose them, and placed where a mob stands the
//! way `LivingEntityRenderer` places any living entity.

use voxel_core::math::{mth_cos_f64, mth_lerp_f32 as lerp_f32, mth_sin_f64};
use voxel_data::entity_model::ModelLayer;

use crate::entity_textures::{TextureRect, uv};
use crate::joml::{Pose, Quaternion};
use crate::player_model::{ModelVertex, Part, PlayerRenderState, humanoid_anim, rot_lerp_rad};
use crate::scene::Vertex;

/// What `QuadrupedModel.setupAnim` reads of a mob.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct QuadrupedState {
    /// The head's turn from the body and its pitch, in degrees.
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
}

/// What a villager's (or a wandering trader's) model is posed from.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VillagerState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// Shaking its head: `AbstractVillager.getUnhappyCounter` is above zero.
    pub unhappy: bool,
}

/// `AbstractIllager.IllagerArmPose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IllagerArmPose {
    Crossed,
    Attacking,
    Spellcasting,
    BowAndArrow,
    CrossbowHold,
    CrossbowCharge,
    Celebrating,
    Neutral,
}

impl IllagerArmPose {
    /// The pose of the reference's name (`CROSSBOW_HOLD`).
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "CROSSED" => Self::Crossed,
            "ATTACKING" => Self::Attacking,
            "SPELLCASTING" => Self::Spellcasting,
            "BOW_AND_ARROW" => Self::BowAndArrow,
            "CROSSBOW_HOLD" => Self::CrossbowHold,
            "CROSSBOW_CHARGE" => Self::CrossbowCharge,
            "CELEBRATING" => Self::Celebrating,
            "NEUTRAL" => Self::Neutral,
            _ => return None,
        })
    }
}

/// What an illager's model is posed from (`IllagerRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IllagerState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub riding: bool,
    pub arm_pose: IllagerArmPose,
    /// `mainArm` is the left.
    pub main_arm_left: bool,
    /// How far through the swing (`attackTime`, which `attackAnim` equals).
    pub attack_time: f32,
    /// Whether the main hand holds anything.
    pub has_item: bool,
    pub max_crossbow_charge: i32,
    pub ticks_using_item: f32,
    /// The hat layer of the head: only an illusioner shows it.
    pub hat_visible: bool,
}

/// What the witch's model is posed from (`WitchRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WitchState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// The entity's number: how fast the nose twitches.
    pub entity_id: i32,
    pub holding_item: bool,
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

/// What an enderman's model is posed from: the humanoid state, whether it
/// carries a block (`carriedBlock` is not empty) and `isCreepy`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EndermanState {
    pub humanoid: PlayerRenderState,
    pub carrying: bool,
    pub creepy: bool,
}

/// `Crackiness.Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crackiness {
    None,
    Low,
    Medium,
    High,
}

/// `Crackiness.GOLEM.byFraction`: how cracked an iron golem with this
/// fraction of its health left looks.
pub fn golem_crackiness(fraction: f32) -> Crackiness {
    if fraction < 0.25 {
        Crackiness::High
    } else if fraction < 0.5 {
        Crackiness::Medium
    } else if fraction < 0.75 {
        Crackiness::Low
    } else {
        Crackiness::None
    }
}

/// `Mth.triangleWave`.
fn triangle_wave(index: f32, period: f32) -> f32 {
    ((index % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
}

/// What `IronGolemRenderer.setupRotations` rolls the pose by while the golem
/// walks, in degrees, or none while it stands.
pub fn iron_golem_roll(walk_position: f32, walk_speed: f32) -> Option<f32> {
    (walk_speed >= 0.01).then(|| 6.5 * triangle_wave(walk_position + 6.0, 13.0))
}

/// What a goat's models are posed from (`GoatRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoatState {
    pub quadruped: QuadrupedState,
    pub has_left_horn: bool,
    pub has_right_horn: bool,
    /// `rammingXHeadRot`: how far the head is lowered to ram, in radians.
    pub ramming_x_head_rot: f32,
    pub baby: bool,
}

/// What a rabbit's models are posed from (`RabbitRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RabbitState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub hop: voxel_world::animation_state::AnimationState,
    pub idle_head_tilt: voxel_world::animation_state::AnimationState,
    pub baby: bool,
}

/// What a copper golem's model is posed from (`CopperGolemRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CopperGolemModelState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// Either hand holds something (`rightHandItemState` or `leftHandItemState`).
    pub holds_item: bool,
    pub animations: voxel_world::copper_golem::CopperGolemPlaying,
}

/// What a bee's models are posed from (`BeeRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeeState {
    pub age_in_ticks: f32,
    pub roll_amount: f32,
    pub has_stinger: bool,
    /// Resting on the ground: the wings and legs stay as modelled.
    pub on_ground: bool,
    pub angry: bool,
    pub baby: bool,
}

/// Which equine model: the horse family's (the horse, the undead horses), the
/// donkey family's (donkey, mule, which carry chests), grown or baby. The baby
/// donkey's parts hang under its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquineKind {
    Horse,
    BabyHorse,
    Donkey,
    BabyDonkey,
}

/// What an equine's model is posed from (`EquineRenderState`,
/// `DonkeyRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquineState {
    pub kind: EquineKind,
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// `eatAnimation`, `standAnimation` and `feedingAnimation`: how far the
    /// head is down to graze, the animal is reared and its mouth is open.
    pub eat_animation: f32,
    pub stand_animation: f32,
    pub feeding_animation: f32,
    pub animate_tail: bool,
    pub in_water: bool,
    pub age_scale: f32,
    pub has_chest: bool,
}

/// What a camel's models are posed from (`CamelRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// The dash cooldown left, which tips the head down while it runs.
    pub jump_cooldown: f32,
    pub sit: voxel_world::animation_state::AnimationState,
    pub sit_pose: voxel_world::animation_state::AnimationState,
    pub sit_up: voxel_world::animation_state::AnimationState,
    pub idle: voxel_world::animation_state::AnimationState,
    pub dash: voxel_world::animation_state::AnimationState,
    pub baby: bool,
}

/// What a wolf's models are posed from (`WolfRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WolfState {
    pub quadruped: QuadrupedState,
    pub angry: bool,
    pub sitting: bool,
    pub baby: bool,
    /// How far the tail is raised, and how far the head rolls to a side.
    pub tail_angle: f32,
    pub head_roll_angle: f32,
    /// The shake's progress, which rolls the body.
    pub shake_anim: f32,
    /// `LivingEntityRenderState.ageScale`: what the sitting pose is scaled by.
    pub age_scale: f32,
}

impl WolfState {
    /// `WolfRenderState.getBodyRollAngle`.
    fn body_roll_angle(&self, offset: f32) -> f32 {
        let pi = std::f64::consts::PI as f32;
        let progress = ((self.shake_anim + offset) / 1.8).clamp(0.0, 1.0);
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        sin(progress * pi) * sin(progress * pi * 11.0) * 0.15 * pi
    }
}

/// What a panda's models are posed from (`PandaRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PandaState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub unhappy: bool,
    pub sneezing: bool,
    pub sneeze_time: i32,
    pub eating: bool,
    pub scared: bool,
    pub sit_amount: f32,
    pub lie_on_back_amount: f32,
    pub roll_amount: f32,
    pub baby: bool,
}

/// What `PandaRenderer.setupRotations` turns the whole panda by: the roll it
/// somersaults in (`rollTime`, zero when it does not), how far it sits and
/// lies on its back, and its fright.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PandaRotation {
    pub roll_time: f32,
    pub sit_amount: f32,
    pub lie_on_back_amount: f32,
    /// The look's pitch the sitting tips are measured from.
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub scared: bool,
    pub baby: bool,
}

/// What a fox's models are posed from (`FoxRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FoxState {
    pub quadruped: QuadrupedState,
    /// How far its head rolls to a side while it is interested, and how far
    /// it has crouched (zero to five).
    pub head_roll_angle: f32,
    pub crouch_amount: f32,
    pub crouching: bool,
    pub sleeping: bool,
    pub sitting: bool,
    pub faceplanted: bool,
    pub pouncing: bool,
    pub age_in_ticks: f32,
    /// `LivingEntityRenderState.ageScale`: what a baby's sitting shifts are scaled by.
    pub age_scale: f32,
    pub baby: bool,
}

/// What a strider's models are posed from (`StriderRenderState`): the look
/// and walk, the age (the bristles stir with it), and whether it is ridden
/// (its body then faces front).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StriderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub ridden: bool,
    pub baby: bool,
}

/// What a ravager's model is posed from (`RavagerRenderState`): the quadruped's
/// look and walk, and how long the stun, the attack swing and the roar have
/// to run (the ticks left less the partial tick; the roar as a fraction).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RavagerState {
    pub quadruped: QuadrupedState,
    pub stunned_ticks_remaining: f32,
    pub attack_ticks_remaining: f32,
    pub roar_animation: f32,
}

/// What a hoglin's or a zoglin's models are posed from (`HoglinRenderState`):
/// the walk, the head's turn from the body (there is no pitch) and the ticks
/// left of its attack swing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HoglinState {
    pub head_y_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
    pub attack_ticks: i32,
    pub baby: bool,
}

/// What a polar bear's models are posed from: the quadruped's, how far it
/// has stood up (`PolarBearRenderState.standScale`, zero to one) and the age
/// scale its shifts are scaled by.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolarBearState {
    pub quadruped: QuadrupedState,
    pub stand_scale: f32,
    pub age_scale: f32,
}

/// What a cat's or an ocelot's models are posed from (`FelineRenderState`,
/// `CatRenderState`): an ocelot never sits, lies down or relaxes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FelineState {
    pub quadruped: QuadrupedState,
    pub crouching: bool,
    pub sprinting: bool,
    pub sitting: bool,
    pub baby: bool,
    /// How far it has lain down, its tail's own progress, and how far it has
    /// relaxed (its head lifted).
    pub lie_down_amount: f32,
    pub lie_down_amount_tail: f32,
    pub relax_state_one_amount: f32,
    /// `LivingEntityRenderState.ageScale`: what the pose's shifts are scaled by.
    pub age_scale: f32,
}

const LEGS: [&str; 4] = [
    "right_hind_leg",
    "left_hind_leg",
    "right_front_leg",
    "left_front_leg",
];

impl MobPlacement {
    /// `CatRenderer.setupRotations`: a cat that lies down tips onto its side
    /// (and further over a sleeping player).
    pub fn cat_lying(&mut self, amount: f32, on_sleeping_player: bool) {
        if amount > 0.0 {
            self.shift = Some([0.4 * amount, 0.15 * amount, 0.1 * amount]);
            // `Mth.rotLerp(amount, 0, 90)`.
            self.roll = Some(amount * 90.0);
            if on_sleeping_player {
                self.roll_shift = Some([0.15 * amount, 0.0, 0.0]);
            }
        }
    }

    /// `FoxRenderer.setupRotations`: a fox that pounces or has its face
    /// planted tips with its pitch.
    pub fn fox_tipping(&mut self, pitch: f32) {
        self.pitch = Some(-pitch);
    }
}

/// `PandaRenderer.setupRotations`, after what `LivingEntityRenderer` does:
/// the somersault in four quarter turns (eased between whole ticks), the
/// sitting tip (and the frightened shake) and lying on the back.
fn panda_rotate(pose: &mut Pose, panda: &PandaRotation) {
    let rotate_x = |pose: &mut Pose, degrees: f32| {
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            degrees,
        )));
    };
    if panda.roll_time > 0.0 {
        let transition = panda.roll_time - panda.roll_time.floor();
        let roll_pos = panda.roll_time.floor() as i32;
        let next = roll_pos + 1;
        let y = if panda.baby { 0.3 } else { 0.8 };
        let angle_of = |this: f32, next_angle: f32, threshold: f32| {
            if (next as f32) < threshold {
                lerp_f32(transition, this, next_angle)
            } else {
                this
            }
        };
        let (position, degrees) = if roll_pos < 8 {
            let angle = angle_of(90.0 * roll_pos as f32 / 7.0, 90.0 * next as f32 / 7.0, 8.0);
            ((y + 0.2) * (angle / 90.0), angle)
        } else if roll_pos < 16 {
            let counter = (roll_pos as f32 - 8.0) / 7.0;
            let angle = angle_of(
                90.0 + 90.0 * counter,
                90.0 + 90.0 * (next as f32 - 8.0) / 7.0,
                16.0,
            );
            (y + 0.2 + (y - 0.2) * (angle - 90.0) / 90.0, angle)
        } else if roll_pos < 24 {
            let counter = (roll_pos as f32 - 16.0) / 7.0;
            let angle = angle_of(
                180.0 + 90.0 * counter,
                180.0 + 90.0 * (next as f32 - 16.0) / 7.0,
                24.0,
            );
            (y + y * (270.0 - angle) / 90.0, angle)
        } else if roll_pos < 32 {
            let counter = (roll_pos as f32 - 24.0) / 7.0;
            let angle = angle_of(
                270.0 + 90.0 * counter,
                270.0 + 90.0 * (next as f32 - 24.0) / 7.0,
                32.0,
            );
            (y * ((360.0 - angle) / 90.0), angle)
        } else {
            (f32::NAN, 0.0)
        };
        if !position.is_nan() {
            pose.translate(0.0, position, 0.0);
            rotate_x(pose, -degrees);
        }
    }
    let sit = panda.sit_amount;
    if sit > 0.0 {
        pose.translate(0.0, 0.8 * sit, 0.0);
        rotate_x(pose, lerp_f32(sit, panda.x_rot, panda.x_rot + 90.0));
        pose.translate(0.0, -sit, 0.0);
        if panda.scared {
            // `Math.cos` in doubles, not `Mth.cos`.
            let shake = (f64::from(panda.age_in_ticks * 1.25).cos()
                * std::f64::consts::PI
                * f64::from(0.05_f32)) as f32;
            pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
                shake,
            )));
            if panda.baby {
                pose.translate(0.0, 0.8, 0.55);
            }
        }
    }
    let lie = panda.lie_on_back_amount;
    if lie > 0.0 {
        let y = if panda.baby { 0.5 } else { 1.3 };
        pose.translate(0.0, y * lie, 0.0);
        rotate_x(pose, lerp_f32(lie, panda.x_rot, panda.x_rot + 180.0));
    }
}

/// `PandaHoldsItemLayer`: what a sitting panda that is not frightened holds
/// is carried in front of it, bobbing while it eats.
pub fn panda_item_pose(base: &Pose, age_in_ticks: f32, eating: bool) -> Pose {
    let (mut z, mut y) = (-0.6_f32, 1.4_f32);
    if eating {
        let bob = mth_sin_f64(f64::from(age_in_ticks * 0.6));
        z -= 0.2 * bob + 0.2;
        y -= 0.09 * bob;
    }
    let mut pose = *base;
    pose.translate(0.1, y, z);
    pose
}

/// `Mth.rotLerp` (which wraps the radians it is given as if they were degrees).
fn rot_lerp(amount: f32, from: f32, to: f32) -> f32 {
    from + amount * voxel_core::math::wrap_degrees_f32(to - from)
}

/// What a sheep's models are posed from: the quadruped's, and how far the
/// head is down and tipped while it eats (`SheepRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SheepState {
    pub quadruped: QuadrupedState,
    pub head_eat_position_scale: f32,
    pub head_eat_angle_scale: f32,
    /// `LivingEntityRenderState.ageScale`: a half for a baby.
    pub age_scale: f32,
}

/// What a chicken's model is posed from: the head's turn, the walk and the
/// flap of the wings.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChickenState {
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
    pub flap: f32,
    pub flap_speed: f32,
}

/// What an armor stand's models are posed from (`ArmorStandRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorStandState {
    /// The rotations of the head, the body, the left and right arm and the
    /// left and right leg, in degrees about x, y, z.
    pub poses: [[f32; 3]; 6],
    pub show_arms: bool,
    pub show_base_plate: bool,
    /// The stand's turn, in degrees (the base plate follows it).
    pub y_rot: f32,
}

impl Default for ArmorStandState {
    fn default() -> Self {
        Self {
            poses: [
                [0.0; 3],
                [0.0; 3],
                [-10.0, 0.0, -10.0],
                [-15.0, 0.0, 10.0],
                [-1.0, 0.0, -1.0],
                [1.0, 0.0, 1.0],
            ],
            show_arms: false,
            show_base_plate: true,
            y_rot: 0.0,
        }
    }
}

/// What a zombie's model is posed from: the humanoid state, and the flags
/// `ZombieRenderState` adds. The held item is not read yet, so the arms are
/// raised for the baby as for the grown (`state.getMainHandItemStack() ==
/// ItemStack.EMPTY`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ZombieState {
    pub humanoid: PlayerRenderState,
    /// The mob's aggressive flag: the arms raise higher.
    pub aggressive: bool,
    /// `DrownedModel`: the swimming arms and legs.
    pub drowned: bool,
}

/// What a skeleton's model is posed from. `holding_bow` is whether the main
/// hand holds a bow, which the held item says (not read yet: the live caller
/// takes a bow to be in hand, as a skeleton spawns with one).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SkeletonState {
    pub humanoid: PlayerRenderState,
    pub aggressive: bool,
    pub holding_bow: bool,
}

/// What a piglin's model is posed from. A zombified piglin raises its arms
/// as a zombie does; a piglin dances when its data says so (the poses of its
/// held items are not read yet).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PiglinState {
    pub humanoid: PlayerRenderState,
    pub aggressive: bool,
    pub zombified: bool,
    pub baby: bool,
    pub dancing: bool,
}

/// The arms of `IllagerModel.setupAnim`'s arm pose: each arm's rotations
/// about x, y and z, given the head's turn and pitch.
fn illager_arms(
    state: &IllagerState,
    (head_y_rot, head_x_rot): (f32, f32),
    (right_arm, left_arm): (&mut [f32; 3], &mut [f32; 3]),
) {
    let pi = std::f64::consts::PI as f32;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let age = state.age_in_ticks;
    match state.arm_pose {
        IllagerArmPose::Attacking => {
            let attack_time = state.attack_time;
            if state.has_item {
                // `AnimationUtils.swingWeaponDown`.
                let attack_2 = sin(attack_time * pi);
                let attack = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
                right_arm[2] = 0.0;
                left_arm[2] = 0.0;
                right_arm[1] = (std::f64::consts::PI / 20.0) as f32;
                left_arm[1] = (-std::f64::consts::PI / 20.0) as f32;
                if state.main_arm_left {
                    right_arm[0] = -0.0 + cos(age * 0.19) * 0.5;
                    left_arm[0] = -1.884_955_8 + cos(age * 0.09) * 0.15;
                    right_arm[0] += attack_2 * 1.2 - attack * 0.4;
                    left_arm[0] += attack_2 * 2.2 - attack * 0.4;
                } else {
                    right_arm[0] = -1.884_955_8 + cos(age * 0.09) * 0.15;
                    left_arm[0] = -0.0 + cos(age * 0.19) * 0.5;
                    right_arm[0] += attack_2 * 2.2 - attack * 0.4;
                    left_arm[0] += attack_2 * 1.2 - attack * 0.4;
                }
            } else {
                // `AnimationUtils.animateZombieArms`, aggressive.
                let arm_drop = -pi / 1.5;
                let y_modifier = sin(attack_time * pi);
                let x_modifier = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
                let x_rot = arm_drop + y_modifier * 1.2 - x_modifier * 0.4;
                let y_rot = 0.1 - y_modifier * 0.6;
                *right_arm = [x_rot, -y_rot, 0.0];
                *left_arm = [x_rot, y_rot, 0.0];
            }
            // `bobArms`.
            for (arm, scale) in [(&mut *right_arm, 1.0_f32), (&mut *left_arm, -1.0)] {
                arm[2] += scale * (cos(age * 0.09) * 0.05 + 0.05);
                arm[0] += scale * (sin(age * 0.067) * 0.05);
            }
        }
        IllagerArmPose::Spellcasting => {
            // The arms' offsets the reference sets are the ones they start from.
            right_arm[0] = cos(age * 0.6662) * 0.25;
            left_arm[0] = cos(age * 0.6662) * 0.25;
            right_arm[2] = (std::f64::consts::PI * 3.0 / 4.0) as f32;
            left_arm[2] = (-std::f64::consts::PI * 3.0 / 4.0) as f32;
            right_arm[1] = 0.0;
            left_arm[1] = 0.0;
        }
        IllagerArmPose::BowAndArrow => {
            right_arm[1] = -0.1 + head_y_rot;
            right_arm[0] = (-std::f64::consts::PI / 2.0) as f32 + head_x_rot;
            left_arm[0] = -0.942_477_9 + head_x_rot;
            left_arm[1] = head_y_rot - 0.4;
            left_arm[2] = (std::f64::consts::PI / 2.0) as f32;
        }
        IllagerArmPose::CrossbowHold => {
            // `animateCrossbowHold`, held in the right arm.
            right_arm[1] = -0.3 + head_y_rot;
            left_arm[1] = 0.6 + head_y_rot;
            right_arm[0] = (-std::f64::consts::PI / 2.0) as f32 + head_x_rot + 0.1;
            left_arm[0] = -1.5 + head_x_rot;
        }
        IllagerArmPose::CrossbowCharge => {
            // `animateCrossbowCharge`, held in the right arm.
            right_arm[1] = -0.8;
            right_arm[0] = -0.970_796_35;
            left_arm[0] = right_arm[0];
            let maximum = state.max_crossbow_charge as f32;
            let use_ticks = if state.ticks_using_item < 0.0 {
                0.0
            } else if state.ticks_using_item > maximum {
                maximum
            } else {
                state.ticks_using_item
            };
            let alpha = use_ticks / maximum;
            left_arm[1] = 0.4 + alpha * (0.85 - 0.4);
            let half_pi = (-std::f64::consts::PI / 2.0) as f32;
            left_arm[0] += alpha * (half_pi - left_arm[0]);
        }
        IllagerArmPose::Celebrating => {
            right_arm[0] = cos(age * 0.6662) * 0.05;
            right_arm[2] = 2.670_354;
            right_arm[1] = 0.0;
            left_arm[0] = cos(age * 0.6662) * 0.05;
            left_arm[2] = (-std::f64::consts::PI * 3.0 / 4.0) as f32;
            left_arm[1] = 0.0;
        }
        IllagerArmPose::Crossed | IllagerArmPose::Neutral => {}
    }
}

/// A part of an equine's model: a baby donkey's hang under its body.
fn equine_part<'a>(root: &'a mut Part, nested: bool, name: &str) -> &'a mut Part {
    let holder = if nested && name != "body" {
        root.child_mut("body").expect("the baby donkey has a body")
    } else {
        root
    };
    holder.child_mut(name).expect("an equine has this part")
}

/// A mob's model: the parts of one layer.
#[derive(Debug, Clone)]
pub struct MobModel {
    root: Part,
    /// `FoxModel.legMotionPos`: the model's own count, which a faceplanted
    /// fox's legs follow and which grows with every pose of such a fox.
    leg_motion_pos: f32,
}

impl MobModel {
    /// `LayerDefinition.bakeRoot`.
    pub fn bake(layer: &ModelLayer) -> Self {
        Self {
            root: Part::baked(&layer.root, (layer.texture_width, layer.texture_height)),
            leg_motion_pos: 0.0,
        }
    }

    pub fn root(&self) -> &Part {
        &self.root
    }

    /// `QuadrupedModel.setupAnim`: the head follows the look, and the legs
    /// swing in diagonal pairs with the walk.
    pub fn setup_quadruped(&mut self, state: &QuadrupedState) {
        // `EntityModel.setupAnim`: the pose is reset first.
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
        let (position, speed) = (state.walk_position, state.walk_speed);
        let swing = [
            ("right_hind_leg", cos(position * 0.6662) * 1.4 * speed),
            ("left_hind_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("right_front_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("left_front_leg", cos(position * 0.6662) * 1.4 * speed),
        ];
        for (name, x_rot) in swing {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
    }

    /// `ArmorStandModel.setupAnim` (and `ArmorStandArmorModel`'s, `stand` false
    /// for the armor): the humanoid pose with each part turned to the stand's
    /// rotation; the stand itself also hides its hat and its arms and base
    /// plate by its flags, and turns its sticks with the body.
    pub fn setup_armor_stand(&mut self, state: &ArmorStandState, stand: bool) {
        humanoid_anim(&mut self.root, &PlayerRenderState::default());
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let parts = [
            "head",
            "body",
            "left_arm",
            "right_arm",
            "left_leg",
            "right_leg",
        ];
        for (name, [x, y, z]) in parts.into_iter().zip(state.poses) {
            if let Some(part) = self.root.child_mut(name) {
                part.x_rot = degrees * x;
                part.y_rot = degrees * y;
                part.z_rot = degrees * z;
            }
        }
        if !stand {
            return;
        }
        if let Some(hat) = self
            .root
            .child_mut("head")
            .and_then(|head| head.child_mut("hat"))
        {
            hat.visible = false;
        }
        for name in ["left_arm", "right_arm"] {
            if let Some(arm) = self.root.child_mut(name) {
                arm.visible = state.show_arms;
            }
        }
        let [x, y, z] = state.poses[1];
        for name in ["right_body_stick", "left_body_stick", "shoulder_stick"] {
            if let Some(stick) = self.root.child_mut(name) {
                stick.x_rot = degrees * x;
                stick.y_rot = degrees * y;
                stick.z_rot = degrees * z;
            }
        }
        if let Some(plate) = self.root.child_mut("base_plate") {
            plate.y_rot = degrees * -state.y_rot;
            plate.visible = state.show_base_plate;
        }
    }

    /// `HumanoidModel.setupAnim` for a humanoid model of the table whose class
    /// adds nothing of its own (an armor model: the wearer's pose, the arms'
    /// poses its held items ask for).
    pub fn setup_humanoid(&mut self, state: &PlayerRenderState) {
        humanoid_anim(&mut self.root, state);
    }

    /// `SpiderModel.setupAnim`: the head follows the look; each leg is
    /// swung about y and lifted about z in turn, the pairs a quarter of a
    /// stride apart.
    pub fn setup_spider(&mut self, state: &QuadrupedState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
        let position = state.walk_position * 0.6662;
        let speed = state.walk_speed;
        let quarter = (std::f64::consts::PI / 2.0) as f32;
        let three_quarters = (std::f64::consts::PI * 3.0 / 2.0) as f32;
        let swing = |phase: f32| -(cos(position * 2.0 + phase) * 0.4) * speed;
        let step = |phase: f32| (sin(position + phase) * 0.4).abs() * speed;
        // The pairs, back to front: each leg on the right moves one way and
        // its opposite the other.
        for (right, left, phase) in [
            ("right_hind_leg", "left_hind_leg", 0.0),
            ("right_middle_hind_leg", "left_middle_hind_leg", pi),
            ("right_middle_front_leg", "left_middle_front_leg", quarter),
            ("right_front_leg", "left_front_leg", three_quarters),
        ] {
            let (turn, lift) = (swing(phase), step(phase));
            if let Some(leg) = self.root.child_mut(right) {
                leg.y_rot += turn;
                leg.z_rot += lift;
            }
            if let Some(leg) = self.root.child_mut(left) {
                leg.y_rot -= turn;
                leg.z_rot -= lift;
            }
        }
    }

    /// `CreeperModel.setupAnim`: the head follows the look, and the legs
    /// swing in diagonal pairs (the model names its sides the other way
    /// about, which the swing follows).
    pub fn setup_creeper(&mut self, state: &QuadrupedState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = state.head_y_rot * degrees;
            head.x_rot = state.x_rot * degrees;
        }
        let (position, speed) = (state.walk_position, state.walk_speed);
        let swing = [
            ("left_hind_leg", cos(position * 0.6662) * 1.4 * speed),
            ("right_hind_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("left_front_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("right_front_leg", cos(position * 0.6662) * 1.4 * speed),
        ];
        for (name, x_rot) in swing {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
    }

    /// `VillagerModel.setupAnim` (the baby's model inherits it): the head
    /// follows the look, or shakes while the villager is unhappy, and the legs
    /// swing as a pair.
    pub fn setup_villager(&mut self, state: &VillagerState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let look = &state.quadruped;
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = look.head_y_rot * degrees;
            head.x_rot = look.x_rot * degrees;
            if state.unhappy {
                head.z_rot = 0.3 * mth_sin_f64(f64::from(0.45 * state.age_in_ticks));
                head.x_rot = 0.4;
            } else {
                head.z_rot = 0.0;
            }
        }
        let (position, speed) = (look.walk_position, look.walk_speed);
        let swing = [
            ("right_leg", cos(position * 0.6662) * 1.4 * speed * 0.5),
            ("left_leg", cos(position * 0.6662 + pi) * 1.4 * speed * 0.5),
        ];
        for (name, x_rot) in swing {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
                leg.y_rot = 0.0;
            }
        }
    }

    /// `IllagerModel.setupAnim`: the head follows the look; riding or walking
    /// sets the limbs; then the arm pose takes the arms over, and only a
    /// crossed pose shows the folded arms in place of the two.
    pub fn setup_illager(&mut self, state: &IllagerState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (head_y_rot, head_x_rot) = (
            state.quadruped.head_y_rot * degrees,
            state.quadruped.x_rot * degrees,
        );
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = head_y_rot;
            head.x_rot = head_x_rot;
            if let Some(hat) = head.child_mut("hat") {
                hat.visible = state.hat_visible;
            }
        }
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        // Rotations of each limb: x, y, z.
        let (mut right_arm, mut left_arm) = ([0.0_f32; 3], [0.0_f32; 3]);
        let (right_leg, left_leg);
        if state.riding {
            let fifth = (-std::f64::consts::PI / 5.0) as f32;
            right_arm = [fifth, 0.0, 0.0];
            left_arm = [fifth, 0.0, 0.0];
            let tenth = (std::f64::consts::PI / 10.0) as f32;
            right_leg = [-1.413_716_7, tenth, 0.078_539_82];
            left_leg = [-1.413_716_7, -tenth, -0.078_539_82];
        } else {
            right_arm[0] = cos(position * 0.6662 + pi) * 2.0 * speed * 0.5;
            left_arm[0] = cos(position * 0.6662) * 2.0 * speed * 0.5;
            right_leg = [cos(position * 0.6662) * 1.4 * speed * 0.5, 0.0, 0.0];
            left_leg = [cos(position * 0.6662 + pi) * 1.4 * speed * 0.5, 0.0, 0.0];
        }
        illager_arms(
            state,
            (head_y_rot, head_x_rot),
            (&mut right_arm, &mut left_arm),
        );
        let set = |root: &mut Part, name: &str, [x_rot, y_rot, z_rot]: [f32; 3]| {
            if let Some(part) = root.child_mut(name) {
                part.x_rot = x_rot;
                part.y_rot = y_rot;
                part.z_rot = z_rot;
            }
        };
        set(&mut self.root, "right_arm", right_arm);
        set(&mut self.root, "left_arm", left_arm);
        set(&mut self.root, "right_leg", right_leg);
        set(&mut self.root, "left_leg", left_leg);
        let crossed = state.arm_pose == IllagerArmPose::Crossed;
        if let Some(arms) = self.root.child_mut("arms") {
            arms.visible = crossed;
        }
        for name in ["left_arm", "right_arm"] {
            if let Some(arm) = self.root.child_mut(name) {
                arm.visible = !crossed;
            }
        }
    }

    /// `WitchModel.setupAnim`: the head follows the look, the legs swing as a
    /// pair, and the nose twitches at a rate the entity's number sets; holding
    /// something draws it up and forward.
    pub fn setup_witch(&mut self, state: &WitchState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let look = &state.quadruped;
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = look.head_y_rot * degrees;
            head.x_rot = look.x_rot * degrees;
        }
        let (position, speed) = (look.walk_position, look.walk_speed);
        for (name, x_rot) in [
            ("right_leg", cos(position * 0.6662) * 1.4 * speed * 0.5),
            ("left_leg", cos(position * 0.6662 + pi) * 1.4 * speed * 0.5),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
        let twitch = 0.01 * (state.entity_id % 10) as f32;
        if let Some(nose) = self
            .root
            .child_mut("head")
            .and_then(|head| head.child_mut("nose"))
        {
            nose.x_rot = mth_sin_f64(f64::from(state.age_in_ticks * twitch)) * 4.5 * degrees;
            nose.z_rot = cos(state.age_in_ticks * twitch) * 2.5 * degrees;
            if state.holding_item {
                nose.x = 0.0;
                nose.y = 1.0;
                nose.z = -1.5;
                nose.x_rot = -0.9;
            }
        }
    }

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

    /// `EndermanModel.setupAnim`: the humanoid pose with the limbs' swing
    /// halved and held within 0.4; a carried block holds the arms forward and
    /// a creepy one lifts its head off its hat.
    pub fn setup_enderman(&mut self, state: &EndermanState) {
        humanoid_anim(&mut self.root, &state.humanoid);
        if let Some(head) = self.root.child_mut("head") {
            head.visible = true;
        }
        for name in ["right_arm", "left_arm", "right_leg", "left_leg"] {
            if let Some(part) = self.root.child_mut(name) {
                part.x_rot = (part.x_rot * 0.5).clamp(-0.4, 0.4);
            }
        }
        if state.carrying {
            for (name, x_rot, z_rot) in [("right_arm", -0.5, 0.05), ("left_arm", -0.5, -0.05)] {
                if let Some(arm) = self.root.child_mut(name) {
                    arm.x_rot = x_rot;
                    arm.z_rot = z_rot;
                }
            }
        }
        if state.creepy
            && let Some(head) = self.root.child_mut("head")
        {
            head.y -= 5.0;
            if let Some(hat) = head.child_mut("hat") {
                hat.y += 5.0;
            }
        }
    }

    /// `KeyframeAnimation.apply(millisSinceStart, targetScale)` of a definition
    /// on the pose the model is in.
    pub fn animate(
        &mut self,
        definition: &voxel_data::animation::Definition,
        millis: i64,
        target_scale: f32,
    ) {
        crate::keyframe::apply(&mut self.root, definition, millis, target_scale);
    }

    /// `RabbitModel.setupAnim`: the head follows the look unless the idle head
    /// tilt is playing; then the hop and the tilt, each if it has started.
    pub fn setup_rabbit(&mut self, state: &RabbitState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if !state.idle_head_tilt.is_started()
            && let Some(head) = self
                .root
                .child_mut("body")
                .and_then(|body| body.child_mut("head"))
        {
            head.y_rot = state.quadruped.head_y_rot * degrees;
            head.x_rot = state.quadruped.x_rot * degrees;
        }
        let prefix = if state.baby { "BabyRabbit" } else { "Rabbit" };
        for (animation, running) in [("HOP", state.hop), ("IDLE_HEAD_TILT", state.idle_head_tilt)] {
            if !running.is_started() {
                continue;
            }
            if let Some(definition) =
                voxel_data::animation::definition(&format!("{prefix}.{animation}"))
            {
                let millis = (running.time_in_millis(state.age_in_ticks) as f32 * 1.0) as i64;
                self.animate(definition, millis, 1.0);
            }
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
            const RAISE: f32 = -0.87266463;
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

    /// `SilverfishModel.setupAnim` and `EndermiteModel.setupAnim`: each
    /// segment sways about y and slides sideways in a wave that travels down
    /// the body (the silverfish's three layers follow the segments under them).
    pub fn setup_segmented(&mut self, age_in_ticks: f32, silverfish: bool) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let (segments, turn, slide) = if silverfish {
            (7_i32, 0.05_f32, 0.2_f32)
        } else {
            (4, 0.01, 0.1)
        };
        for index in 0..segments {
            let phase = age_in_ticks * 0.9 + index as f32 * 0.15 * pi;
            let distance = (index - 2).abs();
            let y_rot = mth_cos_f64(f64::from(phase)) * pi * turn * (1 + distance) as f32;
            let x = mth_sin_f64(f64::from(phase)) * pi * slide * distance as f32;
            if let Some(segment) = self.root.child_mut(&format!("segment{index}")) {
                segment.y_rot = y_rot;
                segment.x = x;
            }
        }
        if !silverfish {
            return;
        }
        let pose = |root: &mut Part, index: usize| {
            root.child_mut(&format!("segment{index}"))
                .map(|segment| (segment.y_rot, segment.x))
        };
        for (layer, segment, slides) in [(0, 2, false), (1, 4, true), (2, 1, true)] {
            let Some((y_rot, x)) = pose(&mut self.root, segment) else {
                continue;
            };
            if let Some(layer) = self.root.child_mut(&format!("layer{layer}")) {
                layer.y_rot = y_rot;
                if slides {
                    layer.x = x;
                }
            }
        }
    }

    /// `BeeModel.setupAnim` (and the adult's antennas): the stinger while it
    /// has one; in the air the wings beat and the legs hang; a bee that is not
    /// angry bobs; a rolling one turns over.
    pub fn setup_bee(&mut self, state: &BeeState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let sin_cos = |value: f32| mth_cos_f64(f64::from(value));
        let age = state.age_in_ticks;
        let Some(bone) = self.root.child_mut("bone") else {
            return;
        };
        if let Some(stinger) = bone
            .child_mut("body")
            .and_then(|body| body.child_mut("stinger"))
        {
            stinger.visible = state.has_stinger;
        }
        if !state.on_ground {
            let speed = age * 120.321_13 * (std::f64::consts::PI / 180.0) as f32;
            let flap = sin_cos(speed) * pi * 0.15;
            let right_x = bone.child_mut("right_wing").map_or(0.0, |wing| {
                wing.y_rot = 0.0;
                wing.z_rot = flap;
                wing.x_rot
            });
            if let Some(wing) = bone.child_mut("left_wing") {
                wing.x_rot = right_x;
                wing.y_rot = 0.0;
                wing.z_rot = -flap;
            }
            for name in ["front_legs", "middle_legs", "back_legs"] {
                if let Some(leg) = bone.child_mut(name) {
                    leg.x_rot = (std::f64::consts::PI / 4.0) as f32;
                }
            }
        }
        if !state.angry && !state.on_ground {
            let speed = sin_cos(age * 0.18);
            bone.x_rot = 0.1 + speed * pi * 0.025;
            bone.y -= sin_cos(age * 0.18) * 0.9;
            if let Some(leg) = bone.child_mut("front_legs") {
                leg.x_rot = -speed * pi * 0.1 + (std::f64::consts::PI / 8.0) as f32;
            }
            if let Some(leg) = bone.child_mut("back_legs") {
                leg.x_rot = -speed * pi * 0.05 + (std::f64::consts::PI / 4.0) as f32;
            }
            if !state.baby
                && let Some(body) = bone.child_mut("body")
            {
                for name in ["left_antenna", "right_antenna"] {
                    if let Some(antenna) = body.child_mut(name) {
                        antenna.x_rot = speed * pi * 0.03;
                    }
                }
            }
        }
        if state.roll_amount > 0.0 {
            bone.x_rot = rot_lerp_rad(state.roll_amount, bone.x_rot, 3.091_592_8);
        }
    }

    /// `AbstractEquineModel.setupAnim` (and the baby and donkey subclasses'
    /// overrides): the head eases between its grazing, standing and ordinary
    /// poses, the legs stride or paw the air, the tail lifts with the walk and
    /// swishes; a donkey shows its chests.
    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub fn setup_equine(&mut self, state: &EquineState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let lerp = |alpha: f32, from: f32, to: f32| from + alpha * (to - from);
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let kind = state.kind;
        let nested = kind == EquineKind::BabyDonkey;
        // The overrides of each kind.
        let leg_stand_angle = if nested { pi / 3.0 } else { pi / 12.0 };
        let leg_standing_y = match kind {
            EquineKind::BabyHorse => 4.0,
            EquineKind::BabyDonkey => 1.0,
            _ => 12.0,
        };
        let leg_standing_z = match kind {
            EquineKind::BabyHorse => 0.0,
            EquineKind::BabyDonkey => 0.5,
            _ => 4.0,
        };
        let leg_standing_x_rot = if nested { 0.0 } else { -pi / 3.0 };
        let tail_x_rot_offset = match kind {
            EquineKind::BabyHorse => -pi / 2.0,
            EquineKind::BabyDonkey => -pi / 4.0,
            _ => 0.0,
        };
        let look = &state.quadruped;
        let clamped_y_rot = look.head_y_rot.clamp(-20.0, 20.0);
        let mut head_rot_x = look.x_rot * degrees;
        let (speed, position) = (look.walk_speed, look.walk_position);
        if speed > 0.2 {
            head_rot_x += cos(position * 0.8) * 0.15 * speed;
        }
        let eating = state.eat_animation;
        let standing = state.stand_animation;
        let not_standing = 1.0 - standing;
        let feeding = state.feeding_animation;
        let age = state.age_in_ticks;
        let root = &mut self.root;
        let water = if state.in_water { 0.2 } else { 1.0 };
        let leg_anim = cos(water * position * 0.6662 + pi);
        let leg_x_rot_anim = leg_anim * 0.8 * speed;
        let base_head_angle =
            (1.0 - standing.max(eating)) * (pi / 6.0 + head_rot_x + feeding * sin(age) * 0.05);
        // The head parts: first turned, then eased between the poses.
        let head = equine_part(root, nested, "head_parts");
        head.x_rot = pi / 6.0 + head_rot_x;
        head.y_rot = clamped_y_rot * degrees;
        head.x_rot = standing * (pi / 12.0 + head_rot_x)
            + eating * (2.181_661_6 + sin(age) * 0.05)
            + base_head_angle;
        head.y_rot = standing * clamped_y_rot * degrees + (1.0 - standing.max(eating)) * head.y_rot;
        // `animateHeadPartsPlacement`.
        match kind {
            EquineKind::BabyHorse => {
                head.y += lerp(eating, lerp(standing, 0.0, -2.0), 2.0);
                head.z = lerp(standing, head.z, -4.0);
            }
            EquineKind::BabyDonkey => {
                head.y = lerp(eating, head.y, -1.2);
                head.z = lerp(standing, head.z, -3.6);
            }
            _ => {
                head.y += lerp(eating, lerp(standing, 0.0, -8.0), 7.0);
                head.z = lerp(standing, head.z, -4.0);
            }
        }
        let body = root.child_mut("body").expect("an equine has a body");
        body.x_rot = standing * (-pi / 4.0) + not_standing * body.x_rot;
        let left_front = equine_part(root, nested, "left_front_leg");
        left_front.y -= leg_standing_y * standing;
        left_front.z += leg_standing_z * standing;
        let (front_y, front_z) = (left_front.y, left_front.z);
        let right_front = equine_part(root, nested, "right_front_leg");
        right_front.y = front_y;
        right_front.z = front_z;
        let stand_angle = leg_stand_angle * standing;
        let bob = cos(age * 0.6 + pi);
        let right_leg_rot = (leg_standing_x_rot + bob) * standing + leg_x_rot_anim * not_standing;
        let left_leg_rot = (leg_standing_x_rot - bob) * standing - leg_x_rot_anim * not_standing;
        equine_part(root, nested, "left_hind_leg").x_rot =
            stand_angle - leg_anim * 0.5 * speed * not_standing;
        equine_part(root, nested, "right_hind_leg").x_rot =
            stand_angle + leg_anim * 0.5 * speed * not_standing;
        equine_part(root, nested, "left_front_leg").x_rot = right_leg_rot;
        equine_part(root, nested, "right_front_leg").x_rot = left_leg_rot;
        // `offsetLegPositionWhenStanding`: the baby donkey's hind legs drop a little.
        if nested {
            let left = equine_part(root, nested, "left_hind_leg");
            left.y = lerp(standing, left.y, -0.3);
            let left_y = left.y;
            equine_part(root, nested, "right_hind_leg").y = lerp(standing, left_y, -0.3);
        }
        let tail = root
            .child_mut("body")
            .and_then(|body| body.child_mut("tail"))
            .expect("an equine has a tail");
        tail.x_rot = tail_x_rot_offset + pi / 6.0 + speed * 0.75;
        tail.y += speed * state.age_scale;
        tail.z += speed * 2.0 * state.age_scale;
        tail.y_rot = if state.animate_tail {
            cos(age * 0.7)
        } else {
            0.0
        };
        if nested {
            // `BabyDonkeyModel.setupAnim`: the head tipped down at a fixed pitch.
            let head_rot_x = -30.0 * degrees;
            let base_head_angle =
                (1.0 - standing.max(eating)) * (pi / 6.0 + head_rot_x + feeding * sin(age) * 0.05);
            equine_part(root, nested, "head_parts").x_rot = standing * (pi / 12.0 + head_rot_x)
                + eating * (pi / 2.0 + sin(age) * 0.05)
                + base_head_angle;
        }
        // `DonkeyModel.setupAnim`: the chests show when the animal carries them.
        if matches!(kind, EquineKind::Donkey | EquineKind::BabyDonkey) {
            for name in ["left_chest", "right_chest"] {
                if let Some(chest) = root.child_mut("body").and_then(|body| body.child_mut(name)) {
                    chest.visible = state.has_chest;
                }
            }
        }
    }

    /// `CamelModel.setupAnim`: the head follows the look within limits (and
    /// dips while the dash cools down), then the walk and each animation state
    /// that has started.
    pub fn setup_camel(&mut self, state: &CamelState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let y_rot = state.quadruped.head_y_rot.clamp(-30.0, 30.0);
        let mut x_rot = state.quadruped.x_rot.clamp(-25.0, 45.0);
        if state.jump_cooldown > 0.0 {
            let head_rotation = 45.0 * state.jump_cooldown / 55.0;
            x_rot = (x_rot + head_rotation).clamp(-25.0, 70.0);
        }
        if let Some(head) = self
            .root
            .child_mut("body")
            .and_then(|body| body.child_mut("head"))
        {
            head.y_rot = y_rot * degrees;
            head.x_rot = x_rot * degrees;
        }
        let prefix = if state.baby {
            "CamelBaby.CAMEL_BABY"
        } else {
            "Camel.CAMEL"
        };
        let walk = voxel_data::animation::definition(&format!("{prefix}_WALK"));
        if let Some(definition) = walk {
            // `applyWalk(pos, speed, 2.0, 2.5)`.
            let millis = (state.quadruped.walk_position * 50.0 * 2.0) as i64;
            let scale = (state.quadruped.walk_speed * 2.5).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("SIT", state.sit),
            ("SIT_POSE", state.sit_pose),
            ("STANDUP", state.sit_up),
            ("IDLE", state.idle),
            ("DASH", state.dash),
        ] {
            if !running.is_started() {
                continue;
            }
            if let Some(definition) = voxel_data::animation::definition(&format!("{prefix}_{name}"))
            {
                let millis = (running.time_in_millis(state.age_in_ticks) as f32 * 1.0) as i64;
                self.animate(definition, millis, 1.0);
            }
        }
    }

    /// `LlamaModel.setupAnim` (and the baby's): the head follows the look, the
    /// legs swing in diagonal pairs, and the chests show on a loaded llama.
    pub fn setup_llama(&mut self, state: &QuadrupedState, has_chest: bool) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
        let (position, speed) = (state.walk_position, state.walk_speed);
        for (name, x_rot) in [
            ("right_hind_leg", cos(position * 0.6662) * 1.4 * speed),
            ("left_hind_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("right_front_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("left_front_leg", cos(position * 0.6662) * 1.4 * speed),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
        for name in ["right_chest", "left_chest"] {
            if let Some(chest) = self.root.child_mut(name) {
                chest.visible = has_chest;
            }
        }
    }

    /// `WolfModel.setupAnim` (grown and baby): the tail and legs by the walk
    /// (the legs folded while it sits), the body rolled by the shake, then
    /// the head follows the look and the tail rises.
    pub fn setup_wolf(&mut self, state: &WolfState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let scale = state.age_scale;
        let leg_swing = |offset: f32| cos(position * 0.6662 + offset) * 1.4 * speed;
        let tail_y_rot = if state.angry { 0.0 } else { leg_swing(0.0) };
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        set(self, "tail", &|tail| tail.y_rot = tail_y_rot);
        if state.sitting {
            set(self, "body", &|part| {
                part.y += 4.0 * scale;
                part.z -= 2.0 * scale;
                part.x_rot = (std::f64::consts::PI / 4.0) as f32;
                if state.baby {
                    part.x_rot -= (std::f64::consts::PI / 2.0) as f32;
                }
            });
            set(self, "tail", &|part| {
                part.y += 9.0 * scale;
                part.z -= 2.0 * scale;
            });
            for name in ["right_hind_leg", "left_hind_leg"] {
                set(self, name, &|part| {
                    part.y += 6.7 * scale;
                    part.z -= 5.0 * scale;
                    part.x_rot = (std::f64::consts::PI * 3.0 / 2.0) as f32;
                });
            }
            for (name, side) in [("right_front_leg", 1.0), ("left_front_leg", -1.0)] {
                set(self, name, &|part| {
                    part.x_rot = 5.811_947;
                    part.x += side * 0.01 * scale;
                    part.y += 1.0 * scale;
                });
            }
            if !state.baby {
                set(self, "upper_body", &|part| {
                    part.y += 2.0;
                    part.x_rot = (std::f64::consts::PI * 2.0 / 5.0) as f32;
                    part.y_rot = 0.0;
                });
            }
        } else {
            for (name, offset) in [
                ("right_hind_leg", 0.0),
                ("left_hind_leg", pi),
                ("right_front_leg", pi),
                ("left_front_leg", 0.0),
            ] {
                set(self, name, &|part| part.x_rot = leg_swing(offset));
            }
        }
        // `shakeOffWater`: the body rolls; the grown wolf's head, upper body
        // and tail roll in their own parts, the baby's whole head and tail.
        set(self, "body", &|part| {
            part.z_rot = state.body_roll_angle(-0.16)
        });
        if state.baby {
            set(self, "head", &|part| {
                part.z_rot = state.head_roll_angle + state.body_roll_angle(0.0);
            });
            set(self, "tail", &|part| {
                part.z_rot = state.body_roll_angle(-0.2)
            });
        } else {
            set(self, "head", &|head| {
                if let Some(real) = head.child_mut("real_head") {
                    real.z_rot = state.head_roll_angle + state.body_roll_angle(0.0);
                }
            });
            set(self, "upper_body", &|part| {
                part.z_rot = state.body_roll_angle(-0.08)
            });
            set(self, "tail", &|tail| {
                if let Some(real) = tail.child_mut("real_tail") {
                    real.z_rot = state.body_roll_angle(-0.2);
                }
            });
        }
        set(self, "head", &|head| {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        });
        set(self, "tail", &|tail| tail.x_rot = state.tail_angle);
    }

    /// `AdultFelineModel.setupAnim` and `BabyFelineModel.setupAnim`: crouching
    /// or sprinting lifts the tail and lowers the body, the legs swing with
    /// the walk (not while it sits), and lying down and relaxing turn the
    /// head and fold the legs.
    pub fn setup_feline(&mut self, state: &FelineState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let half_pi = (std::f64::consts::PI / 2.0) as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let scale = state.age_scale;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.crouching {
            set(self, "body", &|part| part.y += scale);
            set(self, "head", &|part| part.y += 2.0 * scale);
            set(self, "tail1", &|part| {
                part.y += scale;
                part.x_rot = half_pi;
            });
            set(self, "tail2", &|part| {
                part.y += -4.0 * scale;
                part.z += 2.0 * scale;
                part.x_rot = half_pi;
            });
        } else if state.sprinting {
            let tail1_y = self.root.child_mut("tail1").map_or(0.0, |part| part.y);
            set(self, "tail1", &|part| part.x_rot = half_pi);
            set(self, "tail2", &|part| {
                part.y = tail1_y;
                part.z += 2.0 * scale;
                part.x_rot = half_pi;
            });
        }
        set(self, "head", &|head| {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        });
        if !state.sitting {
            if !state.baby {
                set(self, "body", &|part| part.x_rot = half_pi);
            }
            let swing = |offset: f32| cos(position * 0.6662 + offset) * speed;
            if state.sprinting {
                set(self, "left_hind_leg", &|part| part.x_rot = swing(0.0));
                set(self, "right_hind_leg", &|part| part.x_rot = swing(0.3));
                set(self, "left_front_leg", &|part| {
                    part.x_rot = cos(position * 0.6662 + pi + 0.3) * speed;
                });
                set(self, "right_front_leg", &|part| part.x_rot = swing(pi));
                set(self, "tail2", &|part| {
                    part.x_rot =
                        1.727_876_1 + (std::f64::consts::PI / 10.0) as f32 * cos(position) * speed;
                });
            } else {
                set(self, "left_hind_leg", &|part| part.x_rot = swing(0.0));
                set(self, "right_hind_leg", &|part| part.x_rot = swing(pi));
                set(self, "left_front_leg", &|part| part.x_rot = swing(pi));
                set(self, "right_front_leg", &|part| part.x_rot = swing(0.0));
                let sway = if state.crouching {
                    0.471_238_94
                } else {
                    (std::f64::consts::PI / 4.0) as f32
                };
                set(self, "tail2", &|part| {
                    part.x_rot = 1.727_876_1 + sway * cos(position) * speed;
                });
            }
        } else {
            self.setup_feline_sitting(state);
        }
        self.setup_feline_lying(state);
        if state.relax_state_one_amount > 0.0 {
            set(self, "head", &|part| {
                part.x_rot = rot_lerp(state.relax_state_one_amount, part.x_rot, -0.581_776_44);
            });
        }
    }

    /// The sitting part of `setup_feline`: the baby's small shifts, or the
    /// grown cat's folded pose.
    fn setup_feline_sitting(&mut self, state: &FelineState) {
        let scale = state.age_scale;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.baby {
            set(self, "body", &|part| {
                part.x_rot += -0.436_332_32;
                part.y += 1.25;
            });
            set(self, "head", &|part| part.z += 0.75);
            set(self, "tail1", &|part| {
                part.x_rot += 0.545_415_4;
                part.y += 4.0;
                part.z -= 0.9;
            });
            for name in ["left_hind_leg", "right_hind_leg"] {
                set(self, name, &|part| part.z -= 0.9);
            }
        } else {
            set(self, "body", &|part| {
                part.x_rot = (std::f64::consts::PI / 4.0) as f32;
                part.y += -4.0 * scale;
                part.z += 5.0 * scale;
            });
            set(self, "head", &|part| {
                part.y += -3.3 * scale;
                part.z += scale;
            });
            set(self, "tail1", &|part| {
                part.y += 8.0 * scale;
                part.z += -2.0 * scale;
                part.x_rot = 1.727_876_1;
            });
            set(self, "tail2", &|part| {
                part.y += 2.0 * scale;
                part.z += -0.8 * scale;
                part.x_rot = 2.670_354;
            });
            for name in ["left_front_leg", "right_front_leg"] {
                set(self, name, &|part| {
                    part.x_rot = (-std::f64::consts::PI / 20.0) as f32;
                    part.y += 2.0 * scale;
                    part.z -= 2.0 * scale;
                });
            }
            for name in ["left_hind_leg", "right_hind_leg"] {
                set(self, name, &|part| {
                    part.x_rot = (-std::f64::consts::PI / 2.0) as f32;
                    part.y += 3.0 * scale;
                    part.z -= 4.0 * scale;
                });
            }
        }
    }

    /// The lying-down part of `setup_feline`: the baby's small shifts, or the
    /// grown cat's turned head and folded legs and tail.
    fn setup_feline_lying(&mut self, state: &FelineState) {
        let half_pi = (std::f64::consts::PI / 2.0) as f32;
        let scale = state.age_scale;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        let (lie, tail) = (state.lie_down_amount, state.lie_down_amount_tail);
        if lie > 0.0 && state.baby {
            set(self, "body", &|part| part.x += 1.0);
            set(self, "head", &|part| {
                part.x_rot = rot_lerp(lie, part.x_rot, (std::f64::consts::PI / 18.0) as f32);
                part.z_rot = rot_lerp(lie, part.z_rot, (-std::f64::consts::PI * 5.0 / 12.0) as f32);
                part.x += 1.5;
                part.y += 0.75;
                part.z -= 0.5;
            });
            set(self, "right_front_leg", &|part| {
                part.x_rot = (-std::f64::consts::PI / 4.0) as f32;
                part.x += 3.5;
                part.y -= 0.5;
            });
            set(self, "left_front_leg", &|part| {
                part.x_rot = -half_pi;
                part.x += 1.5;
                part.y -= 1.0;
                part.z -= 2.0;
            });
            set(self, "right_hind_leg", &|part| {
                part.x_rot = (std::f64::consts::PI * 2.0 / 9.0) as f32;
                part.y_rot = (std::f64::consts::PI / 9.0) as f32;
                part.z_rot = (-std::f64::consts::PI / 9.0) as f32;
                part.x += 2.5;
                part.y -= 0.25;
                part.z += 0.5;
            });
            set(self, "left_hind_leg", &|part| {
                part.x += 1.5;
                part.z -= 1.0;
            });
            set(self, "tail1", &|part| {
                part.x_rot += rot_lerp(tail, part.x_rot, (-std::f64::consts::PI / 6.0) as f32);
                part.y_rot += rot_lerp(tail, part.y_rot, 0.0);
                part.z_rot += rot_lerp(tail, part.z_rot, (-std::f64::consts::PI / 18.0) as f32);
                part.x += 1.0;
                part.y += 0.5;
                part.z -= 0.25;
            });
        } else if lie > 0.0 {
            set(self, "head", &|part| {
                part.z_rot = rot_lerp(lie, part.z_rot, -1.270_796_3);
                part.y_rot = rot_lerp(lie, part.y_rot, 1.270_796_3);
            });
            set(self, "left_front_leg", &|part| part.x_rot = -1.270_796_3);
            set(self, "right_front_leg", &|part| {
                part.x_rot = -0.470_796_35;
                part.z_rot = -0.2;
                part.x += scale;
            });
            set(self, "left_hind_leg", &|part| part.x_rot = -0.4);
            set(self, "right_hind_leg", &|part| {
                part.x_rot = 0.5;
                part.z_rot = -0.5;
                part.x += 0.8 * scale;
                part.y += 2.0 * scale;
            });
            set(self, "tail1", &|part| {
                part.x_rot = rot_lerp(tail, part.x_rot, 0.8);
            });
            set(self, "tail2", &|part| {
                part.x_rot = rot_lerp(tail, part.x_rot, -0.4);
            });
        }
    }

    /// `PandaModel.setupAnim` (and `BabyPandaModel`'s sitting): the
    /// quadruped's pose, then the unhappy shake, the sneeze, sitting (and
    /// eating or fright), lying on the back and the roll.
    pub fn setup_panda(&mut self, state: &PandaState) {
        self.setup_quadruped(&state.quadruped);
        let pi = std::f64::consts::PI as f32;
        let age = state.age_in_ticks;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.unhappy {
            set(self, "head", &|head| {
                head.y_rot = 0.35 * sin(0.6 * age);
                head.z_rot = 0.35 * sin(0.6 * age);
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = -0.75 * sin(0.3 * age)
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = 0.75 * sin(0.3 * age)
            });
        } else {
            set(self, "head", &|head| head.z_rot = 0.0);
        }
        if state.sneezing {
            let time = state.sneeze_time;
            if time < 15 {
                set(self, "head", &|head| {
                    head.x_rot = -pi / 4.0 * time as f32 / 14.0;
                });
            } else if time < 20 {
                // The reference divides these as ints.
                let position = ((time - 15) / 5) as f32;
                set(self, "head", &|head| {
                    head.x_rot = -pi / 4.0 + pi / 4.0 * position;
                });
            }
        }
        if state.sit_amount > 0.0 {
            self.panda_sit(state);
            if state.eating {
                set(self, "head", &|head| {
                    head.x_rot = pi / 2.0 + 0.2 * sin(age * 0.6);
                });
                for name in ["right_front_leg", "left_front_leg"] {
                    set(self, name, &|leg| leg.x_rot = -0.4 - 0.2 * sin(age * 0.6));
                }
            }
            if state.scared {
                set(self, "head", &|head| head.x_rot = 2.170_796_4);
                for name in ["right_front_leg", "left_front_leg"] {
                    set(self, name, &|leg| leg.x_rot = -0.9);
                }
            }
        } else {
            for name in LEGS {
                set(self, name, &|leg| leg.z_rot = 0.0);
            }
        }
        if state.lie_on_back_amount > 0.0 {
            let amount = state.lie_on_back_amount;
            set(self, "right_hind_leg", &|leg| {
                leg.x_rot = -0.6 * sin(age * 0.15)
            });
            set(self, "left_hind_leg", &|leg| {
                leg.x_rot = 0.6 * sin(age * 0.15)
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = 0.3 * sin(age * 0.25)
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = -0.3 * sin(age * 0.25)
            });
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, pi / 2.0)
            });
        }
        if state.roll_amount > 0.0 {
            let amount = state.roll_amount;
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, 2.056_194_5)
            });
            set(self, "right_hind_leg", &|leg| {
                leg.x_rot = -0.5 * sin(age * 0.5)
            });
            set(self, "left_hind_leg", &|leg| {
                leg.x_rot = 0.5 * sin(age * 0.5)
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = 0.5 * sin(age * 0.5)
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = -0.5 * sin(age * 0.5)
            });
        }
    }

    /// `PandaModel.animateSitting` and the baby's own.
    fn panda_sit(&mut self, state: &PandaState) {
        let pi = std::f64::consts::PI as f32;
        let amount = state.sit_amount;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.baby {
            set(self, "body", &|body| {
                body.x_rot = rot_lerp_rad(amount, body.x_rot, (std::f64::consts::PI / 18.0) as f32);
                body.z = lerp_f32(amount, body.z, -1.5);
            });
            set(self, "head", &|head| {
                head.z = lerp_f32(amount, head.z, -11.5);
                head.y = lerp_f32(amount, head.y, 17.5);
            });
            for (name, z) in [
                ("right_front_leg", -5.0),
                ("left_front_leg", -5.0),
                ("right_hind_leg", 3.0),
                ("left_hind_leg", 3.0),
            ] {
                set(self, name, &|leg| leg.z = lerp_f32(amount, leg.z, z));
            }
        } else {
            set(self, "body", &|body| {
                body.x_rot = rot_lerp_rad(amount, body.x_rot, 1.740_796_3);
            });
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, pi / 2.0)
            });
        }
        set(self, "right_front_leg", &|leg| leg.z_rot = -0.270_796_42);
        set(self, "left_front_leg", &|leg| leg.z_rot = 0.270_796_42);
        set(self, "right_hind_leg", &|leg| leg.z_rot = 0.570_796_4);
        set(self, "left_hind_leg", &|leg| leg.z_rot = -0.570_796_4);
    }

    /// `FoxModel.setupAnim` (and `AdultFoxModel`'s and `BabyFoxModel`'s): the
    /// walk, then the crouching, sleeping or sitting pose, the pounce, the
    /// head's look (not while it sleeps, crouches or has its face planted)
    /// and the legs of a faceplanted fox.
    pub fn setup_fox(&mut self, state: &FoxState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        // `setWalkingPose`.
        set(self, "head", &|head| head.z_rot = state.head_roll_angle);
        for name in LEGS {
            set(self, name, &|leg| leg.visible = true);
        }
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        if state.baby {
            if let Some(definition) = voxel_data::animation::definition("FoxBaby.FOX_BABY_WALK") {
                // `applyWalk(pos, speed, 1.0, 2.5)`.
                self.animate(definition, (position * 50.0) as i64, (speed * 2.5).min(1.0));
            }
        } else {
            for (name, offset) in [
                ("right_hind_leg", 0.0),
                ("left_hind_leg", pi),
                ("right_front_leg", pi),
                ("left_front_leg", 0.0),
            ] {
                set(self, name, &|leg| {
                    leg.x_rot = cos(position * 0.6662 + offset) * 1.4 * speed;
                });
            }
        }
        if state.crouching {
            self.fox_crouch(state);
        } else if state.sleeping {
            self.fox_sleep(state);
        } else if state.sitting {
            self.fox_sit(state);
        }
        if state.pouncing && !state.baby {
            let crouch = state.crouch_amount / 2.0;
            set(self, "body", &|body| body.y -= crouch);
            set(self, "head", &|head| head.y -= crouch);
        }
        if !state.sleeping && !state.faceplanted && !state.crouching {
            set(self, "head", &|head| {
                head.x_rot = state.quadruped.x_rot * degrees;
                head.y_rot = state.quadruped.head_y_rot * degrees;
            });
        }
        if state.sleeping {
            set(self, "head", &|head| {
                head.x_rot = 0.0;
                head.y_rot = (-std::f64::consts::PI * 2.0 / 3.0) as f32;
                head.z_rot = cos(state.age_in_ticks * 0.027) / 22.0;
            });
        }
        if state.faceplanted {
            self.leg_motion_pos += 0.67;
            let phase = self.leg_motion_pos * 0.4662;
            for (name, offset) in [
                ("right_hind_leg", 0.0),
                ("left_hind_leg", pi),
                ("right_front_leg", pi),
                ("left_front_leg", 0.0),
            ] {
                set(self, name, &|leg| leg.x_rot = cos(phase + offset) * 0.1);
            }
        }
    }

    /// `FoxModel.setCrouchingPose` and the model's own: the body sinks and
    /// the tail and legs wiggle.
    fn fox_crouch(&mut self, state: &FoxState) {
        let wiggle = mth_cos_f64(f64::from(state.age_in_ticks)) * 0.05;
        let (crouch, scale) = (state.crouch_amount, state.age_scale);
        let sink = if state.baby { crouch / 6.0 } else { crouch };
        if let Some(body) = self.root.child_mut("body") {
            body.x_rot += 0.104_719_76;
            body.y_rot = wiggle;
            body.y += sink;
        }
        if let Some(head) = self.root.child_mut("head") {
            head.y += crouch * scale;
        }
        for (name, roll) in [
            ("right_hind_leg", wiggle),
            ("left_hind_leg", wiggle),
            ("right_front_leg", wiggle / 2.0),
            ("left_front_leg", wiggle / 2.0),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.z_rot = roll;
            }
        }
    }

    /// `FoxModel.setSleepingPose` and the model's own: the legs are hidden
    /// and the fox curls up on its side.
    fn fox_sleep(&mut self, state: &FoxState) {
        let pi = std::f64::consts::PI;
        for name in LEGS {
            if let Some(leg) = self.root.child_mut(name) {
                leg.visible = false;
            }
        }
        let baby = state.baby;
        if let Some(body) = self.root.child_mut("body") {
            body.z_rot = (-pi / 2.0) as f32;
            if baby {
                body.x_rot = (-pi / 18.0) as f32;
                body.y += 1.5;
                body.z -= 1.5;
                body.x -= 1.5;
            } else {
                body.y += 5.0;
            }
            if let Some(tail) = body.child_mut("tail") {
                if baby {
                    tail.x_rot = -2.181_661_6;
                    tail.x -= 0.7;
                    tail.z += 0.6;
                    tail.y += 0.9;
                } else {
                    tail.x_rot = (-pi * 5.0 / 6.0) as f32;
                }
            }
        }
        if let Some(head) = self.root.child_mut("head") {
            if baby {
                head.x -= 2.0;
                head.y += 2.8;
                head.z -= 4.0;
            } else {
                head.x += 2.0;
                head.y += 2.99;
            }
            head.y_rot = (-pi * 2.0 / 3.0) as f32;
            head.z_rot = 0.0;
        }
    }

    /// `FoxModel.setSittingPose` and the model's own: the head faces front
    /// and the body tips back onto the haunches.
    fn fox_sit(&mut self, state: &FoxState) {
        let pi = std::f64::consts::PI;
        let scale = state.age_scale;
        let front = (-pi / 12.0) as f32;
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = 0.0;
            head.y_rot = 0.0;
            if state.baby {
                head.y -= 0.75;
            } else {
                head.y -= 6.5;
                head.z += 2.75;
            }
        }
        if let Some(body) = self.root.child_mut("body") {
            if state.baby {
                body.x_rot = -0.959_931;
                body.z -= 4.5 * scale;
                body.y += 3.0 * scale;
            } else {
                body.x_rot = (pi / 6.0) as f32;
                body.y -= 7.0;
                body.z += 3.0;
            }
            if let Some(tail) = body.child_mut("tail") {
                if state.baby {
                    tail.y -= 0.6;
                    tail.z -= 2.0 * scale;
                    tail.x_rot = 0.959_931_14;
                } else {
                    tail.x_rot = (pi / 4.0) as f32;
                    tail.z -= 1.0;
                }
            }
        }
        for (name, side) in [("right_front_leg", 1.0), ("left_front_leg", -1.0)] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = front;
                if state.baby {
                    leg.z -= 1.5;
                    leg.x += side * 0.01;
                }
            }
        }
        for (name, side) in [("right_hind_leg", 1.0), ("left_hind_leg", -1.0)] {
            if let Some(leg) = self.root.child_mut(name) {
                if state.baby {
                    leg.z -= 3.75;
                    leg.x += side * 0.01;
                } else {
                    leg.x_rot = (-pi * 5.0 / 12.0) as f32;
                    leg.y += 4.0;
                    leg.z -= 0.25;
                }
            }
        }
    }

    /// `StriderModel.setupAnim` (and `AdultStriderModel`'s and
    /// `BabyStriderModel`'s own part): the body turns with the look unless it
    /// is ridden and rocks with the walk, the legs swing, and the bristles
    /// flow and stir.
    pub fn setup_strider(&mut self, state: &StriderState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let position = state.quadruped.walk_position;
        let speed = state.quadruped.walk_speed.min(0.25);
        let age = state.age_in_ticks;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        set(self, "body", &|body| {
            if state.ridden {
                body.x_rot = 0.0;
                body.y_rot = 0.0;
            } else {
                body.x_rot = state.quadruped.x_rot * degrees;
                body.y_rot = state.quadruped.head_y_rot * degrees;
            }
            body.z_rot = 0.1 * sin(position * 1.5) * 4.0 * speed;
        });
        let half = position * 1.5 * 0.5;
        set(self, "left_leg", &|leg| {
            leg.x_rot = sin(half) * 2.0 * speed;
            leg.z_rot = (std::f64::consts::PI / 18.0) as f32 * cos(half) * speed;
        });
        set(self, "right_leg", &|leg| {
            leg.x_rot = sin(half + pi) * 2.0 * speed;
            leg.z_rot = (std::f64::consts::PI / 18.0) as f32 * cos(half + pi) * speed;
        });
        let flow = cos(position * 1.5 + pi) * speed;
        let leg_swing = |sign: f32| 2.0 * sin(half + sign) * 2.0 * speed;
        if state.baby {
            set(self, "body", &|body| {
                body.y = 17.25;
                body.y -= 1.0 * cos(position * 1.5) * 2.0 * speed;
                for (name, scale, stir) in [
                    ("bristle2", 0.6, 0.1 * sin(age * 0.4)),
                    ("bristle1", 1.2, 0.1 * sin(age * 0.2)),
                    ("bristle0", 1.3, 0.05 * sin(age * -0.4)),
                ] {
                    if let Some(bristle) = body.child_mut(name) {
                        bristle.x_rot += flow * scale;
                        bristle.x_rot += stir;
                    }
                }
            });
            set(self, "left_leg", &|leg| leg.y = 20.0 + leg_swing(pi));
            set(self, "right_leg", &|leg| leg.y = 20.0 + leg_swing(0.0));
        } else {
            set(self, "body", &|body| {
                body.y = 2.0;
                body.y -= 2.0 * cos(position * 1.5) * 2.0 * speed;
                // The sides' bristles rest turned, then take the flow and the stirs.
                for (name, rest, scale, stir) in [
                    (
                        "right_bottom_bristle",
                        -1.221_730_5,
                        1.3,
                        0.05 * sin(age * -0.4),
                    ),
                    (
                        "right_middle_bristle",
                        -1.134_464,
                        1.2,
                        0.1 * sin(age * 0.2),
                    ),
                    (
                        "right_top_bristle",
                        -0.872_664_63,
                        0.6,
                        0.1 * sin(age * 0.4),
                    ),
                    ("left_top_bristle", 0.872_664_63, 0.6, 0.1 * sin(age * 0.4)),
                    ("left_middle_bristle", 1.134_464, 1.2, 0.1 * sin(age * 0.2)),
                    (
                        "left_bottom_bristle",
                        1.221_730_5,
                        1.3,
                        0.05 * sin(age * -0.4),
                    ),
                ] {
                    if let Some(bristle) = body.child_mut(name) {
                        bristle.z_rot = rest;
                        bristle.z_rot += flow * scale;
                        bristle.z_rot += stir;
                    }
                }
            });
            set(self, "left_leg", &|leg| leg.y = 8.0 + leg_swing(pi));
            set(self, "right_leg", &|leg| leg.y = 8.0 + leg_swing(0.0));
        }
    }

    /// `RavagerModel.setupAnim`: the neck thrusts and the mouth snaps in the
    /// attack; otherwise the neck rests, sways while stunned, and the mouth
    /// opens in the roar; the head follows the look and the legs swing.
    pub fn setup_ravager(&mut self, state: &RavagerState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (stunned, attack) = (state.stunned_ticks_remaining, state.attack_ticks_remaining);
        let Some(neck_x_rot) = self.root.child_mut("neck").map(|neck| neck.x_rot) else {
            return;
        };
        let mut neck: (f32, f32, f32);
        let mut neck_x = None;
        let mouth_x_rot;
        if attack > 0.0 {
            // `Mth.triangleWave(attackTick, 10)`.
            let wave = ((attack % 10.0 - 10.0 * 0.5).abs() - 10.0 * 0.25) / (10.0 * 0.25);
            let scaled = (1.0 + wave) * 0.5;
            let head_pos = scaled * scaled * scaled * 12.0;
            let y_offset = head_pos * sin(neck_x_rot);
            neck = (0.0, -7.0 - y_offset, -6.5 + head_pos);
            mouth_x_rot = if attack > 5.0 {
                sin((-4.0 + attack) / 4.0) * pi * 0.4
            } else {
                (std::f64::consts::PI / 20.0) as f32 * sin(pi * attack / 10.0)
            };
        } else {
            let y_offset = -sin(neck_x_rot);
            neck = (0.0, -7.0 - y_offset, 5.5);
            let is_stunned = stunned > 0.0;
            neck_x = Some(if is_stunned { 0.219_911_49 } else { 0.0 });
            let mut mouth = pi * if is_stunned { 0.05 } else { 0.01 };
            if is_stunned {
                let speed = f64::from(stunned) / 40.0;
                neck.0 = (speed * 10.0).sin() as f32 * 3.0;
            } else if state.roar_animation > 0.0 {
                let mouth_anim = sin(state.roar_animation * pi * 0.25);
                mouth = (std::f64::consts::PI / 2.0) as f32 * mouth_anim;
            }
            mouth_x_rot = mouth;
        }
        if let Some(part) = self.root.child_mut("neck") {
            (part.x, part.y, part.z) = neck;
            if let Some(x_rot) = neck_x {
                part.x_rot = x_rot;
            }
            if let Some(head) = part.child_mut("head") {
                head.x_rot = state.quadruped.x_rot * degrees;
                head.y_rot = state.quadruped.head_y_rot * degrees;
                if let Some(mouth) = head.child_mut("mouth") {
                    mouth.x_rot = mouth_x_rot;
                }
            }
        }
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let leg_rot = 0.4 * speed;
        for (name, offset) in [
            ("right_hind_leg", 0.0),
            ("left_hind_leg", pi),
            ("right_front_leg", pi),
            ("left_front_leg", 0.0),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = cos(position * 0.6662 + offset) * leg_rot;
            }
        }
    }

    /// `HoglinModel.setupAnim` (and `BabyHoglinModel`'s): the ears flap with
    /// the walk, the head turns and butts by the attack swing (a baby's
    /// lowers with it), and the legs swing in diagonal pairs.
    pub fn setup_hoglin(&mut self, state: &HoglinState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let (position, speed) = (state.walk_position, state.walk_speed);
        let ear_swing = speed * mth_sin_f64(f64::from(position));
        let ear_rest = (std::f64::consts::PI * 2.0 / 9.0) as f32;
        let butt = 1.0 - ((10 - 2 * state.attack_ticks).abs() as f32) / 10.0;
        if let Some(head) = self.root.child_mut("head") {
            if let Some(ear) = head.child_mut("right_ear") {
                ear.z_rot = -ear_rest - ear_swing;
            }
            if let Some(ear) = head.child_mut("left_ear") {
                ear.z_rot = ear_rest + ear_swing;
            }
            head.y_rot = state.head_y_rot * degrees;
            head.x_rot = lerp_f32(butt, 0.872_664_63, (-std::f64::consts::PI / 9.0) as f32);
            if state.baby {
                head.y += butt * 2.5;
            }
        }
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let right_front = cos(position) * 1.2 * speed;
        let left_front = cos(position + pi) * 1.2 * speed;
        for (name, x_rot) in [
            ("right_front_leg", right_front),
            ("left_front_leg", left_front),
            ("right_hind_leg", left_front),
            ("left_hind_leg", right_front),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
    }

    /// `PolarBearModel.setupAnim` (grown and baby): the quadruped's pose, then
    /// standing up lifts the body and head and raises the front legs.
    pub fn setup_polar_bear(&mut self, state: &PolarBearState) {
        self.setup_quadruped(&state.quadruped);
        let pi = std::f64::consts::PI as f32;
        let stand = state.stand_scale * state.stand_scale;
        let age = state.age_scale;
        let (mut leg_y, mut leg_z) = (0.0, 0.0);
        if let Some(leg) = self.root.child_mut("right_front_leg") {
            leg.y -= stand * age * 20.0;
            leg.z += stand * age * 4.0;
            leg.x_rot -= stand * pi * 0.45;
            (leg_y, leg_z) = (leg.y, leg.z);
        }
        if let Some(leg) = self.root.child_mut("left_front_leg") {
            leg.y = leg_y;
            leg.z = leg_z;
            leg.x_rot -= stand * pi * 0.45;
        }
        if let Some(body) = self.root.child_mut("body") {
            body.x_rot -= stand * pi * 0.35;
            body.y += stand * age * 2.0;
        }
        if let Some(head) = self.root.child_mut("head") {
            head.y -= stand * 24.0;
            head.z += stand * 13.0;
            head.x_rot += stand * pi * 0.15;
        }
    }

    /// `GoatModel.setupAnim` (and `BabyGoatModel`'s): the quadruped's pose, the
    /// horns the goat still has, and the head lowered to ram; a baby's head
    /// tips forward when it is not ramming.
    pub fn setup_goat(&mut self, state: &GoatState) {
        self.setup_quadruped(&state.quadruped);
        if let Some(head) = self.root.child_mut("head") {
            for (name, visible) in [
                ("left_horn", state.has_left_horn),
                ("right_horn", state.has_right_horn),
            ] {
                if let Some(horn) = head.child_mut(name) {
                    horn.visible = visible;
                }
            }
            if state.ramming_x_head_rot != 0.0 {
                head.x_rot = state.ramming_x_head_rot;
            } else if state.baby {
                head.x_rot = (std::f64::consts::PI / 8.0) as f32;
            }
        }
    }

    /// `SheepModel.setupAnim` (and the wool's): the quadruped's pose, with
    /// the head lowered and tipped to eat.
    pub fn setup_sheep(&mut self, state: &SheepState) {
        self.setup_quadruped(&state.quadruped);
        if let Some(head) = self.root.child_mut("head") {
            head.y += state.head_eat_position_scale * 9.0 * state.age_scale;
            head.x_rot = state.head_eat_angle_scale;
        }
    }

    /// `ChickenModel.setupAnim`: the legs swing with the walk, the wings beat
    /// with the flap, and a grown chicken's head follows the look.
    pub fn setup_chicken(&mut self, state: &ChickenState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let flap_angle = (sin(state.flap) + 1.0) * state.flap_speed;
        let (position, speed) = (state.walk_position, state.walk_speed);
        let moves = [
            (
                "right_leg",
                Some(cos(position * 0.6662) * 1.4 * speed),
                None,
            ),
            (
                "left_leg",
                Some(cos(position * 0.6662 + pi) * 1.4 * speed),
                None,
            ),
            ("right_wing", None, Some(flap_angle)),
            ("left_wing", None, Some(-flap_angle)),
        ];
        for (name, x_rot, z_rot) in moves {
            if let Some(part) = self.root.child_mut(name) {
                if let Some(x_rot) = x_rot {
                    part.x_rot = x_rot;
                }
                if let Some(z_rot) = z_rot {
                    part.z_rot = z_rot;
                }
            }
        }
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
    }

    /// `ZombieModel.setupAnim` (and `DrownedModel.setupAnim`): the humanoid
    /// pose, then the arms held out in front.
    pub fn setup_zombie(&mut self, state: &ZombieState) {
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        humanoid_anim(&mut self.root, &state.humanoid);
        self.animate_zombie_arms(&state.humanoid, state.aggressive);
        let humanoid = &state.humanoid;
        let swim = humanoid.swim_amount;
        if state.drowned && swim > 0.0 {
            let age = humanoid.age_in_ticks;
            let sway = swim * 0.35 * sin(0.1 * age);
            let kick = swim * 0.55 * sin(0.1 * age);
            let reach = (-std::f64::consts::PI * 4.0 / 5.0) as f32;
            if let Some(arm) = self.root.child_mut("right_arm") {
                arm.x_rot = rot_lerp_rad(swim, arm.x_rot, reach) + sway;
                arm.z_rot = rot_lerp_rad(swim, arm.z_rot, -0.15);
            }
            if let Some(arm) = self.root.child_mut("left_arm") {
                arm.x_rot = rot_lerp_rad(swim, arm.x_rot, reach) - sway;
                arm.z_rot = rot_lerp_rad(swim, arm.z_rot, 0.15);
            }
            if let Some(leg) = self.root.child_mut("left_leg") {
                leg.x_rot -= kick;
            }
            if let Some(leg) = self.root.child_mut("right_leg") {
                leg.x_rot += kick;
            }
            if let Some(head) = self.root.child_mut("head") {
                head.x_rot = 0.0;
            }
        }
    }

    /// `AnimationUtils.animateZombieArms` with the arms raised and the whack
    /// swing: `animateAttackArms` over a drop that depends on the flag, then
    /// the arms' breathing again.
    fn animate_zombie_arms(&mut self, humanoid: &PlayerRenderState, aggressive: bool) {
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let pi = std::f64::consts::PI as f32;
        let arm_drop = -pi / if aggressive { 1.5 } else { 2.25 };
        let attack_time = humanoid.attack_time;
        let y_modifier = sin(attack_time * pi);
        let x_modifier = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
        let x_rot = arm_drop + y_modifier * 1.2 - x_modifier * 0.4;
        let y_rot = 0.1 - y_modifier * 0.6;
        for (name, y, bob) in [("right_arm", -y_rot, 1.0_f32), ("left_arm", y_rot, -1.0)] {
            if let Some(arm) = self.root.child_mut(name) {
                arm.x_rot = x_rot;
                arm.y_rot = y;
                arm.z_rot = 0.0;
                // `bobArms`.
                arm.z_rot += bob * (cos(humanoid.age_in_ticks * 0.09) * 0.05 + 0.05);
                arm.x_rot += bob * (sin(humanoid.age_in_ticks * 0.067) * 0.05);
            }
        }
    }

    /// `SkeletonModel.setupAnim`: the humanoid pose, and an aggressive
    /// skeleton without a bow swings its arms out in front.
    pub fn setup_skeleton(&mut self, state: &SkeletonState) {
        humanoid_anim(&mut self.root, &state.humanoid);
        if !state.aggressive || state.holding_bow {
            return;
        }
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let pi = std::f64::consts::PI as f32;
        let attack_time = state.humanoid.attack_time;
        let attack_2 = sin(attack_time * pi);
        let attack = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
        let half_pi = (-std::f64::consts::PI / 2.0) as f32;
        for (name, y, bob) in [
            ("right_arm", -(0.1 - attack_2 * 0.6), 1.0_f32),
            ("left_arm", 0.1 - attack_2 * 0.6, -1.0),
        ] {
            if let Some(arm) = self.root.child_mut(name) {
                arm.z_rot = 0.0;
                arm.y_rot = y;
                arm.x_rot = half_pi;
                arm.x_rot -= attack_2 * 1.2 - attack * 0.4;
                arm.z_rot += bob * (cos(state.humanoid.age_in_ticks * 0.09) * 0.05 + 0.05);
                arm.x_rot += bob * (sin(state.humanoid.age_in_ticks * 0.067) * 0.05);
            }
        }
    }

    /// `AbstractPiglinModel.setupAnim` and what `PiglinModel` and
    /// `ZombifiedPiglinModel` add: the ears flap with the walk, a zombified
    /// piglin holds its arms out, and a dancing piglin sways.
    pub fn setup_piglin(&mut self, state: &PiglinState) {
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let pi = std::f64::consts::PI as f32;
        let humanoid = &state.humanoid;
        humanoid_anim(&mut self.root, humanoid);
        let default_angle =
            if state.baby { 5.0_f32 } else { 30.0 } * (std::f64::consts::PI / 180.0) as f32;
        let frequency = humanoid.age_in_ticks * 0.1 + humanoid.walk_position * 0.5;
        let amplitude = 0.08 + humanoid.walk_speed * 0.4;
        if let Some(head) = self.root.child_mut("head") {
            if let Some(ear) = head.child_mut("left_ear") {
                ear.z_rot = -default_angle - cos(frequency * 1.2) * amplitude;
            }
            if let Some(ear) = head.child_mut("right_ear") {
                ear.z_rot = default_angle + cos(frequency) * amplitude;
            }
        }
        if state.zombified {
            self.animate_zombie_arms(humanoid, state.aggressive);
        } else if state.dancing {
            let dance = humanoid.age_in_ticks / 60.0;
            let degrees = (std::f64::consts::PI / 180.0) as f32;
            if let Some(head) = self.root.child_mut("head") {
                if let Some(ear) = head.child_mut("right_ear") {
                    ear.z_rot = pi / 6.0 + degrees * sin(dance * 30.0) * 10.0;
                }
                if let Some(ear) = head.child_mut("left_ear") {
                    ear.z_rot = -pi / 6.0 - degrees * cos(dance * 30.0) * 10.0;
                }
                head.x += sin(dance * 10.0);
                head.y += sin(dance * 40.0) + 0.4;
            }
            let right_z = degrees * (70.0 + cos(dance * 40.0) * 10.0);
            if let Some(arm) = self.root.child_mut("right_arm") {
                arm.z_rot = right_z;
                arm.y += sin(dance * 40.0) * 0.5 - 0.5;
            }
            if let Some(arm) = self.root.child_mut("left_arm") {
                arm.z_rot = -right_z;
                arm.y += sin(dance * 40.0) * 0.5 + 0.5;
            }
            if let Some(body) = self.root.child_mut("body") {
                body.y += sin(dance * 40.0) * 0.35;
            }
        }
    }

    /// `ModelPart.render` of the whole model.
    pub fn render(&self, pose: &Pose) -> Vec<(String, [ModelVertex; 4])> {
        let mut out = Vec::new();
        self.root.render(pose, "", &mut out);
        out
    }
}

/// Where a mob is and how it is lit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobPlacement {
    /// The feet, from the mesh origin.
    pub feet: [f32; 3],
    /// The body's facing, in degrees.
    pub body_rot: f32,
    /// The entity's scale (the attribute; one unless the server says other).
    pub scale: f32,
    /// Packed block and sky light at the mob.
    pub light: i32,
    /// `LivingEntityRenderState.deathTime`: zero while it lives.
    pub death_time: f32,
    /// `LivingEntityRenderer.getFlipDegrees`: how far a dead mob falls over.
    pub flip_degrees: f32,
    /// Drawn with the hurt overlay (`hasRedOverlay`).
    pub red_overlay: bool,
    /// `getWhiteOverlayProgress`: how far the mob is flashed white.
    pub white_overlay: f32,
    /// What the renderer's own `scale` hook scales the pose by, after the
    /// flip (a creeper's swelling), or none.
    pub hook_scale: Option<[f32; 3]>,
    /// A roll about z the renderer's own `setupRotations` adds after the
    /// fall of a dead mob, in degrees (the walking iron golem's sway).
    pub roll: Option<f32>,
    /// A shift the renderer's `setupRotations` makes before that roll, and one
    /// after it, along the rolled axes (a cat lying down), in blocks.
    pub shift: Option<[f32; 3]>,
    pub roll_shift: Option<[f32; 3]>,
    /// A turn about x the renderer's `setupRotations` adds last, in degrees
    /// (a fox that pounces or has its face planted tips with its pitch).
    pub pitch: Option<f32>,
    /// The panda renderer's turns (the somersault, sitting, lying on its back).
    pub panda: Option<PandaRotation>,
    /// `LivingEntityRenderer.isShaking`: the age in ticks of a mob whose body
    /// shudders, or none.
    pub shaking: Option<f32>,
}

impl Default for MobPlacement {
    /// A mob of the ordinary size, upright where the mesh starts, unlit.
    fn default() -> Self {
        Self {
            feet: [0.0; 3],
            body_rot: 0.0,
            scale: 1.0,
            light: 0,
            death_time: 0.0,
            flip_degrees: FLIP_DEGREES,
            red_overlay: false,
            white_overlay: 0.0,
            hook_scale: None,
            roll: None,
            shift: None,
            roll_shift: None,
            pitch: None,
            panda: None,
            shaking: None,
        }
    }
}

/// `OverlayTexture`'s row for what a vertex is drawn with over its texel,
/// as the entity shader reads it: the hurt overlay is one, a white flash is
/// twice its column (`OverlayTexture.u`).
pub fn overlay_code(red: bool, white_progress: f32) -> f32 {
    if red {
        1.0
    } else {
        // `OverlayTexture.u`: truncated to a whole column.
        (2 * (white_progress * 15.0) as i32) as f32
    }
}

/// `CreeperRenderer.scale`: a creeper swells, wobbling as it nears its blast;
/// the factors along x and z, and along y.
pub fn creeper_scale(swelling: f32) -> [f32; 3] {
    let wobble = 1.0 + mth_sin_f64(f64::from(swelling * 100.0)) * swelling * 0.01;
    let mut swell = swelling.clamp(0.0, 1.0);
    swell *= swell;
    swell *= swell;
    let wide = (1.0 + swell * 0.4) * wobble;
    let tall = (1.0 + swell * 0.1) / wobble;
    [wide, tall, wide]
}

/// `CreeperRenderer.getWhiteOverlayProgress`: a swelling creeper flashes
/// white every other tenth of its swell.
pub fn creeper_white_overlay(swelling: f32) -> f32 {
    if (swelling * 10.0) as i32 % 2 == 0 {
        0.0
    } else {
        swelling.clamp(0.5, 1.0)
    }
}

/// `LivingEntityRenderer.getFlipDegrees` of most mobs.
pub const FLIP_DEGREES: f32 = 90.0;

/// `LivingEntityRenderer.setupRotations`'s fall of a dying entity: the
/// turn about z, in degrees, once its death has begun. It eases out as the
/// square root of the time since the first tick, and stops at the full flip.
pub fn death_roll(death_time: f32, flip_degrees: f32) -> Option<f32> {
    (death_time > 0.0).then(|| {
        let fall = ((death_time - 1.0) / 20.0 * 1.6).sqrt();
        fall.min(1.0) * flip_degrees
    })
}

/// `LivingEntityRenderer.submit` for a mob standing on its feet: scaled,
/// turned to face away from the body's facing, flipped upright and lowered so
/// the feet meet the ground.
pub fn placement_pose(placement: &MobPlacement) -> Pose {
    let mut pose = placement_rotations(placement);
    pose.scale(-1.0, -1.0, 1.0);
    if let Some([x, y, z]) = placement.hook_scale {
        pose.scale(x, y, z);
    }
    pose.translate(0.0, -1.501, 0.0);
    pose
}

/// What `LivingEntityRenderer.setupRotations` adds to the body's facing while
/// it shakes: a cosine of the whole tick, in degrees.
pub fn shake_degrees(age_in_ticks: f32) -> f32 {
    (f64::from(age_in_ticks.floor() * 3.25_f32).cos() * std::f64::consts::PI * f64::from(0.4_f32))
        as f32
}

/// What `LivingEntityRenderer.submit` has done by the end of the renderer's
/// `setupRotations`: the feet, the scale, the turn to face away from the
/// body's facing, the fall of a dead mob and the renderer's own turns.
pub fn placement_rotations(placement: &MobPlacement) -> Pose {
    let mut pose = Pose::default();
    pose.translate(placement.feet[0], placement.feet[1], placement.feet[2]);
    pose.scale(placement.scale, placement.scale, placement.scale);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0 - (placement.body_rot + placement.shaking.map_or(0.0, shake_degrees)),
    )));
    if let Some(roll) = death_roll(placement.death_time, placement.flip_degrees) {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            roll,
        )));
    }
    if let Some([x, y, z]) = placement.shift {
        pose.translate(x, y, z);
    }
    if let Some(roll) = placement.roll {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            roll,
        )));
    }
    if let Some([x, y, z]) = placement.roll_shift {
        pose.translate(x, y, z);
    }
    if let Some(pitch) = placement.pitch {
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            pitch,
        )));
    }
    if let Some(panda) = placement.panda {
        panda_rotate(&mut pose, &panda);
    }
    pose
}

/// The posed model's triangles, placed, each vertex carrying its normal in
/// its tint for the entity shader, with texture coordinates in the packed
/// entity textures.
pub fn mob_vertices(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    mob_vertices_coloured(model, placement, texture, None)
}

/// A glowing layer's triangles (`EyesLayer`, `RenderTypes.eyes`): the same
/// posed model, drawn unlit and without the hurt overlay, each vertex's tint
/// the white it is multiplied by.
pub fn mob_vertices_glow(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: 0.0,
                tint: [1.0; 4],
                light: [0.0; 2],
            });
        }
    }
    vertices
}

/// An energy swirl's triangles (`EnergySwirlLayer`, `RenderTypes.energySwirl`):
/// the same posed model, each vertex carrying where its texture is in the
/// packed image (the corner and the size, in its tint), the texture offset
/// (in its light) and the colour (`0xRRGGBB`, in its shade) that the shader
/// moves, wraps and multiplies the texture by.
pub fn mob_vertices_swirl(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    offset: [f32; 2],
    colour: u32,
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let rect = [
        texture[0],
        texture[1],
        texture[2] - texture[0],
        texture[3] - texture[1],
    ];
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: (colour & 0x00FF_FFFF) as f32,
                tint: rect,
                light: offset,
            });
        }
    }
    vertices
}

/// [`mob_vertices`], each texel multiplied by a colour (`0xRRGGBB`): a
/// sheep's wool.
pub fn mob_vertices_coloured(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    colour: Option<u32>,
) -> Vec<Vertex> {
    let colour = colour.map_or(0.0, crate::scene::entity_colour);
    let pose = placement_pose(placement);
    let light = crate::block_light::light_uv(placement.light);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: overlay_code(placement.red_overlay, placement.white_overlay),
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], colour],
                light,
            });
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_data::entity_model::layer;

    fn pig() -> MobModel {
        MobModel::bake(layer("minecraft:pig#main").expect("the pig's layer"))
    }

    #[test]
    fn a_pig_stands_on_the_ground_and_reaches_the_height_of_its_head() {
        let mut model = pig();
        model.setup_quadruped(&QuadrupedState::default());
        let vertices = mob_vertices(&model, &MobPlacement::default(), [0.0, 0.0, 1.0, 1.0]);
        let lowest = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::MAX, f32::min);
        let highest = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::MIN, f32::max);
        // The model hangs from 24 sixteenths, a thousandth above the ground
        // (`-1.501`), and the pig's head tops it at 8 sixteenths below that.
        assert!((lowest - 0.001).abs() < 1e-5, "{lowest}");
        assert!((highest - (1.0 + 0.001)).abs() < 1e-5, "{highest}");
    }

    #[test]
    fn walking_swings_the_legs_in_diagonal_pairs() {
        let mut model = pig();
        model.setup_quadruped(&QuadrupedState {
            walk_position: 1.0,
            walk_speed: 0.5,
            ..QuadrupedState::default()
        });
        let x_rot = |name: &str| {
            model
                .root()
                .children()
                .iter()
                .find(|part| part.name() == name)
                .unwrap()
                .x_rot
        };
        assert!(x_rot("right_hind_leg") > 0.0);
        assert!((x_rot("right_hind_leg") + x_rot("left_hind_leg")).abs() < 1e-6);
        assert_eq!(x_rot("right_hind_leg"), x_rot("left_front_leg"));
        assert_eq!(x_rot("left_hind_leg"), x_rot("right_front_leg"));
    }
}
