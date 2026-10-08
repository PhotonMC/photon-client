//! A sign's text (`AbstractSignRenderer`, `StandingSignRenderer`, `HangingSignRenderer`): where
//! each face's lines are drawn, in what colour and light, and with what outline. The board and
//! the post are the block's own model.

use voxel_core::direction::Direction;
use voxel_game::sign::{BLACK, DYE_TEXT_COLORS};
use voxel_game::text::{Run, Style};

use crate::block_entity::skull::{block_pose, segment_degrees};
use crate::gui::font::Font;
use crate::joml::{Matrix4, Pose, Quaternion};

/// `Font.DisplayMode`'s light for text that glows (`LightCoordsUtil.FULL_BRIGHT`).
const FULL_BRIGHT: i32 = 0x00F0_00F0;

/// `AbstractSignRenderer.OUTLINE_RENDER_DISTANCE`: glowing text is outlined within sixteen blocks.
pub const OUTLINE_DISTANCE_SQR: f64 = 16.0 * 16.0;

/// Which renderer draws a sign's text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignKind {
    /// A standing or wall sign (`StandingSignRenderer`).
    Standing,
    /// A ceiling or wall hanging sign (`HangingSignRenderer`).
    Hanging,
}

impl SignKind {
    /// `getMaxTextLineWidth`.
    pub fn max_line_width(self) -> i32 {
        match self {
            Self::Standing => 90,
            Self::Hanging => 60,
        }
    }

    /// `getTextLineHeight`.
    pub fn line_height(self) -> i32 {
        match self {
            Self::Standing => 10,
            Self::Hanging => 9,
        }
    }

    /// `TRANSFORMATIONS.wallTransformation` or `freeTransformations`: the front and the back
    /// text's transformation for a sign on a wall facing `wall`, or one standing (or hanging from
    /// a ceiling) turned by the sixteenth `rotation`.
    pub fn transformations(self, wall: Option<Direction>, rotation: i32) -> [Matrix4; 2] {
        let angle = wall.map_or_else(|| segment_degrees(rotation), Direction::to_y_rot);
        [true, false].map(|front| match self {
            Self::Standing => standing_text(wall.is_some(), angle, front),
            Self::Hanging => hanging_text(angle, front),
        })
    }
}

fn turn_y(matrix: &mut Matrix4, degrees: f32) {
    matrix.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        degrees,
    )));
}

/// `StandingSignRenderer.textTransformation`.
fn standing_text(on_wall: bool, angle: f32, front: bool) -> Matrix4 {
    let mut matrix = Matrix4::translation(0.5, 0.5, 0.5);
    turn_y(&mut matrix, -angle);
    if on_wall {
        matrix.translate(0.0, -0.3125, -0.4375);
    }
    if !front {
        turn_y(&mut matrix, 180.0);
    }
    matrix.translate(0.0, 0.333_333_34, 0.046_666_667);
    let scale = 0.010_416_667;
    matrix.scale(scale, -scale, scale);
    matrix
}

/// `HangingSignRenderer.textTransformation`.
fn hanging_text(angle: f32, front: bool) -> Matrix4 {
    let mut matrix = Matrix4::translation(0.5, 0.9375, 0.5);
    turn_y(&mut matrix, -angle);
    matrix.translate(0.0, -0.3125, 0.0);
    if !front {
        turn_y(&mut matrix, 180.0);
    }
    matrix.translate(0.0, -0.32, 0.073);
    let scale = 0.014_062_5;
    matrix.scale(scale, -scale, scale);
    matrix
}

/// One face's text as the render state holds it: the four lines (written or filtered, as the
/// player filters text) in words, the dye (an index into `DYES`) and whether it glows.
#[derive(Debug, Clone, Copy)]
pub struct SignFace<'a> {
    pub lines: &'a [Vec<Run>; 4],
    pub color: usize,
    pub glowing: bool,
}

/// One `submitText` a sign makes: the pose, the line's left edge and top, the line, the light,
/// the colour and the outline's colour (none when 0). Signs never drop a shadow and draw with
/// `Font.DisplayMode.POLYGON_OFFSET`.
#[derive(Debug, Clone, PartialEq)]
pub struct SignTextSubmit {
    pub pose: Pose,
    pub x: f32,
    pub y: f32,
    pub line: Vec<Run>,
    pub light: i32,
    pub color: u32,
    pub outline: u32,
}

/// `AbstractSignRenderer.getDarkColor`: glowing black text is a pale cream, any other dye
/// two-fifths as bright (`ARGB.scaleRGB`).
pub fn dark_color(color: usize, glowing: bool) -> u32 {
    let text = DYE_TEXT_COLORS[color];
    if color == BLACK && glowing {
        return 0xFFF0_EBCC;
    }
    [16, 8, 0]
        .into_iter()
        .fold(text & 0xFF00_0000, |sum, shift| {
            let channel = ((((text >> shift) & 0xFF) as f32 * 0.4) as u32).min(255);
            sum | channel << shift
        })
}

/// `AbstractSignRenderer.submit`: the lines of the front face then the back, each face in its
/// transformation; `base` is the pose of the block's corner, `draw_outline` whether the camera is
/// near enough (or scoping) for glowing text to be outlined, `light` the block's.
pub fn sign_text_submits(
    base: &Pose,
    kind: SignKind,
    (transformations, faces): (&[Matrix4; 2], [SignFace<'_>; 2]),
    (draw_outline, light): (bool, i32),
    font: &Font,
) -> Vec<SignTextSubmit> {
    let line_height = kind.line_height();
    let midpoint = 4 * line_height / 2;
    let mut submits = Vec::new();
    for (transformation, face) in transformations.iter().zip(faces) {
        let pose = block_pose(base, transformation);
        let dark = dark_color(face.color, face.glowing);
        let (color, outline, light) = if face.glowing {
            let color = DYE_TEXT_COLORS[face.color];
            let outlined = color == DYE_TEXT_COLORS[BLACK] || draw_outline;
            (color, if outlined { dark } else { 0 }, FULL_BRIGHT)
        } else {
            (dark, 0, light)
        };
        for (index, line) in face.lines.iter().enumerate() {
            // Only the first of the lines the text splits into at the sign's width is drawn.
            let line = crate::gui::chat::split(line, kind.max_line_width(), font)
                .into_iter()
                .next()
                .unwrap_or_default();
            let x = (-font.width_of_runs(&line) / 2) as f32;
            submits.push(SignTextSubmit {
                pose,
                x,
                y: (index as i32 * line_height - midpoint) as f32,
                line,
                light,
                color,
                outline,
            });
        }
    }
    submits
}

/// The world triangles of a sign's text (`TextFeatureRenderer`): an outlined line is drawn eight
/// times a glyph's shadow offset around in the outline's colour (whatever the line's styles say)
/// then once over them in its own colour.
pub fn sign_text_vertices(
    submits: &[SignTextSubmit],
    (font, atlas): (&Font, &crate::gui::GuiAtlas),
) -> Vec<crate::scene::Vertex> {
    use crate::gui::frame::glyph_quads;
    use crate::objects::display::world_glyphs;
    let mut vertices = Vec::new();
    for submit in submits {
        let light = crate::block::light::light_uv(submit.light);
        if submit.outline != 0 {
            let plain: Vec<Run> = submit
                .line
                .iter()
                .map(|run| Run {
                    text: run.text.clone(),
                    style: Style {
                        colour: None,
                        ..run.style.clone()
                    },
                })
                .collect();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let at = (submit.x + dx as f32, submit.y + dy as f32);
                    let quads = glyph_quads(atlas, font, &plain, at, submit.outline, false);
                    world_glyphs(&submit.pose.pose, quads, light, &mut vertices);
                }
            }
        }
        let quads = glyph_quads(
            atlas,
            font,
            &submit.line,
            (submit.x, submit.y),
            submit.color,
            false,
        );
        world_glyphs(&submit.pose.pose, quads, light, &mut vertices);
    }
    vertices
}
