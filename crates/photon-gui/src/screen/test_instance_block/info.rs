//! The description area (`FittingMultiLineTextWidget`): the words the server sent, scrolled by
//! the wheel, the arrow keys and a bar beside the frame.

use super::*;
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};

/// `defaultSettings(9)`: a line a step.
const SCROLL_RATE: f64 = 9.0;
/// `INNER_PADDING`.
const PADDING: i32 = 4;

impl TestInstanceBlockScreen {
    /// The area as a scroll list sees it: the frame's top and height.
    fn info_place(&self) -> Place {
        let at = self.rectangle(Slot::Info);
        Place {
            y: at.y,
            width: at.width,
            height: at.height,
            in_world: true,
        }
    }

    /// The words broken into lines that fit the frame.
    pub(super) fn info_lines(&self, font: &Font) -> Vec<Vec<Run>> {
        let at = self.rectangle(Slot::Info);
        crate::chat::split(&self.info, at.width - 2 * PADDING, font)
    }

    /// `contentHeight`: the lines and the padding round them.
    fn info_content_height(&self) -> i32 {
        9 * self.info_lines(&self.font).len() as i32 + 2 * PADDING
    }

    /// `setScrollAmount`: within what the words leave.
    fn set_info_scroll(&mut self, scroll: f64) {
        let max = self.info_place().max_scroll(self.info_content_height());
        self.info_scroll = scroll.clamp(0.0, f64::from(max));
    }

    /// `isMouseOver`: the frame and the bar beside it.
    pub(super) fn over_info(&self, (x, y): (i32, i32)) -> bool {
        let at = self.rectangle(Slot::Info);
        x >= at.x && y >= at.y && x < at.x + at.width + SCROLLBAR_WIDTH && y < at.y + at.height
    }

    /// `updateScrolling`: a press on the bar takes hold of it.
    pub(super) fn grab_info_bar(&mut self, (x, y): (i32, i32)) {
        let at = self.rectangle(Slot::Info);
        let scrollable = self.info_place().max_scroll(self.info_content_height()) > 0;
        self.info_scrolling = scrollable
            && x >= at.x + at.width
            && x <= at.x + at.width + SCROLLBAR_WIDTH
            && y >= at.y
            && y < at.y + at.height;
    }

    /// The wheel over the area.
    pub(super) fn scroll_info(&mut self, scroll_y: f64) {
        self.set_info_scroll(self.info_scroll - scroll_y * SCROLL_RATE);
    }

    /// Up and Down while the area has the focus: whether the words moved.
    pub(super) fn info_key(&mut self, key: glfw::Key) -> bool {
        let step = match key {
            glfw::Key::Up => -SCROLL_RATE,
            glfw::Key::Down => SCROLL_RATE,
            _ => return false,
        };
        let before = self.info_scroll;
        self.set_info_scroll(before + step);
        before != self.info_scroll
    }

    /// `mouseDragged` with the bar held: the mouse at `y` moved `dy` since the last event.
    pub fn dragged(&mut self, y: f64, dy: f64) {
        if self.focus != Slot::Info || !self.info_scrolling {
            return;
        }
        let content = self.info_content_height();
        let scroll = self
            .info_place()
            .dragged_scroll(self.info_scroll, y, dy, content);
        self.set_info_scroll(scroll);
    }

    /// `mouseReleased`: the bar is let go.
    pub fn released(&mut self) {
        self.info_scrolling = false;
    }

    /// `setStatus`: the words under the identifier become the error the last run left (red, after
    /// "Failed: ") and the status; a size that came with it fills the size boxes.
    pub fn set_status(
        &mut self,
        lang: &Lang,
        error: Option<&Component>,
        status: &Component,
        size: Option<(i32, i32, i32)>,
    ) {
        let mut description = Component::empty();
        if let Some(error) = error {
            let red = Component::empty().with_colour("red").append(error.clone());
            description = description
                .append(Component::translatable_with(
                    "test_instance.description.failed",
                    vec![red],
                ))
                .append(Component::literal("\n\n"));
        }
        description = description.append(status.clone());
        self.info = lang.runs(&description, &Style::default());
        if let Some((x, y, z)) = size {
            let font = self.font.clone();
            for (index, value) in [x, y, z].into_iter().enumerate() {
                self.edits[index + 1].set_value(&value.to_string(), &font);
            }
        }
    }

    /// The frame, then the words wrapped inside it and moved up by the scroll, and the bar.
    pub(super) fn draw_info(&self, (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font)) {
        let sprite = if self.focus == Slot::Info {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        let at = self.rectangle(Slot::Info);
        frame.nine_slice(
            atlas,
            sprite,
            (200, 20),
            1,
            false,
            (at.x, at.y, at.width, at.height),
        );
        let mark = frame.mark();
        for (row, line) in self.info_lines(font).iter().enumerate() {
            let y = at.y + PADDING + 9 * row as i32;
            frame.styled_text(atlas, font, line, (at.x + PADDING, y), 0xFFFF_FFFF, true);
        }
        if self.info_scroll != 0.0 {
            frame.transform_from(mark, [1.0, 0.0, 0.0, 1.0, 0.0, -(self.info_scroll as f32)]);
        }
        let clip = (
            at.x + 1,
            at.y + 1,
            at.x + at.width - 1,
            at.y + at.height - 1,
        );
        frame.clip_from(mark, clip);
        self.info_place().draw_scrollbar(
            frame,
            atlas,
            at.x + at.width,
            self.info_scroll,
            self.info_content_height(),
        );
    }
}
