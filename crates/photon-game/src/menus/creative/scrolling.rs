use super::*;

impl CreativeMenu {
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    pub fn set_scroll(&mut self, scroll: f32) {
        self.scroll = scroll;
        self.scroll_to(scroll);
    }

    /// The stack in a window slot.
    pub fn window_stack(&self, slot: usize) -> Option<&Stack> {
        self.window[slot].as_ref().map(|shown| &shown.stack)
    }

    /// `calculateRowCount`: the rows past the five the window shows.
    pub(super) fn row_count(&self) -> i32 {
        (self.items.len() as i32 + 8) / 9 - 5
    }

    /// `getRowIndexForScroll`.
    pub fn row_index_for_scroll(&self, scroll: f32) -> i32 {
        ((scroll * self.row_count() as f32 + 0.5) as i32).max(0)
    }

    /// `getScrollForRowIndex`.
    pub fn scroll_for_row_index(&self, row: i32) -> f32 {
        (row as f32 / self.row_count() as f32).clamp(0.0, 1.0)
    }

    /// `subtractInputFromScroll`: the wheel's turn as a step of the scroll.
    pub fn subtract_input_from_scroll(&self, scroll: f32, input: f64) -> f32 {
        (scroll - (input / f64::from(self.row_count())) as f32).clamp(0.0, 1.0)
    }

    /// `scrollTo`: the window shows the rows from the scroll's row.
    pub fn scroll_to(&mut self, scroll: f32) {
        let row = self.row_index_for_scroll(scroll);
        for y in 0..ROWS {
            for x in 0..COLUMNS {
                let index = x as i32 + (y as i32 + row) * 9;
                self.window[x + y * COLUMNS] = usize::try_from(index)
                    .ok()
                    .and_then(|index| self.items.get(index))
                    .cloned()
                    .flatten();
            }
        }
    }

    /// The search tab's stacks, by place, shown in place of the tab's own, as
    /// a search finds them, scrolled to the top.
    pub fn show_items(&mut self, places: &[u16]) {
        let items = search_tab().items;
        self.items = places
            .iter()
            .map(|&place| &items[usize::from(place)])
            .filter(|entry| offered(entry))
            .map(|entry| {
                Some(Shown {
                    stack: table_stack(entry, &self.registries),
                    locked: false,
                })
            })
            .collect();
        self.scroll = 0.0;
        self.scroll_to(0.0);
    }

    /// `canScroll` of the menu: more stacks than the window holds.
    pub fn can_scroll(&self) -> bool {
        self.items.len() > WINDOW
    }
}

/// The rows the window shows and the columns of a row.
pub const ROWS: usize = 5;

pub const COLUMNS: usize = 9;

pub(super) const WINDOW: usize = ROWS * COLUMNS;

/// A stack in the picker: `CREATIVE_SLOT_LOCK` stacks (the hotbar tab's
/// placeholders) cannot be picked up.
#[derive(Debug, Clone)]
pub struct Shown {
    pub stack: Stack,
    pub locked: bool,
}
