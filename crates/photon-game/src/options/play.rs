//! The play options of the controls, mouse, chat and video screens that `options.txt` keeps: the
//! toggles of the keys, auto jump, the chat's links and suggestions, and the view's extras.

use super::json::{Json, int_value};

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
    ChatLinks,
    ChatLinksPrompt,
    AutoSuggestions,
    ReducedDebugInfo,
    Vignette,
    AutosaveIndicator,
}

/// The flags and the sprint window of the group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub chat_links: bool,
    pub chat_links_prompt: bool,
    pub auto_suggestions: bool,
    pub reduced_debug_info: bool,
    pub vignette: bool,
    /// `showAutosaveIndicator`.
    pub autosave_indicator: bool,
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
            chat_links: true,
            chat_links_prompt: true,
            auto_suggestions: true,
            reduced_debug_info: false,
            vignette: true,
            autosave_indicator: true,
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
            PlayKey::ChatLinks => Some(&mut self.chat_links),
            PlayKey::ChatLinksPrompt => Some(&mut self.chat_links_prompt),
            PlayKey::AutoSuggestions => Some(&mut self.auto_suggestions),
            PlayKey::ReducedDebugInfo => Some(&mut self.reduced_debug_info),
            PlayKey::Vignette => Some(&mut self.vignette),
            PlayKey::AutosaveIndicator => Some(&mut self.autosave_indicator),
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
        match own.flag(key) {
            Some(flag) => flag.to_string(),
            None => self.sprint_window.to_string(),
        }
    }
}
