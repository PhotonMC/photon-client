use super::*;

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
    pub(super) fn body_roll_angle(&self, offset: f32) -> f32 {
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

pub(super) const LEGS: [&str; 4] = [
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
    pub arm_pose: PiglinArmPose,
}

/// `PiglinArmPose`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PiglinArmPose {
    #[default]
    Default,
    Dancing,
    AttackingWithMeleeWeapon,
    CrossbowHold,
    CrossbowCharge,
    AdmiringItem,
}

impl PiglinArmPose {
    /// The pose with this name in the reference's enum, lower-cased.
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "default" => Self::Default,
            "dancing" => Self::Dancing,
            "attacking_with_melee_weapon" => Self::AttackingWithMeleeWeapon,
            "crossbow_hold" => Self::CrossbowHold,
            "crossbow_charge" => Self::CrossbowCharge,
            "admiring_item" => Self::AdmiringItem,
            _ => return None,
        })
    }
}

/// A mob's model: the parts of one layer.
#[derive(Debug, Clone)]
pub struct MobModel {
    pub(super) root: Part,
    /// `FoxModel.legMotionPos`: the model's own count, which a faceplanted
    /// fox's legs follow and which grows with every pose of such a fox.
    pub(super) leg_motion_pos: f32,
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
    /// What the renderer's `scale` hook then moves the pose by (a phantom's lift), or none.
    pub hook_translate: Option<[f32; 3]>,
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
    /// `SquidRenderer.setupRotations`, which replaces the renderer's own turns.
    pub squid: Option<SquidTurns>,
    /// What the fish renderers' `setupRotations` add to the pose.
    pub fish: Option<FishTurns>,
    /// `AbstractCubeMobRenderer.scale`, which comes before the hooks above.
    pub cube_mob: Option<CubeMobScale>,
    /// `ShulkerRenderer.setupRotations`: the data value of the face the shulker
    /// is attached to, which turns the pose about the middle of its box.
    pub attach_face: Option<i32>,
}

/// What a slime's or a magma cube's renderer scales the pose by
/// (`AbstractCubeMobRenderer.applySizeAndSquish`, and for a slime
/// `downscaleSlightly` before it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubeMobScale {
    pub size: i32,
    pub squish: f32,
    pub downscale: bool,
    pub sulfur: Option<SulfurCubeScale>,
}

/// What a sulfur cube's renderer scales the pose by besides the cube mobs'
/// (`SulfurCubeRenderer.scale`): the swell of a fuse about to run out, and
/// the fit of the small or the full shell about the model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SulfurCubeScale {
    /// `SulfurCubeRenderState.fuseRemainingTicks`: zero while it is not primed.
    pub fuse_remaining: f32,
    pub baby: bool,
    /// Whether it holds a block that is drawn (`SulfurCubeRenderState.containedBlock`
    /// is not empty): it then keeps no squish.
    pub holds_block: bool,
}

impl CubeMobScale {
    pub(super) fn apply(&self, pose: &mut Pose) {
        if self.downscale {
            pose.scale(0.999, 0.999, 0.999);
            pose.translate(0.0, 0.001, 0.0);
        }
        let size = self.size as f32;
        // `SulfurCubeRenderer.applySizeAndSquish`: a held block does not squash.
        let squish = if self.sulfur.is_some_and(|sulfur| sulfur.holds_block) {
            0.0
        } else {
            self.squish
        };
        let squash = squish / (size * 0.5 + 1.0);
        let wide = 1.0 / (squash + 1.0);
        pose.scale(wide * size, 1.0 / wide * size, wide * size);
        if let Some(sulfur) = self.sulfur {
            let fuse = sulfur.fuse_remaining;
            if fuse < 10.0 && fuse > 0.0 {
                let swell = 1.0 + tnt_swell_amount(fuse);
                pose.scale(swell, swell, swell);
            }
            let (offset, fit) = if sulfur.baby {
                (1.24, 1.0)
            } else {
                (0.98, 0.5)
            };
            pose.scale(fit, fit, fit);
            // One pixel up while it is seen; `translate(-0.0F, ..., -0.0F)`.
            pose.translate(-0.0, offset - 1.0 / 16.0, -0.0);
        }
    }
}
