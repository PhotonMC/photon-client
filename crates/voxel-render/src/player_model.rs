//! The player's model: its parts, their boxes and how they are posed.
//!
//! A part is a pose (an offset in sixteenths of a block and three angles)
//! and a list of boxes. Every box face is a quad with its corner texture
//! coordinates in the 64x64 skin, laid out the way the reference's cube
//! builder lays them out; a mirrored box swaps its x extent and reverses
//! each face. Posing resets every part to where the mesh put it, then turns
//! and moves parts the way the reference's humanoid model does for a walking,
//! crouching, swimming or swinging player.

use crate::joml::{Matrix3, Pose, Quaternion};
use crate::spear_animation::{HandUse, third_person_attack_hand, third_person_hand_use};
use voxel_core::direction::Direction;
use voxel_core::math::{mth_cos_f64, mth_sin_f64};

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

/// `ModelPart.Polygon`: corner texture coordinates from the face's texel
/// rectangle, and a mirrored face reversed, with its x normal flipped.
fn polygon(
    corners: [[f32; 3]; 4],
    uv: [f32; 4],
    mirror: bool,
    facing: Direction,
    texture: (f32, f32),
) -> Polygon {
    let [u0, v0, u1, v1] = uv;
    let facing = if mirror && matches!(facing, Direction::West | Direction::East) {
        facing.opposite()
    } else {
        facing
    };
    let (x, y, z) = facing.step();
    let mut corners = [
        (corners[0], [u1 / texture.0, v0 / texture.1]),
        (corners[1], [u0 / texture.0, v0 / texture.1]),
        (corners[2], [u0 / texture.0, v1 / texture.1]),
        (corners[3], [u1 / texture.0, v1 / texture.1]),
    ];
    if mirror {
        corners.reverse();
    }
    Polygon {
        corners,
        normal: [x as f32, y as f32, z as f32],
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

impl Part {
    pub(crate) fn new(
        name: &'static str,
        cubes: Vec<Cube>,
        offset: [f32; 3],
        children: Vec<Part>,
    ) -> Self {
        Self::posed(name, cubes, offset, [0.0; 3], [1.0; 3], children)
    }

    /// A part with a pose of its own: `PartPose.offsetAndRotation`, and
    /// `scaled`.
    pub(crate) fn posed(
        name: &'static str,
        cubes: Vec<Cube>,
        offset: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
        children: Vec<Part>,
    ) -> Self {
        let initial = PartPose {
            offset,
            rotation,
            scale,
        };
        Self {
            name,
            cubes,
            children,
            initial,
            x: offset[0],
            y: offset[1],
            z: offset[2],
            x_rot: rotation[0],
            y_rot: rotation[1],
            z_rot: rotation[2],
            scale,
            visible: true,
            skip_draw: false,
        }
    }

    /// `PartDefinition.bake`: a part made from a layer's definition, in a
    /// texture `texture` big.
    pub fn baked(definition: &voxel_data::entity_model::ModelPart, texture: (u32, u32)) -> Self {
        use voxel_data::entity_model::Face;
        let cubes = definition
            .cubes
            .iter()
            .map(|cube| {
                let faces: Vec<Direction> = cube
                    .faces
                    .iter()
                    .map(|face| match face {
                        Face::Down => Direction::Down,
                        Face::East => Direction::East,
                        Face::North => Direction::North,
                        Face::South => Direction::South,
                        Face::Up => Direction::Up,
                        Face::West => Direction::West,
                    })
                    .collect();
                Cube::grown(
                    (cube.uv[0] as i32, cube.uv[1] as i32),
                    cube.origin,
                    cube.dimensions,
                    cube.grow,
                    cube.mirror,
                    (
                        texture.0 as f32 * cube.uv_scale[0],
                        texture.1 as f32 * cube.uv_scale[1],
                    ),
                    &faces,
                )
            })
            .collect();
        Self::posed(
            definition.name,
            cubes,
            definition.pose.position,
            definition.pose.rotation,
            definition.pose.scale,
            definition
                .children
                .iter()
                .map(|child| Self::baked(child, texture))
                .collect(),
        )
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn children(&self) -> &[Part] {
        &self.children
    }

    /// `ModelPart.getChild`, by name.
    /// The child at this place in the list (`children()` order).
    pub fn child_at_mut(&mut self, index: usize) -> &mut Part {
        &mut self.children[index]
    }

    pub fn child_mut(&mut self, name: &str) -> Option<&mut Part> {
        self.children.iter_mut().find(|child| child.name == name)
    }

    /// `Model.resetPose`: every part back where the mesh put it.
    pub fn reset(&mut self) {
        [self.x, self.y, self.z] = self.initial.offset;
        [self.x_rot, self.y_rot, self.z_rot] = self.initial.rotation;
        self.scale = self.initial.scale;
        for child in &mut self.children {
            child.reset();
        }
    }

    pub(crate) fn child(&mut self, name: &str) -> &mut Part {
        self.children
            .iter_mut()
            .find(|child| child.name == name)
            .expect("the player model has this part")
    }

    /// `ModelPart.rotateBy`: this part's rotation turned further by a quaternion,
    /// through the matrix and back to the three angles.
    pub fn rotate_by(&mut self, rotation: &Quaternion) {
        let mut matrix = Matrix3::rotation_zyx(self.z_rot, self.y_rot, self.x_rot);
        matrix.rotate(rotation);
        let [x, y, z] = matrix.euler_angles_zyx();
        (self.x_rot, self.y_rot, self.z_rot) = (x, y, z);
    }

    /// `ModelPart.translateAndRotate`.
    pub fn translate_and_rotate(&self, pose: &mut Pose) {
        self.translate_and_rotate_shifted(pose, 0.0);
    }

    /// `translateAndRotate` with the part's x moved by `shift` for the
    /// duration (a slim arm's hand).
    pub fn translate_and_rotate_shifted(&self, pose: &mut Pose, shift: f32) {
        pose.translate((self.x + shift) / 16.0, self.y / 16.0, self.z / 16.0);
        if self.x_rot != 0.0 || self.y_rot != 0.0 || self.z_rot != 0.0 {
            pose.rotate(&Quaternion::rotation_zyx(
                self.z_rot, self.y_rot, self.x_rot,
            ));
        }
        if self.scale != [1.0; 3] {
            pose.scale(self.scale[0], self.scale[1], self.scale[2]);
        }
    }

    /// `ModelPart.render`: every visible face, posed, each with the path of
    /// the part it belongs to.
    pub fn render(&self, pose: &Pose, path: &str, out: &mut Vec<(String, [ModelVertex; 4])>) {
        if !self.visible || (self.cubes.is_empty() && self.children.is_empty()) {
            return;
        }
        let mut pose = *pose;
        self.translate_and_rotate(&mut pose);
        for cube in self.cubes.iter().filter(|_| !self.skip_draw) {
            for polygon in &cube.polygons {
                let normal = pose.transform_normal(polygon.normal);
                let corner = |index: usize| {
                    let (position, uv) = polygon.corners[index];
                    ModelVertex {
                        position: pose.pose.transform_position(
                            position[0] / 16.0,
                            position[1] / 16.0,
                            position[2] / 16.0,
                        ),
                        uv,
                        normal,
                    }
                };
                out.push((
                    path.to_owned(),
                    [corner(0), corner(1), corner(2), corner(3)],
                ));
            }
        }
        for child in &self.children {
            child.render(&pose, &format!("{path}/{}", child.name), out);
        }
    }
}

/// `ChestModel.createSingleBodyLayer` with the lid shut, posed at the origin:
/// the bottom, the lid and the lock, each face's corners in blocks with the
/// texture coordinate in the chest's 64x64 texture.
pub fn chest_quads() -> Vec<[ModelVertex; 4]> {
    let cube =
        |tex: (i32, i32), min: [f32; 3], size: [f32; 3]| Cube::new(tex, min, size, 0.0, false);
    let root = Part::new(
        "root",
        Vec::new(),
        [0.0; 3],
        vec![
            Part::new(
                "bottom",
                vec![cube((0, 19), [1.0, 0.0, 1.0], [14.0, 10.0, 14.0])],
                [0.0; 3],
                Vec::new(),
            ),
            Part::new(
                "lid",
                vec![cube((0, 0), [1.0, 0.0, 0.0], [14.0, 5.0, 14.0])],
                [0.0, 9.0, 1.0],
                Vec::new(),
            ),
            Part::new(
                "lock",
                vec![cube((0, 0), [7.0, -2.0, 14.0], [2.0, 4.0, 1.0])],
                [0.0, 9.0, 1.0],
                Vec::new(),
            ),
        ],
    );
    let mut faces = Vec::new();
    root.render(&Pose::default(), "", &mut faces);
    faces.into_iter().map(|(_, face)| face).collect()
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
    pub right_spear: Option<&'static voxel_data::spear::Spear>,
    pub left_spear: Option<&'static voxel_data::spear::Spear>,
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

    /// `PlayerModel.setupAnim` and `HumanoidModel.setupAnim` for empty hands,
    /// on foot.
    pub fn setup_anim(&mut self, state: &PlayerRenderState) {
        humanoid_anim(&mut self.root, state);
    }

    /// `AvatarRenderer.renderHand`: one arm alone, as the first-person view
    /// draws it, reset to where the mesh put it, turned out a tenth of a
    /// radian, with its sleeve.
    pub fn render_arm(&mut self, arm: Arm, pose: &Pose) -> Vec<(String, [ModelVertex; 4])> {
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
        self.root
            .child(name)
            .render(pose, &format!("/{name}"), &mut out);
        out
    }

    /// Every visible face of the posed model, from `pose`.
    pub fn render(&self, pose: &Pose) -> Vec<(String, [ModelVertex; 4])> {
        let mut out = Vec::new();
        self.root.render(pose, "", &mut out);
        out
    }
}

/// `HumanoidModel.setupAnim` for empty hands, on foot: the head follows the
/// look, the limbs swing with the walk, the arms breathe, with the attack
/// swing, the crouch and the swim. `root` holds the humanoid mesh's parts
/// (the player's, a zombie's).
pub fn humanoid_anim(root: &mut Part, state: &PlayerRenderState) {
    use std::f64::consts::PI;
    let pi = PI as f32;
    root.reset();
    let show_body = !state.spectator;
    for name in ["body", "right_arm", "left_arm", "right_leg", "left_leg"] {
        root.child(name).visible = show_body;
    }

    let degrees_to_radians = (PI / 180.0) as f32;
    let swim_amount = state.swim_amount;
    {
        let head = root.child("head");
        head.x_rot = state.x_rot * degrees_to_radians;
        head.y_rot = state.head_y_rot * degrees_to_radians;
        if state.is_fall_flying {
            head.x_rot = (-PI / 4.0) as f32;
        } else if swim_amount > 0.0 {
            head.x_rot = rot_lerp_rad(swim_amount, head.x_rot, (-PI / 4.0) as f32);
        }
    }

    let position = state.walk_position;
    let speed = state.walk_speed;
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let slowdown = state.speed_value;
    root.child("right_arm").x_rot = cos(position * 0.6662 + pi) * 2.0 * speed * 0.5 / slowdown;
    root.child("left_arm").x_rot = cos(position * 0.6662) * 2.0 * speed * 0.5 / slowdown;
    root.child("right_leg").x_rot = cos(position * 0.6662) * 1.4 * speed / slowdown;
    root.child("left_leg").x_rot = cos(position * 0.6662 + pi) * 1.4 * speed / slowdown;
    root.child("right_leg").y_rot = 0.005;
    root.child("left_leg").y_rot = -0.005;
    root.child("right_leg").z_rot = 0.005;
    root.child("left_leg").z_rot = -0.005;
    if state.is_passenger {
        sit(root);
    }
    pose_arms(root, state);

    setup_attack_animation(root, state);

    if state.crouching {
        root.child("body").x_rot = 0.5;
        root.child("right_arm").x_rot += 0.4;
        root.child("left_arm").x_rot += 0.4;
        root.child("right_leg").z += 4.0;
        root.child("left_leg").z += 4.0;
        root.child("head").y += 4.2;
        root.child("body").y += 3.2;
        root.child("left_arm").y += 3.2;
        root.child("right_arm").y += 3.2;
    }

    // `AnimationUtils.bobModelPart`: the arms breathe.
    for (name, scale, pose) in [
        ("right_arm", 1.0_f32, state.right_arm_pose),
        ("left_arm", -1.0, state.left_arm_pose),
    ] {
        // A spyglass held up does not breathe.
        if pose == ArmPose::Spyglass {
            continue;
        }
        let age = state.age_in_ticks;
        let arm = root.child(name);
        arm.z_rot += scale * (cos(age * 0.09) * 0.05 + 0.05);
        arm.x_rot += scale * (sin(age * 0.067) * 0.05);
    }

    if swim_amount > 0.0 {
        swim(root, state, pi);
    }
}

/// `HumanoidModel.setupAnim`'s riding: the arms lift and the legs sit.
fn sit(root: &mut Part) {
    let pi = std::f64::consts::PI;
    let lift = (-pi / 5.0) as f32;
    root.child("right_arm").x_rot += lift;
    root.child("left_arm").x_rot += lift;
    for (name, side) in [("right_leg", 1.0_f32), ("left_leg", -1.0)] {
        let leg = root.child(name);
        leg.x_rot = -1.413_716_7;
        leg.y_rot = side * (pi / 10.0) as f32;
        leg.z_rot = side * 0.078_539_82;
    }
}

/// The order `HumanoidModel.setupAnim` poses the arms in: the arm of the hand in use first, else
/// the one a two-handed pose in the off hand leaves for last.
fn pose_arms(root: &mut Part, state: &PlayerRenderState) {
    let right_handed = state.main_arm == Arm::Right;
    if state.is_using_item {
        // The arm of the hand in use is posed first.
        if state.use_item_main_hand == right_handed {
            pose_arm(root, state, Arm::Right);
            if !state.right_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Left);
            }
        } else {
            pose_arm(root, state, Arm::Left);
            if !state.left_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Right);
            }
        }
    } else {
        // Which arm is posed first: a two-handed pose in the off hand makes the
        // other arm the one that is posed last.
        let two_handed_offhand = if right_handed {
            state.left_arm_pose.is_two_handed()
        } else {
            state.right_arm_pose.is_two_handed()
        };
        if right_handed != two_handed_offhand {
            pose_arm(root, state, Arm::Left);
            if !state.left_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Right);
            }
        } else {
            pose_arm(root, state, Arm::Right);
            if !state.right_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Left);
            }
        }
    }
}

/// `HumanoidModel.setupAttackAnimation`: the body turns with the swing and the swinging arm
/// whacks, or stabs.
fn setup_attack_animation(root: &mut Part, state: &PlayerRenderState) {
    use std::f64::consts::PI;
    let pi = PI as f32;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let attack_time = state.attack_time;
    // The reference swings unless the time is at most zero, so a NaN
    // time swings too.
    if state.swing_weapon_down && attack_time > 0.0 {
        swing_weapon_down(root, state.main_arm, attack_time, state.age_in_ticks);
    } else if attack_time > 0.0 || attack_time.is_nan() {
        let mut body_y_rot = sin(f64::from(attack_time).sqrt() as f32 * (PI * 2.0) as f32) * 0.2;
        if state.attack_arm == Arm::Left {
            body_y_rot *= -1.0;
        }
        root.child("body").y_rot = body_y_rot;
        {
            let right = root.child("right_arm");
            right.z = sin(body_y_rot) * 5.0 * 1.0;
            right.x = -cos(body_y_rot) * 5.0 * 1.0;
        }
        {
            let left = root.child("left_arm");
            left.z = -sin(body_y_rot) * 5.0 * 1.0;
            left.x = cos(body_y_rot) * 5.0 * 1.0;
        }
        root.child("right_arm").y_rot += body_y_rot;
        root.child("left_arm").y_rot += body_y_rot;
        root.child("left_arm").x_rot += body_y_rot;
        let swing = 1.0 - square(square(1.0 - attack_time));
        let aa = sin(swing * pi);
        let head_x_rot = root.child("head").x_rot;
        let bb = sin(attack_time * pi) * -(head_x_rot - 0.7) * 0.75;
        match state.swing_animation {
            SwingAnimation::Whack => {
                let arm = root.child(match state.attack_arm {
                    Arm::Left => "left_arm",
                    Arm::Right => "right_arm",
                });
                arm.x_rot -= aa * 1.2 + bb;
                arm.y_rot += body_y_rot * 2.0;
                arm.z_rot += sin(attack_time * pi) * -0.4;
            }
            SwingAnimation::Stab => {
                third_person_attack_hand(root, attack_time, state.attack_arm);
            }
        }
    }
}

/// `AnimationUtils.animateCrossbowCharge`: the arm that holds the crossbow and the one that pulls.
pub(crate) fn animate_crossbow_charge(
    root: &mut Part,
    (max_duration, ticks_using_item): (f32, f32),
    holding_in_right_arm: bool,
) {
    let pi = std::f64::consts::PI;
    let (holding, pulling) = if holding_in_right_arm {
        ("right_arm", "left_arm")
    } else {
        ("left_arm", "right_arm")
    };
    root.child(holding).y_rot = if holding_in_right_arm { -0.8 } else { 0.8 };
    root.child(holding).x_rot = -0.970_796_35;
    let held_x = root.child(holding).x_rot;
    let use_ticks = ticks_using_item.clamp(0.0, max_duration);
    let alpha = use_ticks / max_duration;
    let sign = if holding_in_right_arm { 1.0 } else { -1.0 };
    let pull = root.child(pulling);
    pull.y_rot = (0.4 + alpha * (0.85 - 0.4)) * sign;
    pull.x_rot = held_x + alpha * ((-pi / 2.0) as f32 - held_x);
}

/// `AnimationUtils.animateCrossbowHold`.
pub(crate) fn animate_crossbow_hold(root: &mut Part, holding_in_right_arm: bool) {
    let pi = std::f64::consts::PI;
    let (head_x, head_y) = {
        let head = root.child("head");
        (head.x_rot, head.y_rot)
    };
    let (holding, shooting) = if holding_in_right_arm {
        ("right_arm", "left_arm")
    } else {
        ("left_arm", "right_arm")
    };
    root.child(holding).y_rot = (if holding_in_right_arm { -0.3 } else { 0.3 }) + head_y;
    root.child(shooting).y_rot = (if holding_in_right_arm { 0.6 } else { -0.6 }) + head_y;
    root.child(holding).x_rot = (-pi / 2.0) as f32 + head_x + 0.1;
    root.child(shooting).x_rot = -1.5 + head_x;
}

/// `AnimationUtils.swingWeaponDown`.
fn swing_weapon_down(root: &mut Part, main_arm: Arm, attack_time: f32, age: f32) {
    let pi = std::f64::consts::PI as f32;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let attack2 = sin(attack_time * pi);
    let attack = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
    root.child("right_arm").z_rot = 0.0;
    root.child("left_arm").z_rot = 0.0;
    root.child("right_arm").y_rot = (std::f64::consts::PI / 20.0) as f32;
    root.child("left_arm").y_rot = (-std::f64::consts::PI / 20.0) as f32;
    if main_arm == Arm::Right {
        root.child("right_arm").x_rot = -1.884_955_8 + cos(age * 0.09) * 0.15;
        root.child("left_arm").x_rot = -0.0 + cos(age * 0.19) * 0.5;
        root.child("right_arm").x_rot += attack2 * 2.2 - attack * 0.4;
        root.child("left_arm").x_rot += attack2 * 1.2 - attack * 0.4;
    } else {
        root.child("right_arm").x_rot = -0.0 + cos(age * 0.19) * 0.5;
        root.child("left_arm").x_rot = -1.884_955_8 + cos(age * 0.09) * 0.15;
        root.child("right_arm").x_rot += attack2 * 1.2 - attack * 0.4;
        root.child("left_arm").x_rot += attack2 * 2.2 - attack * 0.4;
    }
    // `bobArms`.
    for (name, scale) in [("right_arm", 1.0_f32), ("left_arm", -1.0)] {
        let arm = root.child(name);
        arm.z_rot += scale * (cos(age * 0.09) * 0.05 + 0.05);
        arm.x_rot += scale * (sin(age * 0.067) * 0.05);
    }
}

/// `HumanoidModel.poseRightArm` and `poseLeftArm`.
fn pose_arm(root: &mut Part, state: &PlayerRenderState, arm: Arm) {
    let pi = std::f64::consts::PI;
    let (head_x, head_y) = {
        let head = root.child("head");
        (head.x_rot, head.y_rot)
    };
    let (name, pose) = match arm {
        Arm::Right => ("right_arm", state.right_arm_pose),
        Arm::Left => ("left_arm", state.left_arm_pose),
    };
    let right = arm == Arm::Right;
    match pose {
        ArmPose::Empty => root.child(name).y_rot = 0.0,
        ArmPose::Item => {
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - (pi / 10.0) as f32;
            part.y_rot = 0.0;
        }
        ArmPose::Block => {
            // `poseBlockingArm`.
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - 0.942_477_9
                + head_x.clamp((-pi * 4.0 / 9.0) as f32, 0.436_332_32);
            part.y_rot = (if right { -30.0_f32 } else { 30.0 }) * (pi / 180.0) as f32
                + head_y.clamp((-pi / 6.0) as f32, (pi / 6.0) as f32);
        }
        ArmPose::BowAndArrow => {
            let (right_turn, left_turn) = match arm {
                Arm::Right => (-0.1 + head_y, 0.1 + head_y + 0.4),
                Arm::Left => (-0.1 + head_y - 0.4, 0.1 + head_y),
            };
            let raised = (-pi / 2.0) as f32 + head_x;
            root.child("right_arm").y_rot = right_turn;
            root.child("left_arm").y_rot = left_turn;
            root.child("right_arm").x_rot = raised;
            root.child("left_arm").x_rot = raised;
        }
        ArmPose::ThrowTrident => {
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - pi as f32;
            part.y_rot = 0.0;
        }
        ArmPose::CrossbowCharge => animate_crossbow_charge(
            root,
            (state.max_crossbow_charge_duration, state.ticks_using_item),
            right,
        ),
        ArmPose::CrossbowHold => animate_crossbow_hold(root, right),
        ArmPose::Spyglass => {
            let part = root.child(name);
            let crouch = if state.crouching {
                (pi / 12.0) as f32
            } else {
                0.0
            };
            part.x_rot = (head_x - 1.919_862_2 - crouch).clamp(-2.4, 3.3);
            part.y_rot = head_y
                + if right {
                    -(pi / 12.0) as f32
                } else {
                    (pi / 12.0) as f32
                };
        }
        ArmPose::TootHorn => {
            let part = root.child(name);
            part.x_rot = head_x.clamp(-1.2, 1.2) - 1.483_529_8;
            part.y_rot = head_y
                + if right {
                    -(pi / 6.0) as f32
                } else {
                    (pi / 6.0) as f32
                };
        }
        ArmPose::Brush => {
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - (pi / 5.0) as f32;
            part.y_rot = 0.0;
        }
        ArmPose::Spear => {
            let use_state = HandUse {
                spear: match arm {
                    Arm::Right => state.right_spear,
                    Arm::Left => state.left_spear,
                },
                fall_flying_or_swimming: state.is_fall_flying || state.swim_amount > 0.0,
                ticks_using_item: state.ticks_using_item,
                is_using_item: state.is_using_item,
                use_item_main_hand: state.use_item_main_hand,
            };
            third_person_hand_use(root.child(name), (head_x, head_y), right, &use_state);
        }
    }
}

/// `HumanoidModel.setupAnim`'s swimming arms and legs.
fn swim(root: &mut Part, state: &PlayerRenderState, pi: f32) {
    let swim_amount = state.swim_amount;
    let position = state.walk_position;
    let swim_position = position % 26.0;
    let swinging = |arm: Arm| state.attack_arm == arm && state.attack_time > 0.0;
    let right_amount = if state.right_arm_pose == ArmPose::Spear || swinging(Arm::Right) {
        0.0
    } else {
        swim_amount
    };
    let left_amount = if state.left_arm_pose == ArmPose::Spear || swinging(Arm::Left) {
        0.0
    } else {
        swim_amount
    };
    let quadratic = |x: f32| -65.0 * x + x * x;
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    // Targets for the left arm (turned the short way) and the right arm.
    let (x_target, z_left, z_right) = if swim_position < 14.0 {
        (
            (0.0, 0.0),
            pi + 1.870_796_4 * quadratic(swim_position) / quadratic(14.0),
            pi - 1.870_796_4 * quadratic(swim_position) / quadratic(14.0),
        )
    } else if (14.0..22.0).contains(&swim_position) {
        let inner = (swim_position - 14.0) / 8.0;
        (
            (half_pi * inner, half_pi * inner),
            5.012_389 - 1.870_796_4 * inner,
            1.270_796_3 + 1.870_796_4 * inner,
        )
    } else if (22.0..26.0).contains(&swim_position) {
        let inner = (swim_position - 22.0) / 4.0;
        (
            (half_pi - half_pi * inner, half_pi - half_pi * inner),
            pi,
            pi,
        )
    } else {
        ((f32::NAN, f32::NAN), f32::NAN, f32::NAN)
    };
    if !x_target.0.is_nan() && !state.is_using_item {
        let left = root.child("left_arm");
        left.x_rot = rot_lerp_rad(left_amount, left.x_rot, x_target.0);
        left.y_rot = rot_lerp_rad(left_amount, left.y_rot, pi);
        left.z_rot = rot_lerp_rad(left_amount, left.z_rot, z_left);
        let right = root.child("right_arm");
        right.x_rot = lerp(right_amount, right.x_rot, x_target.1);
        right.y_rot = lerp(right_amount, right.y_rot, pi);
        right.z_rot = lerp(right_amount, right.z_rot, z_right);
    }
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let left_leg = root.child("left_leg");
    left_leg.x_rot = lerp(
        swim_amount,
        left_leg.x_rot,
        0.3 * cos(position * 0.333_333_34 + pi),
    );
    let right_leg = root.child("right_leg");
    right_leg.x_rot = lerp(
        swim_amount,
        right_leg.x_rot,
        0.3 * cos(position * 0.333_333_34),
    );
}

fn square(value: f32) -> f32 {
    value * value
}

/// `Mth.lerp(float, float, float)`.
fn lerp(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * (to - from)
}

/// `Mth.rotLerpRad`: the short way round between two angles in radians.
pub(crate) fn rot_lerp_rad(alpha: f32, from: f32, to: f32) -> f32 {
    let turn = (std::f64::consts::PI * 2.0) as f32;
    let mut difference = to - from;
    while difference < (-std::f64::consts::PI) as f32 {
        difference += turn;
    }
    while difference >= std::f64::consts::PI as f32 {
        difference -= turn;
    }
    from + alpha * difference
}
