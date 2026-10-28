//! What the scrolling lists share (`AbstractScrollArea`,
//! `AbstractSelectionList`): where the bar is and how tall, the background
//! that moves with the list, the lines above and below it, and what a drag
//! of the bar does.

use photon_render::gui::{GuiAtlas, GuiFrame};

/// `AbstractScrollArea.SCROLLBAR_WIDTH` and the least the bar is tall.
pub const SCROLLBAR_WIDTH: i32 = 6;
const SCROLLBAR_MIN_HEIGHT: i32 = 32;

/// A list's place: as wide as the screen, `y` down and `height` tall, in a
/// world or not.
#[derive(Debug, Clone, Copy)]
pub struct Place {
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub in_world: bool,
}

impl Place {
    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    /// Whether the point is inside the list.
    pub fn contains(&self, (x, y): (f64, f64)) -> bool {
        x >= 0.0
            && x < f64::from(self.width)
            && y >= f64::from(self.y)
            && y < f64::from(self.bottom())
    }

    /// `maxScrollAmount`.
    pub fn max_scroll(&self, content_height: i32) -> i32 {
        (content_height - self.height).max(0)
    }

    /// `scrollerHeight`.
    pub fn scroller_height(&self, content_height: i32) -> i32 {
        let share = ((self.height * self.height) as f32 / content_height as f32) as i32;
        share.max(SCROLLBAR_MIN_HEIGHT).min(self.height - 8)
    }

    /// `scrollBarY`.
    pub fn scrollbar_y(&self, scroll: f64, content_height: i32) -> i32 {
        let max = self.max_scroll(content_height);
        if max == 0 {
            self.y
        } else {
            self.y.max(
                scroll as i32 * (self.height - self.scroller_height(content_height)) / max + self.y,
            )
        }
    }

    /// `AbstractScrollArea.mouseDragged` with the bar held: the scroll the
    /// mouse at `y`, `dy` further down, takes the list to.
    pub fn dragged_scroll(&self, scroll: f64, y: f64, dy: f64, content_height: i32) -> f64 {
        let max = self.max_scroll(content_height);
        if y < f64::from(self.y) {
            0.0
        } else if y > f64::from(self.bottom()) {
            f64::from(max)
        } else {
            let bar = self.scroller_height(content_height);
            let scale = (f64::from(max.max(1)) / f64::from(self.height - bar)).max(1.0);
            scroll + dy * scale
        }
    }

    /// The name of a texture in a world or out of it.
    fn texture(&self, name: &str) -> String {
        if self.in_world {
            format!("inworld_{name}")
        } else {
            name.to_owned()
        }
    }

    /// `extractListBackground`: the menu list's texture, moving with the
    /// scroll.
    pub fn draw_background(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, scroll: f64) {
        frame.tile_texture_from(
            atlas,
            &self.texture("menu_list_background"),
            (32, 32),
            (self.width, self.bottom() + scroll as i32),
            (0, self.y, self.width, self.height),
        );
    }

    /// `extractListSeparators` and `extractScrollbar`: the lines above and
    /// below the list, and the bar at `bar_x` if the list is longer than it
    /// shows.
    pub fn draw_edges(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (scroll, content_height): (f64, i32),
        bar_x: i32,
    ) {
        frame.blit_texture(
            atlas,
            &self.texture("header_separator"),
            (32, 2),
            (0, 0),
            (0, self.y - 2, self.width, 2),
        );
        frame.blit_texture(
            atlas,
            &self.texture("footer_separator"),
            (32, 2),
            (0, 0),
            (0, self.bottom(), self.width, 2),
        );
        self.draw_scrollbar(frame, atlas, bar_x, scroll, content_height);
    }

    /// `extractScrollbar` of a list that can scroll: the bar's track and the bar, at `bar_x`.
    pub fn draw_scrollbar(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        bar_x: i32,
        scroll: f64,
        content_height: i32,
    ) {
        if self.max_scroll(content_height) > 0 {
            frame.nine_slice(
                atlas,
                "widget/scroller_background",
                (SCROLLBAR_WIDTH, 32),
                1,
                false,
                (bar_x, self.y, SCROLLBAR_WIDTH, self.height),
            );
            frame.nine_slice(
                atlas,
                "widget/scroller",
                (SCROLLBAR_WIDTH, 32),
                1,
                false,
                (
                    bar_x,
                    self.scrollbar_y(scroll, content_height),
                    SCROLLBAR_WIDTH,
                    self.scroller_height(content_height),
                ),
            );
        }
    }
}
