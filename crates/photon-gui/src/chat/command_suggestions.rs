//! `CommandSuggestions`: the matches and the usage of the command a box holds, as the chat and
//! the command block's screen use them. What differs between them is the constructor's
//! arguments ([`SuggestionSettings`]), and where the box sits ([`Place`]).

use std::sync::Arc;

use photon_game::commands::{CommandTree, SuggestContext, format_text, usage_lines};

mod gate;
use photon_game::text::{Lang, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use super::suggestions::{ListKey, SuggestionsList};
use crate::FrameRuns;
use crate::widgets::edit_box::EditBox;

/// The arguments of `CommandSuggestions`' constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuggestionSettings {
    /// Every text is a command, with or without a slash.
    pub commands_only: bool,
    /// Suggest from where the command starts, rather than after its slash.
    pub only_show_if_cursor_past_error: bool,
    pub line_start_offset: usize,
    pub line_limit: usize,
    /// The list and the usage sit above the bottom of the screen, rather than below the box.
    pub anchor_to_bottom: bool,
    pub fill: u32,
    /// The box has a border (`EditBox.isBordered`).
    pub bordered: bool,
}

impl SuggestionSettings {
    /// `ChatScreen`: a command after a slash, in a box without a border.
    pub const CHAT: Self = Self {
        commands_only: false,
        only_show_if_cursor_past_error: false,
        line_start_offset: 1,
        line_limit: 10,
        anchor_to_bottom: true,
        fill: 0xD000_0000,
        bordered: false,
    };

    /// `AbstractCommandBlockEditScreen`: the whole text is a command, in a bordered box.
    pub const COMMAND_BLOCK: Self = Self {
        commands_only: true,
        only_show_if_cursor_past_error: true,
        line_start_offset: 0,
        line_limit: 7,
        anchor_to_bottom: false,
        fill: 0x8000_0000,
        bordered: true,
    };
}

/// Where the box is: the text's left edge, the width inside it and its top (the list and the
/// usage hang from it when they are not at the bottom), and the screen's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub text_x: i32,
    pub inner_width: i32,
    pub screen: (i32, i32),
}

pub struct CommandSuggestions {
    settings: SuggestionSettings,
    font: Arc<Font>,
    place: Place,
    /// The list of matches the server sent for the text, when there is one.
    pub(super) list: Option<SuggestionsList>,
    /// The list is for text since edited: it waits for the server's answer.
    pub(super) stale: bool,
    /// Where the cursor was at the last edit: `updateUsageInfo` reads the usage there, and moving
    /// the cursor does not read it again.
    usage_cursor: usize,
    /// The request to send: its id and text, and the id of the last sent.
    request: Option<(i32, String)>,
    pub(super) request_id: i32,
    /// The command tree, which suggests what continues a command itself.
    commands: Option<Arc<CommandTree>>,
    /// The names the suggestions draw on: online players, and teams.
    players: Vec<String>,
    teams: Vec<String>,
    /// The aimed entity's string UUID, if any (`getSelectedEntities`).
    selected_entities: Vec<String>,
    /// Whether the player may use entity selectors.
    selectors: bool,
    /// What the tree suggested for the text, while the server's answer on
    /// its own arguments is awaited.
    local: Vec<String>,
    /// `setRestrictions`: whether the player may send messages and commands.
    messages_allowed: bool,
    commands_allowed: bool,
}

/// The command a text holds: whether it is one, and where it starts after its slash.
struct Reading {
    is_command: bool,
    offset: usize,
}

impl CommandSuggestions {
    pub fn new(settings: SuggestionSettings, font: Arc<Font>, place: Place) -> Self {
        Self {
            settings,
            font,
            place,
            list: None,
            stale: false,
            usage_cursor: 0,
            request: None,
            request_id: 0,
            commands: None,
            players: Vec::new(),
            teams: Vec::new(),
            selected_entities: Vec::new(),
            selectors: true,
            local: Vec::new(),
            messages_allowed: true,
            commands_allowed: true,
        }
    }

    pub fn set_place(&mut self, place: Place) {
        self.place = place;
    }

    pub fn has_commands(&self) -> bool {
        self.commands.is_some()
    }

    pub fn set_commands(&mut self, commands: Option<Arc<CommandTree>>) {
        self.commands = commands;
    }

    pub fn set_names(&mut self, players: Vec<String>, teams: Vec<String>) {
        self.players = players;
        self.teams = teams;
    }

    pub fn set_selected_entities(&mut self, selected: Vec<String>) {
        self.selected_entities = selected;
    }

    pub fn set_selectors(&mut self, allowed: bool) {
        self.selectors = allowed;
    }

    /// The request the last edit asked for, taken.
    pub fn take_request(&mut self) -> Option<(i32, String)> {
        self.request.take()
    }

    fn reading(&self, text: &str) -> Reading {
        let slash = text.starts_with('/');
        Reading {
            is_command: self.settings.commands_only || slash,
            offset: usize::from(slash),
        }
    }

    /// `updateCommandInfo`: the list goes, and a command's text up to the cursor asks for its
    /// matches.
    pub fn edited(&mut self, edit: &mut EditBox) {
        edit.set_suggestion(None);
        self.stale = true;
        let text = edit.value();
        let cursor = edit.cursor();
        self.usage_cursor = cursor;
        let reading = self.reading(&text);
        let parse_start = if self.settings.only_show_if_cursor_past_error {
            reading.offset
        } else {
            1
        };
        let asks = reading.is_command && cursor >= parse_start;
        // The reference knows the commands and answers within the same frame;
        // here the server answers a moment later, so the list stays up until
        // its answer replaces it, rather than blinking out between keys.
        if !asks {
            self.list = None;
            return;
        }
        self.local.clear();
        let mut ask_server = true;
        let mut start = 0;
        if let Some(tree) = self.commands.clone() {
            // `getCompletionSuggestions` of the whole text at the cursor.
            let parse = tree.parse(&text, reading.offset, self.selectors);
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
        self.request_id += 1;
        if ask_server {
            let typed: String = text.chars().take(cursor).collect();
            self.request = Some((self.request_id, typed));
        } else {
            // The tree knew it all: no request, and the list is final.
            let matches = std::mem::take(&mut self.local);
            self.show(edit, start, cursor - start, matches);
        }
    }

    /// The server's answer, when it is to the last request:
    /// `showSuggestions`, with what the tree suggested added to it.
    pub fn suggested(
        &mut self,
        edit: &mut EditBox,
        (id, start, length): (i32, usize, usize),
        mut matches: Vec<String>,
    ) {
        if id != self.request_id {
            return;
        }
        // The server's answer comes sorted; it is merged only with the tree's.
        if !self.local.is_empty() {
            matches.append(&mut self.local);
            photon_game::commands::sort_suggestions(&mut matches);
        }
        self.show(edit, start, length, matches);
    }

    /// The list for matches, the text up to the cursor having been read.
    pub(super) fn show(
        &mut self,
        edit: &mut EditBox,
        start: usize,
        length: usize,
        matches: Vec<String>,
    ) {
        self.stale = false;
        let value = edit.value();
        let typed: String = value.chars().take(edit.cursor()).collect();
        let Place {
            text_x,
            inner_width,
            screen,
        } = self.place;
        self.list = SuggestionsList::new(
            matches,
            (start, length),
            &value,
            &typed,
            (&self.settings, screen),
            &self.font,
            (text_x, inner_width),
        );
        self.update_ghost(edit);
    }

    /// `select`'s `setSuggestion`: what the chosen match adds to the text.
    pub fn update_ghost(&self, edit: &mut EditBox) {
        let ghost = self
            .list
            .as_ref()
            .filter(|_| !self.stale)
            .and_then(|list| list.ghost(&edit.value()));
        edit.set_suggestion(ghost);
    }

    /// `useSuggestion`: the box takes the match's text, the cursor after it.
    fn take_match(&mut self, edit: &mut EditBox, (text, end): (String, usize)) {
        edit.set_value(&text, &self.font);
        edit.set_cursor_position(end, &self.font);
        self.update_ghost(edit);
    }

    /// `keyPressed`: whether the list took the key.
    pub fn list_key(&mut self, edit: &mut EditBox, key: ListKey) -> bool {
        let Some(list) = &mut self.list else {
            return false;
        };
        if self.stale && key != ListKey::Escape {
            // The list is for older text; the answer is on its way.
            return true;
        }
        if key == ListKey::Escape {
            self.list = None;
            edit.set_suggestion(None);
            return true;
        }
        let (taken, used) = list.key(key);
        if let Some(used) = used {
            self.take_match(edit, used);
        } else {
            self.update_ghost(edit);
        }
        taken
    }

    /// A press in the list (`mouseClicked`): whether it was the list's.
    pub fn mouse_clicked(&mut self, edit: &mut EditBox, mouse: (i32, i32)) -> bool {
        let Some(list) = &mut self.list else {
            return false;
        };
        match list.clicked(mouse) {
            None => false,
            Some(used) => {
                if let Some(used) = used {
                    self.take_match(edit, used);
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

    /// A line taken from the history shows no list until the next edit (`setAllowSuggestions`).
    pub fn hide(&mut self, edit: &mut EditBox) {
        edit.set_suggestion(None);
        self.list = None;
        self.stale = false;
        self.request = None;
    }

    /// The text read against the tree, when it is a command and the tree is known.
    pub fn parse<'a>(
        &self,
        text: &str,
        commands: Option<(&'a CommandTree, &'a Lang)>,
    ) -> Option<Parsed<'a>> {
        let reading = self.reading(text);
        commands
            .filter(|_| reading.is_command)
            .map(|(tree, lang)| Parsed {
                tree,
                lang,
                parse: tree.parse(text, reading.offset, self.selectors),
            })
    }

    /// Where the usage lines start: under the argument they are about, kept on the screen.
    fn usage_x(&self, text: &str, start: i32, longest: i32) -> i32 {
        let chars: Vec<char> = text.chars().collect();
        let start = start.max(0) as usize;
        let Place {
            text_x,
            inner_width,
            ..
        } = self.place;
        let start_x = if start > chars.len() {
            text_x
        } else {
            text_x + self.font.width(&chars[..start].iter().collect::<String>())
        };
        // `Mth.clamp`: the upper bound wins when it is below the lower one.
        start_x.max(0).min(text_x + inner_width - longest)
    }

    /// `extractRenderState`: the list of matches, or the usage and errors of the command in its
    /// place.
    pub fn draw(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        mouse: (i32, i32),
        (edit, parsed): (&mut EditBox, Option<&Parsed<'_>>),
        lang: &Lang,
    ) {
        if let Some(list) = &mut self.list {
            list.draw(frame, atlas, &self.font, mouse);
        } else if let (Some(parsed), false) = (parsed, self.stale) {
            self.draw_usage(frame, atlas, edit, parsed);
        } else if parsed.is_none() {
            self.draw_message_usage(frame, atlas, &edit.value(), lang);
        }
        self.update_ghost(edit);
    }

    fn draw_usage(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        edit: &EditBox,
        parsed: &Parsed<'_>,
    ) {
        let text = edit.value();
        let view = usage_lines(parsed.tree, &parsed.parse, self.usage_cursor, true);
        let mut view = view;
        view.lines.extend(self.restriction_lines(&text));
        let lines: Vec<String> = view
            .lines
            .iter()
            .map(|line| parsed.lang.plain(line))
            .collect();
        let longest = lines
            .iter()
            .map(|line| self.font.width(line))
            .max()
            .unwrap_or(0);
        let x = self.usage_x(&text, view.start, longest);
        for (index, line) in view.lines.iter().enumerate() {
            let step = 12 * index as i32;
            let y = if self.settings.anchor_to_bottom {
                self.place.screen.1 - 27 - step
            } else {
                72 + step
            };
            frame.fill(
                atlas,
                (x - 1, y, x + longest + 1, y + 12),
                self.settings.fill,
            );
            let runs = parsed.lang.runs(line, &Style::default());
            frame.styled_text(atlas, &self.font, &runs, x, y + 2, 0xFFFF_FFFF, true);
        }
    }
}

/// A command text read against the tree.
pub struct Parsed<'a> {
    tree: &'a CommandTree,
    lang: &'a Lang,
    parse: photon_game::commands::ParseResults,
}

impl Parsed<'_> {
    /// `formatText`: the parts of a stretch of the box's text, in the colours of what read it.
    pub fn format(&self, text: &str, offset: usize) -> Vec<(String, u32, bool)> {
        format_text(&self.parse, text, offset)
            .into_iter()
            .map(|(part, colour)| (part, colour, false))
            .collect()
    }
}
