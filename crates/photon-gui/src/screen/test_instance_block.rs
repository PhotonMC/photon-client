//! The test instance block's screen (`TestInstanceBlockEditScreen`): the test's identifier, the
//! size of its structure, the rotation, whether entities are included, and the actions that go
//! to the server.

mod draw;
mod info;
mod input;

use std::sync::Arc;
use std::time::Instant;

use photon_core::identifier::Identifier;
use photon_game::structure_block::Rotation;
use photon_game::test_instance::{InstanceAction, InstanceRequest, TestInstanceContents};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::layout::Arena;
use crate::widgets::cycle::CycleButton;
use crate::widgets::edit_box::EditBox;
use crate::widgets::navigation::{Candidate, Rectangle};
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};

/// `-6250336`: the labels' grey.
const LABEL_GREY: u32 = 0xFFA0_A0A0;

/// The width of the screen's column.
const COLUMN: i32 = 316;

/// What the player's last action did to the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceOutcome {
    Stay,
    /// The screen closes (what it sent is in [`TestInstanceBlockScreen::take_sent`]).
    Close,
}

/// Every widget, in the order the screen added them (the order Tab goes through).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Id,
    Info,
    SizeX,
    SizeY,
    SizeZ,
    Rotation,
    Entities,
    Reset,
    Save,
    Run,
    Done,
    Cancel,
    /// No widget has the focus, as after opening with the mouse (`setInitialFocus` focuses
    /// only when the last input was the keyboard's).
    Nowhere,
}

const ORDER: [Slot; 12] = [
    Slot::Id,
    Slot::Info,
    Slot::SizeX,
    Slot::SizeY,
    Slot::SizeZ,
    Slot::Rotation,
    Slot::Entities,
    Slot::Reset,
    Slot::Save,
    Slot::Run,
    Slot::Done,
    Slot::Cancel,
];

/// `exactWidgetSize`.
fn exact_size(count: i32) -> f32 {
    (COLUMN - (count - 1) * 8) as f32 / count as f32
}

/// `widgetX`, from the left edge of the column.
fn widget_x(index: i32, count: i32) -> i32 {
    (index as f32 * (8.0 + exact_size(count))) as i32
}

/// Where each widget sits, as its column among those of a row (index, count), its top and its
/// height.
const LAYOUT: [(Slot, (i32, i32), i32, i32); 12] = [
    (Slot::Id, (0, 1), 40, 20),
    (Slot::Info, (0, 1), 70, 8 * 9),
    (Slot::SizeX, (0, 5), 160, 20),
    (Slot::SizeY, (1, 5), 160, 20),
    (Slot::SizeZ, (2, 5), 160, 20),
    (Slot::Rotation, (3, 5), 160, 20),
    (Slot::Entities, (4, 5), 160, 20),
    (Slot::Reset, (0, 2), 185, 20),
    (Slot::Save, (1, 2), 185, 20),
    (Slot::Run, (0, 3), 210, 20),
    (Slot::Done, (1, 3), 210, 20),
    (Slot::Cancel, (2, 3), 210, 20),
];

/// Where each widget sits: the offset of its left edge from the column's, its top and its size.
fn place(slot: Slot) -> (i32, i32, i32, i32) {
    LAYOUT.iter().find(|(each, ..)| *each == slot).map_or(
        (0, 0, 0, 0),
        |&(_, (index, count), y, height)| {
            (widget_x(index, count), y, exact_size(count) as i32, height)
        },
    )
}

/// The index of a slot's box among the four.
fn box_index(slot: Slot) -> Option<usize> {
    match slot {
        Slot::Id => Some(0),
        Slot::SizeX => Some(1),
        Slot::SizeY => Some(2),
        Slot::SizeZ => Some(3),
        _ => None,
    }
}

fn rotation_text(rotation: &Rotation) -> Component {
    Component::literal(match rotation {
        Rotation::None => "0",
        Rotation::Clockwise90 => "90",
        Rotation::Clockwise180 => "180",
        Rotation::Counterclockwise90 => "270",
    })
}

fn on_off_text(on: &bool) -> Component {
    Component::translatable(if *on { "options.on" } else { "options.off" })
}

fn button(slot: Slot, label: Vec<Run>) -> Widget {
    Widget {
        rect: rect_of(slot),
        active: true,
        kind: Kind::Button { label },
    }
}

fn rect_of(slot: Slot) -> usize {
    ORDER.iter().position(|each| *each == slot).unwrap_or(0)
}

/// `parseSize`: a number clamped to what a test may be, or one.
fn parse_size(value: &str) -> i32 {
    value.parse::<i32>().map_or(1, |size| size.clamp(1, 48))
}

/// The widgets' rectangles, in the order the screen added them.
fn arena_for(size: (i32, i32)) -> Arena {
    let left = size.0 / 2 - COLUMN / 2;
    let mut arena = Arena::default();
    for slot in ORDER {
        let (x, y, width, height) = place(slot);
        let rect = arena.add(width, height);
        arena.0[rect].x = left + x;
        arena.0[rect].y = y;
    }
    arena
}

/// The identifier's box and the three sizes', with what the block holds.
fn edit_boxes(font: &Font, contents: &TestInstanceContents) -> [EditBox; 4] {
    let box_of = |slot: Slot, length: usize, text: &str| {
        let mut edit = EditBox::new(place(slot).2 - 8, length);
        edit.set_value(text, font);
        edit
    };
    let (sx, sy, sz) = contents.size;
    [
        box_of(Slot::Id, 128, contents.test.as_deref().unwrap_or_default()),
        box_of(Slot::SizeX, 15, &sx.to_string()),
        box_of(Slot::SizeY, 15, &sy.to_string()),
        box_of(Slot::SizeZ, 15, &sz.to_string()),
    ]
}

pub struct TestInstanceBlockScreen {
    size: (i32, i32),
    font: Arc<Font>,
    /// What the block held when the screen was made, which a resize shows again.
    contents: TestInstanceContents,
    arena: Arena,
    /// The identifier's box, then the three sizes'.
    edits: [EditBox; 4],
    rotation: CycleButton<Rotation>,
    entities: CycleButton<bool>,
    reset: Widget,
    save: Widget,
    run: Widget,
    done: Widget,
    cancel: Widget,
    /// The words in the description area: the red note for an identifier that does not parse.
    info: Vec<Run>,
    /// How far the words are scrolled, and whether the bar is held.
    info_scroll: f64,
    info_scrolling: bool,
    focus: Slot,
    focused_at: Instant,
    /// Whether the last input was a key's (`InputType.isKeyboard`).
    keyboard: bool,
    sent: Vec<InstanceRequest>,
}

impl TestInstanceBlockScreen {
    /// `new TestInstanceBlockEditScreen` and `init`: the screen for a block with these contents,
    /// in a screen `size` big. It asks the server for the test's description at once.
    pub fn new(
        (font, lang): (&Arc<Font>, &Lang),
        contents: &TestInstanceContents,
        size: (i32, i32),
    ) -> Self {
        Self::made(font, lang, contents, size, (Vec::new(), false))
    }

    fn made(
        font: &Arc<Font>,
        lang: &Lang,
        contents: &TestInstanceContents,
        size: (i32, i32),
        (sent, keyboard): (Vec<InstanceRequest>, bool),
    ) -> Self {
        let arena = arena_for(size);
        let edits = edit_boxes(font, contents);
        let words = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let mut screen = Self {
            size,
            font: font.clone(),
            contents: contents.clone(),
            arena,
            edits,
            rotation: CycleButton::new(
                lang,
                rect_of(Slot::Rotation),
                Component::translatable("test_instance_block.rotation"),
                Rotation::ALL.to_vec(),
                rotation_text,
                &contents.rotation,
            )
            .only_value(lang),
            entities: CycleButton::new(
                lang,
                rect_of(Slot::Entities),
                Component::translatable("test_instance_block.entities"),
                vec![true, false],
                on_off_text,
                &!contents.ignore_entities,
            )
            .only_value(lang),
            reset: button(Slot::Reset, words("test_instance.action.reset")),
            save: button(Slot::Save, words("test_instance.action.save")),
            run: button(Slot::Run, words("test_instance.action.run")),
            done: button(Slot::Done, words("gui.done")),
            cancel: button(Slot::Cancel, words("gui.cancel")),
            info: Vec::new(),
            info_scroll: 0.0,
            info_scrolling: false,
            // `setInitialFocus` focuses the first widget after a key, and none after the mouse.
            focus: if keyboard { Slot::Id } else { Slot::Nowhere },
            focused_at: Instant::now(),
            keyboard,
            sent,
        };
        screen.update_test_info(lang, true);
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `resize`: the widgets are made again from what the block held, and the server is asked
    /// again (what was typed is not kept).
    pub fn resize(&mut self, lang: &Lang, size: (i32, i32)) {
        let sent = std::mem::take(&mut self.sent);
        let keyboard = self.keyboard;
        *self = Self::made(
            &self.font.clone(),
            lang,
            &self.contents.clone(),
            size,
            (sent, keyboard),
        );
    }

    /// What the screen sent since it was last asked, in order.
    pub fn take_sent(&mut self) -> Vec<InstanceRequest> {
        std::mem::take(&mut self.sent)
    }

    /// The identifier the box holds, if it parses (`Identifier.tryParse`).
    fn parsed_id(&self) -> Option<String> {
        Identifier::parse(&self.edits[0].value())
            .ok()
            .map(|id| id.as_string())
    }

    /// `sendToServer`: whether the identifier parsed.
    fn send(&mut self, action: InstanceAction) -> bool {
        let test = self.parsed_id();
        let size = |index: usize| parse_size(&self.edits[index].value());
        let request = InstanceRequest {
            action,
            size: (size(1), size(2), size(3)),
            rotation: *self.rotation.value(),
            ignore_entities: !*self.entities.value(),
            test,
        };
        let valid = request.test.is_some();
        self.sent.push(request);
        valid
    }

    /// `updateTestInfo`: ask the server and, for an identifier that does not parse, say so.
    fn update_test_info(&mut self, lang: &Lang, init: bool) {
        let valid = self.send(if init {
            InstanceAction::Init
        } else {
            InstanceAction::Query
        });
        if !valid {
            let red = Style {
                colour: Some(0xFF_5555),
                ..Style::default()
            };
            self.info = lang.runs(
                &Component::translatable("test_instance.description.invalid_id"),
                &red,
            );
        }
        self.update_save_state();
    }

    /// `updateSaveState`: a structure can be saved at no rotation and under a name that parses.
    fn update_save_state(&mut self) {
        self.save.active = *self.rotation.value() == Rotation::None && self.parsed_id().is_some();
    }

    fn rectangle(&self, slot: Slot) -> Rectangle {
        let rect = self.arena.0[rect_of(slot)];
        Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    /// What can take the focus, in the order Tab goes through it.
    fn candidates(&self) -> Vec<Candidate> {
        ORDER
            .iter()
            .map(|slot| Candidate {
                rectangle: self.rectangle(*slot),
                focusable: *slot != Slot::Save || self.save.active,
            })
            .collect()
    }
}
