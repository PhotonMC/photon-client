//! The music toast (`NowPlayingToast`): the song that plays, with notes that change colour.

use super::{Toast, ToastEnv, Visibility};
use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Style};
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `NowPlayingToast.VISIBILITY_DURATION`, in milliseconds.
const VISIBILITY_MS: f64 = 5000.0;

/// `DyeColor.LIGHT_GRAY.getTextColor()`.
const TEXT_COLOUR: u32 = 0xFFD3_D3D3;

/// `ColorLerper.MUSIC_NOTE_COLORS`, as `DyeColor.getId`: white, light gray, light blue, blue,
/// cyan, green, lime, yellow, orange, pink, red, magenta.
const NOTE_DYES: [usize; 12] = [0, 8, 3, 11, 9, 13, 5, 4, 1, 6, 14, 2];

/// `ColorLerper.Type.MUSIC_NOTE`'s `colorDuration`, in ticks.
const NOTE_COLOUR_TICKS: i32 = 30;

/// `ColorLerper.getModifiedColor` at a brightness of 1.25 for a dye.
fn modified_colour(dye: usize) -> u32 {
    if dye == 0 {
        // `-1644826`.
        return 0xFFE6_E6E6;
    }
    let source = photon_data::entity_type::dye_texture_colour(dye).unwrap_or(0);
    let scale = |shift: u32| {
        let channel = ((source >> shift) & 0xFF) as f32;
        (channel * 1.25).floor().clamp(0.0, 255.0) as u32
    };
    0xFF00_0000 | scale(16) << 16 | scale(8) << 8 | scale(0)
}

/// `ColorLerper.getLerpedColor(MUSIC_NOTE, tick)`: the notes' colour, from one dye to the next as
/// the ticks go by.
pub fn music_note_colour(tick: f32) -> u32 {
    let tick_count = photon_core::math::floor_f32(tick);
    let value = tick_count / NOTE_COLOUR_TICKS;
    let count = NOTE_DYES.len() as i32;
    let from = modified_colour(NOTE_DYES[(value % count) as usize]);
    let to = modified_colour(NOTE_DYES[((value + 1) % count) as usize]);
    let sub_step = ((tick_count % NOTE_COLOUR_TICKS) as f32 + photon_core::math::frac_f32(tick))
        / NOTE_COLOUR_TICKS as f32;
    photon_render::sky::srgb_lerp(sub_step, from, to)
}

/// The notes' colour as it is now (`NowPlayingToast.musicNoteColor` and its tick and clock),
/// moved on by whatever draws the toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicNotes {
    pub tick: i32,
    /// `System.currentTimeMillis` when the colour last changed.
    pub last_change: i64,
    pub colour: u32,
}

impl Default for MusicNotes {
    fn default() -> Self {
        Self {
            tick: 0,
            last_change: 0,
            colour: 0xFFFF_FFFF,
        }
    }
}

impl MusicNotes {
    /// `tickMusicNotes`: while a song plays, the colour steps every 25 milliseconds.
    pub fn tick(&mut self, wall_ms: i64, playing: bool) {
        if playing && wall_ms > self.last_change + 25 {
            self.tick += 1;
            self.last_change = wall_ms;
            self.colour = music_note_colour(self.tick as f32);
        }
    }
}

/// `NowPlayingToast`.
pub struct NowPlayingToast {
    update_toast: bool,
    display_time: f64,
    wanted: Visibility,
}

impl NowPlayingToast {
    pub fn new() -> Self {
        Self {
            update_toast: false,
            display_time: 0.0,
            wanted: Visibility::Hide,
        }
    }

    /// `showToast`.
    pub fn show(&mut self, display_time: f64) {
        self.update_toast = true;
        self.display_time = display_time;
        self.wanted = Visibility::Show;
    }

    pub fn set_wanted_visibility(&mut self, visibility: Visibility) {
        self.wanted = visibility;
    }

    /// Whether the last `update` ticked the notes (`tickMusicNotes`).
    pub fn ticks_notes(&self) -> bool {
        self.update_toast
    }

    /// `getNowPlayingString`: the song's name, from its key with the slashes made dots.
    fn song_runs(env: &ToastEnv<'_>) -> Vec<photon_game::text::Run> {
        match env.current_song {
            None => Vec::new(),
            Some(key) => env.lang.runs(
                &Component::translatable(key.replace('/', ".")),
                &Style::default(),
            ),
        }
    }

    fn song_width(env: &ToastEnv<'_>) -> i32 {
        30 + env.font.width_of_runs(&Self::song_runs(env)) + 7
    }

    /// `extractToast`: the box, the notes in their colour and the song, nothing while none plays.
    pub fn draw_with(frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>, notes: u32) {
        if env.current_song.is_none() {
            return;
        }
        frame.sprite(atlas, "toast/now_playing", 0, 0, Self::song_width(env), 30);
        frame.sprite_tinted(atlas, "icon/music_notes", (7, 7, 16, 16), notes);
        frame.styled_text(
            atlas,
            env.font,
            &Self::song_runs(env),
            (30, 15 - 9 / 2),
            TEXT_COLOUR,
            true,
        );
    }
}

impl Default for NowPlayingToast {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast for NowPlayingToast {
    fn wanted_visibility(&self) -> Visibility {
        self.wanted
    }

    fn update(&mut self, _env: &ToastEnv<'_>, fully_visible_for_ms: i64) {
        if self.update_toast {
            self.wanted = if (fully_visible_for_ms as f64) < VISIBILITY_MS * self.display_time {
                Visibility::Show
            } else {
                Visibility::Hide
            };
        }
    }

    fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>) {
        Self::draw_with(frame, atlas, env, env.note_colour);
    }

    fn x_pos(&self, _screen_width: i32, visible_portion: f32, env: &ToastEnv<'_>) -> f32 {
        let width = self.width(env) as f32;
        width * visible_portion - width
    }

    fn y_pos(&self, _first_slot: usize, _env: &ToastEnv<'_>) -> f32 {
        0.0
    }

    fn width(&self, env: &ToastEnv<'_>) -> i32 {
        Self::song_width(env)
    }

    fn height(&self) -> i32 {
        30
    }

    fn on_finished_rendering(&mut self) {
        self.update_toast = false;
    }
}
