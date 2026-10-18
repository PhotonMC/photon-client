//! The dark veil over the view while the local player sleeps (`Hud.extractSleepOverlay`).

use photon_render::gui::{GuiAtlas, GuiFrame};

/// The veil's colour for a sleep timer of a hundred and ten at most: it darkens for a hundred
/// ticks and lifts in the ten after waking. Nothing while the timer is zero.
pub fn colour(timer: i32) -> Option<u32> {
    if timer <= 0 {
        return None;
    }
    let timer = timer as f32;
    let mut amount = timer / 100.0;
    if amount > 1.0 {
        amount = 1.0 - (timer - 100.0) / 10.0;
    }
    Some(((220.0 * amount) as i32 as u32) << 24 | 1_052_704)
}

/// The veil over the whole GUI.
pub fn draw(frame: &mut GuiFrame, atlas: &GuiAtlas, (width, height): (i32, i32), timer: i32) {
    if let Some(colour) = colour(timer) {
        frame.fill(atlas, (0, 0, width, height), colour);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_veil_darkens_for_a_hundred_ticks_and_lifts_in_ten() {
        assert_eq!(colour(0), None);
        assert_eq!(colour(100), Some(0xDC10_1020));
        assert_eq!(colour(50), Some(0x6E10_1020));
        assert_eq!(colour(105), Some(0x6E10_1020));
        assert_eq!(colour(109), Some(0x1610_1020));
    }
}
