//! Which style of a line of text a point is over (`ActiveTextCollector.
//! findElementUnderCursor`): each glyph has an active area, the glyph's box
//! across its advance, and the last glyph whose area holds the point gives
//! its style.

use voxel_game::text::{Run, Style};

use crate::gui::font::{Font, Glyph};

/// `extraThickness`: what a bold glyph is drawn wider by, each side.
const BOLD_THICKNESS: f32 = 0.1;

/// The style at `point` of `runs` drawn with their top left at `origin` (and
/// a shadow, as chat text has).
pub fn style_under<'a>(
    font: &Font,
    runs: &'a [Run],
    (x, y): (i32, i32),
    (point_x, point_y): (f32, f32),
) -> Option<&'a Style> {
    let mut found = None;
    let mut pen = x as f32;
    let y = y as f32;
    for run in runs {
        let bold = run.style.bold == Some(true);
        let italic = run.style.italic == Some(true);
        for codepoint in run.text.chars() {
            let Some(glyph) = font.glyph(codepoint) else {
                continue;
            };
            let (left, top, right, bottom) = match glyph {
                // `EmptyArea`: the advance wide and a line tall.
                Glyph::Space(_) => (pen, y, pen + glyph.advance(bold), y + 9.0),
                Glyph::Bitmap(bitmap) => {
                    let up = 7.0 - bitmap.ascent as f32;
                    let down = up + bitmap.cell[3] as f32 * bitmap.scale;
                    let shear = if italic {
                        (1.0 - 0.25 * up).min(1.0 - 0.25 * down)
                    } else {
                        0.0
                    };
                    let thickness = if bold { BOLD_THICKNESS } else { 0.0 };
                    (
                        pen + shear - thickness,
                        y + up - thickness,
                        pen + glyph.advance(bold),
                        // The shadow is a unit down.
                        y + down + 1.0 + thickness,
                    )
                }
            };
            if point_x >= left && point_x < right && point_y >= top && point_y < bottom {
                found = Some(&run.style);
            }
            pen += glyph.advance(bold);
        }
    }
    found
}

/// The box the glyphs of `runs` fill when drawn with their top left at
/// `origin` (`PreparedText.bounds`): as whole units, or none if no glyph is
/// drawn.
pub fn bounds(
    font: &Font,
    runs: &[Run],
    (x, y): (i32, i32),
    shadow: bool,
) -> Option<(i32, i32, i32, i32)> {
    let mut found: Option<(f32, f32, f32, f32)> = None;
    let mut pen = x as f32;
    let y = y as f32;
    let shadow_offset = if shadow { 1.0 } else { 0.0 };
    for run in runs {
        let bold = run.style.bold == Some(true);
        let italic = run.style.italic == Some(true);
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
