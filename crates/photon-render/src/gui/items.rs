//! Items drawn into the GUI, as the reference draws an item into its GUI item
//! atlas and blits it into a 16 by 16 slot: the item's GUI transform, lit by
//! one of the two item lights, the texel times its tint.

use crate::block::world_mesh::ItemLayer;
use crate::model::ItemTransform;

/// One vertex of an item in the GUI: GUI units, with depth towards the
/// viewer, its atlas coordinate and its lit, tinted colour.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiItemVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub colour: [f32; 4],
}

type Matrix = [[f32; 3]; 3];

fn multiply(a: Matrix, b: Matrix) -> Matrix {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| (0..3).map(|k| a[row][k] * b[k][column]).sum())
    })
}

fn apply(m: Matrix, v: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|row| (0..3).map(|k| m[row][k] * v[k]).sum())
}

fn rotation_x(angle: f32) -> Matrix {
    let (s, c) = angle.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

fn rotation_y(angle: f32) -> Matrix {
    let (s, c) = angle.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

fn rotation_z(angle: f32) -> Matrix {
    let (s, c) = angle.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length == 0.0 {
        v
    } else {
        v.map(|axis| axis / length)
    }
}

const DIFFUSE_LIGHT_0: [f32; 3] = [0.2, 1.0, -0.7];
const DIFFUSE_LIGHT_1: [f32; 3] = [-0.2, 1.0, 0.7];

/// `Lighting`'s two item light pairs: flat items lit from the front, block
/// items lit like a block seen from above and to the side.
fn item_lights(side_lit: bool) -> [[f32; 3]; 2] {
    use std::f32::consts::PI;
    let pose = if side_lit {
        let scale = [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]];
        let first = multiply(rotation_y(1.082_104_1), rotation_x(3.237_585_8));
        let second = multiply(rotation_y(-PI / 8.0), rotation_x(PI * 3.0 / 4.0));
        multiply(multiply(scale, first), second)
    } else {
        multiply(rotation_y(-PI / 8.0), rotation_x(PI * 3.0 / 4.0))
    };
    [
        apply(pose, normalise(DIFFUSE_LIGHT_0)),
        apply(pose, normalise(DIFFUSE_LIGHT_1)),
    ]
}

/// `minecraft_mix_light`: ambient plus the two lights, capped at one.
fn mix_light(lights: [[f32; 3]; 2], normal: [f32; 3]) -> f32 {
    let dot = |light: [f32; 3]| {
        (light[0] * normal[0] + light[1] * normal[1] + light[2] * normal[2]).max(0.0)
    };
    ((dot(lights[0]) + dot(lights[1])) * 0.6 + 0.4).min(1.0)
}

/// `ItemFeatureRenderer.prepareFoilSubmit` for an item in a GUI slot: the same triangles again,
/// their atlas coordinate moved by the glint's matrix into the glint texture at `glint`
/// (`[u0, v0, u1, v1]` in the packed image), which the colour slot carries as its rectangle.
pub fn glint_vertices(
    drawn: &[GuiItemVertex],
    matrix: &[f32; 16],
    glint: [f32; 4],
    decal: Option<&[[f32; 2]]>,
) -> Vec<GuiItemVertex> {
    let size = [glint[2] - glint[0], glint[3] - glint[1]];
    drawn
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let uv = decal.map_or(vertex.uv, |corners| {
                corners[crate::objects::item::foil_decal_index(
                    index,
                    [0, 1, 2, 2, 3, 0],
                    corners.len(),
                )]
            });
            let moved = crate::render_types::glint_uv(matrix, uv);
            GuiItemVertex {
                position: vertex.position,
                uv: [glint[0] + moved[0] * size[0], glint[1] + moved[1] * size[1]],
                colour: [glint[0], glint[1], size[0], size[1]],
            }
        })
        .collect()
}

/// The vertices of an item drawn in the 16 by 16 slot whose corner is
/// `(x, y)` in GUI units.
pub fn item_vertices(layers: &[ItemLayer], x: f32, y: f32, out: &mut Vec<GuiItemVertex>) {
    for layer in layers {
        // `GuiItemAtlas.drawToSlot`: the slot's centre, a block the slot's
        // size, y up; then `ItemTransform.apply` in the GUI context.
        let (rotation, translation, scale) = match layer.transform {
            Some(ItemTransform {
                rotation,
                translation,
                scale,
            }) => {
                let radians = rotation.map(f32::to_radians);
                let turn = multiply(
                    multiply(rotation_x(radians[0]), rotation_y(radians[1])),
                    rotation_z(radians[2]),
                );
                (turn, translation, scale)
            }
            None => (
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                [0.0; 3],
                [1.0; 3],
            ),
        };
        let place = |corner: [f32; 3]| {
            let centred = [corner[0] - 0.5, corner[1] - 0.5, corner[2] - 0.5];
            let scaled = [
                centred[0] * scale[0],
                centred[1] * scale[1],
                centred[2] * scale[2],
            ];
            let turned = apply(rotation, scaled);
            let moved = [
                turned[0] + translation[0],
                turned[1] + translation[1],
                turned[2] + translation[2],
            ];
            [
                x + 8.0 + moved[0] * 16.0,
                y + 8.0 - moved[1] * 16.0,
                moved[2] * 16.0,
            ]
        };
        // `PoseStack.scale`: a uniform scale leaves normals alone but for its
        // signs; the slot's flipped y flips them.
        let normal_of = |direction: photon_core::direction::Direction| {
            let (dx, dy, dz) = direction.step();
            let unit = [dx as f32, dy as f32, dz as f32];
            let signed = [
                unit[0] * scale[0].signum(),
                unit[1] * scale[1].signum(),
                unit[2] * scale[2].signum(),
            ];
            let turned = normalise(apply(rotation, signed));
            [turned[0], -turned[1], turned[2]]
        };
        let lights = item_lights(layer.side_lit);
        for (quad, info) in layer.mesh.quads.iter().enumerate() {
            let brightness = mix_light(lights, normal_of(info.direction));
            let vertex = |corner: usize| {
                let source = &layer.mesh.vertices[quad * 4 + corner];
                let tint = source.colour();
                GuiItemVertex {
                    position: place(source.position),
                    uv: source.uv,
                    colour: [
                        tint[0] * brightness,
                        tint[1] * brightness,
                        tint[2] * brightness,
                        tint[3],
                    ],
                }
            };
            out.extend([0, 1, 2, 2, 3, 0].map(vertex));
        }
    }
}

#[cfg(test)]
mod glint_tests {
    use super::*;

    #[test]
    fn a_glint_keeps_the_positions_and_carries_its_rectangle() {
        let drawn = [GuiItemVertex {
            position: [3.0, 4.0, 5.0],
            uv: [0.25, 0.5],
            colour: [1.0; 4],
        }];
        let identity = crate::render_types::glint_texture_matrix(0, 0.0, 1.0);
        let rect = [0.5, 0.25, 0.75, 0.5];
        let glint = glint_vertices(&drawn, &identity, rect, None);
        assert_eq!(glint[0].position, drawn[0].position);
        assert_eq!(glint[0].colour, [0.5, 0.25, 0.25, 0.25]);
    }
}
