//! The boss bars at the top of the screen (`BossHealthOverlay` and
//! `LerpingBossEvent`): what each keeps, how its progress eases, and how it
//! draws.

use crate::{FontRuns, FrameRuns};
use photon_game::text::Run;

use photon_render::gui::GuiAtlas;
use photon_render::gui::GuiFrame;
use photon_render::gui::font::Font;

const BAR_WIDTH: i32 = 182;
const BAR_HEIGHT: i32 = 5;
/// A progress change eases in over this many milliseconds.
const LERP_MILLISECONDS: u64 = 100;

const COLORS: [&str; 7] = ["pink", "blue", "red", "green", "yellow", "purple", "white"];
/// The notches, by overlay ordinal after `progress`.
const NOTCHES: [&str; 4] = ["notched_6", "notched_10", "notched_12", "notched_20"];

/// `LerpingBossEvent`: a bar and the name over it.
#[derive(Debug, Clone)]
struct Bar {
    id: u128,
    name: Vec<Run>,
    from: f32,
    target: f32,
    set_time: u64,
    color: u8,
    overlay: u8,
    flags: u8,
}

impl Bar {
    /// `LerpingBossEvent.getProgress`.
    fn progress(&self, now: u64) -> f32 {
        let since = now.saturating_sub(self.set_time);
        let lerp = (since as f32 / LERP_MILLISECONDS as f32).clamp(0.0, 1.0);
        self.from + lerp * (self.target - self.from)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BossBars {
    bars: Vec<Bar>,
}

/// `Mth.lerpDiscrete(alpha, 0, 182)`.
fn lerp_discrete(alpha: f32) -> i32 {
    (alpha * (BAR_WIDTH - 1) as f32).floor() as i32 + i32::from(alpha > 0.0)
}

impl BossBars {
    fn find(&mut self, id: u128) -> Option<&mut Bar> {
        self.bars.iter_mut().find(|bar| bar.id == id)
    }

    /// `add`: a bar of the same id is replaced where it stood in the map's
    /// order (`LinkedHashMap.put` keeps the first position).
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the reference's `ClientboundBossEventPacket.Handler.add`"
    )]
    pub fn add(
        &mut self,
        id: u128,
        name: Vec<Run>,
        progress: f32,
        color: u8,
        overlay: u8,
        flags: u8,
        now: u64,
    ) {
        let bar = Bar {
            id,
            name,
            from: progress,
            target: progress,
            set_time: now,
            color,
            overlay,
            flags,
        };
        match self.find(id) {
            Some(existing) => *existing = bar,
            None => self.bars.push(bar),
        }
    }

    pub fn remove(&mut self, id: u128) {
        self.bars.retain(|bar| bar.id != id);
    }

    /// `LerpingBossEvent.setProgress`: from where the bar is now.
    pub fn set_progress(&mut self, id: u128, progress: f32, now: u64) {
        if let Some(bar) = self.find(id) {
            bar.from = bar.progress(now);
            bar.target = progress;
            bar.set_time = now;
        }
    }

    pub fn set_name(&mut self, id: u128, name: Vec<Run>) {
        if let Some(bar) = self.find(id) {
            bar.name = name;
        }
    }

    pub fn set_style(&mut self, id: u128, color: u8, overlay: u8) {
        if let Some(bar) = self.find(id) {
            bar.color = color;
            bar.overlay = overlay;
        }
    }

    pub fn set_flags(&mut self, id: u128, flags: u8) {
        if let Some(bar) = self.find(id) {
            bar.flags = flags;
        }
    }

    /// `reset`: the level changed.
    pub fn clear(&mut self) {
        self.bars.clear();
    }

    /// `shouldDarkenScreen`.
    pub fn darkens_screen(&self) -> bool {
        self.bars.iter().any(|bar| bar.flags & 1 != 0)
    }

    /// `shouldPlayBossMusic`.
    pub fn plays_music(&self) -> bool {
        self.bars.iter().any(|bar| bar.flags & 2 != 0)
    }

    /// `shouldCreateWorldFog`.
    pub fn creates_fog(&self) -> bool {
        self.bars.iter().any(|bar| bar.flags & 4 != 0)
    }

    /// `BossHealthOverlay.extractRenderState`: a bar and its name for each
    /// event, 19 down apiece, until a third of the screen is used.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (width, height): (i32, i32),
        now: u64,
    ) {
        let mut y = 12;
        for bar in &self.bars {
            let x = width / 2 - 91;
            self.draw_bar(frame, atlas, x, y, bar, now);
            let name_x = width / 2 - font.width_of_runs(&bar.name) / 2;
            frame.styled_text(atlas, font, &bar.name, name_x, y - 9, 0xFFFF_FFFF, true);
            y += 10 + 9;
            if y >= height / 3 {
                break;
            }
        }
    }

    fn draw_bar(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        x: i32,
        y: i32,
        bar: &Bar,
        now: u64,
    ) {
        let color = COLORS[usize::from(bar.color)];
        let mut part = |kind: &str, width: i32| {
            frame.sprite_part(
                atlas,
                &format!("boss_bar/{color}_{kind}"),
                (BAR_WIDTH, BAR_HEIGHT),
                (x, y, width, BAR_HEIGHT),
            );
            if bar.overlay > 0 {
                let notches = NOTCHES[usize::from(bar.overlay) - 1];
                frame.sprite_part(
                    atlas,
                    &format!("boss_bar/{notches}_{kind}"),
                    (BAR_WIDTH, BAR_HEIGHT),
                    (x, y, width, BAR_HEIGHT),
                );
            }
        };
        part("background", BAR_WIDTH);
        let width = lerp_discrete(bar.progress(now));
        if width > 0 {
            part("progress", width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_discrete_over_the_bar() {
        assert_eq!(lerp_discrete(0.0), 0);
        assert_eq!(lerp_discrete(0.001), 1);
        assert_eq!(lerp_discrete(0.5), 91);
        assert_eq!(lerp_discrete(1.0), 182);
    }

    #[test]
    fn a_change_eases_in_over_a_tenth_of_a_second() {
        let mut bars = BossBars::default();
        bars.add(1, Vec::new(), 0.0, 0, 0, 0, 1000);
        bars.set_progress(1, 1.0, 1000);
        assert_eq!(bars.bars[0].progress(1000), 0.0);
        assert_eq!(bars.bars[0].progress(1050), 0.5);
        assert_eq!(bars.bars[0].progress(5000), 1.0);
        // A second change starts from where the bar had got to.
        bars.set_progress(1, 0.0, 1050);
        assert_eq!(bars.bars[0].progress(1050), 0.5);
    }

    #[test]
    fn re_adding_keeps_the_place() {
        let mut bars = BossBars::default();
        bars.add(1, Vec::new(), 0.0, 0, 0, 0, 0);
        bars.add(2, Vec::new(), 0.0, 0, 0, 0, 0);
        bars.add(1, Vec::new(), 0.5, 1, 0, 0, 0);
        assert_eq!(bars.bars[0].id, 1);
        assert_eq!(bars.bars[0].color, 1);
    }
}
