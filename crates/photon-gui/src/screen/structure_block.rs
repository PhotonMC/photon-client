//! The structure block's screen (`StructureBlockEditScreen`): the structure's name, where it
//! sits and how big it is (or how it is put down, or its custom data), by the block's mode, and
//! the buttons that save, load, detect, mirror and turn it.

mod buttons;
mod input;
mod layout;
pub(crate) mod number;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Instant;

use photon_game::structure_block::{
    Mirror, Rotation, StructureBlockContents, StructureMode, UpdateType,
};
use photon_game::text::{Component, Lang, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::layout::Arena;
use crate::widgets::edit_box::EditBox;
use crate::widgets::navigation::{Candidate, Rectangle};
use crate::{FontRuns, FrameRuns};
use buttons::{Buttons, CYCLES};
use layout::{BOXES, GROUPS, Item, ORDER, Side, Slot, place, shown};

/// `-6250336`: the labels' grey.
const LABEL_GREY: u32 = 0xFFA0_A0A0;

/// What the screen sends with Done, Save, Load and Detect (`ServerboundSetStructureBlockPacket`
/// but for the block's position).
#[derive(Debug, Clone, PartialEq)]
pub struct StructureBlockEdit {
    pub update: UpdateType,
    pub mode: StructureMode,
    pub name: String,
    pub offset: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub mirror: Mirror,
    pub rotation: Rotation,
    pub data: String,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    pub integrity: f32,
    pub seed: i64,
}

/// The values `onCancel` puts back: what the block had when the screen was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Initial {
    pub mirror: Mirror,
    pub rotation: Rotation,
    pub mode: StructureMode,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
}

/// What the player's last action did to the screen.
#[derive(Debug, Clone, PartialEq)]
pub enum StructureBlockAction {
    Stay,
    /// Cancel or Escape: the screen closes and the block takes its first values back.
    Cancel(Initial),
    /// Done, Save, Load or Detect: what to send, and the screen closes.
    Done(Box<StructureBlockEdit>),
}

pub struct StructureBlockScreen {
    size: (i32, i32),
    font: Arc<Font>,
    initial: Initial,
    /// The block's rotation, which the four turn buttons set.
    rotation: Rotation,
    buttons: Buttons,
    arena: Arena,
    edits: Vec<EditBox>,
    focus: Slot,
    focused_at: Instant,
    /// Whether `onCancel` gave the block its first mode back: the labels follow the block's mode,
    /// the buttons keep what they show.
    cancelled: std::cell::Cell<bool>,
}

/// A box of the screen by its slot.
fn box_index(slot: Slot) -> Option<usize> {
    BOXES.iter().position(|each| *each == slot)
}

/// The most characters a box takes, and the width of its text.
fn box_shape(slot: Slot) -> (usize, i32) {
    let width = place(slot).width - 8;
    let length = match slot {
        Slot::Name | Slot::Data => 128,
        Slot::Seed => 31,
        _ => 15,
    };
    (length, width)
}

impl StructureBlockScreen {
    /// `new StructureBlockEditScreen` and `init`: the screen for a block with these contents, in
    /// a screen `size` big.
    pub fn new(
        (font, lang): (&Arc<Font>, &Lang),
        contents: &StructureBlockContents,
        size: (i32, i32),
    ) -> Self {
        let (arena, buttons) = Buttons::new(lang, size, contents);
        let edits = BOXES
            .iter()
            .map(|slot| {
                let (length, width) = box_shape(*slot);
                let mut edit = EditBox::new(width, length);
                edit.set_value(&initial_text(*slot, contents), font);
                edit
            })
            .collect();
        Self {
            size,
            font: font.clone(),
            initial: Initial {
                mirror: contents.mirror,
                rotation: contents.rotation,
                mode: contents.mode,
                ignore_entities: contents.ignore_entities,
                strict: contents.strict,
                show_air: contents.show_air,
                show_bounding_box: contents.show_bounding_box,
            },
            rotation: contents.rotation,
            buttons,
            arena,
            edits,
            // `setInitialFocus` finds the name's box hidden in data mode and focuses nothing.
            focus: if contents.mode == StructureMode::Data {
                Slot::Nowhere
            } else {
                Slot::Name
            },
            focused_at: Instant::now(),
            cancelled: std::cell::Cell::new(false),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The mode the mode button holds.
    pub fn mode(&self) -> StructureMode {
        *self.buttons.mode.value()
    }

    /// `resize`: the widgets are made again for the new size from what the block now holds (the
    /// buttons wrote their values to it), the typed text kept, the first values taken anew, and
    /// the focus back on the name.
    pub fn resize(&mut self, lang: &Lang, size: (i32, i32)) {
        let contents = self.current(UpdateType::UpdateData);
        let texts: Vec<String> = self.edits.iter().map(EditBox::value).collect();
        let mut fresh = Self::new(
            (&self.font.clone(), lang),
            &StructureBlockContents {
                name: String::new(),
                metadata: String::new(),
                pos: (0, 0, 0),
                size: (0, 0, 0),
                mirror: contents.mirror,
                rotation: contents.rotation,
                mode: contents.mode,
                ignore_entities: contents.ignore_entities,
                strict: contents.strict,
                show_air: contents.show_air,
                show_bounding_box: contents.show_bounding_box,
                integrity: 1.0,
                seed: 0,
            },
            size,
        );
        for (edit, text) in fresh.edits.iter_mut().zip(&texts) {
            edit.set_value(text, &self.font);
        }
        *self = fresh;
    }

    /// What `sendToServer` sends for an update type.
    fn current(&self, update: UpdateType) -> StructureBlockEdit {
        let text = |slot: Slot| self.edits[box_index(slot).unwrap_or(0)].value();
        let triple = |slots: [Slot; 3]| {
            (
                number::coordinate(&text(slots[0])),
                number::coordinate(&text(slots[1])),
                number::coordinate(&text(slots[2])),
            )
        };
        let buttons = &self.buttons;
        StructureBlockEdit {
            update,
            mode: *buttons.mode.value(),
            name: text(Slot::Name),
            offset: triple([Slot::PosX, Slot::PosY, Slot::PosZ]),
            size: triple([Slot::SizeX, Slot::SizeY, Slot::SizeZ]),
            mirror: *buttons.mirror.value(),
            rotation: self.rotation,
            data: text(Slot::Data),
            ignore_entities: !*buttons.entities.value(),
            strict: *buttons.strict.value(),
            show_air: *buttons.air.value(),
            show_bounding_box: *buttons.bounding_box.value(),
            integrity: number::integrity(&text(Slot::Integrity)),
            seed: number::seed(&text(Slot::Seed)),
        }
    }

    /// The values the block holds while the screen is open: the buttons write them at once.
    pub fn held(&self) -> Initial {
        let edit = self.current(UpdateType::UpdateData);
        Initial {
            mirror: edit.mirror,
            rotation: edit.rotation,
            mode: edit.mode,
            ignore_entities: edit.ignore_entities,
            strict: edit.strict,
            show_air: edit.show_air,
            show_bounding_box: edit.show_bounding_box,
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

    /// Whether a widget is shown (`visible`).
    fn is_shown(&self, slot: Slot) -> bool {
        shown(slot, self.mode())
    }

    /// What can take the focus, in the order Tab goes through it: every widget's rectangle, and
    /// whether the widget is shown and active.
    fn candidates(&self) -> Vec<Candidate> {
        ORDER
            .iter()
            .map(|slot| Candidate {
                rectangle: self.rectangle(*slot),
                focusable: self.is_shown(*slot) && self.buttons.is_active(*slot),
            })
            .collect()
    }

    /// `extractRenderState`: the dimmed game, the buttons the mode shows, the title, then the
    /// labels and boxes group by group, and the mode's description.
    pub fn draw(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
        lang: &Lang,
    ) {
        let (width, height) = self.size;
        let middle = width / 2;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        self.draw_buttons((frame, atlas, font), mouse);
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let title = runs("block.minecraft.structure_block");
        let title_width = font.width_of_runs(&title);
        frame.styled_text(
            atlas,
            font,
            &title,
            (middle - title_width / 2, 10),
            0xFFFF_FFFF,
            true,
        );
        let mode = if self.cancelled.get() {
            self.initial.mode
        } else {
            self.mode()
        };
        for (applies, items) in GROUPS {
            if !applies(mode) {
                continue;
            }
            for item in items {
                match item {
                    Item::Label(key, side, y) => {
                        let label = runs(key);
                        let x = match side {
                            Side::Left(x) => middle + x,
                            Side::Right(x) => middle + x - font.width_of_runs(&label),
                        };
                        frame.styled_text(atlas, font, &label, (x, *y), LABEL_GREY, true);
                    }
                    Item::Box(slot) if shown(*slot, self.mode()) => {
                        self.draw_box((frame, atlas, font), *slot);
                    }
                    Item::Box(_) => {}
                }
            }
        }
        let info = runs(&format!("structure_block.mode_info.{}", mode.name()));
        frame.styled_text(atlas, font, &info, (middle - 153, 174), LABEL_GREY, true);
    }

    /// `EditBox.extractWidgetRenderState` with its border: the sprite, the text and the cursor.
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
        frame.nine_slice(
            atlas,
            sprite,
            (200, 20),
            1,
            false,
            (at.x, at.y, at.width, 20),
        );
        self.edits[index].draw_formatted(
            frame,
            atlas,
            font,
            (at.x + 4, at.y + (20 - 8) / 2),
            focused.then_some(self.focused_at),
            None,
        );
    }
}

/// The text a box starts with.
fn initial_text(slot: Slot, contents: &StructureBlockContents) -> String {
    match slot {
        Slot::Name => contents.name.clone(),
        Slot::PosX => contents.pos.0.to_string(),
        Slot::PosY => contents.pos.1.to_string(),
        Slot::PosZ => contents.pos.2.to_string(),
        Slot::SizeX => contents.size.0.to_string(),
        Slot::SizeY => contents.size.1.to_string(),
        Slot::SizeZ => contents.size.2.to_string(),
        Slot::Integrity => number::format_integrity(contents.integrity),
        Slot::Seed => contents.seed.to_string(),
        _ => contents.metadata.clone(),
    }
}
