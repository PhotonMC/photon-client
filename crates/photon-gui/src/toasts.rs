//! The toasts (`ToastManager`): small notices that slide in from the corner of the screen, wait,
//! and slide out; the system notices and the now-playing music one.

mod advancement;
mod now_playing;
mod system;

use std::any::Any;
use std::collections::VecDeque;

use photon_game::text::Lang;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

pub use advancement::AdvancementToast;
pub use now_playing::{MusicNotes, NowPlayingToast, music_note_colour};
pub use system::{SystemToast, SystemToastId};

/// How long a toast takes to slide in or out, in milliseconds (`SLIDE_ANIMATION_DURATION_MS`).
const SLIDE_MS: i64 = 600;

/// How many rows of toasts the screen holds (`SLOT_COUNT`).
const SLOT_COUNT: usize = 5;

/// `Toast.Visibility`: what a toast wants to be doing, and the sound its change makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Show,
    Hide,
}

impl Visibility {
    /// `SoundEvents.UI_TOAST_IN` and `UI_TOAST_OUT`.
    pub fn sound(self) -> &'static str {
        match self {
            Self::Show => "minecraft:ui.toast.in",
            Self::Hide => "minecraft:ui.toast.out",
        }
    }
}

/// What a toast reads of the game: the text it is written with, how long notices are kept (the
/// option's multiplier) and, for the music toast, the song playing.
#[derive(Clone, Copy)]
pub struct ToastEnv<'a> {
    pub font: &'a Font,
    pub lang: &'a Lang,
    pub display_time: f64,
    /// `MusicManager.getCurrentMusicTranslationKey`: the song's language key with its slashes.
    pub current_song: Option<&'a str>,
    /// The colour of the music toast's notes now; the manager sets it where it draws.
    pub note_colour: u32,
}

/// One toast (`Toast`).
pub trait Toast: Any {
    fn wanted_visibility(&self) -> Visibility;

    /// `update`: it may change what it wants as it has been fully shown for this long.
    fn update(&mut self, env: &ToastEnv<'_>, fully_visible_for_ms: i64);

    /// The sound a toast makes as it comes (`getSoundEvent`).
    fn sound_event(&self) -> Option<&'static str> {
        None
    }

    /// `extractRenderState`, at the toast's own corner.
    fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>);

    /// The items a toast draws, at its corner as it has slid (`fakeItem`, which the frame keeps
    /// apart from the sprites and text).
    fn draw_items(&self, _frame: &mut GuiFrame, _item: &mut ItemDrawer<'_>, _origin: (f32, f32)) {}

    /// `xPos`: where its left edge is, with `visible_portion` of it slid in.
    fn x_pos(&self, screen_width: i32, visible_portion: f32, env: &ToastEnv<'_>) -> f32 {
        screen_width as f32 - self.width(env) as f32 * visible_portion
    }

    /// `yPos`.
    fn y_pos(&self, first_slot: usize, _env: &ToastEnv<'_>) -> f32 {
        first_slot as f32 * self.height() as f32
    }

    fn width(&self, _env: &ToastEnv<'_>) -> i32 {
        160
    }

    fn height(&self) -> i32 {
        32
    }

    /// `occcupiedSlotCount`: `Mth.positiveCeilDiv(height, 32)`.
    fn occupied_slots(&self) -> usize {
        (self.height().max(0) as usize).div_ceil(32)
    }

    fn on_finished_rendering(&mut self) {}

    /// The system toast this is, to be found by its id.
    fn as_system_mut(&mut self) -> Option<&mut SystemToast> {
        None
    }
}

/// `ToastManager.ToastInstance`: a toast being shown, in its slots and its slide.
struct ToastInstance<T: ?Sized> {
    first_slot: usize,
    occupied_slots: usize,
    animation_start: i64,
    became_fully_visible_at: i64,
    visibility: Visibility,
    fully_visible_for: i64,
    visible_portion: f32,
    finished: bool,
    toast: Box<T>,
}

impl<T: Toast + ?Sized> ToastInstance<T> {
    fn new(toast: Box<T>, first_slot: usize, occupied_slots: usize) -> Self {
        let mut instance = Self {
            first_slot,
            occupied_slots,
            animation_start: -1,
            became_fully_visible_at: -1,
            visibility: Visibility::Hide,
            fully_visible_for: 0,
            visible_portion: 0.0,
            finished: false,
            toast,
        };
        instance.reset();
        instance
    }

    /// `resetToast`.
    fn reset(&mut self) {
        self.animation_start = -1;
        self.became_fully_visible_at = -1;
        self.visibility = Visibility::Hide;
        self.fully_visible_for = 0;
        self.visible_portion = 0.0;
        self.finished = false;
    }

    fn calculate_visible_portion(&mut self, now: i64) {
        let progress = ((now - self.animation_start) as f32 / SLIDE_MS as f32).clamp(0.0, 1.0);
        let progress = progress * progress;
        self.visible_portion = if self.visibility == Visibility::Hide {
            1.0 - progress
        } else {
            progress
        };
    }

    /// `update`, at `now` milliseconds.
    fn update(&mut self, now: i64, env: &ToastEnv<'_>) {
        if self.animation_start == -1 {
            self.animation_start = now;
            self.visibility = Visibility::Show;
        }
        if self.visibility == Visibility::Show && now - self.animation_start <= SLIDE_MS {
            self.became_fully_visible_at = now;
        }
        self.fully_visible_for = now - self.became_fully_visible_at;
        self.calculate_visible_portion(now);
        self.toast.update(env, self.fully_visible_for);
        let wanted = self.toast.wanted_visibility();
        if wanted != self.visibility {
            self.animation_start = now - ((1.0 - self.visible_portion) * SLIDE_MS as f32) as i64;
            self.visibility = wanted;
        }
        let was_finished = self.finished;
        self.finished =
            self.visibility == Visibility::Hide && now - self.animation_start > SLIDE_MS;
        if self.finished && !was_finished {
            self.toast.on_finished_rendering();
        }
    }

    fn draw(
        &self,
        (frame, item): (&mut GuiFrame, &mut ItemDrawer<'_>),
        atlas: &GuiAtlas,
        screen_width: i32,
        env: &ToastEnv<'_>,
    ) {
        if self.finished {
            return;
        }
        let first = (frame.vertices.len(), frame.log.as_ref().map_or(0, Vec::len));
        self.toast.draw(frame, atlas, env);
        let translate = (
            self.toast.x_pos(screen_width, self.visible_portion, env),
            self.toast.y_pos(self.first_slot, env),
        );
        frame.scale_from(first, 1.0, translate);
        self.toast.draw_items(frame, item, translate);
    }
}

pub use photon_game::options::MusicToast;

/// `ToastManager`: the toasts shown, the ones waiting for room, and the music toast.
pub struct ToastManager {
    visible: Vec<ToastInstance<dyn Toast>>,
    occupied: [bool; SLOT_COUNT],
    queued: VecDeque<Box<dyn Toast>>,
    now_playing: Option<ToastInstance<NowPlayingToast>>,
    /// The music toast's notes (`NowPlayingToast`'s statics).
    pub notes: MusicNotes,
}

impl ToastManager {
    /// A manager for the music toast option's state (`initializeMusicToast`).
    pub fn new(music_toast: MusicToast) -> Self {
        let mut manager = Self {
            visible: Vec::new(),
            occupied: [false; SLOT_COUNT],
            queued: VecDeque::new(),
            now_playing: None,
            notes: MusicNotes::default(),
        };
        manager.init_music_toast(music_toast);
        manager
    }

    /// `initializeMusicToast`: the music toast exists for the option's state, shown for nothing.
    pub fn init_music_toast(&mut self, music_toast: MusicToast) {
        self.now_playing = music_toast
            .render_in_pause_screen()
            .then(|| ToastInstance::new(Box::new(NowPlayingToast::new()), 0, 0));
    }

    /// `update`, at `now` milliseconds: the toasts move on, the finished ones free their slots,
    /// the waiting ones take free ones. The sounds that make (events, in order) are given back.
    pub fn update(&mut self, (now, wall): (i64, i64), env: &ToastEnv<'_>) -> Vec<&'static str> {
        let mut sounds = Vec::new();
        let mut change_sound_played = false;
        let occupied = &mut self.occupied;
        self.visible.retain_mut(|instance| {
            let previous = instance.visibility;
            instance.update(now, env);
            if instance.visibility != previous && !change_sound_played {
                change_sound_played = true;
                sounds.push(instance.visibility.sound());
            }
            if instance.finished {
                occupied[instance.first_slot..instance.first_slot + instance.occupied_slots]
                    .fill(false);
            }
            !instance.finished
        });
        if !self.queued.is_empty() && self.free_slots() > 0 {
            self.start_waiting(&mut sounds);
        }
        if let Some(playing) = &mut self.now_playing {
            playing.update(now, env);
            if playing.toast.ticks_notes() {
                self.notes.tick(wall, env.current_song.is_some());
            }
        }
        sounds
    }

    /// The waiting toasts that fit take slots, in the order they came (`queued.removeIf`): a
    /// small one may start before a large one that has no room yet. A toast's own sound plays
    /// once however many start together.
    fn start_waiting(&mut self, sounds: &mut Vec<&'static str>) {
        let mut played = Vec::new();
        let mut waiting = VecDeque::new();
        while let Some(toast) = self.queued.pop_front() {
            let count = toast.occupied_slots();
            let Some(first) = self.find_free_slots(count) else {
                waiting.push_back(toast);
                continue;
            };
            self.occupied[first..first + count].fill(true);
            let sound = toast.sound_event().filter(|sound| !played.contains(sound));
            if let Some(sound) = sound {
                played.push(sound);
                sounds.push(sound);
            }
            self.visible.push(ToastInstance::new(toast, first, count));
        }
        self.queued = waiting;
    }

    /// `extractRenderState`: the toasts slid in, then the music toast, unless the interface is
    /// hidden; the music toast is also left out under the pause screen, which draws it itself.
    pub fn draw(
        &self,
        (frame, item): (&mut GuiFrame, &mut ItemDrawer<'_>),
        atlas: &GuiAtlas,
        screen_width: i32,
        (hidden, music_toast, pause_screen): (bool, MusicToast, bool),
        env: &ToastEnv<'_>,
    ) {
        if hidden {
            return;
        }
        let env = &ToastEnv {
            note_colour: self.notes.colour,
            ..*env
        };
        for instance in &self.visible {
            instance.draw((frame, &mut *item), atlas, screen_width, env);
        }
        if music_toast.render_toast()
            && !pause_screen
            && let Some(playing) = &self.now_playing
        {
            playing.draw((frame, item), atlas, screen_width, env);
        }
    }

    fn free_slots(&self) -> usize {
        self.occupied.iter().filter(|taken| !**taken).count()
    }

    /// `findFreeSlotsIndex`: the first of `count` free slots in a row.
    fn find_free_slots(&self, count: usize) -> Option<usize> {
        if self.free_slots() < count {
            return None;
        }
        let mut run = 0;
        for (index, taken) in self.occupied.iter().enumerate() {
            if *taken {
                run = 0;
            } else {
                run += 1;
                if run == count {
                    return Some(index + 1 - run);
                }
            }
        }
        None
    }

    /// `addToast`.
    pub fn add(&mut self, toast: Box<dyn Toast>) {
        self.queued.push_back(toast);
    }

    /// `SystemToast.add`.
    pub fn add_system(&mut self, toast: SystemToast) {
        self.add(Box::new(toast));
    }

    /// `SystemToast.addOrUpdate`: the toast with this id, shown or waiting, takes the new text and
    /// is kept longer; another is added if there is none.
    pub fn add_or_update_system(
        &mut self,
        id: SystemToastId,
        env: &ToastEnv<'_>,
        (title, message): (
            &photon_game::text::Component,
            Option<&photon_game::text::Component>,
        ),
    ) {
        match self.find_system(id) {
            Some(toast) => toast.reset(env, title, message),
            None => self.add_system(SystemToast::new(id, env, title, message)),
        }
    }

    /// `ToastManager.getToast` for a system toast: the shown ones first, then the waiting ones.
    fn find_system(&mut self, id: SystemToastId) -> Option<&mut SystemToast> {
        self.visible
            .iter_mut()
            .find_map(|instance| {
                instance
                    .toast
                    .as_system_mut()
                    .filter(|toast| toast.id() == id)
            })
            .or_else(|| {
                self.queued
                    .iter_mut()
                    .find_map(|toast| toast.as_system_mut().filter(|toast| toast.id() == id))
            })
    }

    /// `SystemToast.forceHide`.
    pub fn force_hide_system(&mut self, id: SystemToastId) {
        if let Some(toast) = self.find_system(id) {
            toast.force_hide();
        }
    }

    /// `clear`.
    pub fn clear(&mut self) {
        self.occupied = [false; SLOT_COUNT];
        self.visible.clear();
        self.queued.clear();
    }

    /// `showNowPlayingToast`: the music toast starts over and shows.
    pub fn show_now_playing(&mut self, display_time: f64) {
        if let Some(playing) = &mut self.now_playing {
            playing.reset();
            playing.toast.show(display_time);
        }
    }

    /// `hideNowPlayingToast`.
    pub fn hide_now_playing(&mut self) {
        if let Some(playing) = &mut self.now_playing {
            playing.toast.set_wanted_visibility(Visibility::Hide);
        }
    }

    /// `setMusicToastDisplayState`.
    pub fn set_music_toast(&mut self, state: MusicToast, music_volume: f32, display_time: f64) {
        match state {
            MusicToast::Never => self.now_playing = None,
            MusicToast::Pause | MusicToast::PauseAndToast => {
                let mut playing = ToastInstance::new(Box::new(NowPlayingToast::new()), 0, 0);
                if state == MusicToast::PauseAndToast && music_volume > 0.0 {
                    playing.toast.show(display_time);
                }
                self.now_playing = Some(playing);
            }
        }
    }

    /// The music toast, for the pause screen to draw (`NowPlayingToast.extractToast`).
    pub fn draw_now_playing_for_pause(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        env: &ToastEnv<'_>,
    ) {
        NowPlayingToast::draw_with(frame, atlas, env, self.notes.colour);
    }
}
