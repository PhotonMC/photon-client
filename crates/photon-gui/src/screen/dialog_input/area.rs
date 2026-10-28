//! `TextInputHandler`'s multi-line box: a `MultiLineEditBox` under its label, with the count of
//! characters beneath it.

use photon_game::dialog::{Multiline, TextInput};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::widgets::edit_box::Clipboard;
use crate::widgets::navigation::Rectangle;
use crate::widgets::text_area::MultiLineEditBox;
use crate::{FontRuns, FrameRuns};

/// `CommonLayouts.labeledElement`: the label's height and the space under it.
const LABEL_HEIGHT: i32 = 9;
const LABEL_SPACING: i32 = 4;
/// `MultiLineEditBox.Builder`'s text and cursor colours.
const TEXT: u32 = 0xFFE0_E0E0;
const CURSOR: u32 = 0xFFD0_D0D0;
/// The tallest box the codec allows.
const MAX_HEIGHT: i32 = 512;
/// A box with no height takes this many lines.
const DEFAULT_LINES: i32 = 4;

pub(super) struct Area {
    edit: MultiLineEditBox,
    label: Option<Vec<Run>>,
    lang: Lang,
    limit: usize,
    dragging: bool,
}

impl Area {
    /// The box and its label, and the size of the two together (the count hangs below it).
    pub(super) fn new(
        font: &Font,
        lang: &Lang,
        (spec, multiline): (&TextInput, &Multiline),
    ) -> (Self, (i32, i32)) {
        let height = multiline.height.unwrap_or_else(|| {
            (9 * multiline.max_lines.unwrap_or(DEFAULT_LINES) + 8).min(MAX_HEIGHT)
        });
        let mut edit =
            MultiLineEditBox::new(font, (0, 0, spec.width, height), (TEXT, CURSOR), true);
        let limit = spec.max_length as usize;
        edit.field.set_character_limit(limit);
        if let Some(lines) = multiline.max_lines {
            edit.field.set_line_limit(lines as usize);
        }
        edit.set_value(font, &spec.initial, false);
        let label = spec
            .label_visible
            .then(|| lang.runs(&spec.label, &Style::default()));
        let size = match &label {
            Some(runs) => (
                spec.width.max(font.width_of_runs(runs)),
                LABEL_HEIGHT + LABEL_SPACING + height,
            ),
            None => (spec.width, height),
        };
        let area = Self {
            edit,
            label,
            lang: lang.clone(),
            limit,
            dragging: false,
        };
        (area, size)
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
            width: self.edit.rect.2,
            height: self.edit.rect.3,
        }
    }

    pub(super) fn value(&self) -> String {
        self.edit.field.value()
    }

    pub(super) fn focus(&mut self, gained: bool) {
        self.edit.set_focused(gained, None);
    }

    /// `onClick`: the cursor goes where the mouse is, and a drag may follow.
    pub(super) fn clicked(
        &mut self,
        font: &Font,
        (x, y): (i32, i32),
        (rect, shift): (Rectangle, bool),
    ) {
        let at = self.box_rect(rect);
        self.edit.place(at.x, at.y);
        self.edit
            .clicked(font, (f64::from(x), f64::from(y)), shift, false);
        self.dragging = true;
    }

    /// `onDrag`: the selection stretched to the mouse.
    pub(super) fn dragged(
        &mut self,
        font: &Font,
        (x, y): (i32, i32),
        (rect, shift): (Rectangle, bool),
    ) {
        if self.dragging {
            let at = self.box_rect(rect);
            self.edit.place(at.x, at.y);
            self.edit.dragged(font, (f64::from(x), f64::from(y)), shift);
        }
    }

    pub(super) fn released(&mut self) {
        self.dragging = false;
    }

    pub(super) fn key(
        &mut self,
        font: &Font,
        key: glfw::Key,
        modifiers: (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        self.edit.key_pressed(font, key, modifiers, clipboard)
    }

    pub(super) fn char_typed(&mut self, font: &Font, character: char) -> bool {
        self.edit.char_typed(font, character)
    }

    /// What a screen of another size had (`repositionElements` keeps the widgets).
    pub(super) fn copy_from(&mut self, before: &Self) {
        self.edit = before.edit.clone();
    }

    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        rect: Rectangle,
    ) {
        if let Some(label) = &self.label {
            frame.styled_text(atlas, font, label, (rect.x, rect.y), 0xFFFF_FFFF, true);
        }
        let at = self.box_rect(rect);
        // `gui.multiLineEditBox.character_limit`, in UTF-16 units as `String.length` counts.
        let used = self.edit.field.value().encode_utf16().count();
        let count = self.lang.runs(
            &Component::translatable_with(
                "gui.multiLineEditBox.character_limit",
                vec![
                    Component::literal(used.to_string()),
                    Component::literal(self.limit.to_string()),
                ],
            ),
            &Style::default(),
        );
        self.edit.draw_framed(
            frame,
            (atlas, font),
            (at.x, at.y, at.width, at.height),
            Some(&count),
        );
    }
}
