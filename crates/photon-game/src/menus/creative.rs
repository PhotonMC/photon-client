//! The creative inventory's menu: `CreativeModeInventoryScreen.ItemPickerMenu`
//! and the screen's `slotClicked`, without the drawing.
//!
//! The picker shows a tab's stacks through a window of 45 slots, five rows of
//! nine, that scrolls a row at a time. A click there never reaches the
//! server as a container click: a change to the player's own inventory is
//! sent as a `SetCreativeModeSlot` packet for each slot it changed, and a
//! stack thrown away as one for slot -1.
//!
//! Not ported: the hotbar tab's saved hotbars (they are all empty, so it
//! shows the placeholders), and armour locked by enchantments.

use crate::menus::container::{ClickType, ContainerMenu, Player, max_stack_size, settle};
use crate::registries::Registries;
use photon_data::item::creative_tabs::{CREATIVE_TABS, CreativeStack, DEFAULT_TAB, TabKind};
use photon_world::ClientItemStackState as Stack;
use photon_world::block::tags::ItemTags;

impl CreativeMenu {
    /// A menu on a tab.
    pub fn new(
        tab: usize,
        registries: std::sync::Arc<Registries>,
        versions: std::sync::Arc<photon_data::version_registries::VersionRegistries>,
    ) -> Self {
        let mut menu = Self {
            tab,
            items: Vec::new(),
            window: vec![None; WINDOW],
            picker: ContainerMenu::default(),
            scroll: 0.0,
            registries,
            versions,
        };
        menu.select_tab(tab);
        menu
    }

    pub fn tab(&self) -> usize {
        self.tab
    }

    pub fn kind(&self) -> TabKind {
        CREATIVE_TABS[self.tab].kind
    }
}

/// `ItemPickerMenu` and the tab it shows.
#[derive(Debug, Clone)]
pub struct CreativeMenu {
    pub(super) tab: usize,
    /// `ItemPickerMenu.items`: what the tab lists, with gaps in the hotbar
    /// tab.
    pub(super) items: Vec<Option<Shown>>,
    /// `CONTAINER`: the 45 slots of the window.
    pub(super) window: Vec<Option<Shown>>,
    /// The picker's own menu, which the hotbar slots of a click go through.
    /// It keeps a drag in progress between clicks.
    pub(super) picker: ContainerMenu,
    pub(super) scroll: f32,
    /// The registries the tabs' stacks are written with.
    pub(super) registries: std::sync::Arc<Registries>,
    /// The numbering of the version the session speaks: a tab lists the items it has.
    pub(super) versions: std::sync::Arc<photon_data::version_registries::VersionRegistries>,
}

mod clicks;
pub use clicks::*;
mod scrolling;
pub use scrolling::*;
mod search;
pub use search::*;

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: Player = Player {
        selected: 0,
        infinite_materials: true,
    };

    fn test_versions() -> std::sync::Arc<photon_data::version_registries::VersionRegistries> {
        photon_data::version_registries::VersionRegistries::of(
            photon_core::version::Version::NEWEST,
        )
    }

    fn menu_on(kind: TabKind) -> CreativeMenu {
        let tab = CREATIVE_TABS
            .iter()
            .position(|tab| tab.kind == kind)
            .expect("a tab of the kind");
        CreativeMenu::new(
            tab,
            std::sync::Arc::new(crate::registries::for_tests()),
            test_versions(),
        )
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
            test_versions(),
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
            test_versions(),
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
            test_versions(),
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
            test_versions(),
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
            test_versions(),
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
            test_versions(),
        );
        let mut menu = ContainerMenu {
            carried: Some(Stack::new(5, photon_core::ids::ItemId(1))),
            ..ContainerMenu::default()
        };
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
        menu.slots[10] = Some(Stack::new(3, photon_core::ids::ItemId(1)));
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
        let assets =
            std::path::PathBuf::from(photon_core::content_root::assets().unwrap_or_default());
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
