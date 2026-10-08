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
