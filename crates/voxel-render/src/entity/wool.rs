//! The colour a sheep's wool is multiplied by (`SheepRenderState.getWoolColor`).

use voxel_core::math::{floor_f32, frac_f32};
use voxel_data::entity_type::{DYE_COUNT, sheep_colour};

/// How many ticks a `jeb_` sheep spends from one dye to the next
/// (`ColorLerper.Type.SHEEP`).
const JEB_TICKS_PER_DYE: i32 = 25;

/// A sheep's wool of this dye: the colour, as `0xRRGGBB`.
pub fn wool_colour(dye: usize) -> u32 {
    sheep_colour(dye).unwrap_or(0xFF_FF_FF_FF) & 0x00FF_FFFF
}

/// `ColorLerper.getLerpedColor`: a `jeb_` sheep's wool, between the dyes in
/// turn as it ages.
pub fn jeb_colour(age_in_ticks: f32) -> u32 {
    let tick_count = floor_f32(age_in_ticks);
    let value = tick_count / JEB_TICKS_PER_DYE;
    let count = DYE_COUNT as i32;
    let from = sheep_colour((value % count) as usize).unwrap_or(0);
    let to = sheep_colour(((value + 1) % count) as usize).unwrap_or(0);
    let sub_step = ((tick_count % JEB_TICKS_PER_DYE) as f32 + frac_f32(age_in_ticks))
        / JEB_TICKS_PER_DYE as f32;
    // `ARGB.srgbLerp`: each channel on its own, in whole numbers.
    let lerp = |shift: u32| {
        let a = ((from >> shift) & 0xFF) as i32;
        let b = ((to >> shift) & 0xFF) as i32;
        (a + floor_f32(sub_step * (b - a) as f32)) as u32 & 0xFF
    };
    lerp(16) << 16 | lerp(8) << 8 | lerp(0)
}
