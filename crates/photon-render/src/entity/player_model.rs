//! The player's model: its parts, their boxes and how they are posed.
//!
//! A part is a pose (an offset in sixteenths of a block and three angles)
//! and a list of boxes. Every box face is a quad with its corner texture
//! coordinates in the 64x64 skin, laid out the way the reference's cube
//! builder lays them out; a mirrored box swaps its x extent and reverses
//! each face. Posing resets every part to where the mesh put it, then turns
//! and moves parts the way the reference's humanoid model does for a walking,
//! crouching, swimming or swinging player.

use crate::entity::spear_animation::{HandUse, third_person_attack_hand, third_person_hand_use};
use crate::joml::{Matrix3, Pose, Quaternion};
use photon_core::direction::Direction;
use photon_core::math::{mth_cos_f64, mth_sin_f64};

const ALL_FACES: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::West,
    Direction::North,
    Direction::East,
    Direction::South,
];

/// One corner of a model face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

/// A face of a box, before posing: corners in sixteenths.
#[derive(Debug, Clone, Copy)]
struct Polygon {
    corners: [([f32; 3], [f32; 2]); 4],
    normal: [f32; 3],
}

#[derive(Debug, Clone)]
pub(crate) struct Cube {
    polygons: Vec<Polygon>,
}

impl Cube {
    /// `ModelPart.Cube` in the player's 64x64 texture, with every face
    /// visible.
    fn new(tex: (i32, i32), min: [f32; 3], size: [f32; 3], grow: f32, mirror: bool) -> Self {
        Self::sized(tex, min, size, grow, mirror, (64.0, 64.0), &ALL_FACES)
    }

    /// `ModelPart.Cube` in a texture of any size, with only the faces given
    /// visible.
    pub(crate) fn sized(
        tex: (i32, i32),
        min: [f32; 3],
        size: [f32; 3],
        grow: f32,
        mirror: bool,
        texture: (f32, f32),
        visible: &[Direction],
    ) -> Self {
        Self::grown(tex, min, size, [grow; 3], mirror, texture, visible)
    }

    /// `ModelPart.Cube` with a growth of its own along each axis.
    pub(crate) fn grown(
        tex: (i32, i32),
        min: [f32; 3],
        size: [f32; 3],
        grow: [f32; 3],
        mirror: bool,
        texture: (f32, f32),
        visible: &[Direction],
    ) -> Self {
        let [width, height, depth] = size;
        let mut min_x = min[0] - grow[0];
        let min_y = min[1] - grow[1];
        let min_z = min[2] - grow[2];
        let mut max_x = min[0] + width + grow[0];
        let max_y = min[1] + height + grow[1];
        let max_z = min[2] + depth + grow[2];
        if mirror {
            std::mem::swap(&mut min_x, &mut max_x);
        }
        let t0 = [min_x, min_y, min_z];
        let t1 = [max_x, min_y, min_z];
        let t2 = [max_x, max_y, min_z];
        let t3 = [min_x, max_y, min_z];
        let l0 = [min_x, min_y, max_z];
        let l1 = [max_x, min_y, max_z];
        let l2 = [max_x, max_y, max_z];
        let l3 = [min_x, max_y, max_z];
        let (tex_u, tex_v) = (tex.0 as f32, tex.1 as f32);
        let u0 = tex_u;
        let u1 = tex_u + depth;
        let u2 = tex_u + depth + width;
        let u22 = tex_u + depth + width + width;
        let u3 = tex_u + depth + width + depth;
        let u4 = tex_u + depth + width + depth + width;
        let v0 = tex_v;
        let v1 = tex_v + depth;
        let v2 = tex_v + depth + height;
        let face = |corners: [[f32; 3]; 4], uv: [f32; 4], facing: Direction| {
            polygon(corners, uv, mirror, facing, texture)
        };
        let faces = [
            (
                face([l1, l0, t0, t1], [u1, v0, u2, v1], Direction::Down),
                Direction::Down,
            ),
            (
                face([t2, t3, l3, l2], [u2, v1, u22, v0], Direction::Up),
                Direction::Up,
            ),
            (
                face([t0, l0, l3, t3], [u0, v1, u1, v2], Direction::West),
                Direction::West,
            ),
            (
                face([t1, t0, t3, t2], [u1, v1, u2, v2], Direction::North),
                Direction::North,
            ),
            (
                face([l1, t1, t2, l2], [u2, v1, u3, v2], Direction::East),
                Direction::East,
            ),
            (
                face([l0, l1, l2, l3], [u3, v1, u4, v2], Direction::South),
                Direction::South,
            ),
        ];
        Self {
            polygons: faces
                .into_iter()
                .filter(|(_, facing)| visible.contains(facing))
                .map(|(polygon, _)| polygon)
                .collect(),
        }
    }
}

/// Where a part sits relative to its parent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct PartPose {
    offset: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

/// `ModelPart`.
#[derive(Debug, Clone)]
pub struct Part {
    name: &'static str,
    cubes: Vec<Cube>,
    children: Vec<Part>,
    initial: PartPose,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub x_rot: f32,
    pub y_rot: f32,
    pub z_rot: f32,
    pub scale: [f32; 3],
    pub visible: bool,
    /// `ModelPart.skipDraw`: the part's own cubes are not drawn, its children are.
    pub skip_draw: bool,
}

/// Which arm the player swings with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm {
    Left,
    Right,
}

/// `HumanoidModel.ArmPose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArmPose {
    #[default]
    Empty,
    Item,
    Block,
    BowAndArrow,
    ThrowTrident,
    CrossbowCharge,
    CrossbowHold,
    Spyglass,
    TootHorn,
    Brush,
    Spear,
}

/// `SwingAnimationType`, of the two that move the arm: an item swung down, or thrust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwingAnimation {
    #[default]
    Whack,
    Stab,
}

impl ArmPose {
    /// `ArmPose.isTwoHanded`.
    pub fn is_two_handed(self) -> bool {
        matches!(
            self,
            ArmPose::BowAndArrow | ArmPose::CrossbowCharge | ArmPose::CrossbowHold
        )
    }

    /// `ArmPose.affectsOffhandPose`.
    pub fn affects_offhand_pose(self) -> bool {
        matches!(
            self,
            ArmPose::BowAndArrow
                | ArmPose::ThrowTrident
                | ArmPose::CrossbowCharge
                | ArmPose::CrossbowHold
                | ArmPose::Spear
        )
    }

    /// The pose with this name in the reference's `ArmPose` enum, lower-cased.
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "empty" => Self::Empty,
            "item" => Self::Item,
            "block" => Self::Block,
            "bow_and_arrow" => Self::BowAndArrow,
            "throw_trident" => Self::ThrowTrident,
            "crossbow_charge" => Self::CrossbowCharge,
            "crossbow_hold" => Self::CrossbowHold,
            "spyglass" => Self::Spyglass,
            "toot_horn" => Self::TootHorn,
            "brush" => Self::Brush,
            "spear" => Self::Spear,
            _ => return None,
        })
    }
}

/// What the model is posed from: the reference's avatar render state, the
/// fields a player uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRenderState {
    /// The head's turn from the body and its pitch, in degrees.
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
    pub age_in_ticks: f32,
    pub attack_time: f32,
    pub attack_arm: Arm,
    pub swim_amount: f32,
    pub crouching: bool,
    pub spectator: bool,
    pub right_arm_pose: ArmPose,
    pub left_arm_pose: ArmPose,
    /// `HumanoidRenderState.mainArm`: which arm the main hand is.
    pub main_arm: Arm,
    /// How long the item in use has been used, and how long a crossbow takes to charge.
    pub ticks_using_item: f32,
    pub max_crossbow_charge_duration: f32,
    /// `isUsingItem`, and whether the hand using it is the main hand.
    pub is_using_item: bool,
    pub use_item_main_hand: bool,
    /// `isPassenger`: riding sits the legs and lifts the arms.
    pub is_passenger: bool,
    /// `isFallFlying`: gliding tilts the head down.
    pub is_fall_flying: bool,
    /// `speedValue`: one, or how fast a glider flies, which slows its limbs' swing.
    pub speed_value: f32,
    /// A piglin with a melee weapon swings it down (`PiglinModel.setupAttackAnimation`).
    pub swing_weapon_down: bool,
    /// `swingAnimationType`: how the swing of the hand that swings moves the arm.
    pub swing_animation: SwingAnimation,
    /// `isBaby`, and whether the main hand holds nothing (`getMainHandItemStack() == EMPTY`): a
    /// baby zombie only raises its arms when its hand is empty.
    pub is_baby: bool,
    pub main_hand_empty: bool,
    /// The spear each arm holds, which a spear pose raises the arm with.
    pub right_spear: Option<&'static photon_data::spear::Spear>,
    pub left_spear: Option<&'static photon_data::spear::Spear>,
    /// The skin layers hidden (`PlayerModelPart` masks, `AvatarRenderer.extractRenderState`'s
    /// `show*`): none hides nothing.
    pub hidden_layers: u8,
}

impl Default for PlayerRenderState {
    fn default() -> Self {
        Self {
            head_y_rot: 0.0,
            x_rot: 0.0,
            walk_position: 0.0,
            walk_speed: 0.0,
            age_in_ticks: 0.0,
            attack_time: 0.0,
            attack_arm: Arm::Right,
            swim_amount: 0.0,
            crouching: false,
            spectator: false,
            right_arm_pose: ArmPose::Empty,
            left_arm_pose: ArmPose::Empty,
            main_arm: Arm::Right,
            ticks_using_item: 0.0,
            max_crossbow_charge_duration: 0.0,
            is_using_item: false,
            use_item_main_hand: true,
            is_passenger: false,
            is_fall_flying: false,
            speed_value: 1.0,
            swing_weapon_down: false,
            swing_animation: SwingAnimation::Whack,
            is_baby: false,
            main_hand_empty: true,
            right_spear: None,
            left_spear: None,
            hidden_layers: 0,
        }
    }
}

/// The player model: the reference's player mesh, wide or slim armed.
#[derive(Debug, Clone)]
pub struct PlayerModel {
    pub root: Part,
    /// Whether the arms are three wide.
    pub slim: bool,
}

impl PlayerModel {
    /// `PlayerModel.createMesh` with no deformation, baked for a 64x64 skin.
    pub fn new(slim: bool) -> Self {
        let cube = |tex: (i32, i32), min: [f32; 3], size: [f32; 3], grow: f32| {
            Cube::new(tex, min, size, grow, false)
        };
        let arm_width = if slim { 3.0 } else { 4.0 };
        let right_arm_x = if slim { -2.0 } else { -3.0 };
        let head = Part::new(
            "head",
            vec![cube((0, 0), [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0], 0.0)],
            [0.0, 0.0, 0.0],
            vec![Part::new(
                "hat",
                vec![cube((32, 0), [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0], 0.5)],
                [0.0; 3],
                Vec::new(),
            )],
        );
        let body = Part::new(
            "body",
            vec![cube((16, 16), [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0], 0.0)],
            [0.0, 0.0, 0.0],
            vec![Part::new(
                "jacket",
                vec![cube((16, 32), [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0], 0.25)],
                [0.0; 3],
                Vec::new(),
            )],
        );
        let right_arm = Part::new(
            "right_arm",
            vec![cube(
                (40, 16),
                [right_arm_x, -2.0, -2.0],
                [arm_width, 12.0, 4.0],
                0.0,
            )],
            [-5.0, 2.0, 0.0],
            vec![Part::new(
                "right_sleeve",
                vec![cube(
                    (40, 32),
                    [right_arm_x, -2.0, -2.0],
                    [arm_width, 12.0, 4.0],
                    0.25,
                )],
                [0.0; 3],
                Vec::new(),
            )],
        );
        let left_arm = Part::new(
            "left_arm",
            vec![cube(
                (32, 48),
                [-1.0, -2.0, -2.0],
                [arm_width, 12.0, 4.0],
                0.0,
            )],
            [5.0, 2.0, 0.0],
            vec![Part::new(
                "left_sleeve",
                vec![cube(
                    (48, 48),
                    [-1.0, -2.0, -2.0],
                    [arm_width, 12.0, 4.0],
                    0.25,
                )],
                [0.0; 3],
                Vec::new(),
            )],
        );
        let right_leg = Part::new(
            "right_leg",
            vec![cube((0, 16), [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0], 0.0)],
            [-1.9, 12.0, 0.0],
            vec![Part::new(
                "right_pants",
                vec![cube((0, 32), [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0], 0.25)],
                [0.0; 3],
                Vec::new(),
            )],
        );
        let left_leg = Part::new(
            "left_leg",
            vec![cube((16, 48), [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0], 0.0)],
            [1.9, 12.0, 0.0],
            vec![Part::new(
                "left_pants",
                vec![cube((0, 48), [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0], 0.25)],
                [0.0; 3],
                Vec::new(),
            )],
        );
        Self {
            slim,
            root: Part::new(
                "root",
                Vec::new(),
                [0.0; 3],
                vec![head, body, right_arm, left_arm, right_leg, left_leg],
            ),
        }
    }

    pub fn part(&mut self, name: &str) -> &mut Part {
        self.root.child(name)
    }

    /// The outer layers by the part they hang on and their `PlayerModelPart` bit.
    const LAYERS: [(&str, &str, u8); 6] = [
        ("body", "jacket", 1),
        ("left_arm", "left_sleeve", 2),
        ("right_arm", "right_sleeve", 3),
        ("left_leg", "left_pants", 4),
        ("right_leg", "right_pants", 5),
        ("head", "hat", 6),
    ];

    /// `PlayerModel.setupAnim` and `HumanoidModel.setupAnim` for empty hands,
    /// on foot.
    pub fn setup_anim(&mut self, state: &PlayerRenderState) {
        humanoid_anim(&mut self.root, state);
        // `PlayerModel.setupAnim`: each outer layer shows by its part's bit.
        for (parent, layer, bit) in Self::LAYERS {
            let shown = state.hidden_layers & (1 << bit) == 0;
            if let Some(part) = self
                .part(parent)
                .children
                .iter_mut()
                .find(|child| child.name == layer)
            {
                part.visible = shown;
            }
        }
    }

    /// `AvatarRenderer.renderHand`: one arm alone, as the first-person view
    /// draws it, reset to where the mesh put it, turned out a tenth of a
    /// radian, with its sleeve.
    pub fn render_arm(&mut self, arm: Arm, pose: &Pose) -> Vec<[ModelVertex; 4]> {
        let name = match arm {
            Arm::Left => "left_arm",
            Arm::Right => "right_arm",
        };
        let part = self.part(name);
        part.reset();
        part.visible = true;
        for child in &mut part.children {
            child.visible = true;
        }
        part.z_rot = match arm {
            Arm::Left => -0.1,
            Arm::Right => 0.1,
        };
        let mut out = Vec::new();
        self.root.child(name).render_quads(pose, &mut out);
        out
    }

    /// Every visible face of the posed model, from `pose`.
    pub fn render(&self, pose: &Pose) -> Vec<(String, [ModelVertex; 4])> {
        let mut out = Vec::new();
        self.root.render(pose, "", &mut out);
        out
    }

    /// The faces of `render`, without the paths of their parts.
    pub fn quads(&self, pose: &Pose) -> Vec<[ModelVertex; 4]> {
        let mut out = Vec::new();
        self.root.render_quads(pose, &mut out);
        out
    }
}

mod part;

mod pose;
pub use pose::*;
mod lerp;
pub(crate) use lerp::*;
