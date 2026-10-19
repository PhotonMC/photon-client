//! The layers drawn over the HUD in one go: the tab list, the subtitles of the sounds heard and
//! the toasts.

use super::*;
use photon_gui::hud::subtitles::{Listener, Subtitles};
use photon_gui::toasts::{MusicToast, ToastEnv, ToastManager};

/// What the layers over the HUD keep from frame to frame.
pub(crate) struct Layers {
    subtitles: Subtitles,
    listener: Option<Listener>,
    pub(super) toasts: ToastManager,
    /// The state of the music toast option the manager has, once it has looked (the manager's
    /// constructor only builds the toast, `initializeMusicToast`).
    music_toast: Option<MusicToast>,
    /// The song the music manager has playing, as its language key.
    current_song: Option<String>,
    started_at: std::time::Instant,
}

/// Where in the frame the subtitles are drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SubtitlePass {
    /// Before the screens (and with another screen than a container, sign or book up).
    Hud,
    /// Over a container, sign or book screen.
    OverScreen,
    /// With no screen up, after the toasts.
    AfterToasts,
}

impl SubtitlePass {
    /// The pass the subtitles are drawn in now, and whether the screen up (if any) is part of the
    /// game's own interface.
    fn of(controls: &Controls) -> (Self, bool) {
        let over_screen = controls.screen.is_some()
            || controls.sign_edit.is_some()
            || controls.book_edit.is_some();
        let pass = if !controls.any_screen_open() {
            Self::AfterToasts
        } else if over_screen {
            Self::OverScreen
        } else {
            Self::Hud
        };
        (pass, over_screen || controls.chat.is_some())
    }
}

/// Whether the pause screen is the screen up (`screen instanceof PauseScreen`): the settings screens
/// opened from it are over it.
fn pause_on_top(controls: &Controls) -> bool {
    controls.pause.is_some()
        && [
            controls.options_screen.is_some(),
            controls.video_screen.is_some(),
            controls.chat_options_screen.is_some(),
            controls.skin_screen.is_some(),
            controls.sound_screen.is_some(),
            controls.controls_screen.is_some(),
            controls.language_screen.is_some(),
            controls.mouse_screen.is_some(),
            controls.key_binds_screen.is_some(),
        ]
        .iter()
        .all(|open| !open)
}

impl Layers {
    pub(super) fn new(started_at: std::time::Instant) -> Self {
        Self {
            subtitles: Subtitles::default(),
            listener: None,
            toasts: ToastManager::new(MusicToast::Never),
            music_toast: None,
            current_song: None,
            started_at,
        }
    }

    fn now_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }

    /// What the sound system heard since the last frame joins the subtitles (and it listens only
    /// while the option is on, `SubtitleOverlay.isListening`); the listener is kept for drawing
    /// them.
    fn hear_sounds(
        &mut self,
        sound: &mut crate::sound::SoundSystem,
        options: &photon_game::options::Options,
    ) {
        sound.set_listening(options.show_subtitles);
        let now = self.now_ms();
        for heard in sound.take_heard() {
            if let Some(key) = heard.subtitle {
                self.subtitles.heard(&key, heard.position, heard.range, now);
            }
        }
        let (position, forward, up) = sound.listener_view();
        let vec = |value: [f64; 3]| photon_core::math::Vec3::new(value[0], value[1], value[2]);
        self.listener = Some(Listener::new(position, vec(forward), vec(up)));
    }

    /// `Gui.update`: the toasts move on, with the sounds they make played, and the music toast
    /// follows its option (`Options.musicToast`'s change callback).
    fn update_toasts(
        &mut self,
        sound: Option<&mut crate::sound::SoundSystem>,
        options: &photon_game::options::Options,
        ((font, lang), current_song): ((&Font, &Lang), Option<String>),
    ) {
        self.current_song = current_song;
        match self.music_toast.replace(options.music_toast) {
            None => self.toasts.init_music_toast(options.music_toast),
            Some(before) if before != options.music_toast => {
                let volume = (options
                    .sound_category_volume(photon_game::sounds::SoundSource::Master)
                    * options.sound_category_volume(photon_game::sounds::SoundSource::Music))
                    as f32;
                self.toasts.set_music_toast(
                    options.music_toast,
                    volume,
                    options.notification_display_time,
                );
            }
            Some(_) => {}
        }
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() as i64);
        let env = ToastEnv {
            font,
            lang,
            display_time: options.notification_display_time,
            current_song: self.current_song.as_deref(),
            note_colour: self.toasts.notes.colour,
        };
        let sounds = self.toasts.update((self.now_ms() as i64, wall), &env);
        if let Some(sound) = sound {
            for event in sounds {
                sound.play_ui(event, 1.0, 1.0);
            }
        }
    }

    /// `Hud.extractSubtitleOverlay`: the subtitles over the HUD, or with it hidden only over a
    /// screen that is part of the game's own interface (the chat, the containers, the signs and
    /// the books). Java draws them where the game is at (`pass`): with no screen up they are
    /// deferred to after the toasts; over a container, sign or book screen in the screen's
    /// background, after its darkening and under its panel (here once the screen is drawn); with
    /// any other screen in the HUD pass, before the screen.
    fn draw_subtitles(
        &mut self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (controls, size): (&Controls, &photon_render::window::scale::WindowScale),
        pass: SubtitlePass,
    ) {
        let options = &controls.options;
        let Some(listener) = self.listener.filter(|_| options.show_subtitles) else {
            return;
        };
        let (wanted, in_game_ui) = SubtitlePass::of(controls);
        if pass != wanted || (controls.hud_hidden && !in_game_ui) {
            return;
        }
        // `Options.getBackgroundColor(0.8F)`.
        let opacity = if options.background_for_chat_only {
            0.8_f32
        } else {
            options.text_background_opacity as f32
        };
        let now = self.now_ms();
        self.subtitles.draw(
            frame,
            (atlas, font, lang),
            (size.gui_scaled_width(), size.gui_scaled_height()),
            listener,
            now,
            (
                options.notification_display_time,
                ((opacity * 255.0) as u32) << 24,
            ),
        );
    }

    /// The toasts over everything (`Gui.extractRenderState`: after the screen), and the music
    /// toast too unless the pause screen draws it itself.
    fn draw_toasts(
        &self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (controls, size): (&Controls, &photon_render::window::scale::WindowScale),
        item: &mut photon_render::gui::ItemDrawer<'_>,
    ) {
        let options = &controls.options;
        let env = ToastEnv {
            font,
            lang,
            display_time: options.notification_display_time,
            current_song: self.current_song.as_deref(),
            note_colour: self.toasts.notes.colour,
        };
        self.toasts.draw(
            (frame, item),
            atlas,
            size.gui_scaled_width(),
            (
                controls.hud_hidden,
                options.music_toast,
                pause_on_top(controls),
            ),
            &env,
        );
    }

    /// The music toast of the pause screen (`PauseScreen.rendersNowPlayingToast`): while the music
    /// is heard, drawn at the corner.
    fn draw_pause_toast(
        &self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        controls: &Controls,
    ) {
        let options = &controls.options;
        let volume = options.sound_category_volume(photon_game::sounds::SoundSource::Master)
            * options.sound_category_volume(photon_game::sounds::SoundSource::Music);
        if !pause_on_top(controls) || !options.music_toast.render_in_pause_screen() || volume <= 0.0
        {
            return;
        }
        let env = ToastEnv {
            font,
            lang,
            display_time: options.notification_display_time,
            current_song: self.current_song.as_deref(),
            note_colour: self.toasts.notes.colour,
        };
        self.toasts.draw_now_playing_for_pause(frame, atlas, &env);
    }
}

impl Gui {
    /// What the sounds told since the last frame goes to the subtitles, and the toasts move on.
    pub fn feed_sounds(
        &mut self,
        sound: Option<&mut crate::sound::SoundSystem>,
        options: &photon_game::options::Options,
        current_song: Option<String>,
    ) {
        let mut sound = sound;
        if let Some(sound) = sound.as_deref_mut() {
            self.layers.hear_sounds(sound, options);
        }
        self.layers
            .update_toasts(sound, options, ((&self.font, &self.lang), current_song));
    }

    /// `ClientPacketListener.handleLogin`'s toast for a server that does not enforce secure chat.
    pub fn add_unsecure_server_toast(&mut self) {
        use photon_game::text::Component;
        use photon_gui::toasts::{SystemToast, SystemToastId, ToastEnv};
        let env = ToastEnv {
            font: &self.font,
            lang: &self.lang,
            display_time: 1.0,
            current_song: None,
            note_colour: 0xFFFF_FFFF,
        };
        let title = Component::translatable("multiplayer.unsecureserver.toast.title");
        let message = Component::translatable("multiplayer.unsecureserver.toast");
        let toast = SystemToast::new(
            SystemToastId::UnsecureServerWarning,
            &env,
            &title,
            Some(&message),
        );
        self.layers.toasts.add_system(toast);
    }

    /// The toasts the packets asked for, in the order they came: an advancement's holds what it
    /// showed when the packet came, a recipe is added to the recipe toast.
    pub fn add_pending_toasts(
        &mut self,
        pending: Vec<photon_game::toast::PendingToast>,
        display_time: f64,
    ) {
        use photon_game::toast::PendingToast;
        let env = photon_gui::toasts::ToastEnv {
            font: &self.font,
            lang: &self.lang,
            display_time,
            current_song: None,
            note_colour: 0xFFFF_FFFF,
        };
        for toast in pending {
            match toast {
                PendingToast::Advancement(display) => {
                    let toast = photon_gui::toasts::AdvancementToast::new(display.as_ref(), &env);
                    self.layers.toasts.add(Box::new(toast));
                }
                PendingToast::Recipe { station, result } => {
                    self.layers
                        .toasts
                        .add_or_update_recipe(&env, (station, result));
                }
            }
        }
    }

    /// `Gui.addSocialInteractionsToast`: the tutorial toast of the first server joined, kept
    /// for eight seconds; `key` is the social interactions key as the words name it.
    pub fn add_social_interactions_toast(
        &mut self,
        key: &photon_game::text::Component,
        display_time: f64,
    ) {
        use photon_gui::toasts::{TutorialIcon, TutorialToast, TutorialWords};
        let env = photon_gui::toasts::ToastEnv {
            font: &self.font,
            lang: &self.lang,
            display_time,
            current_song: None,
            note_colour: 0xFFFF_FFFF,
        };
        let title = photon_game::text::Component::translatable("tutorial.socialInteractions.title");
        let mut key = key.clone();
        key.style.bold = Some(true);
        let message = photon_game::text::Component::translatable_with(
            "tutorial.socialInteractions.description",
            vec![key],
        );
        let words = TutorialWords {
            title: &title,
            message: Some(&message),
            progressable: true,
            time_to_display_ms: 8000,
        };
        let toast = TutorialToast::new(&env, TutorialIcon::SocialInteractions, &words);
        self.layers.toasts.add(Box::new(toast));
    }

    /// A tutorial step's toast, kept until the tutorial hides it.
    pub fn add_tutorial_toast(
        &mut self,
        icon: photon_gui::toasts::TutorialIcon,
        (title, message): (
            &photon_game::text::Component,
            Option<&photon_game::text::Component>,
        ),
        (progressable, display_time): (bool, f64),
    ) {
        use photon_gui::toasts::{TutorialToast, TutorialWords};
        let env = photon_gui::toasts::ToastEnv {
            font: &self.font,
            lang: &self.lang,
            display_time,
            current_song: None,
            note_colour: 0xFFFF_FFFF,
        };
        let words = TutorialWords {
            title,
            message,
            progressable,
            time_to_display_ms: 0,
        };
        let toast = TutorialToast::new(&env, icon, &words);
        self.layers.toasts.add(Box::new(toast));
    }

    /// `TutorialToast.hide` or `updateProgress` of the toast with the icon.
    pub fn tutorial_toast(
        &mut self,
        icon: photon_gui::toasts::TutorialIcon,
        progress: Option<f32>,
    ) {
        if let Some(toast) = self.layers.toasts.tutorial_mut(icon) {
            match progress {
                Some(progress) => toast.update_progress(progress),
                None => toast.hide(),
            }
        }
    }

    /// The social interactions toast is hidden once the screen is opened with its key.
    pub fn hide_social_interactions_toast(&mut self) {
        let icon = photon_gui::toasts::TutorialIcon::SocialInteractions;
        if let Some(toast) = self.layers.toasts.tutorial_mut(icon) {
            toast.hide();
        }
    }

    /// `Hud.showNowPlayingToast` and `hideNowPlayingToast` as the music manager asked.
    pub fn music_toast_requests(&mut self, requests: Vec<bool>, display_time: f64) {
        for show in requests {
            if show {
                self.layers.toasts.show_now_playing(display_time);
            } else {
                self.layers.toasts.hide_now_playing();
            }
        }
    }

    /// The tab list (while Tab is held and the interface shown), then the subtitles over it
    /// (`Hud.extractRenderState`: the tab list comes before the subtitle overlay).
    #[expect(clippy::type_complexity, reason = "the fields the layers draw from")]
    pub(super) fn draw_tab_and_subtitles(
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (tab_entries, width): (Option<(Vec<TabEntry>, Option<TabObjective>)>, i32),
        ((health, header, footer), gui_ticks): (
            (
                &mut HealthStates,
                &[photon_game::text::Run],
                &[photon_game::text::Run],
            ),
            i64,
        ),
        (layers, (controls, size)): (
            &mut Layers,
            (&Controls, &photon_render::window::scale::WindowScale),
        ),
    ) {
        if let Some((entries, objective)) = tab_entries {
            tab_list::draw(
                frame,
                atlas,
                font,
                width,
                &entries,
                (header, footer),
                TabScores {
                    objective,
                    health,
                    gui_ticks,
                    lang,
                },
            );
        }
        layers.draw_subtitles(
            frame,
            (atlas, font, lang),
            (controls, size),
            SubtitlePass::Hud,
        );
    }

    /// What is drawn once the screens are: the subtitles over a container, sign or book screen,
    /// the music toast of the pause screen and the toasts.
    pub(super) fn draw_late_layers(
        frame: &mut photon_render::gui::GuiFrame,
        resources: (&GuiAtlas, &Font, &Lang),
        (layers, item): (&mut Layers, &mut photon_render::gui::ItemDrawer<'_>),
        (controls, size, session): (
            &Controls,
            &photon_render::window::scale::WindowScale,
            &PlaySession,
        ),
    ) {
        let view = (controls, size);
        let debug = Self::debug_columns(session, controls);
        layers.draw_subtitles(frame, resources, view, SubtitlePass::OverScreen);
        layers.draw_pause_toast(frame, resources, view.0);
        layers.draw_toasts(frame, resources, view, item);
        // `Gui.extractRenderState`: the debug overlay comes after the toasts, before the
        // subtitles that waited.
        if let Some((left, right)) = debug.as_ref() {
            photon_gui::hud::debug::draw(
                frame,
                resources.0,
                resources.1,
                view.1.gui_scaled_width(),
                (left, right),
            );
        }
        layers.draw_subtitles(frame, resources, view, SubtitlePass::AfterToasts);
    }
}
