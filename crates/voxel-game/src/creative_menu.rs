//! The creative inventory's menu: `CreativeModeInventoryScreen.ItemPickerMenu`
//! and the screen's `slotClicked`, without the drawing.
//!
//! The picker shows a tab's stacks through a window of 45 slots, five rows of
//! nine, that scrolls a row at a time. A click there never reaches the
//! server as a container click: a change to the player's own inventory is
//! sent as a `SetCreativeModeSlot` packet for each slot it changed, and a
//! stack thrown away as one for slot -1.
//!
//! Not ported: the search box (the search tab lists everything and takes no
//! text), the hotbar tab's saved hotbars (they are all empty, so it shows
//! the placeholders), tooltips, and armour locked by enchantments.

use crate::container_menu::{ClickType, ContainerMenu, Player, max_stack_size, settle};
use crate::registries::Registries;
use voxel_data::item::creative_tabs::{CREATIVE_TABS, CreativeStack, DEFAULT_TAB, TabKind};
use voxel_world::ClientItemStackState as Stack;

/// The rows the window shows and the columns of a row.
pub const ROWS: usize = 5;
pub const COLUMNS: usize = 9;
const WINDOW: usize = ROWS * COLUMNS;

/// A stack in the picker: `CREATIVE_SLOT_LOCK` stacks (the hotbar tab's
/// placeholders) cannot be picked up.
#[derive(Debug, Clone)]
pub struct Shown {
    pub stack: Stack,
    pub locked: bool,
}

/// What a click landed on, by what the reference calls its slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A slot of the window, by its place in it.
    Window(usize),
    /// A hotbar slot of the picker's own layout, 0 to 8.
    Hotbar(usize),
    /// The inventory tab's view of a slot of the player's menu.
    Inventory(usize),
    /// The inventory tab's bin.
    Destroy,
}

/// A click as `slotClicked` gets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Click {
    pub target: Option<Target>,
    /// The slot's index, or -999 outside the screen.
    pub slot_id: i32,
    pub button: i32,
    pub input: ClickType,
    /// What `hasClickedOutside` last found.
    pub outside: bool,
}

/// What a click made the client do, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// `Player.drop` on the client: the arm swings, and the swing is sent.
    Swing,
    /// `handleCreativeModeItemAdd`: the slot's new stack.
    SetSlot(i16, Option<Stack>),
    /// `handleCreativeModeItemDrop`: a stack thrown away.
    Drop(Stack),
}

/// The stack a tab declares: the item and the components it carries, as the
/// network carries them. A component naming an entry the registries do not
/// hold is left out.
pub fn table_stack(entry: &CreativeStack, registries: &Registries) -> Stack {
    let item = voxel_data::item::registry::index(entry.item).map_or(0, |index| index as i32);
    let added: Vec<(i32, Vec<u8>)> = entry
        .components
        .iter()
        .filter_map(|component| crate::default_components::encode(component, Some(registries)))
        .collect();
    let positive = added.iter().map(|(id, _)| *id).collect();
    let patch = crate::item_components::Patch {
        added,
        removed: Vec::new(),
    };
    Stack::from_wire(1, item, positive, Vec::new()).with_patch_bytes(patch.trusted())
}

/// The search tab, which lists every stack the other tabs do.
fn search_tab() -> &'static voxel_data::item::creative::CreativeTab {
    CREATIVE_TABS
        .iter()
        .find(|tab| tab.kind == TabKind::Search)
        .expect("there is a search tab")
}

/// The tabs that are shown, in the reference's order: `CreativeModeTabs.tabs`.
pub fn shown_tabs() -> Vec<usize> {
    (0..CREATIVE_TABS.len())
        .filter(|&tab| CREATIVE_TABS[tab].should_display)
        .collect()
}

/// The search tab's items with what the search reads of each: the lines of
/// its tooltip, lowercased, and its id. `SessionSearchTrees` builds it from
/// the search tab's display items.
pub struct SearchIndex {
    /// The search tab's stacks, by place.
    count: usize,
    lines: Vec<Vec<String>>,
    /// Each item's namespace and path, lowercased.
    ids: Vec<(String, String)>,
}

impl SearchIndex {
    pub fn build(lang: &crate::text::Lang, registries: &Registries) -> Self {
        let items = search_tab().items;
        let mut lines = Vec::with_capacity(items.len());
        let mut ids = Vec::with_capacity(items.len());
        for entry in items {
            let stack = table_stack(entry, registries);
            lines.push(
                crate::tooltip::search_lines(&stack, lang, registries)
                    .into_iter()
                    .map(|line| line.to_lowercase())
                    .collect(),
            );
            let name = entry.item;
            let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
            ids.push((namespace.to_lowercase(), path.to_lowercase()));
        }
        Self {
            count: items.len(),
            lines,
            ids,
        }
    }

    /// `SearchTree.search`: the places in the search tab of the stacks a
    /// query finds, in its order. A query with a colon names a namespace and
    /// a path; anything else is read as text of the tooltip.
    pub fn search(&self, text: &str) -> Vec<u16> {
        (0..self.count)
            .filter(|&at| query_matches(&self.lines[at], std::slice::from_ref(&self.ids[at]), text))
            .map(|at| at as u16)
            .collect()
    }
}

/// Whether a query finds what has these tooltip lines (lowercased) and these
/// ids (namespace and path, lowercased): a query with a colon names a
/// namespace and a path, anything else is text of a line.
pub fn query_matches(lines: &[String], ids: &[(String, String)], text: &str) -> bool {
    let by_line = |needle: &str| lines.iter().any(|line| line.contains(needle));
    match text.split_once(':') {
        None => by_line(text),
        Some((namespace, path)) => {
            let (namespace, path) = (namespace.trim(), path.trim());
            ids.iter().any(|(id_namespace, id_path)| {
                id_namespace.contains(namespace) && (id_path.contains(path) || by_line(path))
            })
        }
    }
}

/// The search trees of a session: built from the language the first time a
/// search asks, which takes a few milliseconds.
pub struct SearchTrees {
    lang: std::sync::Arc<crate::text::Lang>,
    registries: std::sync::Arc<Registries>,
    index: std::sync::OnceLock<SearchIndex>,
}

impl std::fmt::Debug for SearchTrees {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SearchTrees")
    }
}

impl SearchTrees {
    pub fn new(
        lang: std::sync::Arc<crate::text::Lang>,
        registries: std::sync::Arc<Registries>,
    ) -> Self {
        Self {
            lang,
            registries,
            index: std::sync::OnceLock::new(),
        }
    }

    /// The registries the trees were built with.
    pub fn registries(&self) -> &std::sync::Arc<Registries> {
        &self.registries
    }

    pub fn index(&self) -> &SearchIndex {
        self.index
            .get_or_init(|| SearchIndex::build(&self.lang, &self.registries))
    }
}

/// `CreativeModeTab.contains`: whether a stack is one the tab gives the
/// search tab, by item and component patch.
pub fn tab_contains(tab: usize, stack: &Stack, registries: &Registries) -> bool {
    let entry = &CREATIVE_TABS[tab];
    let candidates = if entry.search_items.is_empty() {
        entry.items
    } else {
        entry.search_items
    };
    // A stack made here has no patch bytes: the empty patch is two zeros.
    let patch: &[u8] = if stack.patch_bytes().is_empty() {
        &[0, 0]
    } else {
        stack.patch_bytes()
    };
    let item = usize::try_from(stack.item_registry_id()).ok();
    candidates.iter().any(|candidate| {
        voxel_data::item::registry::index(candidate.item) == item
            && table_stack(candidate, registries).patch_bytes() == patch
    })
}

/// `CreativeModeTabs.getDefaultTab`.
pub fn default_tab() -> usize {
    CREATIVE_TABS
        .iter()
        .position(|tab| tab.name == DEFAULT_TAB)
        .expect("the default tab is in the table")
}

/// `ItemPickerMenu` and the tab it shows.
#[derive(Debug, Clone)]
pub struct CreativeMenu {
    tab: usize,
    /// `ItemPickerMenu.items`: what the tab lists, with gaps in the hotbar
    /// tab.
    items: Vec<Option<Shown>>,
    /// `CONTAINER`: the 45 slots of the window.
    window: Vec<Option<Shown>>,
    /// The picker's own menu, which the hotbar slots of a click go through.
    /// It keeps a drag in progress between clicks.
    picker: ContainerMenu,
    scroll: f32,
    /// The registries the tabs' stacks are written with.
    registries: std::sync::Arc<Registries>,
}

/// `AbstractContainerMenu.broadcastChanges`: the creative listener sends
/// each slot that no longer matches what it held.
fn broadcast_changes(before: &[Option<Stack>], after: &[Option<Stack>], effects: &mut Vec<Effect>) {
    for (index, (was, now)) in before.iter().zip(after).enumerate() {
        let same = match (was, now) {
            (None, None) => true,
            (Some(a), Some(b)) => a.matches(b),
            _ => false,
        };
        if !same {
            effects.push(Effect::SetSlot(index as i16, now.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: Player = Player {
        selected: 0,
        infinite_materials: true,
    };

    fn menu_on(kind: TabKind) -> CreativeMenu {
        let tab = CREATIVE_TABS
            .iter()
            .position(|tab| tab.kind == kind)
            .expect("a tab of the kind");
        CreativeMenu::new(tab, std::sync::Arc::new(crate::registries::for_tests()))
    }

    fn click(
        creative: &mut CreativeMenu,
        menu: &mut ContainerMenu,
        target: Option<Target>,
        button: i32,
        input: ClickType,
    ) -> Vec<Effect> {
        let slot_id = match target {
            None => -999,
            Some(Target::Window(slot) | Target::Hotbar(slot)) => slot as i32,
            Some(Target::Inventory(slot)) => slot as i32,
            Some(Target::Destroy) => 46,
        };
        let click = Click {
            target,
            slot_id,
            button,
            input,
            outside: true,
        };
        creative.slot_clicked(click, menu, PLAYER, true)
    }

    #[test]
    fn the_default_tab_lists_its_items() {
        let creative = CreativeMenu::new(
            default_tab(),
            std::sync::Arc::new(crate::registries::for_tests()),
        );
        assert_eq!(creative.kind(), TabKind::Category);
        assert!(creative.window_stack(0).is_some());
        assert!(creative.can_scroll());
        assert!(creative.row_count() > 0);
    }

    #[test]
    fn scrolling_moves_the_window_by_rows() {
        let mut creative = CreativeMenu::new(
            default_tab(),
            std::sync::Arc::new(crate::registries::for_tests()),
        );
        let first = creative.window_stack(0).map(Stack::item_registry_id);
        creative.set_scroll(1.0);
        assert_ne!(creative.window_stack(0).map(Stack::item_registry_id), first);
        assert_eq!(creative.scroll_for_row_index(0), 0.0);
        creative.set_scroll(0.0);
        assert_eq!(creative.window_stack(0).map(Stack::item_registry_id), first);
    }

    #[test]
    fn a_click_on_the_window_takes_a_stack_and_clicks_grow_it() {
        let mut creative = CreativeMenu::new(
            default_tab(),
            std::sync::Arc::new(crate::registries::for_tests()),
        );
        let mut menu = ContainerMenu::default();
        let item = creative.window_stack(0).cloned().expect("a stack");
        click(
            &mut creative,
            &mut menu,
            Some(Target::Window(0)),
            0,
            ClickType::Pickup,
        );
        assert_eq!(menu.carried.as_ref().map(Stack::item_count), Some(1));
        click(
            &mut creative,
            &mut menu,
            Some(Target::Window(0)),
            0,
            ClickType::Pickup,
        );
        assert_eq!(menu.carried.as_ref().map(Stack::item_count), Some(2));
        click(
            &mut creative,
            &mut menu,
            Some(Target::Window(0)),
            0,
            ClickType::QuickMove,
        );
        assert_eq!(
            menu.carried.as_ref().map(Stack::item_count),
            Some(max_stack_size(&item))
        );
        // A click on another kind of stack puts the carried one away.
        click(
            &mut creative,
            &mut menu,
            Some(Target::Window(1)),
            0,
            ClickType::Pickup,
        );
        assert!(menu.carried.is_none());
    }

    #[test]
    fn placing_in_the_hotbar_sends_the_slot() {
        let mut creative = CreativeMenu::new(
            default_tab(),
            std::sync::Arc::new(crate::registries::for_tests()),
        );
        let mut menu = ContainerMenu::default();
        click(
            &mut creative,
            &mut menu,
            Some(Target::Window(0)),
            0,
            ClickType::Pickup,
        );
        let effects = click(
            &mut creative,
            &mut menu,
            Some(Target::Hotbar(2)),
            0,
            ClickType::Pickup,
        );
        assert!(menu.carried.is_none());
        assert!(menu.slots[38].is_some());
        assert!(matches!(effects.as_slice(), [Effect::SetSlot(38, Some(_))]));
    }

    #[test]
    fn a_hotbar_key_over_the_window_fills_the_slot_with_a_full_stack() {
        let mut creative = CreativeMenu::new(
            default_tab(),
            std::sync::Arc::new(crate::registries::for_tests()),
        );
        let mut menu = ContainerMenu::default();
        let effects = click(
            &mut creative,
            &mut menu,
            Some(Target::Window(0)),
            3,
            ClickType::Swap,
        );
        let stack = menu.slots[39].as_ref().expect("filled");
        assert_eq!(stack.item_count(), max_stack_size(stack));
        assert!(matches!(effects.as_slice(), [Effect::SetSlot(39, Some(_))]));
    }

    #[test]
    fn a_click_outside_throws_the_carried_stack_away() {
        let mut creative = CreativeMenu::new(
            default_tab(),
            std::sync::Arc::new(crate::registries::for_tests()),
        );
        let mut menu = ContainerMenu::default();
        menu.carried = Some(Stack::new(5, 1));
        let effects = click(&mut creative, &mut menu, None, 0, ClickType::Pickup);
        assert!(menu.carried.is_none());
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0], Effect::Swing);
        assert!(matches!(&effects[1], Effect::Drop(stack) if stack.item_count() == 5));
    }

    #[test]
    fn the_bin_empties_the_whole_inventory_with_a_quick_click() {
        let mut creative = menu_on(TabKind::Inventory);
        let mut menu = ContainerMenu::default();
        menu.slots[10] = Some(Stack::new(3, 1));
        let effects = click(
            &mut creative,
            &mut menu,
            Some(Target::Destroy),
            0,
            ClickType::QuickMove,
        );
        assert!(menu.slots.iter().all(Option::is_none));
        assert_eq!(effects.len(), menu.slots.len());
        assert!(
            effects
                .iter()
                .all(|effect| matches!(effect, Effect::SetSlot(_, None)))
        );
    }

    fn index() -> Option<SearchIndex> {
        let assets = std::path::PathBuf::from(std::env::var("VOXEL_ASSETS").unwrap_or_default());
        let lang = crate::text::Lang::load(&assets);
        let registries = crate::registries::for_tests();
        lang.get("item.minecraft.stone")
            .is_some()
            .then(|| SearchIndex::build(&lang, &registries))
    }

    fn names(found: &[u16]) -> Vec<&'static str> {
        found
            .iter()
            .map(|&place| search_tab().items[usize::from(place)].item)
            .collect()
    }

    #[test]
    fn a_search_finds_what_the_tooltip_says() {
        let Some(index) = index() else { return };
        let found = names(&index.search("diamond sword"));
        assert_eq!(found, vec!["minecraft:diamond_sword"]);
        // Text of any line of the tooltip: a potion's effect.
        let poison = names(&index.search("poison"));
        assert!(poison.contains(&"minecraft:tipped_arrow"));
        assert!(poison.contains(&"minecraft:potion"));
        assert!(index.search("qqqq").is_empty());
    }

    #[test]
    fn a_search_with_a_colon_reads_a_namespace_and_a_path() {
        let Some(index) = index() else { return };
        let found = names(&index.search("minecraft:oak_lo"));
        assert!(found.contains(&"minecraft:oak_log"));
        assert!(found.iter().all(|name| name.starts_with("minecraft:")));
        assert!(index.search("nomod:oak_log").is_empty());
    }

    #[test]
    fn the_hotbar_tab_shows_locked_placeholders() {
        let mut creative = menu_on(TabKind::Hotbar);
        let mut menu = ContainerMenu::default();
        assert!(creative.window_stack(0).is_some());
        assert!(creative.window_stack(1).is_none());
        click(
            &mut creative,
            &mut menu,
            Some(Target::Window(0)),
            0,
            ClickType::Pickup,
        );
        assert!(menu.carried.is_none());
    }
}

mod creative_menu_scroll;
