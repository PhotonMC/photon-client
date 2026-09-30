//! The recipe book's model (`ClientRecipeBook`, `RecipeCollection` and
//! `RecipeBookComponent`'s state): the recipes the server made known, grouped
//! into collections, which of them fit the open menu and can be crafted from
//! what the player holds, and what a tab, a search and the filter show.

use std::collections::{BTreeMap, BTreeSet};

use voxel_data::item::component_types as ids;
use voxel_world::{
    ClientItemStackState as Stack, Ingredient, RecipeBook, RecipeDisplay, SlotDisplay,
    block::tags::ItemTags,
};

use crate::container_menu::{ContainerMenu, MenuKind, Smelter, max_stack_size};
use crate::item_values::Values;

/// The recipes per page: five columns by four rows (`RecipeBookPage`).
pub const RECIPES_PER_PAGE: usize = 20;

/// The book a menu shows, which decides the recipes it lists and the
/// `RecipeBookType` its settings are kept under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookKind {
    /// The 2×2 grid of the inventory or the 3×3 of a crafting table.
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
            voxel_core::wire::read_varint(&mut input).is_ok_and(|damage| damage > 0)
        });
        if damaged
            || !values.enchantments(ids::ENCHANTMENTS).is_empty()
            || values.has(ids::CUSTOM_NAME)
        {
            return;
        }
        let count = stack.item_count().min(max_stack_size(stack));
        *self.0.entry(stack.item_registry_id()).or_insert(0) += count;
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

/// `ClientRecipeBook.rebuildCollections`: entries of the book's categories
/// grouped by category and group; an ungrouped entry stands alone.
fn build_collections(book: &RecipeBook, kind: BookKind) -> Vec<Collection> {
    let categories = kind.categories();
    let mut grouped: BTreeMap<(i32, Option<i32>, Option<i32>), Vec<i32>> = BTreeMap::new();
    for entry in book.known.values() {
        if !categories.contains(&entry.category) {
            continue;
        }
        let key = match entry.group {
            Some(group) => (entry.category, Some(group), None),
            None => (entry.category, None, Some(entry.id)),
        };
        grouped.entry(key).or_default().push(entry.id);
    }
    let mut all: Vec<Collection> = grouped
        .into_iter()
        .map(|((category, _, _), entries)| Collection {
            category,
            entries,
            selected: BTreeSet::new(),
            craftable: BTreeSet::new(),
        })
        .collect();
    all.sort_by_key(|collection| collection.entries[0]);
    all
}

/// `PlaceRecipeHelper.placeRecipe`: the grid slot of each entry of a recipe
/// `recipe_width` by `recipe_height`, small ones centred in the grid's
/// middle row or column.
pub fn place_recipe_helper(
    grid: (i32, i32),
    recipe: (i32, i32),
    entries: usize,
) -> Vec<(usize, i32)> {
    let (grid_width, grid_height) = grid;
    let (recipe_width, recipe_height) = recipe;
    let mut placed = Vec::new();
    let mut next = 0;
    let mut grid_index = 0;
    let mut y = 0;
    while y < grid_height {
        let centre_y = (recipe_height as f32) < grid_height as f32 / 2.0;
        let start_y = (grid_height as f32 / 2.0 - recipe_height as f32 / 2.0).floor() as i32;
        if centre_y && start_y > y {
            grid_index += grid_width;
            y += 1;
        }
        for x in 0..grid_width {
            if next >= entries {
                return placed;
            }
            let centre_x = (recipe_width as f32) < grid_width as f32 / 2.0;
            let start_x = (grid_width as f32 / 2.0 - recipe_width as f32 / 2.0).floor() as i32;
            let mut total = recipe_width;
            let mut add = x < recipe_width;
            if centre_x {
                total = start_x + recipe_width;
                add = start_x <= x && x < start_x + recipe_width;
            }
            if add {
                placed.push((next, grid_index));
                next += 1;
            } else if total == x {
                grid_index += grid_width - x;
                break;
            }
            grid_index += 1;
        }
        y += 1;
    }
    placed
}

/// A ghost recipe's slots: the menu slot, what it shows and whether it is
/// the result (`fillGhostRecipe`). The furnace's fuel shows only over an
/// empty slot.
pub fn ghost_slots(
    kind: BookKind,
    display: &RecipeDisplay,
    slot_empty: &dyn Fn(usize) -> bool,
) -> Vec<(usize, SlotDisplay, bool)> {
    let mut slots = vec![(0, display.result().clone(), true)];
    match (kind, display) {
        (
            BookKind::Crafting { size },
            RecipeDisplay::Shaped {
                width,
                height,
                ingredients,
                ..
            },
        ) => {
            for (entry, grid) in
                place_recipe_helper((size, size), (*width, *height), ingredients.len())
            {
                slots.push((grid as usize + 1, ingredients[entry].clone(), false));
            }
        }
        (BookKind::Crafting { size }, RecipeDisplay::Shapeless { ingredients, .. }) => {
            // `CraftingRecipeBookComponent.fillGhostRecipe`: the i-th
            // ingredient goes to the i-th slot of the grid.
            let count = ingredients.len().min((size * size) as usize);
            for (index, ingredient) in ingredients.iter().take(count).enumerate() {
                slots.push((index + 1, ingredient.clone(), false));
            }
        }
        (
            BookKind::Furnace(_),
            RecipeDisplay::Furnace {
                ingredient,
                fuel,
                result,
            },
        ) => {
            slots = vec![(2, result.clone(), true), (0, ingredient.clone(), false)];
            if slot_empty(1) {
                slots.push((1, fuel.clone(), false));
            }
        }
        _ => {}
    }
    slots
}

/// `OverlayRecipeComponent.init`'s place for the overlay: where the button
/// is, the middle of the panel and the button's width decide how far the
/// overlay is moved left and up or down to stay near it. `total` recipes.
pub fn overlay_origin(
    button: (i32, i32),
    (center_x, center_y): (i32, i32),
    button_width: f32,
    total: usize,
) -> (i32, i32) {
    let max_row = if total <= 16 { 4 } else { 5 };
    let rows = total.div_ceil(max_row);
    let (mut x, mut y) = button;
    let right = x as f32 + total.min(max_row) as f32 * 25.0;
    let max_left = (center_x + 50) as f32;
    if right > max_left {
        x = (x as f32 - button_width * ((right - max_left) / button_width).trunc()) as i32;
    }
    let bottom = y as f32 + rows as f32 * 25.0;
    let max_bottom = (center_y + 50) as f32;
    if bottom > max_bottom {
        y = (y as f32 - button_width * ((bottom - max_bottom) / button_width).ceil()) as i32;
    }
    let top = y as f32;
    let max_top = (center_y - 100) as f32;
    if top < max_top {
        y = (y as f32 - button_width * ((top - max_top) / button_width).ceil()) as i32;
    }
    (x, y)
}

/// One ingredient of an overlay button: where it sits in the button and what
/// it shows.
pub type OverlaySlot = (i32, i32, SlotDisplay);

/// The ingredients an overlay button draws (`calculateIngredientsPositions`):
/// a crafting recipe on a 3 by 3 grid of 7-unit cells, a furnace recipe's
/// ingredient in the middle one.
pub fn overlay_slots(display: &RecipeDisplay, tags: &ItemTags) -> Vec<OverlaySlot> {
    let cell = |x: i32, y: i32| (3 + x * 7, 3 + y * 7);
    let shown = |slot: &SlotDisplay| !display_stacks(slot, tags).is_empty();
    match display {
        RecipeDisplay::Shaped {
            width,
            height,
            ingredients,
            ..
        } => place_recipe_helper((3, 3), (*width, *height), ingredients.len())
            .into_iter()
            .filter(|(entry, _)| shown(&ingredients[*entry]))
            .map(|(entry, grid)| {
                let (x, y) = cell(grid % 3, grid / 3);
                (x, y, ingredients[entry].clone())
            })
            .collect(),
        RecipeDisplay::Shapeless { ingredients, .. } => ingredients
            .iter()
            .enumerate()
            .filter(|(_, slot)| shown(slot))
            .map(|(index, slot)| {
                let (x, y) = cell(index as i32 % 3, index as i32 / 3);
                (x, y, slot.clone())
            })
            .collect(),
        RecipeDisplay::Furnace { ingredient, .. } if shown(ingredient) => {
            let (x, y) = cell(1, 1);
            vec![(x, y, ingredient.clone())]
        }
        _ => Vec::new(),
    }
}

/// What a button shows for a recipe and how it cycles
/// (`RecipeButton.getDisplayStack`): `results` holds each recipe's stacks as
/// (item, count); `index` counts up with time.
pub fn button_stack(results: &[Vec<(i32, i32)>], index: usize) -> Option<(i32, i32)> {
    let count = results.len();
    if count == 0 {
        return None;
    }
    let offset = index / count;
    let stacks = &results[index - count * offset];
    (!stacks.is_empty()).then(|| stacks[offset % stacks.len()])
}

/// `allRecipesHaveSameResultDisplay`: every stack of every recipe is the same.
pub fn same_display(results: &[Vec<(i32, i32)>]) -> bool {
    let mut all = results.iter().flatten();
    all.next()
        .is_none_or(|first| results.iter().flatten().all(|stack| stack == first))
}

/// The items a display can show, which a ghost slot and a button cycle
/// through (`SlotDisplay.resolveForStacks`). Displays that need more of the
/// world than the client keeps (fuels, potions) show nothing.
pub fn display_items(display: &SlotDisplay, tags: &ItemTags) -> Vec<i32> {
    display_stacks(display, tags)
        .into_iter()
        .map(|(item, _)| item)
        .collect()
}

/// The stacks a display can show, as (item, count).
pub fn display_stacks(display: &SlotDisplay, tags: &ItemTags) -> Vec<(i32, i32)> {
    match display {
        SlotDisplay::Item(item) => vec![(*item, 1)],
        SlotDisplay::ItemStack(item, count) => vec![(*item, *count)],
        SlotDisplay::Tag(tag) => tags.members(tag).map(|item| (item, 1)).collect(),
        SlotDisplay::Composite(all) => all
            .iter()
            .flat_map(|one| display_stacks(one, tags))
            .collect(),
        SlotDisplay::WithRemainder(input, _)
        | SlotDisplay::OnlyWithComponent(input, _)
        | SlotDisplay::WithAnyPotion(input) => display_stacks(input, tags),
        SlotDisplay::Dyed(_, target) => display_stacks(target, tags),
        SlotDisplay::SmithingTrim(base, _, _) => display_stacks(base, tags),
        // `AnyFuel.resolveForStacks`: every item that burns (in registry
        // order; the reference lists them in the order its table was built).
        SlotDisplay::AnyFuel => voxel_data::item::registry::ITEMS
            .iter()
            .enumerate()
            .filter(|(_, item)| item.burn_time > 0)
            .map(|(id, _)| (id as i32, 1))
            .collect(),
        SlotDisplay::Empty => Vec::new(),
    }
}

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
