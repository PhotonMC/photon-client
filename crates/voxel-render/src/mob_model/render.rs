use super::*;

/// What `QuadrupedModel.setupAnim` reads of a mob.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct QuadrupedState {
    /// The head's turn from the body and its pitch, in degrees.
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
}

/// What a shulker's model is posed from (`ShulkerRenderState`): the lid's
/// peek, the age, the pitch, and the head's and the body's own turns.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShulkerRenderState {
    pub peek_amount: f32,
    pub age_in_ticks: f32,
    pub x_rot: f32,
    pub y_head_rot: f32,
    pub y_body_rot: f32,
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

/// What a dolphin's model is posed from (`DolphinRenderState`): the head's
/// turn and pitch, which turn the whole body, the age, and whether it moves.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DolphinState {
    pub y_rot: f32,
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub is_moving: bool,
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
    pub swim: voxel_world::animation_state::AnimationState,
    pub walk: voxel_world::animation_state::AnimationState,
    pub idle_under_water: voxel_world::animation_state::AnimationState,
    pub idle_under_water_on_ground: voxel_world::animation_state::AnimationState,
    pub idle_on_ground: voxel_world::animation_state::AnimationState,
    pub play_dead: voxel_world::animation_state::AnimationState,
}

/// What a frog's model is posed from (`FrogRenderState`): the animation
/// states its pose and floating run, and whether it swims.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrogRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_swimming: bool,
    pub jump: voxel_world::animation_state::AnimationState,
    pub croak: voxel_world::animation_state::AnimationState,
    pub tongue: voxel_world::animation_state::AnimationState,
    pub swim_idle: voxel_world::animation_state::AnimationState,
}

/// What a breeze's models (the body, the wind and the eyes) are posed from
/// (`BreezeRenderState`): the animation states its pose and its ticks run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeRenderState {
    pub age_in_ticks: f32,
    pub idle: voxel_world::animation_state::AnimationState,
    pub shoot: voxel_world::animation_state::AnimationState,
    pub slide: voxel_world::animation_state::AnimationState,
    pub slide_back: voxel_world::animation_state::AnimationState,
    pub inhale: voxel_world::animation_state::AnimationState,
    pub long_jump: voxel_world::animation_state::AnimationState,
}

/// What a warden's models (the body and its four glowing layers) are posed from
/// (`WardenRenderState`): the look, the walk, the pulses of its tendrils, and
/// the animation states its pose and events run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WardenRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub tendril_animation: f32,
    pub roar: voxel_world::animation_state::AnimationState,
    pub sniff: voxel_world::animation_state::AnimationState,
    pub emerge: voxel_world::animation_state::AnimationState,
    pub digging: voxel_world::animation_state::AnimationState,
    pub attack: voxel_world::animation_state::AnimationState,
    pub sonic_boom: voxel_world::animation_state::AnimationState,
}

/// What a creaking's models (the body and its eyes) are posed from
/// (`CreakingRenderState`): the look, the walk (which only a creaking that can
/// move shows), and the animation states its ticks run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreakingRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub can_move: bool,
    pub attack: voxel_world::animation_state::AnimationState,
    pub invulnerable: voxel_world::animation_state::AnimationState,
    pub death: voxel_world::animation_state::AnimationState,
}

/// What an armadillo's model is posed from (`ArmadilloRenderState`): the look,
/// the walk, whether it hides in its shell, and its animation states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmadilloRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_hiding_in_shell: bool,
    pub roll_out: voxel_world::animation_state::AnimationState,
    pub roll_up: voxel_world::animation_state::AnimationState,
    pub peek: voxel_world::animation_state::AnimationState,
}

/// What a sniffer's model is posed from (`SnifferRenderState`): the look, the
/// walk, whether it is searching, and its animation states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnifferRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_searching: bool,
    pub digging: voxel_world::animation_state::AnimationState,
    pub sniffing: voxel_world::animation_state::AnimationState,
    pub rising: voxel_world::animation_state::AnimationState,
    pub feeling_happy: voxel_world::animation_state::AnimationState,
    pub scenting: voxel_world::animation_state::AnimationState,
}

/// What a wither's models (the body and its armor) are posed from
/// (`WitherRenderState`): the middle head's look, the body's turn, and the side
/// heads' turns, all in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitherRenderState {
    pub age_in_ticks: f32,
    pub y_rot: f32,
    pub x_rot: f32,
    pub body_rot: f32,
    pub x_head_rots: [f32; 2],
    pub y_head_rots: [f32; 2],
}

/// What an ender dragon's model is posed from (`EnderDragonRenderState`): the
/// wing beat, the flight history its parts follow (and the partial tick it is read at),
/// and what its neck's pose follows while it lands, takes off or sits.
#[derive(Debug, Clone, PartialEq)]
pub struct EnderDragonRenderState {
    pub flap_time: f32,
    pub history: voxel_world::ender_dragon::FlightHistory,
    pub partial_ticks: f32,
    pub is_landing_or_taking_off: bool,
    pub is_sitting: bool,
    pub distance_to_egg: f64,
}

impl EnderDragonRenderState {
    /// `getHeadPartYOffset`.
    pub(super) fn head_part_y_offset(
        &self,
        part: i32,
        body: voxel_world::ender_dragon::Sample,
        at: voxel_world::ender_dragon::Sample,
    ) -> f32 {
        let result = if self.is_landing_or_taking_off {
            f64::from(part) / (self.distance_to_egg / 4.0).max(1.0)
        } else if self.is_sitting {
            f64::from(part)
        } else if part == 6 {
            0.0
        } else {
            at.y - body.y
        };
        result as f32
    }
}

/// What an end crystal's model is posed from (`EndCrystalRenderState`): its age
/// (which turns and bobs its glass) and whether it shows its base.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndCrystalRenderState {
    pub age_in_ticks: f32,
    pub shows_bottom: bool,
}

/// What an arrow's model is posed from (`ArrowRenderState`): how long it has shaken.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowRenderState {
    pub shake: f32,
}

/// `ParrotModel.Pose`: how a parrot sits, stands or flies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParrotPose {
    Flying,
    Standing,
    Sitting,
    Party,
    OnShoulder,
}

/// What a parrot's model is posed from (`ParrotRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParrotRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub flap_angle: f32,
    pub pose: ParrotPose,
}

/// What a bat's model is posed from (`BatRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatRenderState {
    pub head_y_rot: f32,
    pub age_in_ticks: f32,
    pub is_resting: bool,
    pub fly: voxel_world::animation_state::AnimationState,
    pub rest: voxel_world::animation_state::AnimationState,
}

/// What a vex's model is posed from (`VexRenderState`): the look, whether it
/// is charging, and whether each hand holds anything.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexRenderState {
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub is_charging: bool,
    pub right_hand_item: bool,
    pub left_hand_item: bool,
}

/// What an allay's model is posed from (`AllayRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AllayRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_dancing: bool,
    pub is_spinning: bool,
    pub spinning_progress: f32,
    pub holding_animation_progress: f32,
}

/// What a guardian's eye follows (`GuardianRenderState`): its own eye and
/// look, and the position of what it looks at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianLook {
    pub eye_position: [f64; 3],
    pub look_direction: [f64; 3],
    pub look_at_position: [f64; 3],
}

/// What a guardian's model is posed from (`GuardianRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianState {
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub spikes_animation: f32,
    pub tail_animation: f32,
    pub look: Option<GuardianLook>,
}

/// `GuardianModel`'s tables: where each spike sits, before it breathes.
pub(super) const SPIKE_X: [f32; 12] = [
    0.0, 0.0, 8.0, -8.0, -8.0, 8.0, 8.0, -8.0, 0.0, 0.0, 8.0, -8.0,
];

pub(super) const SPIKE_Y: [f32; 12] = [
    -8.0, -8.0, -8.0, -8.0, 0.0, 0.0, 0.0, 0.0, 8.0, 8.0, 8.0, 8.0,
];

pub(super) const SPIKE_Z: [f32; 12] = [
    8.0, -8.0, 0.0, 0.0, -8.0, -8.0, 8.0, 8.0, 8.0, -8.0, 0.0, 0.0,
];

/// What a nautilus's model is posed from (`NautilusRenderState`): the look
/// and stride, and whether it wears body armor (which hides a zombie
/// nautilus's corals).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NautilusState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub has_body_armor: bool,
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
