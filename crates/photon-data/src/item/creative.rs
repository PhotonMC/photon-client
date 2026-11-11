//! What the creative inventory's tabs declare (`CreativeModeTabs`): each tab
//! with its icon and the stacks it shows, an item and the components it
//! carries beyond its own defaults.

use super::component::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    Category,
    Inventory,
    Hotbar,
    Search,
}

/// An item and the components a stack of it carries beyond the item's
/// defaults, in the order the reference's patch lists them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreativeStack {
    pub item: &'static str,
    pub components: &'static [Component],
}

impl CreativeStack {
    pub const fn of(item: &'static str) -> Self {
        Self {
            item,
            components: &[],
        }
    }

    pub const fn with(mut self, components: &'static [Component]) -> Self {
        self.components = components;
        self
    }
}

#[derive(Debug)]
pub struct CreativeTab {
    pub name: &'static str,
    pub kind: TabKind,
    pub top: bool,
    pub column: i32,
    pub aligned_right: bool,
    pub show_title: bool,
    pub can_scroll: bool,
    pub should_display: bool,
    pub background: &'static str,
    pub title: &'static str,
    pub icon: CreativeStack,
    pub items: &'static [CreativeStack],
    /// What the search tab takes from the tab when it is not the same as
    /// `items`; empty when it is.
    pub search_items: &'static [CreativeStack],
}

/// Puts each of `windows` in its own order inside `items`: a window applies where all its names
/// stand together and none was moved by an earlier (longer) window.
pub fn reorder(items: &mut [CreativeStack], windows: &[&[&str]]) {
    let mut moved = vec![false; items.len()];
    for window in windows {
        let Some(start) = window_start(items, window) else {
            continue;
        };
        let range = start..start + window.len();
        if moved[range.clone()].iter().any(|moved| *moved) {
            continue;
        }
        let old = items[range.clone()].to_vec();
        for (place, name) in window.iter().enumerate() {
            if let Some(entry) = old.iter().find(|entry| bare(entry.item) == *name) {
                items[start + place] = *entry;
            }
        }
        moved[range].fill(true);
    }
}

fn bare(name: &str) -> &str {
    name.split_once(':').map_or(name, |(_, bare)| bare)
}

/// Where the names of `window` stand together in `items`, in any order.
fn window_start(items: &[CreativeStack], window: &[&str]) -> Option<usize> {
    let inside = |entry: &CreativeStack| window.contains(&bare(entry.item));
    let start = items.iter().position(inside)?;
    let run = items.get(start..start + window.len())?;
    let whole = run.iter().all(inside)
        && window
            .iter()
            .all(|name| run.iter().any(|entry| bare(entry.item) == *name));
    let elsewhere = items[start + window.len()..].iter().any(inside);
    (whole && !elsewhere).then_some(start)
}
