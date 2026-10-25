//! What the three lists share (`AbstractSelectionList` as the statistics use it): rows of one
//! height in a row 280 wide, a scroll bar at the right of the row, and a wheel that scrolls by half
//! a row.

use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `getRowWidth` of every list here.
pub(super) const ROW_WIDTH: i32 = 280;
/// `AbstractSelectionList.Entry.CONTENT_PADDING`.
pub(super) const PADDING: i32 = 2;

/// Where a list is and how far it has scrolled.
pub(super) struct ListView {
    pub(super) place: Place,
    pub(super) scroll: f64,
    /// `defaultEntryHeight`, which is also twice the scroll rate.
    pub(super) row_height: i32,
    /// `getSelected`: the entry the last press chose, kept when the list loses the focus.
    pub(super) selected: Option<usize>,
    /// Whether the list has the focus: the screen's focus is on it.
    pub(super) focused: bool,
}

impl ListView {
    pub(super) fn new(row_height: i32) -> Self {
        Self {
            place: Place {
                y: 0,
                width: 0,
                height: 0,
                in_world: true,
            },
            scroll: 0.0,
            row_height,
            selected: None,
            focused: false,
        }
    }

    /// `updateSizeAndPosition`: the place, the selected entry shown, and the scroll held to what it
    /// can be there.
    pub(super) fn place(&mut self, place: Place, rows: usize) {
        self.place = place;
        if let Some(selected) = self.selected {
            self.scroll_to(selected);
        }
        self.scroll = self
            .scroll
            .clamp(0.0, f64::from(place.max_scroll(self.content_height(rows))));
    }

    /// `scrollToEntry`: the least scroll that shows the entry whole.
    pub(super) fn scroll_to(&mut self, index: usize) {
        let y = self.row_y(index);
        let top = y - self.place.y - 2;
        if top < 0 {
            self.scroll += f64::from(top);
        }
        let bottom = self.place.bottom() - y - self.row_height - 2;
        if bottom < 0 {
            self.scroll += f64::from(-bottom);
        }
    }

    /// `getEntryAtPosition`: the entry whose rectangle holds the point.
    pub(super) fn entry_at(&self, (x, y): (i32, i32), rows: usize) -> Option<usize> {
        (0..rows).find(|index| {
            let top = self.row_y(*index);
            x >= self.row_left() && x < self.row_right() && y >= top && y < top + self.row_height
        })
    }

    /// `setSelected` by a press: the entry is chosen, and shown if it is clipped.
    pub(super) fn select(&mut self, index: usize, rows: usize) {
        self.selected = Some(index);
        let y = self.row_y(index);
        let (content_top, content_bottom) = (y + PADDING, y + self.row_height - PADDING);
        if content_top < self.place.y || content_bottom > self.place.bottom() {
            self.scroll_to(index);
        }
        let max = f64::from(self.place.max_scroll(self.content_height(rows)));
        self.scroll = self.scroll.clamp(0.0, max);
    }

    /// The list takes the focus by Tab: the entry selected keeps it, else one at the end the
    /// search comes from is chosen. Whether there was an entry.
    pub(super) fn focus_entry(&mut self, forward: bool, rows: usize) -> bool {
        if rows == 0 {
            return false;
        }
        let index = match self.selected {
            Some(selected) => selected,
            None if forward => 0,
            None => rows - 1,
        };
        self.focused = true;
        self.select(index, rows);
        true
    }

    /// `AbstractSelectionList.extractSelection`: the outline of the entry, white while the list has
    /// the focus and grey after.
    pub(super) fn draw_selection(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, index: usize) {
        if self.selected != Some(index) {
            return;
        }
        let y = self.row_y(index);
        let colour = if self.focused {
            0xFFFF_FFFF
        } else {
            0xFF80_8080
        };
        let (left, right) = (self.row_left(), self.row_right());
        frame.fill(atlas, (left, y, right, y + self.row_height), colour);
        frame.fill(
            atlas,
            (left + 1, y + 1, right - 1, y + self.row_height - 1),
            0xFF00_0000,
        );
    }

    /// `AbstractSelectionList.contentHeight`.
    pub(super) fn content_height(&self, rows: usize) -> i32 {
        rows as i32 * self.row_height + 4
    }

    /// `AbstractScrollArea.mouseScrolled`.
    pub(super) fn scroll_by(&mut self, delta: f64, rows: usize) {
        let max = f64::from(self.place.max_scroll(self.content_height(rows)));
        self.scroll = (self.scroll - delta * f64::from(self.row_height / 2)).clamp(0.0, max);
    }

    pub(super) fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    pub(super) fn row_right(&self) -> i32 {
        self.row_left() + ROW_WIDTH
    }

    /// The `y` of a row.
    pub(super) fn row_y(&self, index: usize) -> i32 {
        self.place.y + 2 - self.scroll as i32 + index as i32 * self.row_height
    }

    /// `extractListItems`: whether a row at `y` shows.
    pub(super) fn shows(&self, y: i32) -> bool {
        y + self.row_height >= self.place.y && y <= self.place.bottom()
    }

    /// The scroll bar, after the rows.
    pub(super) fn draw_scrollbar(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, rows: usize) {
        self.place.draw_scrollbar(
            frame,
            atlas,
            self.row_right() + SCROLLBAR_WIDTH + 2,
            self.scroll,
            self.content_height(rows),
        );
    }
}
