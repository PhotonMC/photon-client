//! The options screens as the title screen opens them: the same screens and the same saving as in
//! a level, made as outside one (`Minecraft.level == null`).

use super::*;

/// The options screens, by what takes the events.
#[derive(Clone, Copy)]
enum OptionScreen {
    CreditsAttribution,
    Language,
    KeyBinds,
    Controls,
    Sound,
    Skin,
    World,
    Online,
    Accessibility,
    Chat,
    Video,
    Options,
}

impl Controls {
    /// The host the title screen keeps for the options screens.
    pub(crate) fn for_title() -> Self {
        let mut host = Self::new(CameraType::FirstPerson, false);
        host.title_host = true;
        host
    }

    /// Whether an options screen, or the link question one asked, is open.
    pub(crate) fn host_open(&self) -> bool {
        self.topmost_option_screen().is_some() || self.link_confirm.is_some()
    }

    /// The screen that takes the events: the topmost of the options screens that are open.
    fn topmost_option_screen(&self) -> Option<OptionScreen> {
        if self.credits.is_some() {
            return None;
        }
        [
            (
                self.credits_attribution_screen.is_some(),
                OptionScreen::CreditsAttribution,
            ),
            (self.language_open(), OptionScreen::Language),
            (self.key_binds_screen.is_some(), OptionScreen::KeyBinds),
            (
                self.mouse_screen.is_some() || self.controls_screen.is_some(),
                OptionScreen::Controls,
            ),
            (self.sound_screen.is_some(), OptionScreen::Sound),
            (self.skin_screen.is_some(), OptionScreen::Skin),
            (
                self.accessibility_screen.is_some(),
                OptionScreen::Accessibility,
            ),
            (self.chat_options_screen.is_some(), OptionScreen::Chat),
            (self.world_screen.is_some(), OptionScreen::World),
            (self.online_screen.is_some(), OptionScreen::Online),
            (self.video_screen.is_some(), OptionScreen::Video),
            (self.options_screen.is_some(), OptionScreen::Options),
        ]
        .into_iter()
        .find_map(|(open, screen)| open.then_some(screen))
    }

    /// An event for the open options screen or one of the screens over it (what the title screen
    /// opens as well as the pause menu): the event comes back when none is open.
    pub(super) fn option_screens_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> std::result::Result<(), glfw::WindowEvent> {
        let Some(screen) = self.topmost_option_screen() else {
            return Err(event);
        };
        match screen {
            OptionScreen::CreditsAttribution => {
                self.credits_attribution_event(event, surface, assets);
            }
            OptionScreen::Language => self.language_event(event, surface, assets),
            OptionScreen::KeyBinds => self.key_binds_event(event, surface, assets),
            OptionScreen::Controls => self.controls_event(event, surface, assets),
            _ => self.lower_option_screen_event(screen, event, surface, assets),
        }
        Ok(())
    }

    /// An event for the settings screens that the others open over.
    fn lower_option_screen_event(
        &mut self,
        screen: OptionScreen,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        match screen {
            OptionScreen::Sound => self.sound_options_event(event, surface, assets),
            OptionScreen::Skin => self.skin_event(event, surface, assets),
            OptionScreen::World => self.world_options_event(event, surface, assets),
            OptionScreen::Online => self.online_event(event, surface, assets),
            OptionScreen::Accessibility => self.accessibility_event(event, surface, assets),
            OptionScreen::Chat => self.chat_options_event(event, surface, assets),
            OptionScreen::Video => self.video_event(event, surface, assets),
            _ => self.options_event(event, surface, assets),
        }
    }

    /// `OptionsScreen`, from the title screen's Options button.
    pub(crate) fn open_title_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.open_options(surface, assets);
    }

    /// `VideoSettingsScreen` directly, for a capture of it.
    pub(crate) fn open_title_video(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.open_video(surface, assets);
    }

    /// `AccessibilityOptionsScreen`, from the title screen's accessibility button.
    pub(crate) fn open_title_accessibility(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.open_accessibility(surface, assets);
    }

    /// `LanguageSelectScreen`, from the title screen's language button.
    pub(crate) fn open_title_language(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.open_language(surface, assets);
    }

    /// One window event for the open screens. The held control key is kept for the screens that
    /// read it.
    pub(crate) fn host_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if let glfw::WindowEvent::Key(key, _, action, _) = event {
            self.track_modifiers(key, action, false);
        }
        if self.link_confirm.is_some() {
            if let Some((action, target)) = self.link_answer(event, surface) {
                match (action, target) {
                    (LinkAction::Yes, ConfirmTarget::Link(url)) => open_uri(&url),
                    (LinkAction::Copy, ConfirmTarget::Link(url)) => surface.set_clipboard(&url),
                    _ => {}
                }
            }
            return;
        }
        let _ = self.option_screens_event(event, surface, assets);
    }
}
