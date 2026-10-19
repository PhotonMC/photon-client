use super::*;

/// `InBedChatScreen.init`: the button, 200 by 20 units, 40 above the bottom.
fn leave_bed_buttons(
    size: (i32, i32),
    label: Vec<Run>,
) -> crate::widgets::button_set::ButtonSet<()> {
    let mut buttons = crate::widgets::button_set::ButtonSet::new(size);
    buttons.add(
        (size.0 / 2 - 100, size.1 - 40, 200, 20),
        (crate::widgets::Kind::Button { label }, true),
        (),
        None,
    );
    buttons
}

/// Where the chat's box sits: the whole width of the screen, at its bottom.
fn place_of(size: (i32, i32)) -> Place {
    Place {
        text_x: 4,
        inner_width: size.0 - 4,
        screen: size,
    }
}

impl ChatScreen {
    /// `ChatScreen.init`: the box holds `initial` (empty, or a slash), and
    /// the character of the key that opened it is to be dropped.
    pub fn new(font: Arc<Font>, size: (i32, i32), initial: &str, history: Vec<String>) -> Self {
        let mut edit = EditBox::new(size.0 - 4, 256);
        edit.set_value(initial, &font);
        let history_pos = history.len();
        let place = place_of(size);
        Self {
            edit,
            suggest: CommandSuggestions::new(SuggestionSettings::CHAT, font.clone(), place),
            font,
            size,
            history,
            history_pos,
            history_buffer: String::new(),
            focused_at: Instant::now(),
            skip_char: true,
            draft: false,
            bed: None,
        }
    }

    /// The tree the server sent: it is what suggests the commands, and the
    /// server is asked only for what the tree cannot tell.
    pub fn set_commands(&mut self, commands: Option<Arc<CommandTree>>) {
        self.suggest.set_commands(commands);
    }

    /// The online players' and the teams' names, which suggestions use.
    pub fn set_names(&mut self, players: Vec<String>, teams: Vec<String>) {
        self.suggest.set_names(players, teams);
    }

    /// The string UUID of the entity in the crosshair, which entity
    /// arguments offer as a name.
    pub fn set_selected_entities(&mut self, selected: Vec<String>) {
        self.suggest.set_selected_entities(selected);
    }

    /// Whether the player's permissions allow entity selectors.
    pub fn set_selectors(&mut self, allowed: bool) {
        self.suggest.set_selectors(allowed);
    }

    /// The box opens on a draft kept from an earlier box.
    pub fn as_draft(mut self) -> Self {
        self.draft = true;
        self
    }

    pub fn is_draft(&self) -> bool {
        self.draft
    }

    /// A script's chat box: no key opened it.
    pub fn without_key(mut self) -> Self {
        self.skip_char = false;
        self
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        if self.size == size {
            return;
        }
        self.size = size;
        self.suggest.set_place(place_of(size));
        if let Some((buttons, label)) = self.bed.take() {
            self.bed = Some((leave_bed_buttons(size, label.clone()), label));
            drop(buttons);
        }
    }

    /// `InBedChatScreen`: the box has the button that gets the player out of bed.
    pub fn in_bed(mut self, lang: &Lang) -> Self {
        let label = lang.runs(
            &Component::translatable("multiplayer.stopSleeping"),
            &Style::default(),
        );
        self.bed = Some((leave_bed_buttons(self.size, label.clone()), label));
        self
    }

    pub fn is_in_bed(&self) -> bool {
        self.bed.is_some()
    }

    /// A press at `mouse` on the leave-bed button.
    pub fn leave_bed_clicked(&self, mouse: (i32, i32)) -> bool {
        self.bed
            .as_ref()
            .is_some_and(|(buttons, _)| buttons.clicked(mouse).is_some())
    }

    /// The text sent, the box empty again (`ChatScreen` without `closeOnSubmit`).
    pub fn clear(&mut self) {
        let font = self.font.clone();
        self.edit.set_value("", &font);
        self.edited();
    }

    /// The request the last edit asked for, taken.
    pub fn take_request(&mut self) -> Option<(i32, String)> {
        self.suggest.take_request()
    }

    /// `CommandSuggestions.updateCommandInfo`: the list goes, and a command's
    /// text up to the cursor asks for its matches.
    pub(super) fn edited(&mut self) {
        // `onEdited`: whatever changes the text makes it the player's own.
        self.draft = false;
        self.suggest.edited(&mut self.edit);
    }

    /// The server's answer, when it is to the last request.
    pub fn suggested(&mut self, id: i32, start: usize, length: usize, matches: Vec<String>) {
        self.suggest
            .suggested(&mut self.edit, (id, start, length), matches);
    }

    /// A press in the list (`mouseClicked`): whether it was the list's.
    pub fn mouse_clicked(&mut self, mouse: (i32, i32)) -> bool {
        self.suggest.mouse_clicked(&mut self.edit, mouse)
    }

    /// The wheel over the list (`mouseScrolled`): whether it was the list's.
    pub fn mouse_scrolled(&mut self, scroll: f64, mouse: (i32, i32)) -> bool {
        self.suggest.mouse_scrolled(scroll, mouse)
    }

    /// The box takes the focus again: its cursor shows from now.
    pub fn refocus(&mut self) {
        self.focused_at = Instant::now();
    }

    /// The font the box's text is set in.
    pub fn font(&self) -> Arc<Font> {
        self.font.clone()
    }

    /// `insertText`: the text goes into the box, or takes its place.
    pub fn insert_text(&mut self, text: &str, replace: bool) {
        let font = self.font.clone();
        if replace {
            self.edit.set_value(text, &font);
        } else {
            self.edit.insert_text(text, &font);
        }
        self.edited();
    }

    /// The box's text and its cursor.
    pub fn value(&self) -> String {
        self.edit.value()
    }

    pub fn cursor(&self) -> usize {
        self.edit.cursor()
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// `ChatScreen.keyPressed`.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> ChatKey {
        use glfw::Key;
        let list_key = match key {
            Key::Up => Some(ListKey::Up),
            Key::Down => Some(ListKey::Down),
            Key::Tab => Some(ListKey::Tab { shift }),
            Key::Escape => Some(ListKey::Escape),
            _ => None,
        };
        if let Some(list_key) = list_key
            && self.suggest.list_key(&mut self.edit, list_key)
        {
            return ChatKey::None;
        }
        if self.draft && key == Key::Backspace {
            // A draft is wiped by one Backspace.
            let font = self.font.clone();
            self.edit.set_value("", &font);
            self.edited();
            return ChatKey::None;
        }
        match key {
            Key::Escape => return ChatKey::Close,
            Key::Enter | Key::KpEnter => {
                let text = normalize(&self.edit.value());
                if !text.is_empty() {
                    if self.history.last() != Some(&text) {
                        self.history.push(text.clone());
                        if self.history.len() > MAX_LINES {
                            self.history.remove(0);
                        }
                    }
                    return ChatKey::Send(text);
                }
                return ChatKey::Done;
            }
            Key::Tab => {}
            Key::Up => self.move_in_history(-1),
            Key::Down => self.move_in_history(1),
            Key::PageUp => return ChatKey::Page(1),
            Key::PageDown => return ChatKey::Page(-1),
            other => {
                if let Some(edit) = edit_key(other) {
                    let font = self.font.clone();
                    let before = self.edit.value();
                    self.edit
                        .key_pressed(edit, control, shift, &font, clipboard);
                    if self.edit.value() != before {
                        self.edited();
                    }
                }
            }
        }
        ChatKey::None
    }

    /// `charTyped`: a character the chat allows.
    pub fn char_typed(&mut self, character: char) {
        if std::mem::take(&mut self.skip_char) {
            return;
        }
        if character != '\u{a7}' && character >= ' ' && character != '\u{7f}' {
            let font = self.font.clone();
            if self.edit.char_typed(character, &font) {
                self.edited();
            }
        }
    }

    /// `ChatScreen.moveInHistory`: the older or newer line, the line being
    /// typed coming back at the end.
    pub(super) fn move_in_history(&mut self, direction: i32) {
        let new_pos = (self.history_pos as i32 + direction).clamp(0, self.history.len() as i32);
        let new_pos = new_pos as usize;
        if new_pos == self.history_pos {
            return;
        }
        if new_pos == self.history.len() {
            self.history_pos = new_pos;
            let buffer = self.history_buffer.clone();
            let font = self.font.clone();
            self.edit.set_value(&buffer, &font);
            // `setValue` tells the responder, which asks for suggestions.
            self.edited();
        } else {
            if self.history_pos == self.history.len() {
                self.history_buffer = self.edit.value();
            }
            self.history_pos = new_pos;
            let line = self.history[new_pos].clone();
            let font = self.font.clone();
            self.edit.set_value(&line, &font);
            // `setAllowSuggestions(false)`: a line taken from the history
            // shows no list until the next edit.
            self.suggest.hide(&mut self.edit);
        }
    }

    /// The box over the chat: a dark band, the text in it (coloured by what
    /// reads it when it is a command), and the list of matches over it, or
    /// the usage and errors of the command in its place.
    pub fn draw(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        mouse: (i32, i32),
        commands: Option<(&CommandTree, &Lang)>,
    ) {
        let (width, height) = self.size;
        frame.fill(atlas, (2, height - 14, width - 2, height - 2), 0x8000_0000);
        let text = self.edit.value();
        // `updateCommandInfo`: a command is read against the tree.
        let parsed = self.suggest.parse(&text, commands);
        // `formatChat`: a restored draft is grey and slanted until it is
        // edited, which comes before the colours of a command.
        let draft = self.draft;
        let format = |part: &str, offset: usize| {
            if draft {
                return Some(vec![(part.to_owned(), 0xAA_AAAA, true)]);
            }
            parsed.as_ref().map(|parsed| parsed.format(part, offset))
        };
        self.edit.draw_formatted(
            frame,
            atlas,
            &self.font,
            (4, height - 12),
            Some(self.focused_at),
            Some(&format),
        );
        if let Some((buttons, _)) = &self.bed {
            buttons.draw(frame, atlas, &self.font, mouse);
        }
        self.suggest
            .draw(frame, atlas, mouse, (&mut self.edit, parsed.as_ref()));
    }
}
