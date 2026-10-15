use super::*;

impl PlaySession {
    /// `ClientboundOpenBookPacket`: the book in a hand is to be shown.
    /// `handleOpenBook`, once: the pages of the book in the hand the server opened, if it holds
    /// a written book or a book and quill (`BookAccess.fromItem`).
    pub fn take_open_book(&mut self) -> Option<Vec<photon_game::text::Component>> {
        let hand = self.opened_book.take()?;
        self.hand_book_pages(hand)
    }

    /// The pages of the book in a hand.
    fn hand_book_pages(&self, hand: u8) -> Option<Vec<photon_game::text::Component>> {
        let slot = if hand == 0 {
            36 + self.status.selected_slot
        } else {
            45
        };
        self.trace
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(slot as i16))
            .and_then(|stack| {
                photon_game::item::values::Values::new(
                    stack.item_registry_id(),
                    stack.patch_bytes(),
                )
                .book_pages()
            })
    }

    /// The hand whose book and quill was used and the book's pages, once: its screen is to open
    /// (`LocalPlayer.openItemGui`).
    pub fn take_opened_book(&mut self) -> Option<(u8, Vec<photon_game::text::Component>)> {
        let hand = self.game_mode_state.take_opened_book()?;
        Some((hand, self.hand_book_pages(hand).unwrap_or_default()))
    }

    /// `ServerboundEditBookPacket`: the book in a hand (its inventory slot: the selected one or
    /// the off hand's, 40) written with these pages, and signed with a title if one is given.
    pub fn edit_book(
        &mut self,
        hand: u8,
        pages: &[String],
        title: Option<&str>,
    ) -> Result<(), ConnectionError> {
        let slot = if hand == 0 {
            self.status.selected_slot
        } else {
            40
        };
        let string = |text: &str, output: &mut Vec<u8>| {
            wire::write_varint(text.len() as i32, output);
            output.extend_from_slice(text.as_bytes());
        };
        self.send(PLAY_SERVERBOUND_EDIT_BOOK_PACKET_ID, |output| {
            wire::write_varint(slot, output);
            wire::write_varint(pages.len() as i32, output);
            for page in pages {
                string(page, output);
            }
            output.push(u8::from(title.is_some()));
            if let Some(title) = title {
                string(title, output);
            }
            Ok(())
        })
    }
}
