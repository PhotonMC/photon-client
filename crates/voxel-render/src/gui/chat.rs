//! The chat: its lines as `ChatComponent` keeps, wraps, fades and draws them
//! over the HUD, and the box a line is typed into (`ChatScreen`).
//!
//! The chat's width, height, scale, opacity, background opacity and line
//! spacing are the options' ([`ChatSettings`]). Not drawn: the chat queue
//! and the prompt of a chat that the options disable.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use voxel_game::commands::{CommandTree, SuggestContext, format_text, usage_lines};
use voxel_game::text::{Component, HoverEvent, Lang, Run, Style};

use crate::gui::GuiAtlas;
use crate::gui::GuiFrame;
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;
use crate::gui::suggestions::{ListKey, SuggestionsList};

/// The lines the chat keeps.
const MAX_LINES: usize = 100;
/// A message's height, and the room under the chat for the hotbar.
const MESSAGE_HEIGHT: i32 = 9;
const BOTTOM_MARGIN: i32 = 40;

/// The options `ChatComponent` reads, each a share from 0 to 1 as the
/// option holds it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatSettings {
    pub width: f64,
    pub scale: f64,
    pub opacity: f64,
    pub background_opacity: f64,
    pub line_spacing: f64,
    pub height_focused: f64,
    pub height_unfocused: f64,
}

impl Default for ChatSettings {
    /// The options' defaults.
    fn default() -> Self {
        Self::from(&voxel_game::options::Options::default())
    }
}

impl From<&voxel_game::options::Options> for ChatSettings {
    fn from(options: &voxel_game::options::Options) -> Self {
        Self {
            width: options.chat_width,
            scale: options.chat_scale,
            opacity: options.chat_opacity,
            background_opacity: options.text_background_opacity,
            line_spacing: options.chat_line_spacing,
            height_focused: options.chat_height_focused,
            height_unfocused: options.chat_height_unfocused,
        }
    }
}

/// `ChatComponent.getWidth`: the width in units of a share.
pub fn width_of(share: f64) -> i32 {
    (share * 280.0 + 40.0).floor() as i32
}

/// `ChatComponent.getHeight`: the height in units of a share.
pub fn height_of(share: f64) -> i32 {
    (share * 160.0 + 20.0).floor() as i32
}

/// `GuiMessageTag`: the mark beside a message that says where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    /// `system`: a message the server sent as a player would.
    System,
    /// `systemSinglePlayer`: what the server says.
    SystemSinglePlayer,
    /// `chatNotSecure`: a player's message with no signature.
    NotSecure,
    /// `chatModified`: signed, but shown changed; holds the original.
    Modified(String),
    /// `chatError`: a message that could not be checked.
    Error,
}

impl Tag {
    /// `indicatorColor`.
    fn colour(&self) -> u32 {
        match self {
            Self::System | Self::SystemSinglePlayer | Self::NotSecure => 0x00D0_D0D0,
            Self::Modified(_) => 0x0060_6060,
            Self::Error => 0x00FF_5555,
        }
    }

    /// Whether the tag has an icon (`GuiMessageTag.Icon`), which is 9 by 9.
    fn has_icon(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    /// The tag's tooltip, as lines of the text it is made of.
    fn tooltip(&self, lang: &voxel_game::text::Lang) -> Vec<Run> {
        let key = match self {
            Self::System => "chat.tag.system",
            Self::SystemSinglePlayer => "chat.tag.system_single_player",
            Self::NotSecure => "chat.tag.not_secure",
            Self::Modified(_) => "chat.tag.modified",
            Self::Error => "chat.tag.error",
        };
        let mut runs = vec![Run {
            text: lang.text(key),
            style: Style::default(),
        }];
        if let Self::Modified(original) = self {
            runs.push(Run {
                text: format!("\n{original}"),
                style: Style {
                    colour: Some(0xAAAAAA),
                    ..Style::default()
                },
            });
        }
        runs
    }
}

/// A `GuiMessage`: what was said, when, and its signature and tag.
#[derive(Debug, Clone)]
struct Message {
    runs: Vec<Run>,
    /// What the runs were made of, if it is known, to be worded again.
    source: Option<Component>,
    added: u64,
    signature: Option<Vec<u8>>,
    tag: Option<Tag>,
}

/// One wrapped line of a message (`GuiMessage.Line`).
#[derive(Debug, Clone)]
struct ChatLine {
    runs: Vec<Run>,
    added: u64,
    tag: Option<Tag>,
}

/// `StringSplitter.splitLines`: the runs broken into lines of at most `max`
/// units, at the last space that fits or inside a word when there is none.
/// The space a line breaks at goes; a line that follows another starts with
/// a space (`ComponentRenderUtils.wrapComponents`).
pub fn wrap(runs: &[Run], max: i32, font: &Font) -> Vec<Vec<Run>> {
    break_lines(runs, max, font, true)
}

/// `Font.split`: the same lines, with no indent on those that follow.
pub fn split(runs: &[Run], max: i32, font: &Font) -> Vec<Vec<Run>> {
    break_lines(runs, max, font, false)
}

fn break_lines(runs: &[Run], max: i32, font: &Font, indent_soft: bool) -> Vec<Vec<Run>> {
    let chars: Vec<(char, usize)> = runs
        .iter()
        .enumerate()
        .flat_map(|(index, run)| run.text.chars().map(move |c| (c, index)))
        .collect();
    let advance = |(c, index): (char, usize)| {
        let bold = runs[index].style.bold == Some(true);
        font.glyph(c).map_or(0.0, |glyph| glyph.advance(bold))
    };
    // Each line, and whether it follows a soft break (one the width made).
    let mut lines: Vec<(Vec<(char, usize)>, bool)> = Vec::new();
    let mut soft = false;
    let mut current: Vec<(char, usize)> = Vec::new();
    let mut width = 0.0_f32;
    let mut last_space: Option<usize> = None;
    for &entry in &chars {
        if entry.0 == '\n' {
            lines.push((std::mem::take(&mut current), soft));
            soft = false;
            width = 0.0;
            last_space = None;
            continue;
        }
        current.push(entry);
        width += advance(entry);
        if entry.0 == ' ' {
            last_space = Some(current.len() - 1);
        }
        if width > max as f32 && current.len() > 1 {
            let (line, rest) = match last_space {
                Some(space) => {
                    let rest = current.split_off(space + 1);
                    current.pop();
                    (std::mem::take(&mut current), rest)
                }
                None => {
                    let rest = current.split_off(current.len() - 1);
                    (std::mem::take(&mut current), rest)
                }
            };
            lines.push((line, soft));
            soft = true;
            current = rest;
            width = current.iter().map(|entry| advance(*entry)).sum();
            last_space = current.iter().rposition(|entry| entry.0 == ' ');
        }
    }
    lines.push((current, soft));
    lines
        .into_iter()
        .map(|(line, soft)| {
            let mut out: Vec<Run> = Vec::new();
            // A line after a soft break starts with a space in no style.
            let indent = (soft && indent_soft).then_some((' ', Style::default()));
            let styled = line
                .into_iter()
                .map(|(c, index)| (c, runs[index].style.clone()));
            for (c, style) in indent.into_iter().chain(styled) {
                match out.last_mut() {
                    Some(run) if run.style == style => run.text.push(c),
                    _ => out.push(Run {
                        text: c.to_string(),
                        style,
                    }),
                }
            }
            out
        })
        .collect()
}

/// `AlphaCalculator.timeBased`: fully shown for 180 ticks, then faded out by
/// the 200th.
fn fade(age: u64) -> f32 {
    let t = ((1.0 - age as f32 / 200.0) * 10.0).clamp(0.0, 1.0);
    t * t
}

/// `ChatComponent.TIME_BEFORE_MESSAGE_DELETION`.
const TIME_BEFORE_DELETION: u64 = 60;

/// The messages the player has been told, and their lines, newest first.
#[derive(Debug, Default)]
pub struct ChatLog {
    messages: VecDeque<Message>,
    lines: VecDeque<ChatLine>,
    ticks: u64,
    scroll: usize,
    /// `newMessageSinceScroll`: the scroll bar shows red.
    new_message: bool,
    /// `messageDeletionQueue`: a signature and the tick it may go at.
    deletions: Vec<(Vec<u8>, u64)>,
    settings: ChatSettings,
}

impl ChatLog {
    /// `ChatComponent.tick`: the clock, and the deletions that have waited
    /// long enough. `marker` is the deleted message's text.
    pub fn tick(&mut self, font: &Font, marker: &[Run]) {
        self.ticks += 1;
        if self.deletions.is_empty() {
            return;
        }
        let now = self.ticks;
        let waiting = std::mem::take(&mut self.deletions);
        for (signature, after) in waiting {
            if now >= after {
                if let Some(delayed) = self.delete_or_delay(&signature, font, marker) {
                    self.deletions.push(delayed);
                }
            } else {
                self.deletions.push((signature, after));
            }
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn settings(&self) -> ChatSettings {
        self.settings
    }

    /// A chat with no messages that wraps for these settings.
    pub fn with_settings(settings: ChatSettings) -> Self {
        Self {
            settings,
            ..Self::default()
        }
    }

    /// The options changed: `rescaleChat` takes the chat back to the bottom
    /// and wraps its messages again for the new width and scale.
    pub fn set_settings(&mut self, settings: ChatSettings, font: &Font) {
        if settings != self.settings {
            self.settings = settings;
            self.rescale(font);
        }
    }

    /// `rescaleChat`.
    pub fn rescale(&mut self, font: &Font) {
        self.reset_scroll();
        self.refresh(font);
    }

    /// `getLineHeight`: a message's line and the space between lines.
    fn entry_height(&self) -> i32 {
        (f64::from(MESSAGE_HEIGHT) * (self.settings.line_spacing + 1.0)) as i32
    }

    /// `getLinesPerPage`: the lines a chat shows, an open one more than a
    /// closed one.
    pub fn lines_per_page(&self, focused: bool) -> usize {
        let share = if focused {
            self.settings.height_focused
        } else {
            self.settings.height_unfocused
        };
        (height_of(share) / self.entry_height()).max(0) as usize
    }

    /// `ChatComponent.addMessage`: the message kept and wrapped, newest
    /// first. A chat that is scrolled back keeps what it shows.
    pub fn add(&mut self, runs: &[Run], font: &Font, focused: bool) {
        self.add_message(runs, None, None, font, focused);
    }

    /// `addMessage` of a component, which is worded in `lang` now and again
    /// when the language changes.
    pub fn add_component(
        &mut self,
        component: &Component,
        lang: &Lang,
        (signature, tag): (Option<Vec<u8>>, Option<Tag>),
        font: &Font,
        focused: bool,
    ) {
        self.add_message(
            &lang.runs(component, &Style::default()),
            signature,
            tag,
            font,
            focused,
        );
        if let Some(message) = self.messages.front_mut() {
            message.source = Some(component.clone());
        }
    }

    /// The language changed: the messages made of a component are worded
    /// again, and the lines wrapped anew (`rescaleChat` on a reload).
    pub fn set_lang(&mut self, lang: &Lang, font: &Font) {
        for message in &mut self.messages {
            if let Some(source) = &message.source {
                message.runs = lang.runs(source, &Style::default());
            }
        }
        self.rescale(font);
    }

    /// `addMessage` with a signature and a tag.
    pub fn add_message(
        &mut self,
        runs: &[Run],
        signature: Option<Vec<u8>>,
        tag: Option<Tag>,
        font: &Font,
        focused: bool,
    ) {
        let message = Message {
            runs: runs.to_vec(),
            source: None,
            added: self.ticks,
            signature,
            tag,
        };
        self.add_to_display(&message, font, focused);
        self.messages.push_front(message);
        self.messages.truncate(MAX_LINES);
    }

    /// `addMessageToDisplayQueue`: the message's lines, the width short of a
    /// tag's icon.
    fn add_to_display(&mut self, message: &Message, font: &Font, focused: bool) {
        let mut width =
            (f64::from(width_of(self.settings.width)) / self.settings.scale).floor() as i32;
        if message.tag.as_ref().is_some_and(Tag::has_icon) {
            width -= 9 + 4 + 2;
        }
        for line in wrap(&message.runs, width, font) {
            self.lines.push_front(ChatLine {
                runs: line,
                added: message.added,
                tag: message.tag.clone(),
            });
            if focused && self.scroll > 0 {
                self.new_message = true;
                self.scroll_by(1, true);
            }
        }
        self.lines.truncate(MAX_LINES);
    }

    /// `ChatComponent.deleteMessage`: a signed message is replaced by the
    /// marker, once it is 60 ticks old.
    pub fn delete(&mut self, signature: &[u8], font: &Font, marker: &[Run]) {
        if let Some(delayed) = self.delete_or_delay(signature, font, marker) {
            self.deletions.push(delayed);
        }
    }

    fn delete_or_delay(
        &mut self,
        signature: &[u8],
        font: &Font,
        marker: &[Run],
    ) -> Option<(Vec<u8>, u64)> {
        let place = self
            .messages
            .iter()
            .position(|message| message.signature.as_deref() == Some(signature))?;
        let after = self.messages[place].added + TIME_BEFORE_DELETION;
        if self.ticks >= after {
            self.messages[place] = Message {
                runs: marker.to_vec(),
                source: None,
                added: self.messages[place].added,
                signature: None,
                tag: Some(Tag::System),
            };
            self.refresh(font);
            return None;
        }
        Some((signature.to_vec(), after))
    }

    /// `refreshTrimmedMessages`: the lines made again from the messages,
    /// oldest first.
    fn refresh(&mut self, font: &Font) {
        self.lines.clear();
        let messages: Vec<Message> = self.messages.iter().rev().cloned().collect();
        for message in &messages {
            self.add_to_display(message, font, false);
        }
    }

    /// `scrollChat`, in lines.
    pub fn scroll_by(&mut self, amount: i32, focused: bool) {
        let limit = self
            .lines
            .len()
            .saturating_sub(self.lines_per_page(focused));
        self.scroll = (self.scroll as i32 + amount).clamp(0, limit as i32) as usize;
        if self.scroll == 0 {
            self.new_message = false;
        }
    }

    /// `resetChatScroll`.
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
        self.new_message = false;
    }

    /// `ChatComponent.extractRenderState`: the lines over the hotbar. An open
    /// chat with the mouse over a tag gives back the tag's tooltip.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        gui: (i32, i32),
        focused: bool,
    ) {
        self.draw_with(frame, atlas, font, gui, focused, None, &Lang::default());
    }

    /// [`Self::draw`] with the mouse, for the tooltip of a tag it is over.
    /// Everything is drawn in the chat's own units, under a pose that scales
    /// by the option and moves 4 right.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the reference's render call"
    )]
    pub fn draw_with(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (gui_width, gui_height): (i32, i32),
        focused: bool,
        mouse: Option<(i32, i32)>,
        lang: &Lang,
    ) -> Option<Vec<Vec<Run>>> {
        let first = (frame.vertices.len(), frame.log.as_ref().map_or(0, Vec::len));
        let scale = self.settings.scale as f32;
        let max_width = (width_of(self.settings.width) as f32 / scale).ceil() as i32;
        let chat_bottom = ((gui_height - BOTTOM_MARGIN) as f32 / scale).floor() as i32;
        let text_opacity = self.settings.opacity as f32 * 0.9 + 0.1;
        let background_opacity = self.settings.background_opacity as f32;
        let spacing = self.settings.line_spacing;
        let entry_height = self.entry_height();
        // `Math.round`.
        let entry_bottom_to_message_y =
            (8.0 * (spacing + 1.0) - 4.0 * spacing + 0.5).floor() as i32;
        let per_page = self.lines_per_page(focused);
        let shown = self.lines.len().saturating_sub(self.scroll).min(per_page);
        // `forEachLine`: from the oldest of the page to the newest, the ones
        // not faded out; the backgrounds first, then each line's text and tag.
        let visible: Vec<(usize, f32)> = (0..shown)
            .rev()
            .filter_map(|i| {
                let line = &self.lines[i + self.scroll];
                let alpha = if focused {
                    1.0
                } else {
                    fade(self.ticks - line.added)
                };
                (alpha > 1.0e-5).then_some((i, alpha))
            })
            .collect();
        for &(i, alpha) in &visible {
            let entry_bottom = chat_bottom - i as i32 * entry_height;
            let background = ((alpha * background_opacity * 255.0) as u32) << 24;
            frame.fill(
                atlas,
                (
                    -4,
                    entry_bottom - entry_height,
                    max_width + 4 + 4,
                    entry_bottom,
                ),
                background,
            );
        }
        let local = mouse.map(|mouse| pose_undone(scale, mouse));
        let over = |local: Option<(f32, f32)>, (x0, y0, x1, y1): (i32, i32, i32, i32)| {
            local.is_some_and(|(x, y)| {
                x >= x0 as f32 && x < x1 as f32 && y >= y0 as f32 && y < y1 as f32
            })
        };
        let mut tooltip = None;
        // The style with a hover event that the mouse is over, the last one
        // drawn (`hoveredTextStyle`).
        let mut hovered: Option<&Style> = None;
        for &(i, alpha) in &visible {
            let entry_bottom = chat_bottom - i as i32 * entry_height;
            let entry_top = entry_bottom - entry_height;
            let text_top = entry_bottom - entry_bottom_to_message_y;
            let opacity = alpha * text_opacity;
            let text = ((opacity * 255.0) as u32) << 24 | 0x00FF_FFFF;
            let line = &self.lines[i + self.scroll];
            frame.styled_text(atlas, font, &line.runs, 0, text_top, text, true);
            if let Some(point) = local
                && focused
                && let Some(style) =
                    crate::gui::text_hit::style_under(font, &line.runs, (0, text_top), point)
                && style.hover.is_some()
            {
                hovered = Some(style);
            }
            let Some(tag) = &line.tag else {
                continue;
            };
            let bar = ((opacity * 255.0) as u32) << 24 | tag.colour();
            frame.fill(atlas, (-4, entry_top, -2, entry_bottom), bar);
            if focused && over(local, (-4, entry_top, -2, entry_bottom)) {
                tooltip = Some(tag.tooltip(lang));
            }
            if tag.has_icon() && focused {
                // `handleTagIcon`: shown while the mouse is over it.
                let left = font.width_of_runs(&line.runs) + 4;
                let bottom_of_text = text_top + MESSAGE_HEIGHT;
                let top = bottom_of_text - 9 - 1;
                if over(local, (left, top, left + 9, bottom_of_text)) {
                    tooltip = Some(tag.tooltip(lang));
                    frame.sprite(atlas, "icon/chat_modified", left, top, 9, 9);
                }
            }
        }
        self.scroll_bar(
            frame,
            atlas,
            (chat_bottom, max_width),
            visible.len(),
            focused,
        );
        frame.scale_from(first, scale, (4.0 * scale, 0.0));
        let tag_tooltip = tooltip.map(|runs| split(&runs, 210, font));
        // `componentHoverEffect`: a tag's tooltip comes first, then a hover
        // event's text, split to half the screen (at least 200).
        tag_tooltip.or_else(|| hover_lines(hovered, lang, font, gui_width))
    }

    /// `ClickableStyleFinder`: the style a press at `mouse` lands on in an
    /// open chat, the last glyph's that has a click event (or, with
    /// `insertions`, an insertion).
    pub fn clicked_style(
        &self,
        font: &Font,
        (_, gui_height): (i32, i32),
        mouse: (i32, i32),
        insertions: bool,
    ) -> Option<Style> {
        let scale = self.settings.scale as f32;
        let chat_bottom = ((gui_height - BOTTOM_MARGIN) as f32 / scale).floor() as i32;
        let entry_height = self.entry_height();
        let spacing = self.settings.line_spacing;
        let entry_bottom_to_message_y =
            (8.0 * (spacing + 1.0) - 4.0 * spacing + 0.5).floor() as i32;
        let shown = self
            .lines
            .len()
            .saturating_sub(self.scroll)
            .min(self.lines_per_page(true));
        let point = pose_undone(scale, mouse);
        let mut found = None;
        for i in (0..shown).rev() {
            let text_top = chat_bottom - i as i32 * entry_height - entry_bottom_to_message_y;
            let line = &self.lines[i + self.scroll];
            if let Some(style) =
                crate::gui::text_hit::style_under(font, &line.runs, (0, text_top), point)
                && (style.click.is_some() || (insertions && style.insertion.is_some()))
            {
                found = Some(style.clone());
            }
        }
        found
    }

    /// The scroll bar of a chat that is open (`extractRenderState`'s last
    /// part): as tall as the lines shown are of all of them.
    fn scroll_bar(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (chat_bottom, max_width): (i32, i32),
        count: usize,
        focused: bool,
    ) {
        let total = self.lines.len() as i32;
        if total == 0 || !focused {
            return;
        }
        let chat_height = count as i32 * self.entry_height();
        let virtual_height = total * self.entry_height();
        if virtual_height == chat_height {
            return;
        }
        let y = self.scroll as i32 * chat_height / total - chat_bottom;
        let height = chat_height * chat_height / virtual_height;
        let alpha = if y > 0 { 170_u32 } else { 96 };
        let colour = if self.new_message {
            13_382_451
        } else {
            3_355_562
        };
        let start_x = max_width + 4;
        frame.fill(
            atlas,
            (start_x, -y, start_x + 2, -y - height),
            alpha << 24 | colour,
        );
        frame.fill(
            atlas,
            (start_x + 2, -y, start_x + 1, -y - height),
            alpha << 24 | 13_421_772,
        );
    }
}

/// A point of the screen in the chat's own units: the pose, which scales
/// and moves 4 right, undone (`Matrix3x2f.invert`, in single precision).
fn pose_undone(scale: f32, (x, y): (i32, i32)) -> (f32, f32) {
    let (m00, m20) = (scale, 4.0 * scale);
    let det = 1.0 / (m00 * m00);
    let inverse_x = m00 * det;
    let inverse_translation = (0.0 - m20 * m00) * det;
    (
        inverse_x * x as f32 + inverse_translation,
        inverse_x * y as f32,
    )
}

/// `componentHoverEffect` for a hover event's text: split to half the screen
/// (at least 200).
fn hover_lines(
    style: Option<&Style>,
    lang: &Lang,
    font: &Font,
    gui_width: i32,
) -> Option<Vec<Vec<Run>>> {
    match style?.hover.as_deref()? {
        HoverEvent::ShowText(text) => Some(split(
            &lang.runs(text, &Style::default()),
            (gui_width / 2).max(200),
            font,
        )),
        _ => None,
    }
}

/// What a key did to the chat box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatKey {
    None,
    /// Escape: the box is closed on purpose.
    Close,
    /// Enter with nothing to send: the box is done.
    Done,
    /// The line to send, and whether the screen closes (it always does).
    Send(String),
    /// Scroll the log a page, less a line: up for 1, down for -1.
    Page(i32),
}

/// `ChatComponent.ChatMethod`: what opened the box, a message or a command
/// (the slash key).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatMethod {
    Message,
    Command,
}

impl ChatMethod {
    /// What the box starts with when there is no draft.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Message => "",
            Self::Command => "/",
        }
    }
}

/// `ChatScreen.ExitReason`: why the box went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// Escape.
    Intentional,
    /// Another screen took its place (a death, say).
    Interrupted,
    /// A line was sent, or none was to send.
    Done,
}

/// `ChatComponent.latestDraft`: the text a closed box left, and what kind
/// of box it was.
#[derive(Debug, Clone, Default)]
pub struct ChatDrafts {
    latest: Option<(String, ChatMethod)>,
}

impl ChatDrafts {
    /// `createScreen`: the text a box of this method opens with, and whether
    /// it is a draft. A message box takes any draft; a command box only a
    /// command's.
    pub fn open(&self, method: ChatMethod) -> (String, bool) {
        match &self.latest {
            Some((text, kept)) if method == ChatMethod::Message || method == *kept => {
                (text.clone(), true)
            }
            _ => (method.prefix().to_owned(), false),
        }
    }

    /// `ChatScreen.removed`: what the box held, kept as the draft unless the
    /// way it went (or a blank text) discards it. A box opened on a draft
    /// and not edited leaves it as it was.
    pub fn removed(&mut self, text: &str, is_draft: bool, reason: ExitReason, save: bool) {
        let discard =
            reason != ExitReason::Interrupted && (reason != ExitReason::Intentional || !save);
        if discard || text.chars().all(char::is_whitespace) {
            self.latest = None;
        } else if !is_draft {
            let method = if text.starts_with('/') {
                ChatMethod::Command
            } else {
                ChatMethod::Message
            };
            self.latest = Some((text.to_owned(), method));
        }
    }

    /// The draft kept, and the method it came from.
    pub fn kept(&self) -> Option<(&str, ChatMethod)> {
        self.latest
            .as_ref()
            .map(|(text, method)| (text.as_str(), *method))
    }
}

/// `ChatScreen`: the box at the bottom of the screen and the history of what
/// was sent.
pub struct ChatScreen {
    edit: EditBox,
    font: Arc<Font>,
    size: (i32, i32),
    history: Vec<String>,
    history_pos: usize,
    history_buffer: String,
    focused_at: Instant,
    /// The key that opened the box also types its character: dropped.
    skip_char: bool,
    /// The list of matches the server sent for the text, when there is one.
    list: Option<SuggestionsList>,
    /// The list is for text since edited: it waits for the server's answer.
    stale: bool,
    /// The request to send: its id and text, and the id of the last sent.
    request: Option<(i32, String)>,
    request_id: i32,
    /// `keepSuggestions`: a match was just used, and that edit asks for nothing.
    applied: bool,
    /// The command tree, which suggests what continues a command itself.
    commands: Option<Arc<CommandTree>>,
    /// The names the suggestions draw on: online players, and teams.
    players: Vec<String>,
    teams: Vec<String>,
    /// The aimed entity's string UUID, if any (`getSelectedEntities`).
    selected_entities: Vec<String>,
    /// Whether the player may use entity selectors.
    selectors: bool,
    /// The text is a draft kept from before, not yet edited (`isDraft`).
    draft: bool,
    /// What the tree suggested for the text, while the server's answer on
    /// its own arguments is awaited.
    local: Vec<String>,
}

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
    fn edited(&mut self) {
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
            voxel_game::commands::sort_suggestions(&mut matches);
        }
        self.show(start, length, matches);
    }

    /// The list for matches, the text up to the cursor having been read.
    fn show(&mut self, start: usize, length: usize, matches: Vec<String>) {
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
    fn update_ghost(&mut self) {
        let ghost = self
            .list
            .as_ref()
            .filter(|_| !self.stale)
            .and_then(|list| list.ghost(&self.edit.value()));
        self.edit.set_suggestion(ghost);
    }

    /// `useSuggestion`: the box takes the match's text, the cursor after it.
    fn take_match(&mut self, (text, end): (String, usize)) {
        self.applied = true;
        let font = self.font.clone();
        self.edit.set_value(&text, &font);
        self.edit.set_cursor_position(end, &font);
        self.update_ghost();
    }

    /// `CommandSuggestions.keyPressed`: whether the list took the key.
    fn list_key(&mut self, key: ListKey) -> bool {
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
    fn move_in_history(&mut self, direction: i32) {
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

/// `ChatScreen.normalizeChatMessage`: trimmed, runs of blanks made one,
/// cut to 256 characters.
pub fn normalize(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    joined.chars().take(256).collect()
}

/// Styled runs of a plain string, for a line the client writes itself.
pub fn plain_runs(text: &str) -> Vec<Run> {
    vec![Run {
        text: text.to_owned(),
        style: Style::default(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_on_text_finds_its_click_event_or_its_insertion() {
        let Some(root) = std::env::var_os("VOXEL_ASSETS") else {
            return;
        };
        let Some(font) = Font::load(std::path::Path::new(&root)) else {
            return;
        };
        let component = Component::from_json(
            &voxel_core::json::parse(
                r#"{"text":"link","click_event":{"action":"open_url","url":"https://a.b"},"insertion":"ins"}"#,
            )
            .expect("json"),
        )
        .expect("component");
        let mut log = ChatLog::default();
        log.add_component(&component, &Lang::default(), (None, None), &font, true);
        let gui = (427, 240);
        // The line's text starts at x 4, 192 down.
        let clicked = log.clicked_style(&font, gui, (8, 196), false);
        assert!(matches!(
            clicked.as_ref().and_then(|style| style.click.as_deref()),
            Some(voxel_game::text::ClickEvent::OpenUrl(url)) if url == "https://a.b"
        ));
        assert_eq!(
            log.clicked_style(&font, gui, (8, 196), true)
                .and_then(|style| style.insertion),
            Some("ins".into())
        );
        // Off the text, nothing.
        assert!(log.clicked_style(&font, gui, (200, 196), false).is_none());
        assert!(log.clicked_style(&font, gui, (8, 100), false).is_none());
    }

    #[test]
    fn the_options_set_the_size_of_the_chat() {
        assert_eq!((width_of(1.0), width_of(0.0)), (320, 40));
        assert_eq!((height_of(1.0), height_of(0.0)), (180, 20));
        let log = ChatLog::default();
        // 180 / 9 lines open, and 90 / 9 closed by default.
        assert_eq!(
            (log.lines_per_page(true), log.lines_per_page(false)),
            (20, 10)
        );
        let spaced = ChatLog::with_settings(ChatSettings {
            line_spacing: 1.0,
            ..ChatSettings::default()
        });
        assert_eq!(spaced.lines_per_page(true), 10);
    }

    #[test]
    fn a_message_fades_between_180_and_200_ticks() {
        assert_eq!(fade(0), 1.0);
        assert_eq!(fade(180), 1.0);
        assert!((fade(190) - 0.25).abs() < 1.0e-6);
        assert_eq!(fade(200), 0.0);
    }

    #[test]
    fn a_newline_breaks_a_line_without_an_indent() {
        let lines = wrap(&plain_runs("ab\ncd"), 320, &Font::default());
        let texts: Vec<String> = lines
            .iter()
            .map(|line| line.iter().map(|run| run.text.as_str()).collect())
            .collect();
        // A newline breaks the line without indenting the next.
        assert_eq!(texts, ["ab", "cd"]);
    }

    #[test]
    fn the_list_stays_up_between_keys_until_the_answer_replaces_it() {
        let Some(font) = Font::load(std::path::Path::new(
            &std::env::var("VOXEL_ASSETS").unwrap_or_default(),
        )) else {
            return;
        };
        let mut screen =
            ChatScreen::new(Arc::new(font), (427, 240), "/g", Vec::new()).without_key();
        screen.suggested(0, 1, 1, vec!["gamemode".into(), "gamerule".into()]);
        assert!(screen.list.is_some());
        // A key asks again; the list does not blink out meanwhile.
        screen.char_typed('a');
        let (id, text) = screen.take_request().expect("a request");
        assert_eq!(text, "/ga");
        assert!(screen.list.is_some() && screen.stale);
        screen.suggested(id, 1, 2, vec!["gamemode".into(), "gamerule".into()]);
        assert!(!screen.stale);
        // Text that is no command has no list.
        for _ in 0..3 {
            screen.key_pressed(glfw::Key::Backspace, (false, false), &mut NoClip);
        }
        assert!(screen.list.is_none());
    }

    #[test]
    fn a_line_from_the_history_shows_no_list_until_the_next_edit() {
        let Some(font) = Font::load(std::path::Path::new(
            &std::env::var("VOXEL_ASSETS").unwrap_or_default(),
        )) else {
            return;
        };
        let mut screen = ChatScreen::new(
            Arc::new(font),
            (427, 240),
            "/g",
            vec!["/gamemode creative".into()],
        )
        .without_key();
        screen.suggested(0, 1, 1, vec!["gamemode".into()]);
        screen.suggested(screen.request_id, 1, 1, vec!["gamemode".into()]);
        screen.move_in_history(-1);
        assert!(screen.list.is_none() && screen.take_request().is_none());
        // Back to the line being typed asks again.
        screen.move_in_history(1);
        assert!(screen.take_request().is_some());
    }

    struct NoClip;

    impl Clipboard for NoClip {
        fn get(&mut self) -> String {
            String::new()
        }

        fn set(&mut self, _text: &str) {}
    }

    #[test]
    fn normalizing_trims_and_joins_blanks() {
        assert_eq!(normalize("  a   b \t c "), "a b c");
        assert_eq!(normalize("   "), "");
    }

    #[test]
    fn history_walks_back_and_returns_the_line_being_typed() {
        let font = Arc::new(Font::default());
        let mut screen =
            ChatScreen::new(font, (427, 240), "", vec!["a".into(), "b".into()]).without_key();
        screen.char_typed('x');
        screen.move_in_history(-1);
        assert_eq!(screen.edit.value(), "b");
        screen.move_in_history(-1);
        assert_eq!(screen.edit.value(), "a");
        screen.move_in_history(1);
        screen.move_in_history(1);
        assert_eq!(screen.edit.value(), "x");
    }
}
