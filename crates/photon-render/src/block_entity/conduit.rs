//! The conduit's shell (`ConduitRenderer`): the shell of a sleeping one, and the cage, the two
//! turning winds and the eye of a waking one, each with the pose it is drawn from.

use crate::joml::{Pose, Quaternion, normalize};
use photon_core::math::mth_sin_f64;

/// The sprites under `textures/`, in the block atlas (`ConduitRenderer.MAPPER`).
pub const SHELL: &str = "entity/conduit/base";
pub const CAGE: &str = "entity/conduit/cage";
pub const WIND: &str = "entity/conduit/wind";
pub const VERTICAL_WIND: &str = "entity/conduit/wind_vertical";
pub const OPEN_EYE: &str = "entity/conduit/open_eye";
pub const CLOSED_EYE: &str = "entity/conduit/closed_eye";

/// Every sprite the shell is drawn in.
pub const SPRITES: [&str; 6] = [SHELL, CAGE, WIND, VERTICAL_WIND, OPEN_EYE, CLOSED_EYE];

/// The model layers of the four parts.
pub const SHELL_LAYER: &str = "minecraft:conduit#shell";
pub const CAGE_LAYER: &str = "minecraft:conduit#cage";
pub const WIND_LAYER: &str = "minecraft:conduit#wind";
pub const EYE_LAYER: &str = "minecraft:conduit#eye";

/// How many ticks a phase of the wind lasts (`tickCount / 66 % 3`).
const PHASE_TICKS: i32 = 66;

/// What a conduit is drawn from at a partial tick (`ConduitRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConduitFrame {
    pub active: bool,
    pub hunting: bool,
    /// `getActiveRotation`, turned by `-0.0375` a tick.
    pub active_rotation: f32,
    /// `tickCount + partialTicks`.
    pub anim_time: f32,
    /// `tickCount / 66 % 3`.
    pub phase: i32,
}

impl ConduitFrame {
    /// `ConduitRenderer.extractRenderState`: the numbers of a conduit that has counted `rotation`
    /// turns and `tick_count` ticks, at a partial tick.
    pub fn of(
        active: bool,
        hunting: bool,
        (tick_count, rotation): (i32, i32),
        partial: f32,
    ) -> Self {
        let turned = if active { partial } else { 0.0 };
        Self {
            active,
            hunting,
            active_rotation: (rotation as f32 + turned) * -0.0375,
            anim_time: tick_count as f32 + partial,
            phase: tick_count / PHASE_TICKS % 3,
        }
    }
}

/// One part of the shell as it is drawn: which model, in which sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Shell,
    Cage,
    /// A wind, in the vertical sprite or not.
    Wind {
        vertical: bool,
    },
    Eye {
        open: bool,
    },
}

impl Piece {
    /// The model layer the piece is drawn from.
    pub fn layer(self) -> &'static str {
        match self {
            Piece::Shell => SHELL_LAYER,
            Piece::Cage => CAGE_LAYER,
            Piece::Wind { .. } => WIND_LAYER,
            Piece::Eye { .. } => EYE_LAYER,
        }
    }

    /// The sprite the piece is drawn in.
    pub fn sprite(self) -> &'static str {
        match self {
            Piece::Shell => SHELL,
            Piece::Cage => CAGE,
            Piece::Wind { vertical: true } => VERTICAL_WIND,
            Piece::Wind { vertical: false } => WIND,
            Piece::Eye { open: true } => OPEN_EYE,
            Piece::Eye { open: false } => CLOSED_EYE,
        }
    }
}

fn degrees_to_radians() -> f32 {
    (std::f64::consts::PI / 180.0) as f32
}

/// `ConduitRenderer.submit`: each piece with the pose it is submitted from, in order. `base` is
/// the pose of the block's corner and `orientation` the camera's, which the eye faces.
pub fn pieces(base: &Pose, frame: &ConduitFrame, orientation: &Quaternion) -> Vec<(Piece, Pose)> {
    let half = std::f32::consts::FRAC_PI_2;
    let pi = std::f32::consts::PI;
    if !frame.active {
        let mut pose = *base;
        pose.translate(0.5, 0.5, 0.5);
        pose.rotate(&Quaternion::rotation_y(
            frame.active_rotation * degrees_to_radians(),
        ));
        return vec![(Piece::Shell, pose)];
    }
    let rotation = frame.active_rotation * (180.0 / pi);
    let mut bob = mth_sin_f64(f64::from(frame.anim_time * 0.1)) / 2.0 + 0.5;
    bob = bob * bob + bob;
    let height = 0.3 + bob * 0.2;
    let mut cage = *base;
    cage.translate(0.5, height, 0.5);
    let axis = normalize([0.5, 1.0, 0.5]);
    cage.rotate(&Quaternion::identity().rotate_axis(
        rotation * degrees_to_radians(),
        axis[0],
        axis[1],
        axis[2],
    ));
    let mut wind = *base;
    wind.translate(0.5, 0.5, 0.5);
    match frame.phase {
        1 => wind.rotate(&Quaternion::rotation_x(half)),
        2 => wind.rotate(&Quaternion::rotation_z(half)),
        _ => {}
    }
    let kind = Piece::Wind {
        vertical: frame.phase == 1,
    };
    let mut inner = *base;
    inner.translate(0.5, 0.5, 0.5);
    inner.scale(0.875, 0.875, 0.875);
    inner.rotate(&Quaternion::rotation_xyz(pi, 0.0, pi));
    let mut eye = *base;
    eye.translate(0.5, height, 0.5);
    eye.scale(0.5, 0.5, 0.5);
    eye.rotate(orientation);
    eye.rotate(&Quaternion::rotation_z(pi).rotate_y(pi));
    eye.scale(1.333_333_4, 1.333_333_4, 1.333_333_4);
    vec![
        (Piece::Cage, cage),
        (kind, wind),
        (kind, inner),
        (
            Piece::Eye {
                open: frame.hunting,
            },
            eye,
        ),
    ]
}
