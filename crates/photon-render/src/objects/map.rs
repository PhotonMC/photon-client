//! A map drawn (`MapRenderer.render`): its 128 by 128 quad, then each decoration's sprite, turned
//! and placed by its place on the map, each a little in front of the one before.

use crate::joml::{Pose, Quaternion};

/// The side of a map in units of its own quad.
pub const MAP_SIZE: f32 = 128.0;

/// What `extractDecorationRenderState` keeps of a decoration that is drawn: where it is, how it
/// is turned and where its sprite is in its atlas (`u0, v0, u1, v1`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecorationDraw {
    pub x: i8,
    pub y: i8,
    pub rot: i8,
    pub render_on_frame: bool,
    pub sprite: [f32; 4],
}

/// Which image a quad samples: the map's own, or the decorations' atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sheet {
    Map,
    Decorations,
}

/// One quad: its four corners through the pose (top left to the bottom, as submitted) and their
/// texture coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub sheet: Sheet,
    pub corners: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4],
}

/// `MapRenderer.render`'s geometry: `pose` is where the map's corner is (the frame's pose after
/// `translate(-64, -64, -1)`); a frame shows only the decorations that say so.
pub fn map_quads(pose: &Pose, decorations: &[DecorationDraw], show_only_frame: bool) -> Vec<Quad> {
    let place = |pose: &Pose, corners: [[f32; 3]; 4]| {
        corners.map(|[x, y, z]| pose.pose.transform_position(x, y, z))
    };
    let mut quads = vec![Quad {
        sheet: Sheet::Map,
        corners: place(
            pose,
            [
                [0.0, MAP_SIZE, -0.01],
                [MAP_SIZE, MAP_SIZE, -0.01],
                [MAP_SIZE, 0.0, -0.01],
                [0.0, 0.0, -0.01],
            ],
        ),
        uvs: [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    }];
    let mut count = 0_i32;
    for decoration in decorations {
        if show_only_frame && !decoration.render_on_frame {
            continue;
        }
        let mut at = *pose;
        at.translate(
            f32::from(decoration.x) / 2.0 + 64.0,
            f32::from(decoration.y) / 2.0 + 64.0,
            -0.02,
        );
        at.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            (i32::from(decoration.rot) * 360) as f32 / 16.0,
        )));
        at.scale(4.0, 4.0, 3.0);
        at.translate(-0.125, 0.125, 0.0);
        let z = count as f32 * -0.001;
        let [u0, v0, u1, v1] = decoration.sprite;
        quads.push(Quad {
            sheet: Sheet::Decorations,
            corners: place(
                &at,
                [
                    [-1.0, 1.0, z],
                    [1.0, 1.0, z],
                    [1.0, -1.0, z],
                    [-1.0, -1.0, z],
                ],
            ),
            uvs: [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
        });
        count += 1;
    }
    quads
}
