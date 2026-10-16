//! The title, the subtitle and the action bar (`Hud.extractTitle` and
//! `extractOverlayMessage`): what `Hud` keeps of them, how long, and how they
//! fade.

use crate::{FontRuns, FrameRuns};
use photon_game::text::Run;

use photon_render::gui::GuiAtlas;
use photon_render::gui::GuiFrame;
use photon_render::gui::font::Font;

/// The action bar shows for this many ticks.
const OVERLAY_TICKS: i32 = 60;

#[derive(Debug, Clone)]
pub struct Titles {
    title: Option<Vec<Run>>,
    subtitle: Option<Vec<Run>>,
    overlay: Option<Vec<Run>>,
    /// Whether the overlay message cycles through the colours (`animateOverlayMessageColor`).
    overlay_animates: bool,
    fade_in: i32,
    stay: i32,
    fade_out: i32,
    title_time: i32,
    overlay_time: i32,
}

impl Default for Titles {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            overlay: None,
            overlay_animates: false,
            fade_in: 10,
            stay: 70,
            fade_out: 20,
            title_time: 0,
            overlay_time: 0,
        }
    }
}

impl Titles {
    /// `Hud.tick`: the times run down; a title that ran out takes its subtitle.
    pub fn tick(&mut self) {
        if self.overlay_time > 0 {
            self.overlay_time -= 1;
        }
        if self.title_time > 0 {
            self.title_time -= 1;
            if self.title_time == 0 {
                self.title = None;
                self.subtitle = None;
            }
        }
    }

    /// `setTitle`.
    pub fn set_title(&mut self, title: Vec<Run>) {
        self.title = Some(title);
        self.title_time = self.fade_in + self.stay + self.fade_out;
    }

    /// `setSubtitle`.
    pub fn set_subtitle(&mut self, subtitle: Vec<Run>) {
        self.subtitle = Some(subtitle);
    }

    /// `setOverlayMessage` (the action bar).
    pub fn set_overlay(&mut self, message: Vec<Run>) {
        self.overlay = Some(message);
        self.overlay_time = OVERLAY_TICKS;
        self.overlay_animates = false;
    }

    /// `setNowPlaying`: the overlay message, in colours that cycle.
    pub fn set_now_playing(&mut self, message: Vec<Run>) {
        self.set_overlay(message);
        self.overlay_animates = true;
    }

    /// `setTimes`: only a value that is not negative changes.
    pub fn set_times(&mut self, fade_in: i32, stay: i32, fade_out: i32) {
        if fade_in >= 0 {
            self.fade_in = fade_in;
        }
        if stay >= 0 {
            self.stay = stay;
        }
        if fade_out >= 0 {
            self.fade_out = fade_out;
        }
        if self.title_time > 0 {
            self.title_time = self.fade_in + self.stay + self.fade_out;
        }
    }

    /// `clearTitles` and `resetTitleTimes`.
    pub fn clear(&mut self, reset_times: bool) {
        self.title = None;
        self.subtitle = None;
        self.title_time = 0;
        if reset_times {
            *self = Self {
                overlay: self.overlay.take(),
                overlay_animates: self.overlay_animates,
                overlay_time: self.overlay_time,
                ..Self::default()
            };
        }
    }

    /// The title's alpha at `partial` of a tick into its time, 0 to 255.
    fn title_alpha(&self, partial: f32) -> i32 {
        let t = self.title_time as f32 - partial;
        let total = (self.fade_in + self.stay + self.fade_out) as f32;
        let mut alpha = 255.0;
        if self.title_time > self.fade_out + self.stay {
            alpha = if self.fade_in == 0 {
                255.0
            } else {
                (total - t) * 255.0 / self.fade_in as f32
            };
        }
        if self.title_time <= self.fade_out {
            alpha = if self.fade_out == 0 {
                0.0
            } else {
                t * 255.0 / self.fade_out as f32
            };
        }
        (alpha as i32).clamp(0, 255)
    }

    /// The action bar's colour: white, or for a record's name the hue that turns with the time left.
    fn overlay_colour(&self, partial: f32, alpha: i32) -> u32 {
        if self.overlay_animates {
            let hue = (self.overlay_time as f32 - partial) / 50.0;
            if let Ok(colour) = photon_core::math::mth_hsv_to_argb_f32(hue, 0.7, 0.6, alpha) {
                return colour as u32;
            }
        }
        (alpha as u32) << 24 | 0x00FF_FFFF
    }

    /// The action bar's alpha: full until 20 ticks remain.
    fn overlay_alpha(&self, partial: f32) -> i32 {
        let t = self.overlay_time as f32 - partial;
        ((t * 255.0 / 20.0) as i32).min(255)
    }

    /// The action bar over the hotbar, then the title and subtitle in the
    /// middle of the screen.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (width, height): (i32, i32),
        partial: f32,
    ) {
        if let (Some(message), true) = (&self.overlay, self.overlay_time > 0) {
            let alpha = self.overlay_alpha(partial);
            if alpha > 0 {
                let x = width / 2 - font.width_of_runs(message) / 2;
                frame.styled_text(
                    atlas,
                    font,
                    message,
                    x,
                    height - 72,
                    self.overlay_colour(partial, alpha),
                    true,
                );
            }
        }
        let (Some(title), true) = (&self.title, self.title_time > 0) else {
            return;
        };
        let alpha = self.title_alpha(partial);
        if alpha <= 0 {
            return;
        }
        let colour = (alpha as u32) << 24 | 0x00FF_FFFF;
        let centre = ((width / 2) as f32, (height / 2) as f32);
        let mut line = |runs: &[Run], scale: f32, y: i32| {
            let first = (frame.vertices.len(), frame.log.as_ref().map_or(0, Vec::len));
            let x = -(font.width_of_runs(runs) / 2);
            frame.styled_text(atlas, font, runs, x, y, colour, true);
            frame.scale_from(first, scale, centre);
        };
        line(title, 4.0, -10);
        if let Some(subtitle) = &self.subtitle {
            line(subtitle, 2.0, 5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titled() -> Titles {
        let mut titles = Titles::default();
        titles.set_title(Vec::new());
        titles
    }

    #[test]
    fn a_title_fades_in_stays_and_fades_out() {
        let mut titles = titled();
        // 100 ticks: alpha climbs over the first 10, holds, falls over the
        // last 20.
        assert_eq!(titles.title_alpha(0.0), 0);
        titles.title_time = 95;
        assert_eq!(titles.title_alpha(0.0), 127);
        titles.title_time = 50;
        assert_eq!(titles.title_alpha(0.0), 255);
        titles.title_time = 10;
        assert_eq!(titles.title_alpha(0.0), 127);
    }

    #[test]
    fn times_below_zero_keep_what_was_and_a_running_title_restarts() {
        let mut titles = titled();
        titles.title_time = 5;
        titles.set_times(-1, 100, 0);
        assert_eq!((titles.fade_in, titles.stay, titles.fade_out), (10, 100, 0));
        assert_eq!(titles.title_time, 110);
    }

    #[test]
    fn a_title_that_runs_out_takes_its_subtitle() {
        let mut titles = titled();
        titles.set_subtitle(Vec::new());
        for _ in 0..100 {
            titles.tick();
        }
        assert!(titles.title.is_none() && titles.subtitle.is_none());
    }

    #[test]
    fn no_fade_in_does_not_divide_by_zero() {
        let mut titles = Titles::default();
        titles.set_times(0, 70, 20);
        titles.set_title(Vec::new());
        assert_eq!(titles.title_alpha(0.0), 255);
    }

    #[test]
    fn the_action_bar_holds_until_twenty_ticks_remain() {
        let mut titles = Titles::default();
        titles.set_overlay(Vec::new());
        assert_eq!(titles.overlay_alpha(0.0), 255);
        titles.overlay_time = 10;
        assert_eq!(titles.overlay_alpha(0.0), 127);
    }
}
