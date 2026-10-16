//! What a sign's text is made of (`AbstractSignRenderer.submit`): each face's lines split to the
//! sign's width, in the dye's colour and the face's light. The renderer draws the result.

use photon_game::sign::{BLACK, DYE_TEXT_COLORS};
use photon_game::text::Run;
use photon_render::block_entity::sign::{FULL_BRIGHT, SignKind, SignTextSubmit};
use photon_render::block_entity::skull::block_pose;
use photon_render::gui::font::Font;
use photon_render::joml::{Matrix4, Pose};

use crate::runs::{FontRuns, draw_runs};

/// One face's text as the render state holds it: the four lines (written or filtered, as the
/// player filters text) in words, the dye (an index into `DYES`) and whether it glows.
#[derive(Debug, Clone, Copy)]
pub struct SignFace<'a> {
    pub lines: &'a [Vec<Run>; 4],
    pub color: usize,
    pub glowing: bool,
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
            let line = crate::chat::split(line, kind.max_line_width(), font)
                .into_iter()
                .next()
                .unwrap_or_default();
            let x = (-font.width_of_runs(&line) / 2) as f32;
            submits.push(SignTextSubmit {
                pose,
                x,
                y: (index as i32 * line_height - midpoint) as f32,
                line: draw_runs(&line)
                    .into_iter()
                    .map(photon_render::gui::TextRun::into_owned)
                    .collect(),
                light,
                color,
                outline,
            });
        }
    }
    submits
}
