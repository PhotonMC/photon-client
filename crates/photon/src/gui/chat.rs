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

    /// The chat's clock, and the deletions that have waited long enough.
    pub fn tick_chat(&self, log: &mut ChatLog) {
        log.tick(&self.font, &self.deleted_marker());
    }

    /// `ChatListener.showMessageToPlayer`: a player's message, tagged by how
    /// far it can be trusted, and cut by what the server filtered.
    pub(super) fn show_player_chat(
        &self,
        packet: photon_net::connection::PlayerChatPacket,
        sender_is_local: bool,
        log: &mut ChatLog,
        focused: bool,
    ) {
        use photon_net::connection::FilterMask;
        use photon_render::gui::chat::Tag;
        if packet.filter == FilterMask::FullyFiltered {
            return;
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
    }

    /// The chat the server sent, worded and added to the log.
    pub fn receive_chat(&mut self, events: Vec<ChatEvent>, log: &mut ChatLog, focused: bool) {
        use photon_render::gui::chat::Tag;
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
                    (None, Some(Tag::SystemSinglePlayer)),
                    &self.font,
                    focused,
                ),
                ChatEvent::Disguised { content, bound } => {
                    let decorated = self.decorate(content, &bound);
                    log.add_component(
                        &decorated,
                        &self.lang,
                        (None, Some(Tag::System)),
                        &self.font,
                        focused,
                    );
                }
                ChatEvent::Player {
                    packet,
                    sender_known,
                    sender_is_local,
                } => {
                    if sender_known {
                        self.show_player_chat(packet, sender_is_local, log, focused);
                    } else {
                        // `handleChatMessageError`: a sender the client has
                        // no player for.
                        let style = Style {
                            colour: photon_game::text::named_colour("red"),
                            italic: Some(true),
                            ..Style::default()
                        };
                        let error =
                            photon_game::text::Component::translatable("chat.validation_error")
                                .with_style(&style);
                        let decorated = self.decorate(error, &packet.bound);
                        log.add_component(
                            &decorated,
                            &self.lang,
                            (None, Some(Tag::Error)),
                            &self.font,
                            focused,
                        );
                    }
                }
                ChatEvent::Delete(signature) => {
                    log.delete(&signature, &self.font, &self.deleted_marker());
                }
            }
        }
    }
}
