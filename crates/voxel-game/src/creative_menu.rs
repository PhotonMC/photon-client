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

use crate::inventory_menu::{ClickType, InventoryMenu, Player, max_stack_size, settle};
use voxel_data::item::creative_tabs::{CREATIVE_STACKS, CREATIVE_TABS, DEFAULT_TAB, TabKind};
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

/// The stack of an entry in the table: one item with its patch.
pub fn table_stack(index: u16) -> Stack {
    let entry = &CREATIVE_STACKS[usize::from(index)];
    Stack::from_wire(
        1,
        entry.item,
        entry.positive.to_vec(),
        entry.negative.to_vec(),
    )
    .with_patch_bytes(entry.patch.to_vec())
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
    /// The items, as indices into the table's stacks.
    items: Vec<u16>,
    lines: Vec<Vec<String>>,
    /// Each item's namespace and path, lowercased.
    ids: Vec<(String, String)>,
}

impl SearchIndex {
    pub fn build(lang: &crate::text::Lang, registries: &crate::registries::Registries) -> Self {
        let tab = CREATIVE_TABS
            .iter()
            .find(|tab| tab.kind == TabKind::Search)
            .expect("there is a search tab");
        let items: Vec<u16> = tab.items.to_vec();
        let mut lines = Vec::with_capacity(items.len());
        let mut ids = Vec::with_capacity(items.len());
        for &index in &items {
            let stack = table_stack(index);
            lines.push(
                crate::tooltip::search_lines(&stack, lang, registries)
                    .into_iter()
                    .map(|line| line.to_lowercase())
                    .collect(),
            );
            let name = voxel_data::item::registry::ITEMS[stack.item_registry_id() as usize].name;
            let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
            ids.push((namespace.to_lowercase(), path.to_lowercase()));
        }
        Self { items, lines, ids }
    }

    /// `SearchTree.search`: the items a query finds, in the search tab's
    /// order. A query with a colon names a namespace and a path; anything
    /// else is read as text of the tooltip.
    pub fn search(&self, text: &str) -> Vec<u16> {
        let by_line =
            |needle: &str, at: usize| self.lines[at].iter().any(|line| line.contains(needle));
        (0..self.items.len())
            .filter(|&at| match text.split_once(':') {
                None => by_line(text, at),
                Some((namespace, path)) => {
                    let (namespace, path) = (namespace.trim(), path.trim());
                    self.ids[at].0.contains(namespace)
                        && (self.ids[at].1.contains(path) || by_line(path, at))
                }
            })
            .map(|at| self.items[at])
            .collect()
    }
}

/// The search trees of a session: built from the language the first time a
/// search asks, which takes a few milliseconds.
pub struct SearchTrees {
    lang: std::sync::Arc<crate::text::Lang>,
    registries: std::sync::Arc<crate::registries::Registries>,
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
        registries: std::sync::Arc<crate::registries::Registries>,
    ) -> Self {
        Self {
            lang,
            registries,
            index: std::sync::OnceLock::new(),
        }
    }

    pub fn index(&self) -> &SearchIndex {
        self.index
            .get_or_init(|| SearchIndex::build(&self.lang, &self.registries))
    }
}

/// `CreativeModeTab.contains`: whether a stack is one the tab gives the
/// search tab, by item and component patch.
pub fn tab_contains(tab: usize, stack: &Stack) -> bool {
    let entry = &CREATIVE_TABS[tab];
    let indices = if entry.search_items.is_empty() {
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
    indices.iter().any(|&index| {
        let candidate = &CREATIVE_STACKS[usize::from(index)];
        candidate.item == stack.item_registry_id() && candidate.patch == patch
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
    picker: InventoryMenu,
    scroll: f32,
}

impl CreativeMenu {
    /// A menu on a tab.
    pub fn new(tab: usize) -> Self {
        let mut menu = Self {
            tab,
            items: Vec::new(),
            window: vec![None; WINDOW],
            picker: InventoryMenu::default(),
            scroll: 0.0,
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

    /// `selectTab`: the tab's stacks, scrolled to the top.
    pub fn select_tab(&mut self, tab: usize) {
        self.tab = tab;
        self.items.clear();
        match CREATIVE_TABS[tab].kind {
            TabKind::Hotbar => {
                // Every saved hotbar is empty: a paper in the diagonal.
                let paper = crate::item::id_of("minecraft:paper").expect("paper is an item");
                for hotbar in 0..COLUMNS {
                    for slot in 0..COLUMNS {
                        self.items.push((slot == hotbar).then(|| Shown {
                            stack: Stack::new(1, paper),
                            locked: true,
                        }));
                    }
                }
            }
            TabKind::Category | TabKind::Search => {
                for &index in CREATIVE_TABS[tab].items {
                    self.items.push(Some(Shown {
                        stack: table_stack(index),
                        locked: false,
                    }));
                }
            }
            TabKind::Inventory => {}
        }
        self.scroll = 0.0;
        self.scroll_to(0.0);
    }

    /// `calculateRowCount`: the rows past the five the window shows.
    fn row_count(&self) -> i32 {
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

    /// The stacks of the table shown in place of the tab's own, as a search
    /// finds them, scrolled to the top.
    pub fn show_items(&mut self, indices: &[u16]) {
        self.items = indices
            .iter()
            .map(|&index| {
                Some(Shown {
                    stack: table_stack(index),
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

    /// `Slot.mayPickup` for what `slotClicked` asks it of.
    fn may_pickup(&self, target: Target) -> bool {
        match target {
            // `CustomCreativeSlot`: an empty slot may be, a full one unless
            // it is locked.
            Target::Window(slot) => self.window[slot].as_ref().is_none_or(|shown| !shown.locked),
            _ => true,
        }
    }

    /// `CreativeModeInventoryScreen.slotClicked`.
    ///
    /// `can_drop` is whether `canDropItems` allows a drop.
    pub fn slot_clicked(
        &mut self,
        click: Click,
        menu: &mut InventoryMenu,
        player: Player,
        can_drop: bool,
    ) -> Vec<Effect> {
        let Click {
            target,
            slot_id,
            button,
            input,
            outside: clicked_outside,
        } = click;
        let quick_key = input == ClickType::QuickMove;
        let input = if slot_id == -999 && input == ClickType::Pickup {
            ClickType::Throw
        } else {
            input
        };
        let mut effects = Vec::new();
        if input == ClickType::Throw && !can_drop {
            return effects;
        }
        let inventory_tab = self.kind() == TabKind::Inventory;
        if target.is_none() && !inventory_tab && input != ClickType::QuickCraft {
            if let Some(carried) = menu.carried.clone()
                && clicked_outside
            {
                if !can_drop {
                    return effects;
                }
                if button == 0 {
                    effects.push(Effect::Swing);
                    effects.push(Effect::Drop(carried));
                    menu.carried = None;
                }
                if button == 1
                    && let Some(carried) = menu.carried.as_mut()
                {
                    let removed = carried.split(1);
                    settle(&mut menu.carried);
                    effects.push(Effect::Swing);
                    effects.push(Effect::Drop(removed));
                }
            }
            return effects;
        }
        if let Some(target) = target
            && !self.may_pickup(target)
        {
            return effects;
        }
        if target == Some(Target::Destroy) && quick_key {
            for index in 0..menu.slots.len() {
                menu.slots[index] = None;
                effects.push(Effect::SetSlot(index as i16, None));
            }
        } else if inventory_tab {
            self.inventory_tab_click(target, slot_id, button, input, menu, player, &mut effects);
        } else if input != ClickType::QuickCraft
            && let Some(Target::Window(slot)) = target
        {
            self.window_click(slot, button, input, quick_key, menu, &mut effects);
        } else {
            self.picker_click(target, slot_id, button, input, menu, player, &mut effects);
        }
        effects
    }

    /// The inventory tab: the player's own menu, seen through other places.
    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    fn inventory_tab_click(
        &mut self,
        target: Option<Target>,
        slot_id: i32,
        button: i32,
        input: ClickType,
        menu: &mut InventoryMenu,
        player: Player,
        effects: &mut Vec<Effect>,
    ) {
        if target == Some(Target::Destroy) {
            menu.carried = None;
        } else if input == ClickType::Throw
            && let Some(Target::Inventory(index)) = target
            && menu.slots[index].is_some()
        {
            let amount = if button == 0 {
                1
            } else {
                menu.slots[index].as_ref().map_or(1, max_stack_size)
            };
            if let Some(dropped) = menu.take_from_slot(index, amount) {
                effects.push(Effect::Swing);
                effects.push(Effect::Drop(dropped));
            }
            effects.push(Effect::SetSlot(index as i16, menu.slots[index].clone()));
        } else if input == ClickType::Throw && slot_id == -999 && menu.carried.is_some() {
            let carried = menu.carried.take().expect("checked above");
            effects.push(Effect::Swing);
            effects.push(Effect::Drop(carried));
        } else {
            let index = match target {
                Some(Target::Inventory(index)) => index as i32,
                _ => slot_id,
            };
            let before = menu.slots.clone();
            let swings = menu.click_local(index, button, input, player);
            effects.extend(std::iter::repeat_n(Effect::Swing, swings));
            broadcast_changes(&before, &menu.slots, effects);
        }
    }

    /// A click on a window slot: stacks are made, not moved.
    fn window_click(
        &mut self,
        slot: usize,
        button: i32,
        input: ClickType,
        quick_key: bool,
        menu: &mut InventoryMenu,
        effects: &mut Vec<Effect>,
    ) {
        let clicked = self.window_stack(slot).cloned();
        match input {
            ClickType::Swap => {
                if let Some(clicked) = clicked {
                    // `Inventory.setItem`: the hotbar, or the off hand.
                    let menu_slot = match button {
                        0..=8 => Some(36 + button as usize),
                        40 => Some(45),
                        _ => None,
                    };
                    if let Some(menu_slot) = menu_slot {
                        let before = menu.slots.clone();
                        menu.slots[menu_slot] =
                            Some(clicked.copy_with_count(max_stack_size(&clicked)));
                        broadcast_changes(&before, &menu.slots, effects);
                    }
                }
            }
            ClickType::Clone => {
                if menu.carried.is_none()
                    && let Some(clicked) = clicked
                {
                    menu.carried = Some(clicked.copy_with_count(max_stack_size(&clicked)));
                }
            }
            ClickType::Throw => {
                if let Some(clicked) = clicked {
                    let count = if button == 0 {
                        1
                    } else {
                        max_stack_size(&clicked)
                    };
                    effects.push(Effect::Swing);
                    effects.push(Effect::Drop(clicked.copy_with_count(count)));
                }
            }
            _ => {
                let carried = menu.carried.clone();
                match (carried, clicked) {
                    (Some(mut carried), Some(clicked))
                        if carried.is_same_item_same_components(&clicked) =>
                    {
                        if button == 0 {
                            if quick_key {
                                carried.set_count(max_stack_size(&carried));
                            } else if carried.item_count() < max_stack_size(&carried) {
                                carried.grow(1);
                            }
                        } else {
                            carried.shrink(1);
                        }
                        menu.carried = Some(carried);
                        settle(&mut menu.carried);
                    }
                    (None, Some(clicked)) => {
                        let count = if quick_key {
                            max_stack_size(&clicked)
                        } else {
                            clicked.item_count()
                        };
                        menu.carried = Some(clicked.copy_with_count(count));
                    }
                    _ if button == 0 => menu.carried = None,
                    (Some(mut carried), _) => {
                        carried.shrink(1);
                        menu.carried = Some(carried);
                        settle(&mut menu.carried);
                    }
                    _ => {}
                }
            }
        }
    }

    /// The picker's own slots: the hotbar, and a drag over it. The click runs
    /// in a menu that holds only the hotbar, since the window's slots take no
    /// part in it.
    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    fn picker_click(
        &mut self,
        target: Option<Target>,
        slot_id: i32,
        button: i32,
        input: ClickType,
        menu: &mut InventoryMenu,
        player: Player,
        effects: &mut Vec<Effect>,
    ) {
        let hotbar = match target {
            Some(Target::Hotbar(slot)) => Some(slot),
            Some(_) => return,
            None => None,
        };
        let old = hotbar.and_then(|slot| menu.slots[36 + slot].clone());
        let before = menu.slots.clone();
        let mut slots = vec![None; menu.slots.len()];
        slots[36..45].clone_from_slice(&menu.slots[36..45]);
        self.picker.load(slots, menu.carried.clone(), menu.state_id);
        let index = hotbar.map_or(slot_id, |slot| 36 + slot as i32);
        let swings = if input == ClickType::QuickMove && (button == 0 || button == 1) {
            // `quickMoveStack` of the picker throws the stack away.
            if let Some(slot) = hotbar {
                self.picker.slots[36 + slot] = None;
            }
            0
        } else {
            self.picker.click_local(index, button, input, player)
        };
        menu.slots[36..45].clone_from_slice(&self.picker.slots[36..45]);
        menu.carried = self.picker.carried.clone();
        effects.extend(std::iter::repeat_n(Effect::Swing, swings));
        if button & 3 == 2 {
            // `getQuickcraftHeader(buttonNum) == 2`, whatever the click was.
            for slot in 0..COLUMNS {
                effects.push(Effect::SetSlot(
                    (36 + slot) as i16,
                    menu.slots[36 + slot].clone(),
                ));
            }
        } else if let Some(slot) = hotbar {
            if input == ClickType::Throw
                && let Some(old) = old
                && menu.carried.is_some()
            {
                let count = if button == 0 { 1 } else { old.item_count() };
                let dropped = old.copy_with_count(count);
                // The old stack is shrunk where it lies.
                if let Some(present) = menu.slots[36 + slot].as_mut() {
                    present.shrink(count);
                    settle(&mut menu.slots[36 + slot]);
                }
                effects.push(Effect::Swing);
                effects.push(Effect::Drop(dropped));
            }
            broadcast_changes(&before, &menu.slots, effects);
        }
    }
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
        CreativeMenu::new(tab)
    }

    fn click(
        creative: &mut CreativeMenu,
        menu: &mut InventoryMenu,
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
        let creative = CreativeMenu::new(default_tab());
        assert_eq!(creative.kind(), TabKind::Category);
        assert!(creative.window_stack(0).is_some());
        assert!(creative.can_scroll());
        assert!(creative.row_count() > 0);
    }

    #[test]
    fn scrolling_moves_the_window_by_rows() {
        let mut creative = CreativeMenu::new(default_tab());
        let first = creative.window_stack(0).map(Stack::item_registry_id);
        creative.set_scroll(1.0);
        assert_ne!(creative.window_stack(0).map(Stack::item_registry_id), first);
        assert_eq!(creative.scroll_for_row_index(0), 0.0);
        creative.set_scroll(0.0);
        assert_eq!(creative.window_stack(0).map(Stack::item_registry_id), first);
    }

    #[test]
    fn a_click_on_the_window_takes_a_stack_and_clicks_grow_it() {
        let mut creative = CreativeMenu::new(default_tab());
        let mut menu = InventoryMenu::default();
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
        let mut creative = CreativeMenu::new(default_tab());
        let mut menu = InventoryMenu::default();
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
        let mut creative = CreativeMenu::new(default_tab());
        let mut menu = InventoryMenu::default();
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
        let mut creative = CreativeMenu::new(default_tab());
        let mut menu = InventoryMenu::default();
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
        let mut menu = InventoryMenu::default();
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
        let registries = crate::registries::Registries::vanilla(&assets.with_file_name("data"));
        lang.get("item.minecraft.stone")
            .is_some()
            .then(|| SearchIndex::build(&lang, &registries))
    }

    fn names(found: &[u16]) -> Vec<&'static str> {
        found
            .iter()
            .map(|&index| {
                voxel_data::item::registry::ITEMS[table_stack(index).item_registry_id() as usize]
                    .name
            })
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
        let mut menu = InventoryMenu::default();
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
