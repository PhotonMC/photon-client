use super::*;

/// `ClientRecipeBook.rebuildCollections`: entries of the book's categories
/// grouped by category and group; an ungrouped entry stands alone.
pub(super) fn build_collections(book: &RecipeBook, kind: BookKind) -> Vec<Collection> {
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
        SlotDisplay::AnyFuel => photon_data::item::registry::ITEMS
            .iter()
            .enumerate()
            .filter(|(_, item)| item.burn_time > 0)
            .map(|(id, _)| (id as i32, 1))
            .collect(),
        SlotDisplay::Empty => Vec::new(),
    }
}
