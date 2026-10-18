//! The layers drawn over the HUD in one go: the tab list and the subtitles of the sounds heard.

use super::*;

impl Gui {
    /// What the sound system heard since the last frame joins the subtitles (and it listens only
    /// while the option is on, `SubtitleOverlay.isListening`); the listener is kept for drawing them.
    pub fn hear_sounds(
        &mut self,
        sound: Option<&mut crate::sound::SoundSystem>,
        options: &photon_game::options::Options,
    ) {
        let Some(sound) = sound else {
            return;
        };
        sound.set_listening(options.show_subtitles);
        let now = self.started_at.elapsed().as_millis() as u64;
        for heard in sound.take_heard() {
            if let Some(key) = heard.subtitle {
                self.subtitles.heard(&key, heard.position, heard.range, now);
            }
        }
        let (position, forward, up) = sound.listener_view();
        let vec = |value: [f64; 3]| photon_core::math::Vec3::new(value[0], value[1], value[2]);
        self.listener = Some(photon_gui::hud::subtitles::Listener::new(
            position,
            vec(forward),
            vec(up),
        ));
    }

    /// `Hud.extractSubtitleOverlay`: the subtitles over the HUD, or with it hidden only over a
    /// screen that is part of the game's own interface (the chat, the containers, the signs and
    /// the books). Java draws them in the screen's background, after its darkening and under its
    /// panel; here a container, sign or book screen has them drawn once it is (`over_screen`),
    /// any other time they come before the screens.
    pub(super) fn draw_subtitles(
        frame: &mut photon_render::gui::GuiFrame,
        (subtitles, listener, started_at): (
            &mut photon_gui::hud::subtitles::Subtitles,
            Option<photon_gui::hud::subtitles::Listener>,
            std::time::Instant,
        ),
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (controls, size): (&Controls, &photon_render::window::scale::WindowScale),
        over_screen: bool,
    ) {
        let options = &controls.options;
        let drawn_over_screen = controls.screen.is_some()
            || controls.sign_edit.is_some()
            || controls.book_edit.is_some();
        let in_game_ui = drawn_over_screen || controls.chat.is_some();
        let Some(listener) = listener.filter(|_| options.show_subtitles) else {
            return;
        };
        if over_screen != drawn_over_screen || (controls.hud_hidden && !in_game_ui) {
            return;
        }
        // `Options.getBackgroundColor(0.8F)`.
        let opacity = if options.background_for_chat_only {
            0.8_f32
        } else {
            options.text_background_opacity as f32
        };
        subtitles.draw(
            frame,
            (atlas, font, lang),
            (size.gui_scaled_width(), size.gui_scaled_height()),
            listener,
            started_at.elapsed().as_millis() as u64,
            (
                options.notification_display_time,
                ((opacity * 255.0) as u32) << 24,
            ),
        );
    }

    /// The tab list (while Tab is held and the interface shown), then the subtitles over it
    /// (`Hud.extractRenderState`: the tab list comes before the subtitle overlay).
    #[expect(
        clippy::type_complexity,
        reason = "the fields the two layers draw from"
    )]
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
        (subtitles, (controls, size)): (
            (
                &mut photon_gui::hud::subtitles::Subtitles,
                Option<photon_gui::hud::subtitles::Listener>,
                std::time::Instant,
            ),
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
        Self::draw_subtitles(
            frame,
            subtitles,
            (atlas, font, lang),
            (controls, size),
            false,
        );
    }
}
