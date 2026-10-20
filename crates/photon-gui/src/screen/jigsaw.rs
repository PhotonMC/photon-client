//! The jigsaw block's screen (`JigsawBlockEditScreen`): the pool, name, target and final state of
//! the block, its priorities and joint, and the levels a generate goes down.

mod input;
mod layout;
mod levels;

use std::sync::Arc;
use std::time::Instant;

use photon_game::jigsaw::{JigsawContents, Joint, identifier_text, is_valid_identifier};
use photon_game::text::{Component, Lang, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::FrameRuns;
use crate::layout::Arena;
use crate::tooltip::{Holder, Pending};
use crate::widgets::cycle::CycleButton;
use crate::widgets::edit_box::EditBox;
use crate::widgets::navigation::{Candidate, Rectangle};
use crate::widgets::slider::InputType;
use crate::widgets::{Kind, Widget};
use layout::{BOXES, ITEMS, Item, ORDER, Slot, place};
use levels::LevelsSlider;

/// `-6250336`: the labels' grey.
const LABEL_GREY: u32 = 0xFFA0_A0A0;

/// What Done and Generate send first (`ServerboundSetJigsawBlockPacket`) but for the block's
/// position.
#[derive(Debug, Clone, PartialEq)]
pub struct JigsawEdit {
    pub name: String,
    pub target: String,
    pub pool: String,
    pub final_state: String,
    pub joint: Joint,
    pub selection_priority: i32,
    pub placement_priority: i32,
}

/// What a key or a click did to the screen.
#[derive(Debug, Clone, PartialEq)]
pub enum JigsawAction {
    Stay,
    /// Cancel or Escape: the screen closes and sends nothing.
    Cancel,
    /// Done: the screen closes and the block's settings are sent.
    Done(Box<JigsawEdit>),
    /// Generate: the settings are sent, then the levels and whether to keep the jigsaws.
    Generate(Box<JigsawEdit>, i32, bool),
}

pub struct JigsawScreen {
    size: (i32, i32),
    font: Arc<Font>,
    vertical: bool,
    arena: Arena,
    edits: Vec<EditBox>,
    joint: CycleButton<Joint>,
    slider: LevelsSlider,
    keep: CycleButton<bool>,
    generate: Widget,
    done: Widget,
    cancel: Widget,
    focus: Slot,
    focused_at: Instant,
    input: InputType,
    tips: [Holder; 2],
}

/// A box of the screen by its slot.
fn box_index(slot: Slot) -> Option<usize> {
    BOXES.iter().position(|each| *each == slot)
}

/// The most characters a box takes.
fn box_length(slot: Slot) -> usize {
    match slot {
        Slot::FinalState => 256,
        Slot::Selection | Slot::Placement => 3,
        _ => 128,
    }
}

/// The slot's place in the arena: its index in the order.
fn rect_of(slot: Slot) -> usize {
    ORDER.iter().position(|each| *each == slot).unwrap_or(0)
}

fn button(slot: Slot, label: Vec<photon_game::text::Run>) -> Widget {
    Widget {
        rect: rect_of(slot),
        active: true,
        kind: Kind::Button { label },
    }
}

fn joint_text(joint: &Joint) -> Component {
    Component::translatable(format!("jigsaw_block.joint.{}", joint.name()))
}

fn on_off_text(on: &bool) -> Component {
    Component::translatable(if *on { "options.on" } else { "options.off" })
}

/// The text a box starts with.
fn initial_text(slot: Slot, contents: &JigsawContents) -> String {
    match slot {
        Slot::Pool => contents.pool.clone(),
        Slot::Name => contents.name.clone(),
        Slot::Target => contents.target.clone(),
        Slot::FinalState => contents.final_state.clone(),
        Slot::Selection => contents.selection_priority.to_string(),
        _ => contents.placement_priority.to_string(),
    }
}

/// What the screen keeps over a `resize`: the values the game's own fields hold.
struct Kept {
    levels: i32,
    keep: bool,
}

/// The rectangles of the widgets, in the order of `ORDER`, for a screen `size` big.
fn widget_arena(size: (i32, i32)) -> Arena {
    let mut arena = Arena::default();
    for slot in ORDER {
        let at = place(slot);
        let rect = arena.add(at.width, at.height);
        arena.0[rect].x = size.0 / 2 + at.x;
        arena.0[rect].y = at.y;
    }
    arena
}

/// The boxes, holding the block's text.
fn new_edits(font: &Font, contents: &JigsawContents) -> Vec<EditBox> {
    BOXES
        .iter()
        .map(|slot| {
            let mut edit = EditBox::new(place(*slot).width - 8, box_length(*slot));
            edit.set_value(&initial_text(*slot, contents), font);
            edit
        })
        .collect()
}

impl JigsawScreen {
    /// `new JigsawBlockEditScreen` and `init`.
    pub fn new(
        (font, lang): (&Arc<Font>, &Lang),
        contents: &JigsawContents,
        size: (i32, i32),
    ) -> Self {
        Self::build(
            (font, lang),
            contents,
            size,
            &Kept {
                levels: 0,
                keep: true,
            },
        )
    }

    fn build(
        (font, lang): (&Arc<Font>, &Lang),
        contents: &JigsawContents,
        size: (i32, i32),
        kept: &Kept,
    ) -> Self {
        let arena = widget_arena(size);
        let edits = new_edits(font, contents);
        let words = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let mut joint = CycleButton::new(
            lang,
            rect_of(Slot::Joint),
            Component::translatable("jigsaw_block.joint_label"),
            Joint::ALL.to_vec(),
            joint_text,
            &contents.joint,
        )
        .only_value(lang);
        joint.set_active(contents.vertical);
        let tip = |key: &str| Holder::of(lang, &Component::translatable(key));
        Self {
            size,
            font: font.clone(),
            vertical: contents.vertical,
            arena,
            edits,
            joint,
            slider: LevelsSlider::new(lang, rect_of(Slot::Levels), kept.levels),
            keep: CycleButton::new(
                lang,
                rect_of(Slot::Keep),
                Component::translatable("jigsaw_block.keep_jigsaws"),
                vec![true, false],
                on_off_text,
                &kept.keep,
            ),
            generate: button(Slot::Generate, words("jigsaw_block.generate")),
            done: button(Slot::Done, words("gui.done")),
            cancel: button(Slot::Cancel, words("gui.cancel")),
            focus: Slot::Pool,
            focused_at: Instant::now(),
            input: InputType::Mouse,
            tips: [
                tip("jigsaw_block.placement_priority.tooltip"),
                tip("jigsaw_block.selection_priority.tooltip"),
            ],
        }
        .validated()
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The text of a box.
    fn text(&self, slot: Slot) -> String {
        self.edits[box_index(slot).unwrap_or(0)].value()
    }

    /// `updateValidity`: Done and Generate are active while the three identifiers parse.
    fn validated(mut self) -> Self {
        self.update_validity();
        self
    }

    fn update_validity(&mut self) {
        let valid = [Slot::Name, Slot::Target, Slot::Pool]
            .iter()
            .all(|slot| is_valid_identifier(&self.text(*slot)));
        self.done.active = valid;
        self.generate.active = valid;
    }

    /// `resize`: the widgets are made again from what the block holds, the typed text, the joint,
    /// the levels and keep switch put back, the slider's handle at the left, and the focus on the
    /// pool's box.
    pub fn resize(&mut self, lang: &Lang, size: (i32, i32)) {
        let texts: Vec<String> = self.edits.iter().map(EditBox::value).collect();
        let kept = Kept {
            levels: self.slider.levels(),
            keep: *self.keep.value(),
        };
        let contents = JigsawContents {
            name: String::new(),
            target: String::new(),
            pool: String::new(),
            final_state: String::new(),
            joint: *self.joint.value(),
            selection_priority: 0,
            placement_priority: 0,
            vertical: self.vertical,
        };
        let mut fresh = Self::build((&self.font.clone(), lang), &contents, size, &kept);
        for (edit, text) in fresh.edits.iter_mut().zip(&texts) {
            edit.set_value(text, &self.font);
        }
        fresh.input = self.input;
        fresh.update_validity();
        *self = fresh;
    }

    /// What `sendToServer` sends.
    fn current(&self) -> JigsawEdit {
        let number =
            |slot: Slot| crate::screen::structure_block::number::coordinate(&self.text(slot));
        JigsawEdit {
            name: identifier_text(&self.text(Slot::Name)),
            target: identifier_text(&self.text(Slot::Target)),
            pool: identifier_text(&self.text(Slot::Pool)),
            final_state: self.text(Slot::FinalState),
            joint: *self.joint.value(),
            selection_priority: number(Slot::Selection),
            placement_priority: number(Slot::Placement),
        }
    }

    /// The rectangle of a widget in the screen.
    fn rectangle(&self, slot: Slot) -> Rectangle {
        let at = place(slot);
        Rectangle {
            x: self.size.0 / 2 + at.x,
            y: at.y,
            width: at.width,
            height: at.height,
        }
    }

    /// Whether a widget can take the focus: it is shown and active.
    fn focusable(&self, slot: Slot) -> bool {
        match slot {
            Slot::Joint => self.vertical,
            Slot::Generate => self.generate.active,
            Slot::Done => self.done.active,
            _ => true,
        }
    }

    /// What can take the focus, in the order Tab goes through it.
    fn candidates(&self) -> Vec<Candidate> {
        ORDER
            .iter()
            .map(|slot| Candidate {
                rectangle: self.rectangle(*slot),
                focusable: self.focusable(*slot),
            })
            .collect()
    }

    /// `extractRenderState`: the dimmed game, the buttons, then the labels and boxes, then the
    /// tooltips of the priorities.
    pub fn draw(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
        lang: &Lang,
    ) {
        let (width, height) = self.size;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        self.draw_buttons((frame, atlas, font), mouse);
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        for item in &ITEMS {
            match item {
                Item::Label((key, x, y)) => {
                    if *key != "jigsaw_block.joint_label" || self.vertical {
                        frame.styled_text(
                            atlas,
                            font,
                            &runs(key),
                            width / 2 + x,
                            *y,
                            LABEL_GREY,
                            true,
                        );
                    }
                }
                Item::Box(slot) => self.draw_box((frame, atlas, font), *slot),
            }
        }
        self.draw_tooltips((frame, atlas, font), mouse);
    }

    /// The buttons, in the order they were added.
    fn draw_buttons(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let arena = &self.arena;
        if self.vertical {
            self.joint.draw(frame, atlas, font, arena, mouse);
        }
        self.slider.draw(frame, atlas, font, arena, mouse);
        self.keep.draw(frame, atlas, font, arena, mouse);
        for (slot, widget) in [
            (Slot::Generate, &self.generate),
            (Slot::Done, &self.done),
            (Slot::Cancel, &self.cancel),
        ] {
            let highlighted = widget.contains(arena, mouse) || self.focus == slot;
            widget.draw(frame, atlas, font, arena, highlighted);
        }
    }

    fn draw_box(&self, (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font), slot: Slot) {
        let Some(index) = box_index(slot) else {
            return;
        };
        let focused = self.focus == slot;
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        let at = self.rectangle(slot);
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, at.x, at.y, at.width, 20);
        self.edits[index].draw_formatted(
            frame,
            atlas,
            font,
            (at.x + 4, at.y + (20 - 8) / 2),
            focused.then_some(self.focused_at),
            None,
        );
    }

    /// The tooltips the two priority boxes ask for, as `setTooltipForNextFrame` keeps them: the
    /// first, unless a focused box asks later.
    fn draw_tooltips(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let mut chosen: Option<Pending> = None;
        for (slot, holder) in [
            (Slot::Placement, &self.tips[0]),
            (Slot::Selection, &self.tips[1]),
        ] {
            let rect = self.rectangle(slot);
            let hovered = mouse.0 >= rect.x
                && mouse.1 >= rect.y
                && mouse.0 < rect.x + rect.width
                && mouse.1 < rect.y + rect.height;
            let focused = self.focus == slot;
            let asked = holder.refresh(frame.millis, (hovered, focused), self.input, rect);
            if let Some(asked) = asked
                && (chosen.is_none() || focused)
            {
                chosen = Some(asked);
            }
        }
        if let Some(pending) = chosen {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }
}
