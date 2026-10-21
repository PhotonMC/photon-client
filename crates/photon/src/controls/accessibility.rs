use super::*;
use photon_gui::options::accessibility::{AccessibilityInput, AccessibilityScreen};
use photon_gui::screen::confirm::ConfirmBackground;

/// `CommonLinks.ACCESSIBILITY_HELP`.
const ACCESSIBILITY_HELP: &str = "https://aka.ms/MinecraftJavaAccessibility";

impl Controls {
    /// `AccessibilityOptionsScreen`, over the options screen.
    pub(super) fn open_accessibility(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        // The server's enabled features are not kept for the screen yet, so the minecart option
        // stays out of reach.
        self.accessibility_screen = Some(AccessibilityScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            (&self.options, false),
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen, and the options are
    /// saved.
    pub(super) fn close_accessibility(&mut self) {
        self.accessibility_screen = None;
        self.save_options();
    }

    /// The options the screen shows that the options hold: the chat wraps its lines again when
    /// they changed (`rescaleChat`).
    pub(super) fn take_accessibility_choices(&mut self, assets: &ScreenAssets) {
        let Some(screen) = self.accessibility_screen.as_ref() else {
            return;
        };
        let choices = screen.choices();
        let options = &mut self.options;
        options.show_subtitles = choices.subtitles;
        options.text_background_opacity = choices.text_background_opacity;
        options.background_for_chat_only = choices.background_for_chat_only;
        options.chat_opacity = choices.chat_opacity;
        options.chat_line_spacing = choices.chat_line_spacing;
        options.chat_delay = choices.chat_delay;
        options.notification_display_time = choices.notification_display_time;
        options.screen_effect_scale = choices.screen_effect_scale;
        options.hide_lightning_flashes = choices.hide_lightning_flashes;
        options.view_bobbing = choices.view_bobbing;
        options.high_contrast_block_outline = choices.high_contrast_block_outline;
        options.fov_effect_scale = choices.fov_effect_scale;
        options.darkness_effect_scale = choices.darkness_effect_scale;
        options.damage_tilt_strength = choices.damage_tilt_strength;
        options.glint_speed = choices.glint_speed;
        options.glint_strength = choices.glint_strength;
        options.menu_background_blurriness = choices.menu_blurriness;
        let settings = photon_gui::chat::ChatSettings::from(&self.options);
        self.chat_log.set_settings(settings, &assets.creative.font);
    }

    /// An event for the open accessibility settings.
    pub(super) fn accessibility_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.accessibility_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            self.take_accessibility_choices(assets);
            return;
        }
        let Some(screen) = self.accessibility_screen.as_mut() else {
            return;
        };
        let mut input = AccessibilityInput::default();
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => input = screen.key_pressed(lang, key, shift),
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => input.done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                input = screen.clicked(lang, self.gui_mouse, shift);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(self.gui_mouse, y * self.options.mouse_wheel_sensitivity);
            }
            _ => {}
        }
        self.take_accessibility_choices(assets);
        self.accessibility_asked(input, surface, assets);
    }

    /// A scripted press of the primary button on the screen, at the mouse.
    pub(super) fn accessibility_pressed(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let Some(screen) = self.accessibility_screen.as_mut() else {
            return;
        };
        let input = screen.clicked(&assets.lang, self.gui_mouse, false);
        screen.released(&assets.lang);
        self.take_accessibility_choices(assets);
        self.accessibility_asked(input, surface, assets);
    }

    /// What a press on the screen asks for.
    fn accessibility_asked(
        &mut self,
        input: AccessibilityInput,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        if input.controls {
            self.open_controls(surface, assets);
        } else if input.guide {
            self.open_accessibility_guide(surface, assets);
        } else if input.done {
            self.close_accessibility();
        }
    }

    /// A scripted Escape on the chat or accessibility settings: whether one was open.
    pub(super) fn close_chat_settings(&mut self) -> bool {
        if self.accessibility_screen.is_some() {
            self.close_accessibility();
        } else if self.chat_options_screen.is_some() {
            self.close_chat_options();
        } else {
            return false;
        }
        true
    }

    /// `ConfirmLinkScreen.confirmLink`: the guide's address, to be confirmed.
    fn open_accessibility_guide(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let gui = Self::gui_size(surface);
        let screen = ConfirmScreen::link(
            &assets.creative.font,
            &assets.lang,
            gui,
            ACCESSIBILITY_HELP,
            false,
        )
        .with_background(ConfirmBackground::Menu { in_world: true });
        self.link_confirm = Some((screen, ConfirmTarget::Link(ACCESSIBILITY_HELP.to_owned())));
    }
}
