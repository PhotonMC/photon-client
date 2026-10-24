//! The chat: its lines as `ChatComponent` keeps, wraps, fades and draws them
//! over the HUD, and the box a line is typed into (`ChatScreen`).
//!
//! The chat's width, height, scale, opacity, background opacity and line
//! spacing are the options' ([`ChatSettings`]). Not drawn: the prompt of a
//! chat that the options disable.
pub mod command_suggestions;
pub mod queue;
pub mod suggestions;

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use photon_game::text::{Component, HoverEvent, Lang, Run, Style};

use crate::chat::command_suggestions::{CommandSuggestions, Place, SuggestionSettings};
use crate::chat::suggestions::ListKey;
use crate::widgets::edit_box::{Clipboard, EditBox, edit_key};
use photon_game::chat_abilities::{ChatAbilities, MessageSource};
use photon_game::commands::CommandTree;
use photon_render::gui::GuiAtlas;
use photon_render::gui::GuiFrame;
use photon_render::gui::font::Font;

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
        Self::from(&photon_game::options::Options::default())
    }
}

impl From<&photon_game::options::Options> for ChatSettings {
    fn from(options: &photon_game::options::Options) -> Self {
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
    fn tooltip(&self, lang: &photon_game::text::Lang) -> Vec<Run> {
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
    origin: MessageSource,
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
    if runs.iter().all(|run| run.text.is_empty()) {
        return Vec::new();
    }
    break_lines(runs, max, font, false)
}

fn break_lines(runs: &[Run], max: i32, font: &Font, indent_soft: bool) -> Vec<Vec<Run>> {
    let lines = crate::splitter::split_lines(runs, max, font, &Style::default());
    let mut out: Vec<Vec<Run>> = lines
        .into_iter()
        .map(|(mut line, soft)| {
            if soft && indent_soft {
                line.insert(
                    0,
                    Run {
                        text: " ".into(),
                        style: Style::default(),
                    },
                );
            }
            line
        })
        .collect();
    if out.is_empty() && indent_soft {
        out.push(Vec::new());
    }
    out
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
    /// `ChatListener.queueSize`: the messages held back by the chat delay.
    queue_size: u64,
    /// Clicks on the queue line the game has not yet answered.
    queue_accepts: u32,
    /// F3+D pressed: the game flushes the queue and then clears the lines.
    clear_requested: bool,
    settings: ChatSettings,
    /// `chatColors` is off: the text of the lines loses its formatting codes.
    strip_formatting: bool,
    /// `visibleMessageFilter`: what the player may see of the chat.
    abilities: ChatAbilities,
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
    suggest: CommandSuggestions,
    /// The text is a draft kept from before, not yet edited (`isDraft`).
    draft: bool,
    /// `InBedChatScreen`: the leave-bed button, and its label.
    bed: Option<(crate::widgets::button_set::ButtonSet<()>, Vec<Run>)>,
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

mod screen;

mod formatting;
mod log;
mod queue_line;
mod restricted;
pub use queue_line::QUEUE_EXPAND_ID;
pub use restricted::GO_TO_RESTRICTIONS_ID;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_on_text_finds_its_click_event_or_its_insertion() {
        let Some(root) = std::env::var_os("PHOTON_ASSETS") else {
            return;
        };
        let Some(font) = Font::load(std::path::Path::new(&root)) else {
            return;
        };
        let component = Component::from_json(
            &photon_core::json::parse(
                r#"{"text":"link","click_event":{"action":"open_url","url":"https://a.b"},"insertion":"ins"}"#,
            )
            .expect("json"),
        )
        .expect("component");
        let mut log = ChatLog::default();
        log.add_component(
            &component,
            &Lang::default(),
            (MessageSource::SystemServer, None, None),
            &font,
            true,
        );
        let gui = (427, 240);
        // The line's text starts at x 4, 192 down.
        let clicked = log.clicked_style(&font, &Lang::default(), gui, (8, 196), false);
        assert!(matches!(
            clicked.as_ref().and_then(|style| style.click.as_deref()),
            Some(photon_game::text::ClickEvent::OpenUrl(url)) if url == "https://a.b"
        ));
        assert_eq!(
            log.clicked_style(&font, &Lang::default(), gui, (8, 196), true)
                .and_then(|style| style.insertion),
            Some("ins".into())
        );
        // Off the text, nothing.
        assert!(
            log.clicked_style(&font, &Lang::default(), gui, (200, 196), false)
                .is_none()
        );
        assert!(
            log.clicked_style(&font, &Lang::default(), gui, (8, 100), false)
                .is_none()
        );
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
            &std::env::var("PHOTON_ASSETS").unwrap_or_default(),
        )) else {
            return;
        };
        let mut screen =
            ChatScreen::new(Arc::new(font), (427, 240), "/g", Vec::new()).without_key();
        screen.suggested(0, 1, 1, vec!["gamemode".into(), "gamerule".into()]);
        assert!(screen.suggest.list.is_some());
        // A key asks again; the list does not blink out meanwhile.
        screen.char_typed('a');
        let (id, text) = screen.take_request().expect("a request");
        assert_eq!(text, "/ga");
        assert!(screen.suggest.list.is_some() && screen.suggest.stale);
        screen.suggested(id, 1, 2, vec!["gamemode".into(), "gamerule".into()]);
        assert!(!screen.suggest.stale);
        // Text that is no command has no list.
        for _ in 0..3 {
            screen.key_pressed(glfw::Key::Backspace, (false, false), &mut NoClip);
        }
        assert!(screen.suggest.list.is_none());
    }

    #[test]
    fn a_line_from_the_history_shows_no_list_until_the_next_edit() {
        let Some(font) = Font::load(std::path::Path::new(
            &std::env::var("PHOTON_ASSETS").unwrap_or_default(),
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
        screen.suggested(screen.suggest.request_id, 1, 1, vec!["gamemode".into()]);
        screen.move_in_history(-1);
        assert!(screen.suggest.list.is_none() && screen.take_request().is_none());
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
