//! `TextInputHandler`'s single-line box: an edit box under its label.

use std::time::Instant;

use photon_game::dialog::TextInput;
use photon_game::text::{Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::widgets::edit_box::{Clipboard, EditBox, EditKey};
use crate::widgets::navigation::Rectangle;
use crate::{FontRuns, FrameRuns};

/// `CommonLayouts.labeledElement`: the label's height and the space under it.
const LABEL_HEIGHT: i32 = 9;
const LABEL_SPACING: i32 = 4;
/// The box's height and its border either side.
const BOX_HEIGHT: i32 = 20;
const BORDER: i32 = 4;

pub(super) struct Text {
    edit: EditBox,
    label: Option<Vec<Run>>,
    width: i32,
    /// When the box took the focus, which its cursor blinks from.
    focused_at: Instant,
}

impl Text {
    /// The box and its label, and the size of the two together.
    pub(super) fn new(font: &Font, lang: &Lang, spec: &TextInput) -> (Self, (i32, i32)) {
        let mut edit = EditBox::new(spec.width - 2 * BORDER, spec.max_length as usize);
        edit.set_value(&spec.initial, font);
        let label = spec
            .label_visible
            .then(|| lang.runs(&spec.label, &Style::default()));
        let size = match &label {
            Some(runs) => (
                spec.width.max(font.width_of_runs(runs)),
                LABEL_HEIGHT + LABEL_SPACING + BOX_HEIGHT,
            ),
            None => (spec.width, BOX_HEIGHT),
        };
        let text = Self {
            edit,
            label,
            width: spec.width,
            focused_at: Instant::now(),
        };
        (text, size)
    }

    /// Where the box is inside the control's rectangle: under the label, at the left.
    pub(super) fn box_rect(&self, rect: Rectangle) -> Rectangle {
        let dy = if self.label.is_some() {
            LABEL_HEIGHT + LABEL_SPACING
        } else {
            0
        };
        Rectangle {
            x: rect.x,
            y: rect.y + dy,
            width: self.width,
            height: BOX_HEIGHT,
        }
    }

    pub(super) fn value(&self) -> String {
        self.edit.value()
    }

    pub(super) fn focus(&mut self, gained: bool) {
        if gained {
            self.focused_at = Instant::now();
        }
    }

    /// `onClick`: the cursor goes where the mouse is.
    pub(super) fn clicked(&mut self, x: i32, rect: Rectangle, font: &Font) {
        let at = self.box_rect(rect);
        self.edit.click_at(x - at.x - BORDER, font);
    }

    /// `keyPressed` of the focused box. Whether it took the key.
    pub(super) fn key(
        &mut self,
        font: &Font,
        key: EditKey,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        self.edit.press_key(key, control, shift, font, clipboard)
    }

    pub(super) fn char_typed(&mut self, font: &Font, character: char) -> bool {
        self.edit.char_typed(character, font)
    }

    /// What a screen of another size had (`repositionElements` keeps the widgets).
    pub(super) fn copy_from(&mut self, before: &Self) {
        self.edit = before.edit.clone();
        self.focused_at = before.focused_at;
    }

    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        rect: Rectangle,
        focused: bool,
    ) {
        if let Some(label) = &self.label {
            frame.styled_text(atlas, font, label, rect.x, rect.y, 0xFFFF_FFFF, true);
        }
        let at = self.box_rect(rect);
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        frame.nine_slice(
            atlas,
            sprite,
            (200, 20),
            1,
            false,
            at.x,
            at.y,
            at.width,
            at.height,
        );
        self.edit.draw_formatted(
            frame,
            atlas,
            font,
            (at.x + BORDER, at.y + (BOX_HEIGHT - 8) / 2),
            focused.then_some(self.focused_at),
            None,
        );
    }
}
