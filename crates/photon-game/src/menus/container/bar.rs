//! The bar `GuiGraphics.itemDecorations` draws under an item:
//! `Item.isBarVisible`, `getBarWidth` and `getBarColor`, which a bundle
//! overrides to show how full it is.

use super::components::{damage, max_damage};
use super::*;

/// `BundleItem.FULL_BAR_COLOR`: `ARGB.colorFromFloat(1, 1, 0.33, 0.33)`.
const BUNDLE_FULL: i32 = 0xFFFF_5454_u32 as i32;
/// `BundleItem.BAR_COLOR`: `ARGB.colorFromFloat(1, 0.44, 0.53, 1)`.
const BUNDLE_BAR: i32 = 0xFF70_87FF_u32 as i32;

/// The bar under a stack: its width in pixels (of 13) and its colour, or
/// none when it is not shown.
pub fn item_bar(stack: &Stack) -> Option<(i32, i32)> {
    if let Some((numerator, denominator)) = bundle::bundle_weight(stack) {
        if numerator <= 0 {
            return None;
        }
        // `Math.min(1 + Mth.mulAndTruncate(weight, 12), 13)`, in `int`s that wrap.
        let truncated =
            photon_core::math::mth_mul_and_truncate(numerator as i32, denominator as i32, 12);
        let width = 1_i32.wrapping_add(truncated).min(13);
        let colour = if numerator >= denominator {
            BUNDLE_FULL
        } else {
            BUNDLE_BAR
        };
        return Some((width, colour));
    }
    if !is_damaged(stack) {
        return None;
    }
    let (damage, max) = (damage(stack) as f32, max_damage(stack) as f32);
    // `Math.round`.
    let width = photon_core::math::floor_f32(13.0 - damage * 13.0 / max + 0.5).clamp(0, 13);
    let health = ((max - damage) / max).max(0.0);
    let colour = photon_core::math::mth_hsv_to_rgb_f32(health / 3.0, 1.0, 1.0).unwrap_or(0);
    Some((width, colour))
}
