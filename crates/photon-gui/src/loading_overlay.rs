//! `LoadingOverlay`: the Mojang logo and a progress bar over the game while resources reload.

use photon_render::gui::{GuiAtlas, GuiFrame};

/// `LOGO_BACKGROUND_COLOR`.
const BRAND_BACKGROUND: u32 = 0xFFEF_323D;
/// `LOGO_BACKGROUND_COLOR_DARK`.
const BRAND_BACKGROUND_DARK: u32 = 0xFF00_0000;
/// `FADE_OUT_TIME` and `FADE_IN_TIME`, in milliseconds.
const FADE_OUT_MILLIS: f32 = 1000.0;
const FADE_IN_MILLIS: f32 = 500.0;
/// How long the fade-in must have run before the overlay may finish (`isReadyToFadeOut`).
const READY_AFTER_FADE_IN: i64 = 1000;

/// What the overlay keeps between frames.
#[derive(Debug, Clone)]
pub struct LoadingOverlay {
    fade_in: bool,
    dark: bool,
    current_progress: f32,
    fade_out_start: Option<i64>,
    fade_in_start: Option<i64>,
}

impl LoadingOverlay {
    /// `new LoadingOverlay(.., fadeIn)`, over a brand colour that is black when `dark`
    /// (`darkMojangStudiosBackground`).
    pub fn new(fade_in: bool, dark: bool) -> Self {
        Self {
            fade_in,
            dark,
            current_progress: 0.0,
            fade_out_start: None,
            fade_in_start: None,
        }
    }

    /// The smoothed progress the bar shows.
    pub fn current_progress(&self) -> f32 {
        self.current_progress
    }

    /// Set the smoothed progress, for a test that pins it.
    pub fn set_current_progress(&mut self, progress: f32) {
        self.current_progress = progress;
    }

    /// Pin where the fades began, in `Util.getMillis` terms, for a test.
    pub fn set_fade_starts(&mut self, fade_in: Option<i64>, fade_out: Option<i64>) {
        self.fade_in_start = fade_in;
        self.fade_out_start = fade_out;
    }

    /// Whether the fade-out has begun.
    pub fn fading_out(&self) -> bool {
        self.fade_out_start.is_some()
    }

    /// `tick`: whether the reload is finished and the overlay may answer and start fading out.
    /// The caller answers and then calls `start_fade_out`.
    pub fn ready(&self, done: bool, now: i64) -> bool {
        self.fade_out_start.is_none()
            && done
            && (!self.fade_in
                || self
                    .fade_in_start
                    .is_some_and(|start| now - start >= READY_AFTER_FADE_IN))
    }

    /// The reload was answered: the logo and the bar fade out from `now`.
    pub fn start_fade_out(&mut self, now: i64) {
        self.fade_out_start = Some(now);
    }

    /// Whether the fade-out is over (`setOverlay(null)`).
    pub fn finished(&self, now: i64) -> bool {
        self.fade_out_anim(now) >= 2.0
    }

    fn fade_out_anim(&self, now: i64) -> f32 {
        self.fade_out_start
            .map_or(-1.0, |start| (now - start) as f32 / FADE_OUT_MILLIS)
    }

    fn background(&self) -> u32 {
        if self.dark {
            BRAND_BACKGROUND_DARK
        } else {
            BRAND_BACKGROUND
        }
    }

    /// `extractRenderState` at `now` (`Util.getMillis`), with the reload's `actual_progress`.
    /// `under` draws the screen below the fills where the reference does. The result is the colour
    /// the frame is cleared to instead of being filled, when there is one (`clearColorOverride`).
    pub fn draw(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (width, height): (i32, i32),
        (now, actual_progress): (i64, f32),
        under: &mut dyn FnMut(&mut GuiFrame),
    ) -> Option<u32> {
        if self.fade_in && self.fade_in_start.is_none() {
            self.fade_in_start = Some(now);
        }
        let fade_out = self.fade_out_anim(now);
        let (logo_alpha, clear) = self.backdrop(frame, atlas, (width, height), now, under);
        let content_x = (f64::from(width) * 0.5) as i32;
        let logo_y = (f64::from(height) * 0.5) as i32;
        let logo_height = (f64::from(width) * 0.75).min(f64::from(height)) * 0.25;
        let logo_height_half = (logo_height * 0.5) as i32;
        let logo_width_half = (logo_height * 4.0 * 0.5) as i32;
        let colour = white(logo_alpha);
        let size = logo_height as i32;
        frame.logo_half(
            atlas,
            (-0.0625, 0.0),
            (
                content_x - logo_width_half,
                logo_y - logo_height_half,
                logo_width_half,
                size,
            ),
            colour,
        );
        frame.logo_half(
            atlas,
            (0.0625, 60.0),
            (content_x, logo_y - logo_height_half, logo_width_half, size),
            colour,
        );
        let bar_y = (f64::from(height) * 0.8325) as i32;
        self.current_progress =
            (self.current_progress * 0.95 + actual_progress * 0.050_000_012).clamp(0.0, 1.0);
        if fade_out < 1.0 {
            let (x0, x1) = (width / 2 - logo_width_half, width / 2 + logo_width_half);
            self.progress_bar(
                frame,
                atlas,
                (x0, bar_y - 5, x1, bar_y + 5),
                1.0 - fade_out.clamp(0.0, 1.0),
            );
        }
        clear
    }

    /// The fills over the screen under the overlay, the logo's alpha, and the colour the frame is
    /// cleared to when there are no fills.
    fn backdrop(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (width, height): (i32, i32),
        now: i64,
        under: &mut dyn FnMut(&mut GuiFrame),
    ) -> (f32, Option<u32>) {
        let fade_out = self.fade_out_anim(now);
        let fade_in = self
            .fade_in_start
            .map_or(-1.0, |start| (now - start) as f32 / FADE_IN_MILLIS);
        let rect = (0, 0, width, height);
        if fade_out >= 1.0 {
            under(frame);
            let past = (fade_out - 1.0).clamp(0.0, 1.0);
            let alpha = ceil((1.0 - past) * 255.0);
            frame.fill(atlas, rect, replace_alpha(self.background(), alpha));
            (1.0 - past, None)
        } else if self.fade_in {
            if fade_in < 1.0 {
                under(frame);
            }
            let alpha = (f64::from(fade_in).clamp(0.15, 1.0) * 255.0).ceil() as u32;
            frame.fill(atlas, rect, replace_alpha(self.background(), alpha));
            (fade_in.clamp(0.0, 1.0), None)
        } else {
            (1.0, Some(self.background()))
        }
    }

    /// `extractProgressBar`.
    fn progress_bar(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (x0, y0, x1, y1): (i32, i32, i32, i32),
        fade: f32,
    ) {
        let width = ceil((x1 - x0 - 2) as f32 * self.current_progress) as i32;
        let alpha = (fade * 255.0).round() as u32;
        let white = alpha << 24 | 0x00FF_FFFF;
        frame.fill(atlas, (x0 + 2, y0 + 2, x0 + width, y1 - 2), white);
        frame.fill(atlas, (x0 + 1, y0, x1 - 1, y0 + 1), white);
        frame.fill(atlas, (x0 + 1, y1, x1 - 1, y1 - 1), white);
        frame.fill(atlas, (x0, y0, x0 + 1, y1), white);
        frame.fill(atlas, (x1, y0, x1 - 1, y1), white);
    }
}

/// `Mth.ceil(float)` as an unsigned channel or a count.
fn ceil(value: f32) -> u32 {
    value.ceil().max(0.0) as u32
}

/// `replaceAlpha`.
fn replace_alpha(colour: u32, alpha: u32) -> u32 {
    colour & 0x00FF_FFFF | alpha << 24
}

/// `ARGB.white(float)`.
fn white(alpha: f32) -> u32 {
    ((alpha * 255.0).floor() as u32) << 24 | 0x00FF_FFFF
}
