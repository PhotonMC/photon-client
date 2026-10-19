use super::*;

impl Gui {
    /// The chat's messages are worded in the language now loaded.
    pub fn reword_chat(&self, log: &mut ChatLog) {
        log.set_lang(&self.lang, &self.font);
    }

    /// The decoration of a bound chat type, if the registries know it.
    pub(super) fn decorate(
        &self,
        content: photon_game::text::Component,
        bound: &photon_net::connection::ChatBound,
    ) -> photon_game::text::Component {
        let decoration = match &bound.holder {
            ChatHolder::Direct(decoration) => Some(decoration),
            ChatHolder::Registry(id) => usize::try_from(*id)
                .ok()
                .and_then(|id| self.registries.chat_types.get(id)),
        };
        match decoration {
            Some(decoration) => decoration.decorate(content, &bound.name, bound.target.as_ref()),
            None => content,
        }
    }

    /// `ChatComponent.DELETED_CHAT_MESSAGE`.
    pub(super) fn deleted_marker(&self) -> Vec<photon_game::text::Run> {
        let style = Style {
            colour: photon_game::text::named_colour("gray"),
            italic: Some(true),
            ..Style::default()
        };
        self.lang.runs(
            &photon_game::text::Component::translatable("chat.deleted_marker"),
            &style,
        )
    }

    /// `ChatListener.showMessageToPlayer`: a player's message, tagged by how
    /// far it can be trusted, and cut by what the server filtered.
    pub(super) fn show_player_chat(
        &self,
        packet: photon_net::connection::PlayerChatPacket,
        sender_is_local: bool,
        log: &mut ChatLog,
        focused: bool,
    ) -> bool {
        use photon_gui::chat::Tag;
        use photon_net::connection::FilterMask;
        if packet.filter == FilterMask::FullyFiltered {
            return false;
        }
        let content = packet
            .unsigned
            .clone()
            .unwrap_or_else(|| photon_game::text::Component::literal(packet.signed_text.clone()));
        let decorated = self.decorate(content, &packet.bound);
        let decorated_text = self.lang.plain(&decorated);
        // `ChatTrustLevel.evaluate`: no signature or an old one is not
        // secure; text that is not what was signed is modified.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        let expired = now > packet.timestamp_millis + 7 * 60 * 1000;
        let tag = if sender_is_local {
            None
        } else if packet.signature.is_none() || expired {
            Some(Tag::NotSecure)
        } else if !decorated_text.contains(&packet.signed_text) {
            Some(Tag::Modified(packet.signed_text.clone()))
        } else {
            None
        };
        let shown = if packet.filter == FilterMask::PassThrough {
            decorated
        } else {
            self.decorate(
                filtered_content(&packet.signed_text, &packet.filter),
                &packet.bound,
            )
        };
        log.add_component(
            &shown,
            &self.lang,
            (packet.signature.clone(), tag),
            &self.font,
            focused,
        );
        true
    }

    /// `handleChatMessageError`: a sender the client has no player for.
    fn show_chat_error(
        &self,
        bound: &photon_net::connection::ChatBound,
        log: &mut ChatLog,
        focused: bool,
    ) {
        let style = Style {
            colour: photon_game::text::named_colour("red"),
            italic: Some(true),
            ..Style::default()
        };
        let error =
            photon_game::text::Component::translatable("chat.validation_error").with_style(&style);
        let decorated = self.decorate(error, bound);
        log.add_component(
            &decorated,
            &self.lang,
            (None, Some(photon_gui::chat::Tag::Error)),
            &self.font,
            focused,
        );
    }

    /// `ChatComponent.addClientSystemMessage`: a message of the client's own.
    pub fn client_message(
        &self,
        log: &mut ChatLog,
        content: &photon_game::text::Component,
        focused: bool,
    ) {
        log.add_component(
            content,
            &self.lang,
            (None, Some(photon_gui::chat::Tag::SystemSinglePlayer)),
            &self.font,
            focused,
        );
    }

    /// The chat the server sent, worded and added to the log, or held back by the chat delay.
    /// What comes back is each signed player message and whether it was shown, which the server
    /// is told of (`markMessageAsProcessed`).
    pub fn receive_chat(
        &mut self,
        events: Vec<ChatEvent>,
        log: &mut ChatLog,
        (view, delay): (ChatView<'_>, f64),
    ) -> Vec<(Vec<u8>, bool)> {
        let mut processed = self.set_chat_delay(delay, log, view);
        let now = self.started_at.elapsed().as_millis() as u64;
        for event in events {
            match event {
                ChatEvent::System {
                    content,
                    overlay: true,
                } => {
                    let runs = self.lang.runs(&content, &Style::default());
                    self.titles.set_overlay(runs);
                }
                ChatEvent::System { content, .. } => log.add_component(
                    &content,
                    &self.lang,
                    (None, Some(photon_gui::chat::Tag::SystemSinglePlayer)),
                    &self.font,
                    view.focused,
                ),
                // `handleDeleteChat`: a message still waiting is dropped from the queue.
                ChatEvent::Delete(signature) => {
                    if !self.chat_queue.remove(&signature) {
                        log.delete(&signature, &self.font, &self.deleted_marker());
                    }
                }
                event if self.chat_queue.will_delay(now) => {
                    let signature = match &event {
                        ChatEvent::Player { packet, .. } => packet.signature.clone(),
                        _ => None,
                    };
                    self.chat_queue.push(signature, event);
                }
                event => {
                    if self.show_chat(event, log, view, &mut processed) {
                        self.chat_queue.shown(now);
                    }
                }
            }
        }
        processed
    }

    /// `ChatListener.tick`: the chat's clock, and the messages the delay let through.
    pub fn tick_chat(
        &mut self,
        log: &mut ChatLog,
        (view, delay): (ChatView<'_>, f64),
    ) -> Vec<(Vec<u8>, bool)> {
        log.tick(&self.font, &self.deleted_marker());
        let mut processed = self.set_chat_delay(delay, log, view);
        let now = self.started_at.elapsed().as_millis() as u64;
        let mut queue = std::mem::take(&mut self.chat_queue);
        queue.due(now, |event| {
            self.show_chat(event, log, view, &mut processed)
        });
        self.chat_queue = queue;
        processed
    }

    /// `setMessageDelay`: a delay that goes to none shows what waited.
    fn set_chat_delay(
        &mut self,
        seconds: f64,
        log: &mut ChatLog,
        view: ChatView<'_>,
    ) -> Vec<(Vec<u8>, bool)> {
        let mut processed = Vec::new();
        for event in self.chat_queue.set_delay(seconds) {
            self.show_chat(event, log, view, &mut processed);
        }
        processed
    }

    /// A player's or a disguised message shown, or left out for a hidden player: whether it was
    /// shown, with a signed player message noted for the server.
    fn show_chat(
        &self,
        event: ChatEvent,
        log: &mut ChatLog,
        view: ChatView<'_>,
        processed: &mut Vec<(Vec<u8>, bool)>,
    ) -> bool {
        use photon_gui::chat::Tag;
        match event {
            ChatEvent::Disguised { content, bound } => {
                let decorated = self.decorate(content, &bound);
                log.add_component(
                    &decorated,
                    &self.lang,
                    (None, Some(Tag::System)),
                    &self.font,
                    view.focused,
                );
                true
            }
            ChatEvent::Player {
                packet,
                sender_known,
                sender_is_local,
            } => {
                let signature = packet.signature.clone();
                // `Minecraft.isBlocked`: what a player the local player hid says is not shown
                // (the local player's own words always are).
                let hidden = !sender_is_local
                    && view.hidden.contains(&photon_core::binary::Uuid::new(
                        packet.sender.0,
                        packet.sender.1,
                    ));
                let shown = if hidden {
                    false
                } else if sender_known {
                    self.show_player_chat(packet, sender_is_local, log, view.focused)
                } else {
                    // `handleChatMessageError`: shown as an error, and not as a message.
                    self.show_chat_error(&packet.bound, log, view.focused);
                    false
                };
                processed.extend(signature.map(|signature| (signature, shown)));
                // The error line sets the clock as a shown message does.
                shown || (!hidden && !sender_known)
            }
            ChatEvent::System { .. } | ChatEvent::Delete(_) => false,
        }
    }
}

/// What the chat is drawn and heard for: whether its screen is open, and the players hidden.
#[derive(Clone, Copy)]
pub struct ChatView<'a> {
    pub focused: bool,
    pub hidden: &'a [photon_core::binary::Uuid],
}
