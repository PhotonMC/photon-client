//! A sign's text (`AbstractSignRenderer`, `StandingSignRenderer`, `HangingSignRenderer`): where
//! each face's lines are drawn, in what colour and light, and with what outline. The board and
//! the post are the block's own model.

use photon_core::direction::Direction;

use crate::block_entity::skull::segment_degrees;
use crate::gui::font::Font;
use crate::joml::{Matrix4, Pose, Quaternion};

/// `Font.DisplayMode`'s light for text that glows (`LightCoordsUtil.FULL_BRIGHT`).
pub const FULL_BRIGHT: i32 = 0x00F0_00F0;

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

/// One `submitText` a sign makes: the pose, the line's left edge and top, the line, the light,
/// the colour and the outline's colour (none when 0). Signs never drop a shadow and draw with
/// `Font.DisplayMode.POLYGON_OFFSET`.
#[derive(Debug, Clone, PartialEq)]
pub struct SignTextSubmit {
    pub pose: Pose,
    pub x: f32,
    pub y: f32,
    pub line: Vec<crate::gui::TextRun<'static>>,
    pub light: i32,
    pub color: u32,
    pub outline: u32,
}

/// The world triangles of a sign's text (`TextFeatureRenderer`): an outlined line is drawn eight
/// times a glyph's shadow offset around in the outline's colour (whatever the line's styles say)
/// then once over them in its own colour.
pub fn sign_text_vertices(
    submits: &[SignTextSubmit],
    (font, atlas): (&Font, &crate::gui::GuiAtlas),
) -> Vec<crate::scene::Vertex> {
    use crate::gui::frame::{effect_quads, glyph_quads};
    use crate::gui::text_effect::effects;
    use crate::objects::display::world_glyphs;
    let mut vertices = Vec::new();
    for submit in submits {
        let light = crate::block::light::light_uv(submit.light);
        if submit.outline != 0 {
            let plain: Vec<crate::gui::TextRun> = submit
                .line
                .iter()
                .map(|run| crate::gui::TextRun {
                    colour: None,
                    ..run.clone()
                })
                .collect();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let at = (submit.x + dx as f32, submit.y + dy as f32);
                    let mut quads = glyph_quads(atlas, font, &plain, at, submit.outline, false);
                    let bars = effects(font, &plain, at, (submit.outline, false));
                    quads.extend(effect_quads(atlas, bars));
                    world_glyphs(&submit.pose.pose, quads, light, &mut vertices);
                }
            }
        }
        let at = (submit.x, submit.y);
        let mut quads = glyph_quads(atlas, font, &submit.line, at, submit.color, false);
        let bars = effects(font, &submit.line, at, (submit.color, false));
        quads.extend(effect_quads(atlas, bars));
        world_glyphs(&submit.pose.pose, quads, light, &mut vertices);
    }
    vertices
}
