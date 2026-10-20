//! The command block's screen (`CommandBlockEditScreen`, over `AbstractCommandBlockEditScreen`):
//! the command in a box that suggests, the output of its last run, the buttons that set whether
//! the output is tracked, the mode, whether it is conditional and how it is triggered, and Done
//! and Cancel.
//!
//! The screen opens with the defaults of a block it knows nothing of and its buttons asleep; the
//! block's own values arrive with the server's data ([`CommandBlockScreen::update_gui`]).

mod input;
#[cfg(test)]
mod tests;

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

const ORDER: [Focus; 8] = [
    Focus::Command,
    Focus::Previous,
    Focus::Output,
    Focus::Mode,
    Focus::Conditional,
    Focus::Automatic,
    Focus::Done,
    Focus::Cancel,
];

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
    pub mode: CommandBlockMode,
    pub track_output: bool,
    pub conditional: bool,
    pub automatic: bool,
}

/// The buttons, in the order they were added.
struct Buttons {
    arena: Arena,
    output: CycleButton<bool>,
    mode: CycleButton<CommandBlockMode>,
    conditional: CycleButton<bool>,
    automatic: CycleButton<bool>,
    done: Widget,
    cancel: Widget,
}

/// The values the buttons start from.
#[derive(Clone, Copy)]
struct Values {
    track_output: bool,
    mode: CommandBlockMode,
    conditional: bool,
    automatic: bool,
}

fn mode_text(mode: &CommandBlockMode) -> Component {
    Component::translatable(match mode {
        CommandBlockMode::Sequence => "advMode.mode.sequence",
        CommandBlockMode::Auto => "advMode.mode.auto",
        CommandBlockMode::Redstone => "advMode.mode.redstone",
    })
}

fn output_text(tracked: &bool) -> Component {
    Component::literal(if *tracked { "O" } else { "X" })
}

fn conditional_text(conditional: &bool) -> Component {
    Component::translatable(if *conditional {
        "advMode.mode.conditional"
    } else {
        "advMode.mode.unconditional"
    })
}

fn automatic_text(automatic: &bool) -> Component {
    Component::translatable(if *automatic {
        "advMode.mode.autoexec.bat"
    } else {
        "advMode.mode.redstoneTriggered"
    })
}

/// The rectangles of the six buttons, in the order output, mode, conditional, automatic, Done
/// and Cancel, for a screen `size` big.
fn arena_of((width, height): (i32, i32)) -> (Arena, [usize; 6]) {
    let mut arena = Arena::default();
    let mut place = |x: i32, y: i32, w: i32| {
        let rect = arena.add(w, 20);
        arena.0[rect].x = x;
        arena.0[rect].y = y;
        rect
    };
    let rects = [
        place(width / 2 + 150 - 20, 135, 20),
        place(width / 2 - 50 - 100 - 4, 165, 100),
        place(width / 2 - 50, 165, 100),
        place(width / 2 + 50 + 4, 165, 100),
        place(width / 2 - 4 - 150, height / 4 + 120 + 12, 150),
        place(width / 2 + 4, height / 4 + 120 + 12, 150),
    ];
    (arena, rects)
}

fn button(rect: usize, active: bool, label: Vec<photon_game::text::Run>) -> Widget {
    Widget {
        rect,
        active,
        kind: Kind::Button { label },
    }
}

impl Buttons {
    /// `init` and `addExtraControls`: the buttons for a screen `size` big.
    fn new(lang: &Lang, (width, height): (i32, i32), values: Values, active: bool) -> Self {
        let (arena, [output, mode, conditional, automatic, done, cancel]) =
            arena_of((width, height));
        let name = |key: &str| Component::translatable(key);
        let runs = |key: &str| lang.runs(&name(key), &Style::default());
        let mut buttons = Self {
            output: CycleButton::new(
                lang,
                output,
                name("advMode.trackOutput"),
                vec![true, false],
                output_text,
                &values.track_output,
            )
            .only_value(lang),
            mode: CycleButton::new(
                lang,
                mode,
                name("advMode.mode"),
                vec![
                    CommandBlockMode::Sequence,
                    CommandBlockMode::Auto,
                    CommandBlockMode::Redstone,
                ],
                mode_text,
                &values.mode,
            )
            .only_value(lang),
            conditional: CycleButton::new(
                lang,
                conditional,
                name("advMode.type"),
                vec![true, false],
                conditional_text,
                &values.conditional,
            )
            .only_value(lang),
            automatic: CycleButton::new(
                lang,
                automatic,
                name("advMode.triggering"),
                vec![true, false],
                automatic_text,
                &values.automatic,
            )
            .only_value(lang),
            done: button(done, active, runs("gui.done")),
            cancel: button(cancel, true, runs("gui.cancel")),
            arena,
        };
        buttons.enable(active);
        buttons
    }

    /// `enableControls`.
    fn enable(&mut self, active: bool) {
        self.done.active = active;
        self.output.set_active(active);
        self.mode.set_active(active);
        self.conditional.set_active(active);
        self.automatic.set_active(active);
    }
}

pub struct CommandBlockScreen {
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
        (font, lang): (&Arc<Font>, &Lang),
        contents: &CommandBlockContents,
        size: (i32, i32),
    ) -> Self {
        let values = Values {
            track_output: contents.track_output,
            mode: CommandBlockMode::Redstone,
            conditional: false,
            automatic: false,
        };
        let last_output = contents
            .last_output
            .as_ref()
            .map(|output| lang.plain(output))
            .unwrap_or_default();
        let mut screen = Self {
            size,
            command: EditBox::new(292, MAX_LENGTH),
            previous: EditBox::new(268, MAX_LENGTH).uneditable(),
            suggest: CommandSuggestions::new(
                SuggestionSettings::COMMAND_BLOCK,
                font.clone(),
                place_of(size),
            ),
            buttons: Buttons::new(lang, size, values, false),
            last_output,
            focus: Focus::Command,
            focused_at: Instant::now(),
            font: font.clone(),
        };
        screen.update_previous_output(values.track_output);
        screen.suggest.edited(&mut screen.command);
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
        self.buttons = Buttons::new(lang, size, values, true);
        self.focus = Focus::Command;
        self.focused_at = Instant::now();
        self.command.set_value(&text, &self.font);
        self.update_previous_output(values.track_output);
        self.suggest.edited(&mut self.command);
    }

    fn values(&self) -> Values {
        Values {
            track_output: *self.buttons.output.value(),
            mode: *self.buttons.mode.value(),
            conditional: *self.buttons.conditional.value(),
            automatic: *self.buttons.automatic.value(),
        }
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
        buttons.mode.set_value(lang, mode);
        buttons.conditional.set_value(lang, conditional);
        buttons.automatic.set_value(lang, contents.automatic);
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
            mode: values.mode,
            track_output: values.track_output,
            conditional: values.conditional,
            automatic: values.automatic,
        }
    }

    /// The command tree and names the suggestions draw on.
    pub fn set_suggestion_sources(&mut self, sources: SuggestionSources) {
        self.suggest.set_commands(sources.commands);
        self.suggest.set_selectors(sources.selectors);
        self.suggest.set_names(sources.players, sources.teams);
        self.suggest
            .set_selected_entities(sources.selected_entities);
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
        if !self.previous.is_empty() {
            let at = (width / 2 - 150, 135, 276);
            frame.styled_text(
                atlas,
                font,
                &runs("advMode.previousOutput"),
                at.0 + 1,
                125,
                LABEL_GREY,
                true,
            );
            self.draw_box(
                (frame, atlas, font),
                at,
                (&self.previous, self.focus == Focus::Previous),
                None,
            );
        }
        self.suggest
            .draw(frame, atlas, mouse, (&mut self.command, parsed.as_ref()));
    }

    fn draw_buttons(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let buttons = &self.buttons;
        let arena = &buttons.arena;
        buttons.output.draw(frame, atlas, font, arena, mouse);
        buttons.mode.draw(frame, atlas, font, arena, mouse);
        buttons.conditional.draw(frame, atlas, font, arena, mouse);
        buttons.automatic.draw(frame, atlas, font, arena, mouse);
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

    /// The widgets' rectangles in the order they were added, and whether each can take the focus.
    fn candidates(&self) -> Vec<Candidate> {
        let width = self.size.0;
        let button = |rect: usize, active: bool| {
            let rect = self.buttons.arena.0[rect];
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
        let buttons = &self.buttons;
        vec![
            boxed(width / 2 - 150, 50, 300),
            boxed(width / 2 - 150, 135, 276),
            button(buttons.output.rect(), buttons.output.is_active()),
            button(buttons.mode.rect(), buttons.mode.is_active()),
            button(buttons.conditional.rect(), buttons.conditional.is_active()),
            button(buttons.automatic.rect(), buttons.automatic.is_active()),
            button(buttons.done.rect, buttons.done.active),
            button(buttons.cancel.rect, true),
        ]
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
