use super::*;
use crate::gui::FrameRuns;

impl ChatScreen {
    /// `ChatScreen.init`: the box holds `initial` (empty, or a slash), and
    /// the character of the key that opened it is to be dropped.
    pub fn new(font: Arc<Font>, size: (i32, i32), initial: &str, history: Vec<String>) -> Self {
        let mut edit = EditBox::new(size.0 - 4, 256);
        edit.set_value(initial, &font);
        let history_pos = history.len();
        Self {
            edit,
            font,
            size,
            history,
            history_pos,
            history_buffer: String::new(),
            focused_at: Instant::now(),
            skip_char: true,
            list: None,
            stale: false,
            request: None,
            request_id: 0,
            applied: false,
            commands: None,
            players: Vec::new(),
            teams: Vec::new(),
            selected_entities: Vec::new(),
            selectors: true,
            draft: false,
            local: Vec::new(),
        }
    }

    /// The tree the server sent: it is what suggests the commands, and the
    /// server is asked only for what the tree cannot tell.
    pub fn set_commands(&mut self, commands: Option<Arc<CommandTree>>) {
        self.commands = commands;
    }

    /// The online players' and the teams' names, which suggestions use.
    pub fn set_names(&mut self, players: Vec<String>, teams: Vec<String>) {
        self.players = players;
        self.teams = teams;
    }

    /// The string UUID of the entity in the crosshair, which entity
    /// arguments offer as a name.
    pub fn set_selected_entities(&mut self, selected: Vec<String>) {
        self.selected_entities = selected;
    }

    /// Whether the player's permissions allow entity selectors.
    pub fn set_selectors(&mut self, allowed: bool) {
        self.selectors = allowed;
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
        self.size = size;
    }

    /// The request the last edit asked for, taken.
    pub fn take_request(&mut self) -> Option<(i32, String)> {
        self.request.take()
    }

    /// `CommandSuggestions.updateCommandInfo`: the list goes, and a command's
    /// text up to the cursor asks for its matches. A match just used keeps
    /// the list and asks for nothing.
    pub(super) fn edited(&mut self) {
        // `onEdited`: whatever changes the text makes it the player's own.
        self.draft = false;
        if std::mem::take(&mut self.applied) {
            return;
        }
        self.edit.set_suggestion(None);
        self.stale = true;
        let text = self.edit.value();
        let cursor = self.edit.cursor();
        // The reference knows the commands and answers within the same frame;
        // here the server answers a moment later, so the list stays up until
        // its answer replaces it, rather than blinking out between keys.
        if !(text.starts_with('/') && cursor >= 1) {
            self.list = None;
        }
        if text.starts_with('/') && cursor >= 1 {
            self.local.clear();
            let mut ask_server = true;
            let mut start = 0;
            if let Some(tree) = self.commands.clone() {
                // `getCompletionSuggestions` of the whole text at the cursor.
                let parse = tree.parse(&text, 1, self.selectors);
                let suggested = tree.suggestions(
                    &parse,
                    cursor,
                    &SuggestContext {
                        players: &self.players,
                        teams: &self.teams,
                        selected_entities: &self.selected_entities,
                        selectors: self.selectors,
                    },
                );
                ask_server = suggested.ask_server;
                start = suggested.start;
                self.local = suggested.matches;
            }
            if ask_server {
                self.request_id += 1;
                let typed: String = text.chars().take(cursor).collect();
                self.request = Some((self.request_id, typed));
            } else {
                // The tree knew it all: no request, and the list is final.
                self.request_id += 1;
                let matches = std::mem::take(&mut self.local);
                self.show(start, cursor - start, matches);
            }
        }
    }

    /// The server's answer, when it is to the last request:
    /// `showSuggestions`, with what the tree suggested added to it.
    pub fn suggested(&mut self, id: i32, start: usize, length: usize, matches: Vec<String>) {
        if id != self.request_id {
            return;
        }
        let mut matches = matches;
        // The server's answer comes sorted; it is merged only with the tree's.
        if !self.local.is_empty() {
            matches.append(&mut self.local);
            photon_game::commands::sort_suggestions(&mut matches);
        }
        self.show(start, length, matches);
    }

    /// The list for matches, the text up to the cursor having been read.
    pub(super) fn show(&mut self, start: usize, length: usize, matches: Vec<String>) {
        self.stale = false;
        let value = self.edit.value();
        let typed: String = value.chars().take(self.edit.cursor()).collect();
        self.list = SuggestionsList::new(
            matches,
            (start, length),
            &value,
            &typed,
            self.size.1,
            &self.font,
            (4, self.size.0 - 4),
        );
        self.update_ghost();
    }

    /// `select`'s `setSuggestion`: what the chosen match adds to the text.
    pub(super) fn update_ghost(&mut self) {
        let ghost = self
            .list
            .as_ref()
            .filter(|_| !self.stale)
            .and_then(|list| list.ghost(&self.edit.value()));
        self.edit.set_suggestion(ghost);
    }

    /// `useSuggestion`: the box takes the match's text, the cursor after it.
    pub(super) fn take_match(&mut self, (text, end): (String, usize)) {
        self.applied = true;
        let font = self.font.clone();
        self.edit.set_value(&text, &font);
        self.edit.set_cursor_position(end, &font);
        self.update_ghost();
    }

    /// `CommandSuggestions.keyPressed`: whether the list took the key.
    pub(super) fn list_key(&mut self, key: ListKey) -> bool {
        let Some(list) = &mut self.list else {
            return false;
        };
        if self.stale && key != ListKey::Escape {
            // The list is for older text; the answer is on its way.
            return true;
        }
        if key == ListKey::Escape {
            self.list = None;
            self.edit.set_suggestion(None);
            return true;
        }
        let (taken, used) = list.key(key);
        if let Some(used) = used {
            self.take_match(used);
        } else {
            self.update_ghost();
        }
        taken
    }

    /// A press in the list (`mouseClicked`): whether it was the list's.
    pub fn mouse_clicked(&mut self, mouse: (i32, i32)) -> bool {
        let Some(list) = &mut self.list else {
            return false;
        };
        match list.clicked(mouse) {
            None => false,
            Some(used) => {
                if let Some(used) = used {
                    self.take_match(used);
                }
                true
            }
        }
    }

    /// The wheel over the list (`mouseScrolled`): whether it was the list's.
    pub fn mouse_scrolled(&mut self, scroll: f64, mouse: (i32, i32)) -> bool {
        self.list
            .as_mut()
            .is_some_and(|list| list.scrolled(scroll.clamp(-1.0, 1.0), mouse))
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
            && self.list_key(list_key)
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
            self.edit.set_suggestion(None);
            self.list = None;
            self.stale = false;
            self.request = None;
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
        let parse = commands
            .filter(|_| text.starts_with('/'))
            .map(|(tree, lang)| (tree, lang, tree.parse(&text, 1, self.selectors)));
        // `formatChat`: a restored draft is grey and slanted until it is
        // edited, which comes before the colours of a command.
        let draft = self.draft;
        let format = |part: &str, offset: usize| {
            if draft {
                return Some(vec![(part.to_owned(), 0xAA_AAAA, true)]);
            }
            parse.as_ref().map(|(_, _, parse)| {
                format_text(parse, part, offset)
                    .into_iter()
                    .map(|(part, colour)| (part, colour, false))
                    .collect()
            })
        };
        self.edit.draw_formatted(
            frame,
            atlas,
            &self.font,
            (4, height - 12),
            Some(self.focused_at),
            Some(&format),
        );
        if let Some(list) = &mut self.list {
            list.draw(frame, atlas, &self.font, mouse);
        } else if let (Some((tree, lang, parse)), false) = (&parse, self.stale) {
            let view = usage_lines(tree, parse, self.edit.cursor(), true);
            let lines: Vec<String> = view.lines.iter().map(|line| lang.plain(line)).collect();
            let longest = lines
                .iter()
                .map(|line| self.font.width(line))
                .max()
                .unwrap_or(0);
            let chars: Vec<char> = text.chars().collect();
            let start = view.start.max(0) as usize;
            let start_x = if start > chars.len() {
                4
            } else {
                4 + self.font.width(&chars[..start].iter().collect::<String>())
            };
            let x = start_x.clamp(0, (4 + width - 4 - longest).max(0));
            for (index, line) in view.lines.iter().enumerate() {
                let y = height - 27 - 12 * index as i32;
                frame.fill(atlas, (x - 1, y, x + longest + 1, y + 12), 0xD000_0000);
                let runs = lang.runs(line, &Style::default());
                frame.styled_text(atlas, &self.font, &runs, x, y + 2, 0xFFFF_FFFF, true);
            }
        }
        self.update_ghost();
    }
}
