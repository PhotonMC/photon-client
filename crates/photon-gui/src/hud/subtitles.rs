//! The subtitles of the sounds heard (`SubtitleOverlay`): one line for each text, at the right of
//! the screen, with an arrow when the sound came from the side or behind.

use crate::{FontRuns, FrameRuns};
use photon_core::math::Vec3;
use photon_game::text::{Component, Lang, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// How long a heard sound keeps its line, in milliseconds (`DISPLAY_TIME`), before the option's
/// multiplier.
const DISPLAY_TIME: f64 = 3000.0;

/// Where the sounds are heard from (`ListenerTransform`): the listener's place, and which way is
/// forward and which right.
#[derive(Debug, Clone, Copy)]
pub struct Listener {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
}

impl Listener {
    /// A listener at `position` looking `forward` with `up` above it: its right is forward
    /// crossed with up.
    pub fn new(position: Vec3, forward: Vec3, up: Vec3) -> Self {
        Self {
            position,
            forward,
            right: forward.cross(up),
        }
    }
}

/// One text and the places it was heard at (`SubtitleOverlay.Subtitle`).
#[derive(Debug, Clone, PartialEq)]
struct Subtitle {
    key: String,
    range: f32,
    played_at: Vec<(Vec3Key, u64)>,
}

/// A place, compared by its coordinates.
type Vec3Key = (f64, f64, f64);

fn vec3(place: Vec3Key) -> Vec3 {
    Vec3::new(place.0, place.1, place.2)
}

impl Subtitle {
    /// `getClosest`: the place heard at that is nearest to `position`, the first of equals.
    fn closest(&self, position: Vec3) -> Option<(Vec3Key, u64)> {
        let mut best: Option<((Vec3Key, u64), f64)> = None;
        for played in &self.played_at {
            let distance = vec3(played.0).distance_to(position);
            if best.is_none_or(|(_, nearest)| distance < nearest) {
                best = Some((*played, distance));
            }
        }
        best.map(|(played, _)| played)
    }

    /// `isAudibleFrom`.
    fn is_audible_from(&self, camera: Vec3) -> bool {
        if self.range.is_infinite() {
            return true;
        }
        self.closest(camera).is_some_and(|(place, _)| {
            camera.closer_than_position(vec3(place), f64::from(self.range))
        })
    }

    fn purge_old_instances(&mut self, max_age: f64, now: u64) {
        self.played_at
            .retain(|(_, time)| now.saturating_sub(*time) as f64 <= max_age);
    }
}

/// The overlay's list: a line for each text ever heard, in the order of the first hearing.
#[derive(Debug, Default)]
pub struct Subtitles {
    subtitles: Vec<Subtitle>,
}

impl Subtitles {
    /// `onPlaySound`: a sound whose event has a subtitle (the translation key) starts at a place
    /// `range` units carry. A text heard again keeps its line, and its old hearing at the same place
    /// is taken out.
    pub fn heard(&mut self, key: &str, position: Vec3Key, range: f32, now: u64) {
        if let Some(subtitle) = self
            .subtitles
            .iter_mut()
            .find(|subtitle| subtitle.key == key)
        {
            subtitle.played_at.retain(|(place, _)| *place != position);
            subtitle.played_at.push((position, now));
            return;
        }
        self.subtitles.push(Subtitle {
            key: key.to_owned(),
            range,
            played_at: vec![(position, now)],
        });
    }

    /// `extractRenderState`: each text still heard and within reach, at the right of the screen,
    /// bottom up.
    pub fn draw(
        &mut self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (width, height): (i32, i32),
        listener: Listener,
        now: u64,
        (display_time, background): (f64, u32),
    ) {
        let mut audible: Vec<usize> = (0..self.subtitles.len())
            .filter(|&index| self.subtitles[index].is_audible_from(listener.position))
            .collect();
        if audible.is_empty() {
            return;
        }
        let mut widest = 0;
        audible.retain(|&index| {
            let subtitle = &mut self.subtitles[index];
            subtitle.purge_old_instances(DISPLAY_TIME * display_time, now);
            if subtitle.played_at.is_empty() {
                return false;
            }
            widest = widest.max(font.width_of_runs(&text_of(lang, &subtitle.key)));
            true
        });
        widest += font.width("<") + font.width(" ") + font.width(">") + font.width(" ");
        let view = View {
            size: (width, height),
            widest,
            listener,
            time: (now, display_time),
            background,
        };
        for (row, &index) in audible.iter().enumerate() {
            self.subtitles[index].draw_line(frame, (atlas, font, lang), row as i32, &view);
        }
    }
}

/// What every line is drawn against.
struct View {
    size: (i32, i32),
    /// The width of the widest text and the arrows.
    widest: i32,
    listener: Listener,
    /// The time now, and the option's multiplier of how long a sound is kept.
    time: (u64, f64),
    background: u32,
}

fn text_of(lang: &Lang, key: &str) -> Vec<photon_game::text::Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

impl Subtitle {
    /// One text's line, `row` lines up from the bottom: the dark band, an arrow when the sound is
    /// not in front, and the text, dimmer as the sound grows old.
    fn draw_line(
        &self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        row: i32,
        view: &View,
    ) {
        let Some((place, time)) = self.closest(view.listener.position) else {
            return;
        };
        let delta = vec3(place)
            .subtract_vec3(view.listener.position)
            .normalized();
        let rightness = view.listener.right.dot(delta);
        let in_view = view.listener.forward.dot(delta) > 0.5;
        let half_width = view.widest / 2;
        let half_height = 9 / 2;
        let text = text_of(lang, &self.key);
        let text_width = font.width_of_runs(&text);
        let (now, display_time) = view.time;
        let elapsed = now.saturating_sub(time) as f32 / (DISPLAY_TIME * display_time) as f32;
        let brightness = photon_core::math::clamped_lerp_f32(elapsed, 255.0, 75.0).floor() as u32;
        let centre_x = (view.size.0 as f32 - half_width as f32 * 1.0 - 2.0) as i32;
        let centre_y = (view.size.1 as f32 - 35.0 - row as f32 * (9.0 + 1.0) * 1.0) as i32;
        frame.fill(
            atlas,
            (
                centre_x - half_width - 1,
                centre_y - half_height - 1,
                centre_x + half_width + 1,
                centre_y + half_height + 1,
            ),
            view.background,
        );
        let colour = 0xFF00_0000 | brightness << 16 | brightness << 8 | brightness;
        if !in_view && rightness != 0.0 {
            let (arrow, x) = if rightness > 0.0 {
                (">", centre_x + half_width - font.width(">"))
            } else {
                ("<", centre_x - half_width)
            };
            frame.text_with_shadow(atlas, font, arrow, x, centre_y - half_height, colour);
        }
        frame.styled_text(
            atlas,
            font,
            &text,
            (centre_x - text_width / 2, centre_y - half_height),
            colour,
            true,
        );
    }
}
