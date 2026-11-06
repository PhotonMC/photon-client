//! The play options of the controls, mouse, chat and video screens that `options.txt` keeps: the
//! toggles of the keys, auto jump, the chat's links and suggestions, and the view's extras.

use super::json::{Json, double_value, int_value, java_double};
use super::sliders::GUI_SCALE_MAX;

/// An option of the group, by its name in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlayKey {
    ToggleCrouch,
    ToggleSprint,
    ToggleAttack,
    ToggleUse,
    SprintWindow,
    AutoJump,
    OperatorItemsTab,
    DiscreteMouseScroll,
    AllowCursorChanges,
    QuitShortcuts,
    CtrlClickEmulatesRightClick,
    ChatLinks,
    ChatLinksPrompt,
    AutoSuggestions,
    ReducedDebugInfo,
    Vignette,
    AutosaveIndicator,
    DarkBackground,
    HideSplashTexts,
    PanoramaSpeed,
    Fullscreen,
    DebugGuiScale,
}

/// The flags and the sprint window of the group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayOptions {
    /// `toggleCrouch`, `toggleSprint`, `toggleAttack` and `toggleUse`: the key toggles instead of
    /// being held.
    pub toggle_crouch: bool,
    pub toggle_sprint: bool,
    pub toggle_attack: bool,
    pub toggle_use: bool,
    /// `sprintWindow`, 0 to 10: the ticks a second press of forward starts the sprint within.
    pub sprint_window: i32,
    pub auto_jump: bool,
    pub operator_items_tab: bool,
    /// `discrete_mouse_scroll`: a notch of the wheel counts as one whatever its size.
    pub discrete_mouse_scroll: bool,
    pub allow_cursor_changes: bool,
    /// `quitShortcuts`: the platform's quit shortcut closes the game.
    pub quit_shortcuts: bool,
    /// `ctrlClickEmulatesRightClick`, which only macOS shows.
    pub ctrl_click_emulates_right_click: bool,
    pub chat_links: bool,
    pub chat_links_prompt: bool,
    pub auto_suggestions: bool,
    pub reduced_debug_info: bool,
    pub vignette: bool,
    /// `showAutosaveIndicator`.
    pub autosave_indicator: bool,
    /// `darkMojangStudiosBackground`: the loading overlay is black.
    pub dark_background: bool,
    /// `hideSplashTexts`.
    pub hide_splash_texts: bool,
    /// `panoramaScrollSpeed`, 0 to 1: how fast the title panorama turns.
    pub panorama_speed: f64,
    /// `fullscreen`: the window is fullscreen.
    pub fullscreen: bool,
    /// `debugGuiScale`: the debug overlay's scale; -1 keeps the game's, 0 is automatic.
    pub debug_gui_scale: i32,
}

impl Default for PlayOptions {
    fn default() -> Self {
        Self {
            toggle_crouch: false,
            toggle_sprint: false,
            toggle_attack: false,
            toggle_use: false,
            sprint_window: 7,
            auto_jump: false,
            operator_items_tab: false,
            discrete_mouse_scroll: false,
            allow_cursor_changes: true,
            quit_shortcuts: true,
            ctrl_click_emulates_right_click: false,
            chat_links: true,
            chat_links_prompt: true,
            auto_suggestions: true,
            reduced_debug_info: false,
            vignette: true,
            autosave_indicator: true,
            dark_background: false,
            hide_splash_texts: false,
            panorama_speed: 1.0,
            fullscreen: false,
            debug_gui_scale: 0,
        }
    }
}

impl PlayOptions {
    fn flag(&mut self, key: PlayKey) -> Option<&mut bool> {
        match key {
            PlayKey::ToggleCrouch => Some(&mut self.toggle_crouch),
            PlayKey::ToggleSprint => Some(&mut self.toggle_sprint),
            PlayKey::ToggleAttack => Some(&mut self.toggle_attack),
            PlayKey::ToggleUse => Some(&mut self.toggle_use),
            PlayKey::AutoJump => Some(&mut self.auto_jump),
            PlayKey::OperatorItemsTab => Some(&mut self.operator_items_tab),
            PlayKey::DiscreteMouseScroll => Some(&mut self.discrete_mouse_scroll),
            _ => self.screen_flag(key),
        }
    }

    fn screen_flag(&mut self, key: PlayKey) -> Option<&mut bool> {
        match key {
            PlayKey::AllowCursorChanges => Some(&mut self.allow_cursor_changes),
            PlayKey::QuitShortcuts => Some(&mut self.quit_shortcuts),
            PlayKey::CtrlClickEmulatesRightClick => Some(&mut self.ctrl_click_emulates_right_click),
            PlayKey::ChatLinks => Some(&mut self.chat_links),
            PlayKey::ChatLinksPrompt => Some(&mut self.chat_links_prompt),
            PlayKey::AutoSuggestions => Some(&mut self.auto_suggestions),
            PlayKey::ReducedDebugInfo => Some(&mut self.reduced_debug_info),
            PlayKey::Vignette => Some(&mut self.vignette),
            _ => self.title_flag(key),
        }
    }

    fn title_flag(&mut self, key: PlayKey) -> Option<&mut bool> {
        match key {
            PlayKey::AutosaveIndicator => Some(&mut self.autosave_indicator),
            PlayKey::DarkBackground => Some(&mut self.dark_background),
            PlayKey::HideSplashTexts => Some(&mut self.hide_splash_texts),
            PlayKey::Fullscreen => Some(&mut self.fullscreen),
            _ => None,
        }
    }

    /// A value the file holds; one of the wrong kind or out of range leaves the option.
    pub(super) fn read(&mut self, key: PlayKey, json: &Json) {
        match (key, json) {
            (PlayKey::SprintWindow, Json::Number(number)) => {
                let value = int_value(number);
                if (0..=10).contains(&value) {
                    self.sprint_window = value;
                }
            }
            // `ClampingLazyMaxIntRange(-1, ..)`: past the largest scale it takes the largest.
            (PlayKey::DebugGuiScale, Json::Number(number)) => {
                let value = int_value(number);
                if value >= -1 {
                    self.debug_gui_scale = value.min(GUI_SCALE_MAX);
                }
            }
            (PlayKey::PanoramaSpeed, Json::Number(number)) => {
                let value = double_value(number);
                if (0.0..=1.0).contains(&value) {
                    self.panorama_speed = value;
                }
            }
            (_, Json::Bool(value)) => {
                if let Some(flag) = self.flag(key) {
                    *flag = *value;
                }
            }
            _ => {}
        }
    }

    pub(super) fn write(&self, key: PlayKey) -> String {
        let mut own = *self;
        match (key, own.flag(key)) {
            (_, Some(flag)) => flag.to_string(),
            (PlayKey::PanoramaSpeed, None) => java_double(self.panorama_speed),
            (PlayKey::DebugGuiScale, None) => self.debug_gui_scale.to_string(),
            (_, None) => self.sprint_window.to_string(),
        }
    }
}
