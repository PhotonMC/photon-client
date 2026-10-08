//! `MultiLineEditBox` (an `AbstractTextAreaWidget`) as the book's page uses it: no border, no
//! character count, the text scrolled to keep the cursor in view.

use crate::gui::font::Font;
use crate::gui::widgets::edit_box::Clipboard;
use crate::gui::widgets::multiline::MultilineTextField;
use crate::gui::{GuiAtlas, GuiFrame};

/// `AbstractTextAreaWidget.INNER_PADDING`.
const PADDING: i32 = 4;
/// A line's height.
const LINE: i32 = 9;

/// `MultiLineEditBox` without its background and decorations.
#[derive(Debug, Clone)]
pub struct MultiLineEditBox {
    pub field: MultilineTextField,
    /// `x, y, width, height`.
    pub rect: (i32, i32, i32, i32),
    text_colour: u32,
    cursor_colour: u32,
    text_shadow: bool,
    scroll: f64,
    focused: bool,
    /// `focusedTime`, on the frame's clock: the first frame's when focused before one was drawn.
    focused_at: Option<u64>,
}

impl MultiLineEditBox {
    /// A box at `rect` in a text colour and a cursor colour, its text with a shadow or without.
    pub fn new(
        font: &Font,
        rect: (i32, i32, i32, i32),
        (text_colour, cursor_colour): (u32, u32),
        text_shadow: bool,
    ) -> Self {
        Self {
            field: MultilineTextField::new(font, rect.2 - 2 * PADDING),
            rect,
            text_colour,
            cursor_colour,
            text_shadow,
            scroll: 0.0,
            focused: false,
            focused_at: None,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// `setFocused`: focusing starts the cursor's blink over, at `now` or the next frame.
    pub fn set_focused(&mut self, focused: bool, now: Option<u64>) {
        if focused && !self.focused {
            self.focused_at = now;
        }
        self.focused = focused;
    }

    /// `setValue`, then the scroll that keeps the cursor in view.
    pub fn set_value(&mut self, font: &Font, value: &str, overflow: bool) {
        self.field.set_value(font, value, overflow);
        self.scroll_to_cursor();
    }

    /// `getInnerHeight` plus the padding (`contentHeight`).
    fn content_height(&self) -> i32 {
        LINE * self.field.lines().len() as i32 + 2 * PADDING
    }

    /// `setScrollAmount`: within what the content leaves.
    fn set_scroll(&mut self, scroll: f64) {
        let max = f64::from((self.content_height() - self.rect.3).max(0));
        self.scroll = scroll.clamp(0.0, max);
    }

    /// `scrollToCursor`: the cursor's line brought into view.
    fn scroll_to_cursor(&mut self) {
        let mut scroll = self.scroll;
        let cursor = self.field.cursor();
        let line = self.field.line_at_cursor().map_or(-1, |line| line as i32);
        let first = self.field.line_view((scroll / 9.0) as usize);
        if cursor <= first.0 {
            scroll = f64::from(line * LINE);
        } else {
            let last_index = ((scroll + f64::from(self.rect.3)) / 9.0) as i64 - 1;
            let last = self.field.line_view(last_index.max(0) as usize);
            if cursor > last.1 {
                scroll = f64::from(line * LINE - self.rect.3 + LINE + 2 * PADDING);
            }
        }
        self.set_scroll(scroll);
    }

    /// `keyPressed`: whether the key was taken.
    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        mods: (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        let taken = self.field.key_pressed(font, key, mods, clipboard);
        if taken {
            self.scroll_to_cursor();
        }
        taken
    }

    /// `charTyped`, while focused.
    pub fn char_typed(&mut self, font: &Font, character: char) -> bool {
        if !self.focused || character == '\u{a7}' || character < ' ' || character == '\u{7f}' {
            return false;
        }
        self.field.insert_text(font, &character.to_string());
        self.scroll_to_cursor();
        true
    }

    /// `AbstractWidget.isMouseOver` for a box with no scroll bar showing.
    pub fn contains(&self, (x, y): (f64, f64)) -> bool {
        let (left, top, width, height) = self.rect;
        x >= f64::from(left)
            && y >= f64::from(top)
            && x < f64::from(left + width)
            && y < f64::from(top + height)
    }

    /// `onClick`: a double click selects the word, a click puts the cursor under the mouse (Shift
    /// stretching the selection).
    pub fn clicked(&mut self, font: &Font, at: (f64, f64), shift: bool, double: bool) {
        if double {
            self.field.select_word_at_cursor();
        } else {
            self.field.set_selecting(shift);
            self.seek_to(font, at);
        }
        self.scroll_to_cursor();
    }

    /// `onDrag`: the selection stretched to the mouse.
    pub fn dragged(&mut self, font: &Font, at: (f64, f64), shift: bool) {
        self.field.set_selecting(true);
        self.seek_to(font, at);
        self.field.set_selecting(shift);
        self.scroll_to_cursor();
    }

    /// `seekCursorScreen`.
    fn seek_to(&mut self, font: &Font, (x, y): (f64, f64)) {
        let x = x - f64::from(self.rect.0) - f64::from(PADDING);
        let y = y - f64::from(self.rect.1) - f64::from(PADDING) + self.scroll;
        self.field.seek_cursor_to_point(font, (x, y));
    }

    /// `withinContentAreaTopBottom`.
    fn within(&self, top: i32, bottom: i32) -> bool {
        f64::from(bottom) - self.scroll >= f64::from(self.rect.1)
            && f64::from(top) - self.scroll <= f64::from(self.rect.1 + self.rect.3)
    }

    /// `extractWidgetRenderState` without background, scroll bar or decorations: the contents
    /// moved up by the scroll.
    pub fn draw(&mut self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
        if self.focused {
            self.focused_at.get_or_insert(frame.millis);
        }
        let mark = frame.mark();
        self.draw_contents(frame, atlas, font);
        if self.scroll != 0.0 {
            frame.transform_from(mark, [1.0, 0.0, 0.0, 1.0, 0.0, -(self.scroll as f32)]);
        }
        // `extractScrollbar`, right of the box.
        let (x, y, width, height) = self.rect;
        let place = crate::gui::widgets::scroll_area::Place {
            y,
            width,
            height,
            in_world: true,
        };
        place.draw_scrollbar(frame, atlas, x + width, self.scroll, self.content_height());
    }

    /// `extractContents`: the lines, the cursor while it blinks on, the selection over them. An
    /// empty box that is not focused shows its placeholder, which the page has none of.
    fn draw_contents(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
        let value = self.field.chars();
        if value.is_empty() && !self.focused {
            return;
        }
        let text = |range: &[char]| range.iter().collect::<String>();
        let colour = self.text_colour;
        let cursor = self.field.cursor();
        let show_cursor = self.focused
            && self
                .focused_at
                .is_some_and(|at| (frame.millis.saturating_sub(at) / 300).is_multiple_of(2));
        let insert = cursor < value.len();
        let (left, top) = (self.rect.0 + PADDING, self.rect.1 + PADDING);
        let (mut cursor_x, mut cursor_y) = (0, 0);
        let mut drawn_cursor = false;
        let mut draw_top = top;
        let put = |frame: &mut GuiFrame, line: &str, x: i32, y: i32| {
            if self.text_shadow {
                frame.text_with_shadow(atlas, font, line, x, y, colour);
            } else {
                frame.text(atlas, font, line, x, y, colour);
            }
        };
        for &(begin, end) in self.field.lines() {
            let visible = self.within(draw_top, draw_top + LINE);
            if !drawn_cursor && show_cursor && insert && cursor >= begin && cursor <= end {
                if visible {
                    let before = text(&value[begin..cursor]);
                    let right = left + font.width(&before);
                    put(frame, &before, left, draw_top);
                    put(frame, &text(&value[cursor..end]), right, draw_top);
                    (cursor_x, cursor_y) = (right, draw_top);
                    frame.fill(
                        atlas,
                        (cursor_x, cursor_y - 1, cursor_x + 1, cursor_y + LINE + 1),
                        self.cursor_colour,
                    );
                    drawn_cursor = true;
                }
            } else if visible {
                let line = text(&value[begin..end]);
                put(frame, &line, left, draw_top);
                if show_cursor && !insert {
                    (cursor_x, cursor_y) = (left + font.width(&line), draw_top);
                }
            }
            draw_top += LINE;
        }
        if show_cursor && !insert && self.within(cursor_y, cursor_y + LINE) {
            put(frame, "_", cursor_x, cursor_y);
        }
        if self.field.has_selection() {
            let (from, to) = self.field.selected();
            let mut draw_top = top;
            for &(begin, end) in self.field.lines() {
                if from > end {
                    draw_top += LINE;
                    continue;
                }
                if begin > to {
                    break;
                }
                if self.within(draw_top, draw_top + LINE) {
                    let start = font.width(&text(&value[begin..from.max(begin)]));
                    let stop = if to > end {
                        self.rect.2 - PADDING
                    } else {
                        font.width(&text(&value[begin..to]))
                    };
                    frame.text_highlight(
                        atlas,
                        (left + start, draw_top, left + stop, draw_top + LINE),
                    );
                }
                draw_top += LINE;
            }
        }
    }
}
