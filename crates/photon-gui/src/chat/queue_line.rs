//! The line under the messages that counts the ones held back by the chat
//! delay, and the click on it.

use super::*;
use crate::FrameRuns;
use photon_game::text::ClickEvent;

/// `ChatComponent.QUEUE_EXPAND_ID`: the click that shows the next message.
pub const QUEUE_EXPAND_ID: &str = "minecraft:internal/expand_chat_queue";

impl ChatLog {
    /// Sets the number of held messages (`ChatListener.queueSize`).
    pub fn set_queue_size(&mut self, size: u64) {
        self.queue_size = size;
    }

    /// A click on the line: the game shows the next held message.
    pub fn request_queue_accept(&mut self) {
        self.queue_accepts += 1;
    }

    /// The clicks on the line since the last call.
    pub fn take_queue_accepts(&mut self) -> u32 {
        std::mem::take(&mut self.queue_accepts)
    }

    /// F3+D (`ChatComponent.clearMessages`): the game flushes the held messages, then clears.
    pub fn request_clear(&mut self) {
        self.clear_requested = true;
    }

    pub fn take_clear_request(&mut self) -> bool {
        std::mem::take(&mut self.clear_requested)
    }

    /// `ChatComponent.clearMessages` without the history: the lines and the pending deletions.
    pub fn clear(&mut self) {
        self.deletions.clear();
        self.lines.clear();
        self.messages.clear();
    }

    /// Whether the line is drawn: the chat has lines and messages wait.
    pub(super) fn has_queue_line(&self) -> bool {
        self.queue_size > 0 && !self.lines.is_empty()
    }

    /// `chat.queue` under `QUEUE_EXPAND_TEXT_STYLE`.
    pub(super) fn queue_runs(&self, lang: &Lang) -> Vec<Run> {
        let style = Style {
            click: Some(Box::new(ClickEvent::Custom(QUEUE_EXPAND_ID.into(), None))),
            hover: Some(Box::new(HoverEvent::ShowText(Box::new(
                Component::translatable("chat.queue.tooltip"),
            )))),
            ..Style::default()
        };
        let count = Component::literal(self.queue_size.to_string());
        let text = Component::translatable_with("chat.queue", vec![count]);
        lang.runs(&text, &style)
    }

    /// The background behind the line, below the last message.
    pub(super) fn queue_background(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        chat_bottom: i32,
    ) {
        if !self.has_queue_line() {
            return;
        }
        let max_width =
            (width_of(self.settings.width) as f32 / self.settings.scale as f32).ceil() as i32;
        let alpha = ((self.settings.background_opacity as f32 * 255.0) as u32) << 24;
        frame.fill(
            atlas,
            (-2, chat_bottom, max_width + 4, chat_bottom + MESSAGE_HEIGHT),
            alpha,
        );
    }

    /// Draws the line (half as bright as the messages) and gives back its
    /// style when the open chat's mouse is over it.
    pub(super) fn queue_text(
        &self,
        (frame, atlas, font, lang): (&mut GuiFrame, &GuiAtlas, &Font, &Lang),
        (chat_bottom, text_opacity): (i32, f32),
        hover: Option<(f32, f32)>,
    ) -> Option<Style> {
        if !self.has_queue_line() {
            return None;
        }
        let runs = self.queue_runs(lang);
        let top = chat_bottom + 1;
        let colour = (((0.5 * text_opacity) * 255.0) as u32) << 24 | 0x00FF_FFFF;
        frame.styled_text(atlas, font, &runs, (0, top), colour, true);
        crate::text_hit::style_under(font, &runs, (0, top), hover?).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_requests_are_taken_once() {
        let mut log = ChatLog::default();
        log.request_queue_accept();
        log.request_queue_accept();
        assert_eq!(log.take_queue_accepts(), 2);
        assert_eq!(log.take_queue_accepts(), 0);
        log.request_clear();
        assert!(log.take_clear_request());
        assert!(!log.take_clear_request());
    }

    #[test]
    fn the_line_needs_lines_and_waiting_messages() {
        let mut log = ChatLog::default();
        log.set_queue_size(2);
        assert!(!log.has_queue_line());
    }
}
