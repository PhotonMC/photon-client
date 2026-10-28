//! The test block's screen (`TestBlockEditScreen`): the mode and, but for a start block, the
//! message the block logs.

mod input;

use std::sync::Arc;
use std::time::Instant;

use photon_game::test_block::{TestBlockContents, TestBlockMode};
use photon_game::text::{Component, Lang, Style};
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

/// The most characters the message's box takes.
const MESSAGE_LENGTH: usize = 128;

/// What Done sends (`ServerboundSetTestBlockPacket`) but for the block's position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestBlockEdit {
    pub mode: TestBlockMode,
    pub message: String,
}

/// What the player's last action did to the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestBlockAction {
    Stay,
    /// Cancel or Escape: the screen closes and sends nothing.
    Cancel,
    /// Done: the screen closes and the mode and message are sent.
    Done(TestBlockEdit),
}

/// Every widget, in the order the screen added them (the order Tab goes through).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Message,
    Mode,
    Done,
    Cancel,
    /// No widget has the focus, as when the message's box is hidden at the start.
    Nowhere,
}

const ORDER: [Slot; 4] = [Slot::Message, Slot::Mode, Slot::Done, Slot::Cancel];

/// Where each widget sits: the offset of its left edge from the middle, its top and its size.
fn place(slot: Slot) -> (i32, i32, i32, i32) {
    match slot {
        Slot::Message => (-152, 80, 240, 20),
        Slot::Mode => (-154, 185, 50, 20),
        Slot::Done => (-154, 210, 150, 20),
        _ => (4, 210, 150, 20),
    }
}

pub struct TestBlockScreen {
    size: (i32, i32),
    font: Arc<Font>,
    /// The message the block had when the screen was made, which a resize puts back in the box.
    message: String,
    arena: Arena,
    edit: EditBox,
    mode: CycleButton<TestBlockMode>,
    done: Widget,
    cancel: Widget,
    focus: Slot,
    focused_at: Instant,
}

fn mode_text(mode: &TestBlockMode) -> Component {
    Component::translatable(format!("test_block.mode.{}", mode.name()))
}

fn button(index: usize, label: Vec<photon_game::text::Run>) -> Widget {
    Widget {
        rect: index,
        active: true,
        kind: Kind::Button { label },
    }
}

impl TestBlockScreen {
    /// `new TestBlockEditScreen` and `init`: the screen for a block with these contents, in a
    /// screen `size` big.
    pub fn new(
        (font, lang): (&Arc<Font>, &Lang),
        contents: &TestBlockContents,
        size: (i32, i32),
    ) -> Self {
        let mut arena = Arena::default();
        for slot in ORDER {
            let (x, y, width, height) = place(slot);
            let rect = arena.add(width, height);
            arena.0[rect].x = size.0 / 2 + x;
            arena.0[rect].y = y;
        }
        let mut edit = EditBox::new(place(Slot::Message).2 - 8, MESSAGE_LENGTH);
        edit.set_value(&contents.message, font);
        let words = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let mode = CycleButton::new(
            lang,
            1,
            Component::translatable("block.minecraft.test_block"),
            TestBlockMode::ALL.to_vec(),
            mode_text,
            &contents.mode,
        )
        .only_value(lang);
        let mut screen = Self {
            size,
            font: font.clone(),
            message: contents.message.clone(),
            arena,
            edit,
            mode,
            done: button(2, words("gui.done")),
            cancel: button(3, words("gui.cancel")),
            // `setInitialFocus` finds the box hidden for a start block and focuses nothing.
            focus: Slot::Message,
            focused_at: Instant::now(),
        };
        if !screen.is_shown(Slot::Message) {
            screen.focus = Slot::Nowhere;
        }
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `resize`: the widgets are made again, with the mode the screen holds and the message the
    /// block had (what was typed is not kept), the focus on the box.
    pub fn resize(&mut self, lang: &Lang, size: (i32, i32)) {
        let contents = TestBlockContents {
            mode: *self.mode.value(),
            message: self.message.clone(),
        };
        *self = Self::new((&self.font.clone(), lang), &contents, size);
    }

    /// What Done sends: the mode, and the box's text whether the box is shown or not.
    fn current(&self) -> TestBlockEdit {
        TestBlockEdit {
            mode: *self.mode.value(),
            message: self.edit.value(),
        }
    }

    fn rectangle(&self, slot: Slot) -> Rectangle {
        let (x, y, width, height) = place(slot);
        Rectangle {
            x: self.size.0 / 2 + x,
            y,
            width,
            height,
        }
    }

    /// Whether a widget is shown: the box is hidden for a start block.
    fn is_shown(&self, slot: Slot) -> bool {
        match slot {
            Slot::Message => *self.mode.value() != TestBlockMode::Start,
            Slot::Nowhere => false,
            _ => true,
        }
    }

    /// What can take the focus, in the order Tab goes through it.
    fn candidates(&self) -> Vec<Candidate> {
        ORDER
            .iter()
            .map(|slot| Candidate {
                rectangle: self.rectangle(*slot),
                focusable: self.is_shown(*slot),
            })
            .collect()
    }

    /// `extractRenderState`: the dimmed game, the box and the buttons in the order they were
    /// added, the title, the message's label and the mode's description.
    pub fn draw(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
        lang: &Lang,
    ) {
        let (width, height) = self.size;
        let middle = width / 2;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        if self.is_shown(Slot::Message) {
            self.draw_box((frame, atlas, font));
        }
        self.mode.draw(frame, atlas, font, &self.arena, mouse);
        for (slot, widget) in [(Slot::Done, &self.done), (Slot::Cancel, &self.cancel)] {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == slot;
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let title = runs("block.minecraft.test_block");
        let title_width = font.width_of_runs(&title);
        frame.styled_text(
            atlas,
            font,
            &title,
            middle - title_width / 2,
            10,
            0xFFFF_FFFF,
            true,
        );
        let mode = *self.mode.value();
        if mode != TestBlockMode::Start {
            let label = runs("test_block.message");
            frame.styled_text(atlas, font, &label, middle - 153, 70, LABEL_GREY, true);
        }
        let info = runs(&format!("test_block.mode_info.{}", mode.name()));
        frame.styled_text(atlas, font, &info, middle - 153, 174, LABEL_GREY, true);
    }

    /// `EditBox.extractWidgetRenderState` with its border: the sprite, the text and the cursor.
    fn draw_box(&self, (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font)) {
        let focused = self.focus == Slot::Message;
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        let at = self.rectangle(Slot::Message);
        frame.nine_slice(
            atlas,
            sprite,
            (200, 20),
            1,
            false,
            (at.x, at.y, at.width, 20),
        );
        self.edit.draw_formatted(
            frame,
            atlas,
            font,
            (at.x + 4, at.y + (20 - 8) / 2),
            focused.then_some(self.focused_at),
            None,
        );
    }
}
