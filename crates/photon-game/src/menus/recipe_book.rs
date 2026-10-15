//! The recipe book's model (`ClientRecipeBook`, `RecipeCollection` and
//! `RecipeBookComponent`'s state): the recipes the server made known, grouped
//! into collections, which of them fit the open menu and can be crafted from
//! what the player holds, and what a tab, a search and the filter show.

use std::collections::{BTreeMap, BTreeSet};

use photon_data::item::component_types as ids;
use photon_world::{
    ClientItemStackState as Stack, Ingredient, RecipeBook, RecipeDisplay, SlotDisplay,
    block::tags::ItemTags,
};

use crate::item::values::Values;
use crate::menus::container::{ContainerMenu, MenuKind, Smelter, max_stack_size};

/// The recipes per page: five columns by four rows (`RecipeBookPage`).
pub const RECIPES_PER_PAGE: usize = 20;

/// The book a menu shows, which decides the recipes it lists and the
/// `RecipeBookType` its settings are kept under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookKind {
    /// The 2x2 grid of the inventory or the 3x3 of a crafting table.
    Crafting {
        size: i32,
    },
    Furnace(Smelter),
}

impl BookKind {
    /// The book of a menu, for the menus that have one.
    pub fn of_menu(kind: MenuKind) -> Option<Self> {
        match kind {
            MenuKind::Inventory => Some(Self::Crafting { size: 2 }),
            MenuKind::Crafting => Some(Self::Crafting { size: 3 }),
            MenuKind::Furnace(smelter) => Some(Self::Furnace(smelter)),
            _ => None,
        }
    }

    /// The `RecipeBookType` ordinal: crafting, furnace, blast furnace, smoker.
    pub fn book_type(self) -> usize {
        match self {
            Self::Crafting { .. } => 0,
            Self::Furnace(Smelter::Furnace) => 1,
            Self::Furnace(Smelter::BlastFurnace) => 2,
            Self::Furnace(Smelter::Smoker) => 3,
        }
    }

    /// The menu slots that hold ingredients, besides the player's inventory
    /// (`fillCraftSlotsStackedContents`).
    fn input_slots(self) -> std::ops::Range<usize> {
        match self {
            Self::Crafting { size: 2 } => 1..5,
            Self::Crafting { .. } => 1..10,
            Self::Furnace(_) => 0..3,
        }
    }

    /// The tabs after the search tab: each a list of `RecipeBookCategories`
    /// ids, in the order the book shows them.
    pub fn tabs(self) -> &'static [&'static [i32]] {
        match self {
            // equipment, building blocks, misc, redstone
            Self::Crafting { .. } => &[&[2], &[0], &[3], &[1]],
            Self::Furnace(Smelter::Furnace) => &[&[4], &[5], &[6]],
            Self::Furnace(Smelter::BlastFurnace) => &[&[7], &[8]],
            Self::Furnace(Smelter::Smoker) => &[&[9]],
        }
    }

    /// The categories the search tab lists (`SearchRecipeBookCategory`).
    fn categories(self) -> std::ops::RangeInclusive<i32> {
        match self {
            Self::Crafting { .. } => 0..=3,
            Self::Furnace(Smelter::Furnace) => 4..=6,
            Self::Furnace(Smelter::BlastFurnace) => 7..=8,
            Self::Furnace(Smelter::Smoker) => 9..=9,
        }
    }

    /// Whether the menu can show a recipe (`RecipeCollection.DisplayResolver`).
    fn can_display(self, display: &RecipeDisplay) -> bool {
        match (self, display) {
            (Self::Crafting { size }, RecipeDisplay::Shaped { width, height, .. }) => {
                size >= *width && size >= *height
            }
            (Self::Crafting { size }, RecipeDisplay::Shapeless { ingredients, .. }) => {
                (size * size) as usize >= ingredients.len()
            }
            (Self::Furnace(_), RecipeDisplay::Furnace { .. }) => true,
            _ => false,
        }
    }
}

/// `RecipeCollection`: recipes of one category and group, which a button
/// stands for.
#[derive(Debug, Clone)]
pub struct Collection {
    pub category: i32,
    /// The recipe display ids, ascending.
    pub entries: Vec<i32>,
    /// The ones the menu can show.
    pub selected: BTreeSet<i32>,
    /// The selected ones that can be crafted from what is held.
    pub craftable: BTreeSet<i32>,
}

impl Collection {
    pub fn has_any_selected(&self) -> bool {
        !self.selected.is_empty()
    }

    pub fn has_craftable(&self) -> bool {
        !self.craftable.is_empty()
    }

    /// The recipes a button offers (`RecipeButton.init`): only the craftable
    /// ones while filtering.
    pub fn offered(&self, filtering: bool) -> Vec<i32> {
        self.selected_entries(filtering.then_some(true))
    }

    /// `getSelectedRecipes`: the ids, the craftable ones first only when
    /// asked for.
    pub fn selected_entries(&self, craftable: Option<bool>) -> Vec<i32> {
        self.entries
            .iter()
            .copied()
            .filter(|id| {
                self.selected.contains(id)
                    && craftable.is_none_or(|want| self.craftable.contains(id) == want)
            })
            .collect()
    }
}

/// `StackedItemContents`: how many of each item the player holds, counting
/// only plain stacks (`Inventory.accountSimpleStack`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StackedItems(BTreeMap<i32, i32>);

impl StackedItems {
    /// `accountSimpleStack`: damaged, enchanted and named stacks do not count.
    pub fn account(&mut self, stack: &Stack) {
        let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
        let damaged = values.raw(ids::DAMAGE).is_some_and(|raw| {
            let mut input = raw;
            photon_core::wire::read_varint(&mut input).is_ok_and(|damage| damage > 0)
        });
        if damaged
            || !values.enchantments(ids::ENCHANTMENTS).is_empty()
            || values.has(ids::CUSTOM_NAME)
        {
            return;
        }
        let count = stack.item_count().min(max_stack_size(stack));
        *self.0.entry(stack.item_registry_id().0).or_insert(0) += count;
    }

    /// The player's 36 inventory slots and the book's ingredient slots of the
    /// menu.
    pub fn of_menu(menu: &ContainerMenu, kind: BookKind) -> Self {
        let mut stacked = Self::default();
        let inventory = match menu.kind {
            MenuKind::Inventory => 9..45,
            other => other.top_slots()..other.slot_count(),
        };
        for slot in inventory.chain(kind.input_slots()) {
            if let Some(Some(stack)) = menu.slots.get(slot) {
                stacked.account(stack);
            }
        }
        stacked
    }

    /// `StackedContents.canCraft`: every ingredient takes one item of its
    /// set, no stack giving more than it holds. An assignment as a bipartite
    /// matching, so an ingredient that takes a scarce item does not starve
    /// one that could have taken another.
    pub fn can_craft(&self, ingredients: &[Ingredient], tags: &ItemTags) -> bool {
        let options: Vec<Vec<i32>> = ingredients
            .iter()
            .map(|ingredient| {
                let items: Vec<i32> = match ingredient {
                    Ingredient::Items(items) => items.clone(),
                    Ingredient::Tag(tag) => tags.members(tag).collect(),
                };
                items
                    .into_iter()
                    .filter(|item| self.0.get(item).is_some_and(|count| *count > 0))
                    .collect()
            })
            .collect();
        let mut owners: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
        (0..options.len()).all(|ingredient| {
            let mut seen = BTreeSet::new();
            self.place(ingredient, &options, &mut owners, &mut seen)
        })
    }

    fn place(
        &self,
        ingredient: usize,
        options: &[Vec<i32>],
        owners: &mut BTreeMap<i32, Vec<usize>>,
        seen: &mut BTreeSet<i32>,
    ) -> bool {
        for &item in &options[ingredient] {
            if !seen.insert(item) {
                continue;
            }
            let capacity = self.0.get(&item).copied().unwrap_or(0) as usize;
            if owners.get(&item).map_or(0, Vec::len) < capacity {
                owners.entry(item).or_default().push(ingredient);
                return true;
            }
            // The item is used up: one of its users may take another item.
            let users = owners.get(&item).cloned().unwrap_or_default();
            for (position, user) in users.into_iter().enumerate() {
                if self.place(user, options, owners, seen) {
                    if let Some(list) = owners.get_mut(&item) {
                        list[position] = ingredient;
                    }
                    return true;
                }
            }
        }
        false
    }
}

/// The book of the open menu: its collections and what they show.
#[derive(Debug, Clone)]
pub struct BookModel {
    pub kind: BookKind,
    collections: Vec<Collection>,
    built_for: Option<u64>,
    stacked: Option<StackedItems>,
    /// The tab: 0 is the search tab, then [`BookKind::tabs`] in order.
    pub tab: usize,
    pub page: usize,
    pub search: String,
}

impl BookModel {
    pub fn new(kind: BookKind) -> Self {
        Self {
            kind,
            collections: Vec::new(),
            built_for: None,
            stacked: None,
            tab: 0,
            page: 0,
            search: String::new(),
        }
    }

    /// `rebuildCollections` and `selectRecipes`, when the book or what is
    /// held changed.
    pub fn refresh(&mut self, book: &RecipeBook, menu: &ContainerMenu, tags: &ItemTags) {
        let stacked = StackedItems::of_menu(menu, self.kind);
        if self.built_for == Some(book.revision) && self.stacked.as_ref() == Some(&stacked) {
            return;
        }
        if self.built_for != Some(book.revision) {
            self.collections = build_collections(book, self.kind);
        }
        for collection in &mut self.collections {
            collection.selected.clear();
            collection.craftable.clear();
            for id in &collection.entries {
                let Some(entry) = book.known.get(id) else {
                    continue;
                };
                if !self.kind.can_display(&entry.display) {
                    continue;
                }
                collection.selected.insert(*id);
                if entry
                    .requirements
                    .as_ref()
                    .is_some_and(|needs| !needs.is_empty() && stacked.can_craft(needs, tags))
                {
                    collection.craftable.insert(*id);
                }
            }
        }
        self.built_for = Some(book.revision);
        self.stacked = Some(stacked);
    }

    pub fn collections(&self) -> &[Collection] {
        &self.collections
    }

    /// Whether a tab shows: the search tab always, a category tab when a
    /// collection of it has a recipe the menu can show.
    pub fn tab_visible(&self, tab: usize) -> bool {
        tab == 0
            || self.collections.iter().any(|collection| {
                self.kind.tabs()[tab - 1].contains(&collection.category)
                    && collection.has_any_selected()
            })
    }

    /// The collections the grid lists (`updateCollections`): those of the
    /// tab with a recipe to show, the ones the search finds, and with
    /// something craftable when filtering. The search tab lists the tabs'
    /// categories one after another (`SearchRecipeBookCategory`).
    pub fn listed(&self, filtering: bool, found: &dyn Fn(&Collection) -> bool) -> Vec<usize> {
        let tab_categories: Vec<i32> = if self.tab == 0 {
            self.kind
                .tabs()
                .iter()
                .flat_map(|tab| tab.iter().copied())
                .collect()
        } else {
            self.kind.tabs()[self.tab - 1].to_vec()
        };
        let searching = !self.search.is_empty();
        tab_categories
            .iter()
            .flat_map(|category| {
                self.collections
                    .iter()
                    .enumerate()
                    .filter(move |(_, collection)| collection.category == *category)
            })
            .filter(|(_, collection)| {
                collection.has_any_selected()
                    && (!searching || found(collection))
                    && (!filtering || collection.has_craftable())
            })
            .map(|(index, _)| index)
            .collect()
    }
}

/// One ingredient of an overlay button: where it sits in the button and what
/// it shows.
pub type OverlaySlot = (i32, i32, SlotDisplay);

mod display;
pub use display::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn stacked(items: &[(i32, i32)]) -> StackedItems {
        StackedItems(items.iter().copied().collect())
    }

    #[test]
    fn picker_reassigns_a_shared_item() {
        let tags = ItemTags::default();
        // Ingredient A takes item 1 or 2, B only item 1; one of each held.
        let ingredients = [Ingredient::Items(vec![1, 2]), Ingredient::Items(vec![1])];
        assert!(stacked(&[(1, 1), (2, 1)]).can_craft(&ingredients, &tags));
        assert!(!stacked(&[(1, 1)]).can_craft(&ingredients, &tags));
    }

    #[test]
    fn small_recipes_centre_in_the_three_by_three_grid() {
        // A 1x1 recipe lands in the middle slot, a 2x2 at the top left, a
        // 1x3 in the middle column.
        assert_eq!(place_recipe_helper((3, 3), (1, 1), 1), vec![(0, 4)]);
        assert_eq!(
            place_recipe_helper((3, 3), (2, 2), 4),
            vec![(0, 0), (1, 1), (2, 3), (3, 4)]
        );
        assert_eq!(
            place_recipe_helper((3, 3), (1, 3), 3),
            vec![(0, 1), (1, 4), (2, 7)]
        );
    }

    #[test]
    fn the_overlay_stays_near_its_button_inside_the_panel() {
        // Far enough left, no move; too far right, back by whole buttons.
        assert_eq!(overlay_origin((20, 60), (73, 96), 25.0, 4), (20, 60));
        assert_eq!(overlay_origin((100, 60), (73, 96), 25.0, 4), (25, 60));
        // Too low moves up, by whole buttons.
        assert_eq!(overlay_origin((20, 140), (73, 96), 25.0, 8), (20, 90));
    }

    #[test]
    fn an_overlay_button_lays_its_ingredients_on_seven_unit_cells() {
        let display = RecipeDisplay::Shapeless {
            ingredients: vec![
                SlotDisplay::Item(1),
                SlotDisplay::Empty,
                SlotDisplay::Item(2),
            ],
            result: SlotDisplay::Item(9),
        };
        let slots = overlay_slots(&display, &ItemTags::default());
        let places: Vec<(i32, i32)> = slots.iter().map(|slot| (slot.0, slot.1)).collect();
        assert_eq!(places, [(3, 3), (17, 3)]);
    }

    #[test]
    fn a_button_cycles_recipes_then_their_items() {
        let results = vec![vec![(1, 1), (2, 1)], vec![(3, 1)]];
        // Time steps through the recipes, and each lap through the items.
        let shown: Vec<_> = (0..4).map(|time| button_stack(&results, time)).collect();
        assert_eq!(
            shown,
            [Some((1, 1)), Some((3, 1)), Some((2, 1)), Some((3, 1))]
        );
        assert!(same_display(&[vec![(1, 1)], vec![(1, 1)]]));
        assert!(!same_display(&[vec![(1, 1)], vec![(1, 2)]]));
    }

    #[test]
    fn a_shapeless_ghost_fills_the_grid_in_order() {
        let display = RecipeDisplay::Shapeless {
            ingredients: vec![SlotDisplay::Item(5), SlotDisplay::Item(6)],
            result: SlotDisplay::Item(9),
        };
        let slots = ghost_slots(BookKind::Crafting { size: 3 }, &display, &|_| true);
        let places: Vec<usize> = slots.iter().map(|slot| slot.0).collect();
        assert_eq!(places, [0, 1, 2]);
    }

    #[test]
    fn picker_counts_an_item_once_per_ingredient() {
        let tags = ItemTags::default();
        let two = [Ingredient::Items(vec![5]), Ingredient::Items(vec![5])];
        assert!(stacked(&[(5, 2)]).can_craft(&two, &tags));
        assert!(!stacked(&[(5, 1)]).can_craft(&two, &tags));
    }
}
