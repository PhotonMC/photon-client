//! The box the glyphs of text fill (`PreparedText.bounds`).

use crate::gui::font::{Font, Glyph};

/// `extraThickness`: what a bold glyph is drawn wider by, each side.
const BOLD_THICKNESS: f32 = 0.1;

type Rect = (f32, f32, f32, f32);

/// `found` grown to hold `rect`.
fn union(found: Option<Rect>, rect: Rect) -> Rect {
    match found {
        None => rect,
        Some(old) => (
            old.0.min(rect.0),
            old.1.min(rect.1),
            old.2.max(rect.2),
            old.3.max(rect.3),
        ),
    }
}

/// A glyph's width, top and bottom as it draws them, for one that draws anything.
fn extent(glyph: &Glyph) -> Option<(f32, f32, f32)> {
    match glyph {
        Glyph::Space(_) => None,
        Glyph::Bitmap(bitmap) => {
            let up = 7.0 - bitmap.ascent as f32;
            Some((
                bitmap.cell[2] as f32 * bitmap.scale,
                up,
                up + bitmap.cell[3] as f32 * bitmap.scale,
            ))
        }
        Glyph::Unihex(unihex) => Some((unihex.width() as f32 / 2.0, 0.0, 8.0)),
    }
}

/// The box one glyph fills at `pen` (`up`, `down` and the shear as the glyph draws them).
fn glyph_rect(
    (width, up, down): (f32, f32, f32),
    (pen, y): (f32, f32),
    run: &crate::gui::TextRun,
    shadow_offset: f32,
) -> Rect {
    let (shear_top, shear_bottom) = (1.0 - 0.25 * up, 1.0 - 0.25 * down);
    let (shear_min, shear_max) = if run.italic {
        (shear_top.min(shear_bottom), shear_top.max(shear_bottom))
    } else {
        (0.0, 0.0)
    };
    let thickness = if run.bold { BOLD_THICKNESS } else { 0.0 };
    (
        pen + shear_min - thickness,
        y + up - thickness,
        pen + width + shadow_offset + shear_max + thickness,
        y + down + shadow_offset + thickness,
    )
}

/// The box the glyphs of `runs` fill when drawn with their top left at
/// `origin` (`PreparedText.bounds`): as whole units, or none if no glyph is
/// drawn.
pub fn bounds(
    font: &Font,
    runs: &[crate::gui::TextRun],
    (x, y): (i32, i32),
    shadow: bool,
) -> Option<(i32, i32, i32, i32)> {
    let mut found: Option<Rect> = None;
    let mut pen = x as f32;
    let y = y as f32;
    for run in runs {
        let bold = run.bold;
        for codepoint in run.text.chars() {
            let Some(glyph) = font.glyph(codepoint) else {
                continue;
            };
            if let Some(extent) = extent(&glyph) {
                let shadow_offset = if shadow { glyph.shadow_offset() } else { 0.0 };
                found = Some(union(
                    found,
                    glyph_rect(extent, (pen, y), run, shadow_offset),
                ));
            }
            pen += glyph.advance(bold);
        }
    }
    // `PreparedTextBuilder.addEffect`: an underline or strikethrough counts, with its shadow.
    for effect in crate::gui::text_effect::effects(font, runs, (x as f32, y), (u32::MAX, shadow)) {
        let (left, top, right, bottom) = effect.rect;
        let offset = if effect.shadow_colour != 0 {
            effect.shadow_offset
        } else {
            0.0
        };
        let rect = (left, top, right + offset, bottom + offset);
        found = Some(union(found, rect));
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
