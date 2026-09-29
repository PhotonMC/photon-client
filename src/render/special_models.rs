//! The entity models the client's `special` item models draw: chests,
//! shulker boxes, banners, heads, shields, conduits, decorated pots and
//! copper golem statues.
//!
//! Each is the reference's `LayerDefinition` for the model, posed as its
//! `setupAnim` leaves it for a stack that is not open, moving or animated,
//! and drawn in one or more texture layers. The geometry stays in the
//! model's own space, in blocks, where the item model's `transformation`
//! finds it.

use crate::direction::Direction;
use crate::render::player_model::{Cube, ModelVertex, Part};

/// A model's faces in one texture.
pub struct ModelLayer {
    pub quads: Vec<[ModelVertex; 4]>,
    /// The texture under `textures/`, without its extension.
    pub sprite: String,
    /// The colour the texels are multiplied by, `0xAARRGGBB`.
    pub tint: u32,
}

/// The models a client item can draw specially.
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialKind {
    /// The single chest, with the texture under `entity/chest`.
    Chest { texture: String },
    /// A shulker box with its texture under `entity/shulker`.
    ShulkerBox { texture: String },
    /// A skull or head by kind (`skeleton`, `wither_skeleton`, `zombie`,
    /// `creeper`, `piglin`, `dragon`) and its texture; a player head wears
    /// the default skin.
    Head { kind: String, texture: String },
    /// A standing banner in a dye colour with no patterns.
    Banner { color: u32 },
    /// A shield with no patterns.
    Shield,
    /// The conduit's shell.
    Conduit,
    /// A decorated pot with no decorations.
    DecoratedPot,
    /// A standing copper golem statue with its texture.
    CopperGolemStatue { texture: String },
}

const WHITE: u32 = 0xFFFF_FFFF;

fn cube(texture: (f32, f32), tex: (i32, i32), min: [f32; 3], size: [f32; 3]) -> Cube {
    grown(texture, tex, min, size, 0.0)
}

fn grown(texture: (f32, f32), tex: (i32, i32), min: [f32; 3], size: [f32; 3], grow: f32) -> Cube {
    Cube::sized(tex, min, size, grow, false, texture, &FACES)
}

const FACES: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::West,
    Direction::North,
    Direction::East,
    Direction::South,
];

fn layer(root: &Part, sprite: &str, tint: u32) -> ModelLayer {
    let mut faces = Vec::new();
    root.render(&crate::render::joml::Pose::default(), "", &mut faces);
    ModelLayer {
        quads: faces.into_iter().map(|(_, face)| face).collect(),
        sprite: sprite.to_owned(),
        tint,
    }
}

/// A root part holding the given children.
fn root(children: Vec<Part>) -> Part {
    Part::new("root", Vec::new(), [0.0; 3], children)
}

/// The layers a model draws.
pub fn layers(kind: &SpecialKind) -> Vec<ModelLayer> {
    match kind {
        SpecialKind::Chest { texture } => vec![ModelLayer {
            quads: crate::render::player_model::chest_quads(),
            sprite: texture.clone(),
            tint: WHITE,
        }],
        SpecialKind::ShulkerBox { texture } => shulker_box(texture),
        SpecialKind::Head { kind, texture } => head(kind, texture),
        SpecialKind::Banner { color } => banner(*color),
        SpecialKind::Shield => shield(),
        SpecialKind::Conduit => conduit(),
        SpecialKind::DecoratedPot => decorated_pot(),
        SpecialKind::CopperGolemStatue { texture } => copper_golem(texture),
    }
}

/// `ShulkerModel.createBoxLayer`: the lid and the base, shut.
fn shulker_box(texture: &str) -> Vec<ModelLayer> {
    let size = (64.0, 64.0);
    let model = root(vec![
        Part::new(
            "lid",
            vec![cube(size, (0, 0), [-8.0, -16.0, -8.0], [16.0, 12.0, 16.0])],
            [0.0, 24.0, 0.0],
            Vec::new(),
        ),
        Part::new(
            "base",
            vec![cube(size, (0, 28), [-8.0, -8.0, -8.0], [16.0, 8.0, 16.0])],
            [0.0, 24.0, 0.0],
            Vec::new(),
        ),
    ]);
    vec![layer(&model, texture, WHITE)]
}

/// `SkullModel`, `PiglinHeadModel` and `DragonHeadModel` with the head
/// facing forward and the animation at its start.
fn head(kind: &str, texture: &str) -> Vec<ModelLayer> {
    let head_cube = |size: (f32, f32)| cube(size, (0, 0), [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]);
    let model = match kind {
        // `createMobHeadLayer`: 64x32.
        "skeleton" | "wither_skeleton" | "creeper" => root(vec![Part::new(
            "head",
            vec![head_cube((64.0, 32.0))],
            [0.0; 3],
            Vec::new(),
        )]),
        // `createHumanoidHeadLayer`: 64x64 with the hat around the head.
        "zombie" | "player" => root(vec![Part::new(
            "head",
            vec![head_cube((64.0, 64.0))],
            [0.0; 3],
            vec![Part::new(
                "hat",
                vec![grown(
                    (64.0, 64.0),
                    (32, 0),
                    [-4.0, -8.0, -4.0],
                    [8.0, 8.0, 8.0],
                    0.25,
                )],
                [0.0; 3],
                Vec::new(),
            )],
        )]),
        "piglin" => {
            let size = (64.0, 64.0);
            // The ears at `animationPos` zero.
            let ear = (0.0_f32.cos() + 2.5) * 0.2;
            root(vec![Part::new(
                "head",
                vec![
                    cube(size, (0, 0), [-5.0, -8.0, -4.0], [10.0, 8.0, 8.0]),
                    cube(size, (31, 1), [-2.0, -4.0, -5.0], [4.0, 4.0, 1.0]),
                    cube(size, (2, 4), [2.0, -2.0, -5.0], [1.0, 2.0, 1.0]),
                    cube(size, (2, 0), [-3.0, -2.0, -5.0], [1.0, 2.0, 1.0]),
                ],
                [0.0; 3],
                vec![
                    Part::posed(
                        "left_ear",
                        vec![cube(size, (51, 6), [0.0, 0.0, -2.0], [1.0, 5.0, 4.0])],
                        [4.5, -6.0, 0.0],
                        [0.0, 0.0, -ear],
                        [1.0; 3],
                        Vec::new(),
                    ),
                    Part::posed(
                        "right_ear",
                        vec![cube(size, (39, 6), [-1.0, 0.0, -2.0], [1.0, 5.0, 4.0])],
                        [-4.5, -6.0, 0.0],
                        [0.0, 0.0, ear],
                        [1.0; 3],
                        Vec::new(),
                    ),
                ],
            )])
        }
        // `DragonHeadModel.createHeadLayer`, scaled to three quarters, the
        // jaw opened by `(sin 0 + 1) * 0.2`.
        "dragon" => {
            let size = (256.0, 256.0);
            let mirrored = |tex: (i32, i32), min: [f32; 3], extent: [f32; 3]| {
                Cube::sized(tex, min, extent, 0.0, true, size, &FACES)
            };
            root(vec![Part::posed(
                "head",
                vec![
                    cube(size, (176, 44), [-6.0, -1.0, -24.0], [12.0, 5.0, 16.0]),
                    cube(size, (112, 30), [-8.0, -8.0, -10.0], [16.0, 16.0, 16.0]),
                    mirrored((0, 0), [-5.0, -12.0, -4.0], [2.0, 4.0, 6.0]),
                    mirrored((112, 0), [-5.0, -3.0, -22.0], [2.0, 2.0, 4.0]),
                    cube(size, (0, 0), [3.0, -12.0, -4.0], [2.0, 4.0, 6.0]),
                    cube(size, (112, 0), [3.0, -3.0, -22.0], [2.0, 2.0, 4.0]),
                ],
                [0.0, -7.986_666 * 0.75, 0.0],
                [0.0; 3],
                [0.75; 3],
                vec![Part::posed(
                    "jaw",
                    vec![cube(size, (176, 65), [-6.0, 0.0, -16.0], [12.0, 4.0, 16.0])],
                    [0.0, 4.0, -8.0],
                    [0.2, 0.0, 0.0],
                    [1.0; 3],
                    Vec::new(),
                )],
            )])
        }
        _ => return Vec::new(),
    };
    vec![layer(&model, texture, WHITE)]
}

/// `BannerModel.createBodyLayer(true)` and `BannerFlagModel`: the pole and
/// the bar, then the flag in the plain banner texture, then the flag again
/// in the base pattern, tinted by the dye.
fn banner(color: u32) -> Vec<ModelLayer> {
    let size = (64.0, 64.0);
    let body = root(vec![
        Part::new(
            "pole",
            vec![cube(size, (44, 0), [-1.0, -42.0, -1.0], [2.0, 42.0, 2.0])],
            [0.0; 3],
            Vec::new(),
        ),
        Part::new(
            "bar",
            vec![cube(size, (0, 42), [-10.0, -44.0, -1.0], [20.0, 2.0, 2.0])],
            [0.0; 3],
            Vec::new(),
        ),
    ]);
    // `flag.xRot` at phase zero.
    let tilt = (-0.0125_f32 + 0.01) * std::f32::consts::PI;
    let flag = root(vec![Part::posed(
        "flag",
        vec![cube(size, (0, 0), [-10.0, 0.0, -2.0], [20.0, 40.0, 1.0])],
        [0.0, -44.0, 0.0],
        [tilt, 0.0, 0.0],
        [1.0; 3],
        Vec::new(),
    )]);
    vec![
        layer(&body, "entity/banner/banner_base", WHITE),
        layer(&flag, "entity/banner/banner_base", WHITE),
        layer(&flag, "entity/banner/base", color),
    ]
}

/// `ShieldModel.createLayer`, without patterns.
fn shield() -> Vec<ModelLayer> {
    let size = (64.0, 64.0);
    let model = root(vec![
        Part::new(
            "plate",
            vec![cube(size, (0, 0), [-6.0, -11.0, -2.0], [12.0, 22.0, 1.0])],
            [0.0; 3],
            Vec::new(),
        ),
        Part::new(
            "handle",
            vec![cube(size, (26, 0), [-1.0, -3.0, -1.0], [2.0, 6.0, 6.0])],
            [0.0; 3],
            Vec::new(),
        ),
    ]);
    vec![layer(&model, "entity/shield/shield_base_nopattern", WHITE)]
}

/// `ConduitRenderer.createShellLayer`.
fn conduit() -> Vec<ModelLayer> {
    let model = root(vec![Part::new(
        "shell",
        vec![cube(
            (32.0, 16.0),
            (0, 0),
            [-3.0, -3.0, -3.0],
            [6.0, 6.0, 6.0],
        )],
        [0.0; 3],
        Vec::new(),
    )]);
    vec![layer(&model, "entity/conduit/base", WHITE)]
}

/// `DecoratedPotRenderer.createBaseLayer` and `createSidesLayer`, every side
/// the plain one.
fn decorated_pot() -> Vec<ModelLayer> {
    use std::f32::consts::PI;
    let base_size = (32.0, 32.0);
    let neck = Part::posed(
        "neck",
        vec![
            grown(base_size, (0, 0), [4.0, 17.0, 4.0], [8.0, 3.0, 8.0], -0.1),
            grown(base_size, (0, 5), [5.0, 20.0, 5.0], [6.0, 1.0, 6.0], 0.2),
        ],
        [0.0, 37.0, 16.0],
        [PI, 0.0, 0.0],
        [1.0; 3],
        Vec::new(),
    );
    let plane = |name: &'static str, y: f32| {
        Part::new(
            name,
            vec![cube(
                base_size,
                (-14, 13),
                [0.0, 0.0, 0.0],
                [14.0, 0.0, 14.0],
            )],
            [1.0, y, 1.0],
            Vec::new(),
        )
    };
    let base = root(vec![neck, plane("top", 16.0), plane("bottom", 0.0)]);
    let side_size = (16.0, 16.0);
    let side = |name: &'static str, offset: [f32; 3], rotation: [f32; 3]| {
        Part::posed(
            name,
            vec![Cube::sized(
                (1, 0),
                [0.0, 0.0, 0.0],
                [14.0, 16.0, 0.0],
                0.0,
                false,
                side_size,
                &[Direction::North],
            )],
            offset,
            rotation,
            [1.0; 3],
            Vec::new(),
        )
    };
    let sides = root(vec![
        side("back", [15.0, 16.0, 1.0], [0.0, 0.0, PI]),
        side("left", [1.0, 16.0, 1.0], [0.0, -PI / 2.0, PI]),
        side("right", [15.0, 16.0, 15.0], [0.0, PI / 2.0, PI]),
        side("front", [1.0, 16.0, 15.0], [PI, 0.0, 0.0]),
    ]);
    vec![
        layer(&base, "entity/decorated_pot/decorated_pot_base", WHITE),
        layer(&sides, "entity/decorated_pot/decorated_pot_side", WHITE),
    ]
}

/// `CopperGolemModel.createBodyLayer` as the statue poses it: standing, the
/// whole model turned over by `zRot = π` at the origin.
fn copper_golem(texture: &str) -> Vec<ModelLayer> {
    let size = (64.0, 64.0);
    let head = Part::new(
        "head",
        vec![
            grown(size, (0, 0), [-4.0, -5.0, -5.0], [8.0, 5.0, 10.0], 0.015),
            cube(size, (56, 0), [-1.0, -2.0, -6.0], [2.0, 3.0, 2.0]),
            grown(size, (37, 8), [-1.0, -9.0, -1.0], [2.0, 4.0, 2.0], -0.015),
            grown(size, (37, 0), [-2.0, -13.0, -2.0], [4.0, 4.0, 4.0], -0.015),
        ],
        [0.0, -6.0, 0.0],
        Vec::new(),
    );
    let body = Part::new(
        "body",
        vec![cube(size, (0, 15), [-4.0, -6.0, -3.0], [8.0, 6.0, 6.0])],
        [0.0, -5.0, 0.0],
        vec![
            head,
            Part::new(
                "right_arm",
                vec![cube(size, (36, 16), [-3.0, -1.0, -2.0], [3.0, 10.0, 4.0])],
                [-4.0, -6.0, 0.0],
                Vec::new(),
            ),
            Part::new(
                "left_arm",
                vec![cube(size, (50, 16), [0.0, -1.0, -2.0], [3.0, 10.0, 4.0])],
                [4.0, -6.0, 0.0],
                Vec::new(),
            ),
        ],
    );
    let model = Part::posed(
        "root",
        Vec::new(),
        [0.0; 3],
        [0.0, 0.0, std::f32::consts::PI],
        [1.0; 3],
        vec![
            body,
            Part::new(
                "right_leg",
                vec![cube(size, (0, 27), [-4.0, 0.0, -2.0], [4.0, 5.0, 4.0])],
                [0.0, -5.0, 0.0],
                Vec::new(),
            ),
            Part::new(
                "left_leg",
                vec![cube(size, (16, 27), [0.0, 0.0, -2.0], [4.0, 5.0, 4.0])],
                [0.0, -5.0, 0.0],
                Vec::new(),
            ),
        ],
    );
    vec![layer(&model, texture, WHITE)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_has_faces_inside_a_few_blocks() {
        let kinds = [
            SpecialKind::Chest {
                texture: "entity/chest/normal".into(),
            },
            SpecialKind::ShulkerBox {
                texture: "entity/shulker/shulker".into(),
            },
            SpecialKind::Head {
                kind: "skeleton".into(),
                texture: "entity/skeleton/skeleton".into(),
            },
            SpecialKind::Head {
                kind: "zombie".into(),
                texture: "entity/zombie/zombie".into(),
            },
            SpecialKind::Head {
                kind: "piglin".into(),
                texture: "entity/piglin/piglin".into(),
            },
            SpecialKind::Head {
                kind: "dragon".into(),
                texture: "entity/enderdragon/dragon".into(),
            },
            SpecialKind::Banner { color: WHITE },
            SpecialKind::Shield,
            SpecialKind::Conduit,
            SpecialKind::DecoratedPot,
            SpecialKind::CopperGolemStatue {
                texture: "entity/copper_golem/copper_golem".into(),
            },
        ];
        for kind in kinds {
            let layers = layers(&kind);
            assert!(!layers.is_empty(), "{kind:?}");
            for layer in layers {
                assert!(!layer.quads.is_empty(), "{kind:?}");
                for quad in &layer.quads {
                    for vertex in quad {
                        assert!(
                            vertex.position.iter().all(|axis| axis.abs() < 4.0),
                            "{kind:?}"
                        );
                        assert!(
                            vertex.uv.iter().all(|axis| (-2.0..=2.0).contains(axis)),
                            "{kind:?} {:?}",
                            vertex.uv
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_shulker_box_is_two_blocks_of_sixteen_wide() {
        let layers = layers(&SpecialKind::ShulkerBox {
            texture: "entity/shulker/shulker".into(),
        });
        let (mut low, mut high) = (f32::MAX, f32::MIN);
        for quad in &layers[0].quads {
            for vertex in quad {
                low = low.min(vertex.position[0]);
                high = high.max(vertex.position[0]);
            }
        }
        assert!((high - low - 1.0).abs() < 1e-6);
    }
}
