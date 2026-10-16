//! The box the glyphs of text fill (`PreparedText.bounds`).

use crate::gui::font::{Font, Glyph};

/// `extraThickness`: what a bold glyph is drawn wider by, each side.
const BOLD_THICKNESS: f32 = 0.1;

/// The box the glyphs of `runs` fill when drawn with their top left at
/// `origin` (`PreparedText.bounds`): as whole units, or none if no glyph is
/// drawn.
pub fn bounds(
    font: &Font,
    runs: &[crate::gui::TextRun],
    (x, y): (i32, i32),
    shadow: bool,
) -> Option<(i32, i32, i32, i32)> {
    let mut found: Option<(f32, f32, f32, f32)> = None;
    let mut pen = x as f32;
    let y = y as f32;
    let shadow_offset = if shadow { 1.0 } else { 0.0 };
    for run in runs {
        let bold = run.bold;
        let italic = run.italic;
        for codepoint in run.text.chars() {
            let Some(glyph) = font.glyph(codepoint) else {
                continue;
            };
            if let Glyph::Bitmap(bitmap) = glyph {
                let up = 7.0 - bitmap.ascent as f32;
                let down = up + bitmap.cell[3] as f32 * bitmap.scale;
                let (shear_top, shear_bottom) = (1.0 - 0.25 * up, 1.0 - 0.25 * down);
                let (shear_min, shear_max) = if italic {
                    (shear_top.min(shear_bottom), shear_top.max(shear_bottom))
                } else {
                    (0.0, 0.0)
                };
                let thickness = if bold { BOLD_THICKNESS } else { 0.0 };
                let width = bitmap.cell[2] as f32 * bitmap.scale;
                let rect = (
                    pen + shear_min - thickness,
                    y + up - thickness,
                    pen + width + shadow_offset + shear_max + thickness,
                    y + down + shadow_offset + thickness,
                );
                found = Some(match found {
                    None => rect,
                    Some(union) => (
                        union.0.min(rect.0),
                        union.1.min(rect.1),
                        union.2.max(rect.2),
                        union.3.max(rect.3),
                    ),
                });
            }
            pen += glyph.advance(bold);
        }
    }
    found
        .filter(|rect| rect.0 < rect.2 && rect.1 < rect.3)
        .map(|(left, top, right, bottom)| {
            (
                left.floor() as i32,
                top.floor() as i32,
                right.ceil() as i32,
                bottom.ceil() as i32,
            )
        })
}
