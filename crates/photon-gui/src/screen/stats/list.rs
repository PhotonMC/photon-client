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
        }
    }

    /// `updateSizeAndPosition`: the place, and the scroll held to what it can be there.
    pub(super) fn place(&mut self, place: Place, rows: usize) {
        self.place = place;
        self.scroll = self
            .scroll
            .clamp(0.0, f64::from(place.max_scroll(self.content_height(rows))));
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
