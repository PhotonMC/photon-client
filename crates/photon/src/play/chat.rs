use super::*;
use crate::gui::ChatView;

impl Play {
    /// The chat the server sent, shown or held back by the chat delay, and the server told of
    /// the signed messages that were processed.
    pub(super) fn receive_chat(&mut self) -> Result<()> {
        let chat = self.session.drain_chat();
        if chat.is_empty() {
            return Ok(());
        }
        let view = ChatView {
            focused: self.controls.chat.is_some(),
            hidden: &self.controls.hidden_players,
        };
        let delay = self.controls.options.chat_delay;
        let processed = self
            .gui
            .receive_chat(chat, &mut self.controls.chat_log, (view, delay));
        self.acknowledge_chat(processed)
    }

    /// `ChatListener.tick`, once a client tick.
    pub(super) fn tick_chat(&mut self) -> Result<()> {
        let view = ChatView {
            focused: self.controls.chat.is_some(),
            hidden: &self.controls.hidden_players,
        };
        let delay = self.controls.options.chat_delay;
        let processed = self
            .gui
            .tick_chat(&mut self.controls.chat_log, (view, delay));
        self.acknowledge_chat(processed)
    }

    fn acknowledge_chat(&mut self, processed: Vec<(Vec<u8>, bool)>) -> Result<()> {
        for (signature, shown) in processed {
            self.session.mark_message_processed(&signature, shown)?;
        }
        Ok(())
    }
}
