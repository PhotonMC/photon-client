//! The command block's screen (`CommandBlockEditScreen`, over `AbstractCommandBlockEditScreen`):
//! the command in a box that suggests, the output of its last run, the buttons that set whether
//! the output is tracked, the mode, whether it is conditional and how it is triggered, and Done
//! and Cancel.
//!
//! The screen opens with the defaults of a block it knows nothing of and its buttons asleep; the
//! block's own values arrive with the server's data ([`CommandBlockScreen::update_gui`]).

mod buttons;
mod input;
#[cfg(test)]
mod tests;

use buttons::{Buttons, Extra, Values};

use std::sync::Arc;
use std::time::Instant;

use crate::{FontRuns, FrameRuns};
use photon_game::command_block::CommandBlockContents;
use photon_game::commands::CommandTree;
use photon_game::text::{Component, Lang, Style};
use photon_world::CommandBlockMode;

use crate::chat::command_suggestions::{CommandSuggestions, Place, SuggestionSettings};
use crate::layout::Arena;
use crate::widgets::cycle::CycleButton;
use crate::widgets::edit_box::EditBox;
use crate::widgets::navigation::{Candidate, Rectangle};
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `-6250336`: the labels' grey.
const LABEL_GREY: u32 = 0xFFA0_A0A0;
/// `setMaxLength`.
const MAX_LENGTH: usize = 32500;

/// What can have the focus, in the order Tab goes through it (the order the widgets were added).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Command,
    Previous,
    Output,
    Mode,
    Conditional,
    Automatic,
    Done,
    Cancel,
}

const BLOCK_ORDER: [Focus; 8] = [
    Focus::Command,
    Focus::Previous,
    Focus::Output,
    Focus::Mode,
    Focus::Conditional,
    Focus::Automatic,
    Focus::Done,
    Focus::Cancel,
];

const MINECART_ORDER: [Focus; 5] = [
    Focus::Command,
    Focus::Previous,
    Focus::Output,
    Focus::Done,
    Focus::Cancel,
];

/// Which screen this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBlockKind {
    /// `CommandBlockEditScreen`.
    Block,
    /// `MinecartCommandBlockEditScreen`.
    Minecart,
}

impl CommandBlockKind {
    fn order(self) -> &'static [Focus] {
        match self {
            Self::Block => &BLOCK_ORDER,
            Self::Minecart => &MINECART_ORDER,
        }
    }

    /// `getPreviousY`.
    fn previous_y(self) -> i32 {
        match self {
            Self::Block => 135,
            Self::Minecart => 150,
        }
    }
}

/// What the player's last action did to the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBlockAction {
    Stay,
    /// Cancel or Escape: the screen closes and nothing is sent.
    Close,
    /// Done or Enter: what to send, and the screen closes.
    Done(CommandBlockEdit),
}

/// What `populateAndSendPacket` sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockEdit {
    pub command: String,
    pub track_output: bool,
    /// The block's own settings; a minecart has none.
    pub block: Option<BlockSettings>,
}

/// What a command block's three extra buttons hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSettings {
    pub mode: CommandBlockMode,
    pub conditional: bool,
    pub automatic: bool,
}

pub struct CommandBlockScreen {
    kind: CommandBlockKind,
    size: (i32, i32),
    command: EditBox,
    previous: EditBox,
    suggest: CommandSuggestions,
    buttons: Buttons,
    /// The block's last output, as the client's block entity has it (empty when none).
    last_output: String,
    focus: Focus,
    focused_at: Instant,
    font: Arc<Font>,
}

impl CommandBlockScreen {
    /// `new CommandBlockEditScreen` and `init`: the screen for a block whose client-side contents
    /// are `contents`, in a screen `size` big. Its buttons sleep until the data arrives.
    pub fn new(
        fonts: (&Arc<Font>, &Lang),
        contents: &CommandBlockContents,
        size: (i32, i32),
    ) -> Self {
        Self::open(CommandBlockKind::Block, fonts, contents, size)
    }

    /// `new MinecartCommandBlockEditScreen` and `init`: the screen for a command block minecart
    /// whose client-side contents are `contents`. Nothing sleeps, and the box is filled at once.
    pub fn minecart(
        fonts: (&Arc<Font>, &Lang),
        contents: &CommandBlockContents,
        size: (i32, i32),
    ) -> Self {
        Self::open(CommandBlockKind::Minecart, fonts, contents, size)
    }

    fn open(
        kind: CommandBlockKind,
        (font, lang): (&Arc<Font>, &Lang),
        contents: &CommandBlockContents,
        size: (i32, i32),
    ) -> Self {
        let values = Values {
            track_output: contents.track_output,
            block: BlockSettings {
                mode: CommandBlockMode::Redstone,
                conditional: false,
                automatic: false,
            },
        };
        let last_output = contents
            .last_output
            .as_ref()
            .map(|output| lang.plain(output))
            .unwrap_or_default();
        let awake = kind == CommandBlockKind::Minecart;
        let mut screen = Self {
            kind,
            size,
            command: EditBox::new(292, MAX_LENGTH),
            previous: EditBox::new(268, MAX_LENGTH).uneditable(),
            suggest: CommandSuggestions::new(
                SuggestionSettings::COMMAND_BLOCK,
                font.clone(),
                place_of(size),
            ),
            buttons: Buttons::new(lang, size, (kind, values), awake),
            last_output,
            focus: Focus::Command,
            focused_at: Instant::now(),
            font: font.clone(),
        };
        screen.update_previous_output(values.track_output);
        screen.suggest.edited(&mut screen.command);
        if awake {
            screen.command.set_value(&contents.command, font);
            screen.suggest.edited(&mut screen.command);
        }
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `resize`: the widgets are made again for the new size from the values the screen holds, the
    /// typed text kept, and every control wakes.
    pub fn resize(&mut self, lang: &Lang, size: (i32, i32)) {
        let values = self.values();
        let text = self.command.value();
        self.size = size;
        self.command = EditBox::new(292, MAX_LENGTH);
        self.suggest.set_place(place_of(size));
        self.buttons = Buttons::new(lang, size, (self.kind, values), true);
        self.focus = Focus::Command;
        self.focused_at = Instant::now();
        self.command.set_value(&text, &self.font);
        self.update_previous_output(values.track_output);
        self.suggest.edited(&mut self.command);
    }

    fn values(&self) -> Values {
        let buttons = &self.buttons;
        let block = buttons.extra.as_ref().map_or(
            BlockSettings {
                mode: CommandBlockMode::Redstone,
                conditional: false,
                automatic: false,
            },
            |extra| BlockSettings {
                mode: *extra.mode.value(),
                conditional: *extra.conditional.value(),
                automatic: *extra.automatic.value(),
            },
        );
        Values {
            track_output: *buttons.output.value(),
            block,
        }
    }

    /// Whether the output is tracked, as the button holds it (the client's copy of the block's
    /// `trackOutput`, which the button writes at once).
    pub fn track_output(&self) -> bool {
        *self.buttons.output.value()
    }

    /// `updateGui`: the block's data arrived. The box and the buttons take it, and the controls
    /// wake.
    pub fn update_gui(
        &mut self,
        lang: &Lang,
        (mode, conditional): (CommandBlockMode, bool),
        contents: &CommandBlockContents,
    ) {
        self.command.set_value(&contents.command, &self.font);
        self.suggest.edited(&mut self.command);
        self.last_output = contents
            .last_output
            .as_ref()
            .map(|output| lang.plain(output))
            .unwrap_or_default();
        let buttons = &mut self.buttons;
        buttons.output.set_value(lang, contents.track_output);
        if let Some(extra) = &mut buttons.extra {
            extra.mode.set_value(lang, mode);
            extra.conditional.set_value(lang, conditional);
            extra.automatic.set_value(lang, contents.automatic);
        }
        self.update_previous_output(contents.track_output);
        self.buttons.enable(true);
    }

    /// `updatePreviousOutput`: the last output while it is tracked, a dash when it is not.
    fn update_previous_output(&mut self, tracking: bool) {
        let text = if tracking { &self.last_output } else { "-" };
        self.previous.set_value(text, &self.font);
    }

    /// What Done sends (`populateAndSendPacket`).
    fn edit(&self) -> CommandBlockEdit {
        let values = self.values();
        CommandBlockEdit {
            command: self.command.value(),
            track_output: values.track_output,
            block: (self.kind == CommandBlockKind::Block).then_some(values.block),
        }
    }

    /// The command tree and names the suggestions draw on.
    pub fn set_suggestion_sources(&mut self, sources: SuggestionSources) {
        // The reference has the dispatcher when the box is filled; here it is handed over after
        // the screen is made, so the text it already holds is read again.
        let first = !self.suggest.has_commands() && sources.commands.is_some();
        self.suggest.set_commands(sources.commands);
        self.suggest.set_selectors(sources.selectors);
        self.suggest.set_names(sources.players, sources.teams);
        self.suggest
            .set_selected_entities(sources.selected_entities);
        if first {
            self.suggest.edited(&mut self.command);
        }
    }

    /// The request the last edit asked the server for, taken.
    pub fn take_request(&mut self) -> Option<(i32, String)> {
        self.suggest.take_request()
    }

    /// The server's answer to a request.
    pub fn suggested(&mut self, id: i32, start: usize, length: usize, matches: Vec<String>) {
        self.suggest
            .suggested(&mut self.command, (id, start, length), matches);
    }

    /// The last output's label and box, when there is an output to show.
    fn draw_previous(&self, (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font), lang: &Lang) {
        if self.previous.is_empty() {
            return;
        }
        let (width, _) = self.size;
        let previous_y = self.kind.previous_y();
        let label = lang.runs(
            &Component::translatable("advMode.previousOutput"),
            &Style::default(),
        );
        frame.styled_text(
            atlas,
            font,
            &label,
            width / 2 - 149,
            previous_y - 10,
            LABEL_GREY,
            true,
        );
        self.draw_box(
            (frame, atlas, font),
            (width / 2 - 150, previous_y, 276),
            (&self.previous, self.focus == Focus::Previous),
            None,
        );
    }

    /// `extractRenderState`: the buttons over the dimmed game, the title and the labels, the boxes
    /// (the last output's only when it has something to show), then the suggestions.
    pub fn draw(
        &mut self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
        (commands, lang): (Option<&CommandTree>, &Lang),
    ) {
        let (width, height) = self.size;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        self.draw_buttons(frame, atlas, font, mouse);
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let title = runs("advMode.setCommand");
        let title_width = font.width_of_runs(&title);
        frame.styled_text(
            atlas,
            font,
            &title,
            width / 2 - title_width / 2,
            20,
            0xFFFF_FFFF,
            true,
        );
        frame.styled_text(
            atlas,
            font,
            &runs("advMode.command"),
            width / 2 - 149,
            40,
            LABEL_GREY,
            true,
        );
        let text = self.command.value();
        let parsed = self.suggest.parse(&text, commands.map(|tree| (tree, lang)));
        let format = |part: &str, offset: usize| parsed.as_ref().map(|p| p.format(part, offset));
        self.draw_box(
            (frame, atlas, font),
            (width / 2 - 150, 50, 300),
            (&self.command, self.focus == Focus::Command),
            Some(&format),
        );
        self.draw_previous((frame, atlas, font), lang);
        self.suggest
            .draw(frame, atlas, mouse, (&mut self.command, parsed.as_ref()));
    }

    fn draw_buttons(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let buttons = &self.buttons;
        let arena = &buttons.arena;
        buttons.output.draw(frame, atlas, font, arena, mouse);
        if let Some(extra) = &buttons.extra {
            extra.mode.draw(frame, atlas, font, arena, mouse);
            extra.conditional.draw(frame, atlas, font, arena, mouse);
            extra.automatic.draw(frame, atlas, font, arena, mouse);
        }
        for (widget, focus) in [
            (&buttons.done, Focus::Done),
            (&buttons.cancel, Focus::Cancel),
        ] {
            let highlighted = widget.contains(arena, mouse) || self.focus == focus;
            widget.draw(frame, atlas, font, arena, highlighted);
        }
    }

    /// `EditBox.extractWidgetRenderState` with its border: the sprite, the text and the cursor.
    fn draw_box(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        (x, y, width): (i32, i32, i32),
        (edit, focused): (&EditBox, bool),
        format: Option<&crate::widgets::edit_box::Formatter<'_>>,
    ) {
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, x, y, width, 20);
        edit.draw_formatted(
            frame,
            atlas,
            font,
            (x + 4, y + (20 - 8) / 2),
            focused.then_some(self.focused_at),
            format,
        );
    }

    /// What can take the focus, in the order Tab goes through it: the widgets' rectangles in the
    /// order they were added, and whether each can.
    fn candidates(&self) -> Vec<Candidate> {
        self.kind
            .order()
            .iter()
            .map(|focus| self.candidate_of(*focus))
            .collect()
    }

    fn candidate_of(&self, focus: Focus) -> Candidate {
        let buttons = &self.buttons;
        let width = self.size.0;
        let button = |rect: usize, active: bool| {
            let rect = buttons.arena.0[rect];
            Candidate {
                rectangle: Rectangle {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                },
                focusable: active,
            }
        };
        let boxed = |x: i32, y: i32, w: i32| Candidate {
            rectangle: Rectangle {
                x,
                y,
                width: w,
                height: 20,
            },
            focusable: true,
        };
        let cycle = |select: fn(&Extra) -> (usize, bool)| {
            let (rect, active) = buttons.extra.as_ref().map_or((0, false), select);
            button(rect, active)
        };
        match focus {
            Focus::Command => boxed(width / 2 - 150, 50, 300),
            Focus::Previous => boxed(width / 2 - 150, self.kind.previous_y(), 276),
            Focus::Output => button(buttons.output.rect(), buttons.output.is_active()),
            Focus::Mode => cycle(|extra| (extra.mode.rect(), extra.mode.is_active())),
            Focus::Conditional => {
                cycle(|extra| (extra.conditional.rect(), extra.conditional.is_active()))
            }
            Focus::Automatic => {
                cycle(|extra| (extra.automatic.rect(), extra.automatic.is_active()))
            }
            Focus::Done => button(buttons.done.rect, buttons.done.active),
            Focus::Cancel => button(buttons.cancel.rect, true),
        }
    }
}

/// What the suggestions draw on, as the session knows it.
#[derive(Default)]
pub struct SuggestionSources {
    pub commands: Option<Arc<CommandTree>>,
    pub selectors: bool,
    pub players: Vec<String>,
    pub teams: Vec<String>,
    pub selected_entities: Vec<String>,
}

/// Where the command box sits: 150 left of the middle, 50 down (`EditBox.getScreenX` counts from
/// the box's own edge, not from its text).
fn place_of(size: (i32, i32)) -> Place {
    Place {
        text_x: size.0 / 2 - 150,
        inner_width: 292,
        screen: size,
    }
}
