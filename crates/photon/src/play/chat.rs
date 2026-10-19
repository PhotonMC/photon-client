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
        self.sync_chat_queue();
        self.acknowledge_chat(processed)
    }

    /// `ChatScreen.handleComponentClicked` on the queue line: each click shows the next
    /// message that waits.
    pub(super) fn accept_chat_queue(&mut self) -> Result<()> {
        let clicks = self.controls.chat_log.take_queue_accepts();
        for _ in 0..clicks {
            if self.gui.chat_queue_size() == 0 {
                break;
            }
            let view = ChatView {
                focused: self.controls.chat.is_some(),
                hidden: &self.controls.hidden_players,
            };
            let processed = self
                .gui
                .accept_chat_message(&mut self.controls.chat_log, view);
            self.acknowledge_chat(processed)?;
        }
        self.sync_chat_queue();
        Ok(())
    }

    /// F3+D, `ChatComponent.clearMessages`: the held messages are shown (and told to the
    /// server), then the lines go.
    pub(super) fn clear_chat(&mut self) -> Result<()> {
        if !self.controls.chat_log.take_clear_request() {
            return Ok(());
        }
        let view = ChatView {
            focused: self.controls.chat.is_some(),
            hidden: &self.controls.hidden_players,
        };
        let processed = self.gui.flush_chat_queue(&mut self.controls.chat_log, view);
        self.controls.chat_log.clear();
        self.sync_chat_queue();
        self.acknowledge_chat(processed)
    }

    /// The count the chat's queue line shows is the queue's own.
    fn sync_chat_queue(&mut self) {
        let size = self.gui.chat_queue_size() as u64;
        self.controls.chat_log.set_queue_size(size);
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
        self.sync_chat_queue();
        self.acknowledge_chat(processed)
    }

    fn acknowledge_chat(&mut self, processed: Vec<(Vec<u8>, bool)>) -> Result<()> {
        for (signature, shown) in processed {
            self.session.mark_message_processed(&signature, shown)?;
        }
        Ok(())
    }
}
