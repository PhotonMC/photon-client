//! What a player who may not send messages or commands is told, and kept from sending.

use super::*;
use crate::FontRuns;
use photon_game::commands::has_message_arguments;
use photon_game::text::Component;

impl CommandSuggestions {
    /// `setRestrictions`.
    pub fn set_restrictions(&mut self, messages_allowed: bool, commands_allowed: bool) {
        self.messages_allowed = messages_allowed;
        self.commands_allowed = commands_allowed;
    }

    /// Whether the text is a command, and whether it is a message or holds one
    /// (`currentParseIsCommand`, `currentParseIsMessage`).
    fn kinds(&self, text: &str) -> (bool, bool) {
        let reading = self.reading(text);
        if !reading.is_command {
            return (false, !text.trim().is_empty());
        }
        let message = self.commands.as_ref().is_some_and(|tree| {
            let parse = tree.parse(text, reading.offset, self.selectors);
            has_message_arguments(tree, &parse)
        });
        (true, message)
    }

    /// `hasAllowedInput`: whether Enter may send the text.
    pub fn has_allowed_input(&self, text: &str) -> bool {
        let (command, message) = self.kinds(text);
        if message && !self.messages_allowed {
            return false;
        }
        !command || self.commands_allowed
    }

    /// The red lines that end the usage: commands, then messages, that the player may not send.
    pub(super) fn restriction_lines(&self, text: &str) -> Vec<Component> {
        let (command, message) = self.kinds(text);
        let red = |key: &str| Component::translatable(key).with_colour("red");
        let mut lines = Vec::new();
        if command && !self.commands_allowed {
            lines.push(red("chat_screen.commands_not_allowed"));
        }
        if message && !self.messages_allowed {
            lines.push(red("chat_screen.messages_not_allowed"));
        }
        lines
    }

    /// The usage of a message that may not be sent: its one line, from the screen's edge.
    pub(super) fn draw_message_usage(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        text: &str,
        lang: &Lang,
    ) {
        if self.reading(text).is_command || self.messages_allowed || text.trim().is_empty() {
            return;
        }
        let line = Component::translatable("chat_screen.messages_not_allowed").with_colour("red");
        let runs = lang.runs(&line, &Style::default());
        let width = self.font.width_of_runs(&runs);
        let y = if self.settings.anchor_to_bottom {
            self.place.screen.1 - 27
        } else {
            72
        };
        frame.fill(atlas, (-1, y, width + 1, y + 12), self.settings.fill);
        frame.styled_text(atlas, &self.font, &runs, 0, y + 2, 0xFFFF_FFFF, true);
    }
}
