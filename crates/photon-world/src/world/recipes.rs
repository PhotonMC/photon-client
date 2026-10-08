use crate::*;

impl ClientWorld {
    /// `handleUpdateRecipes`: the items each recipe property set accepts, by
    /// the set's name (`minecraft:furnace_input`).
    pub fn apply_recipe_property_sets(&mut self, sets: Vec<(String, Vec<i32>)>) {
        self.recipe_property_sets = sets
            .into_iter()
            .map(|(name, items)| (name, Arc::new(items.into_iter().collect())))
            .collect();
    }

    pub fn recipe_book(&self) -> &RecipeBook {
        &self.recipe_book
    }

    pub fn recipe_book_mut(&mut self) -> &mut RecipeBook {
        &mut self.recipe_book
    }

    /// `handleUpdateRecipes`'s stonecutter half.
    pub fn apply_stonecutter_recipes(&mut self, entries: Vec<StonecutterEntry>) {
        self.stonecutter = Arc::new(entries);
    }

    /// `SingleInputSet.acceptsInput`, as the items it accepts: every entry's
    /// items, and the members of the tags it names.
    pub fn stonecutter_inputs(&self) -> BTreeSet<i32> {
        let tags = self.block_state_registry.item_tags();
        let mut accepted = BTreeSet::new();
        for entry in self.stonecutter.iter() {
            match &entry.input {
                Ingredient::Items(items) => accepted.extend(items.iter().copied()),
                Ingredient::Tag(tag) => accepted.extend(tags.members(tag)),
            }
        }
        accepted
    }

    /// `SingleInputSet.selectByInput`: what each entry that takes the item
    /// shows, in the server's order (the first item of each display).
    pub fn stonecutter_options(&self, item: i32) -> Vec<i32> {
        let tags = self.block_state_registry.item_tags();
        self.stonecutter
            .iter()
            .filter(|entry| match &entry.input {
                Ingredient::Items(items) => items.contains(&item),
                Ingredient::Tag(tag) => tags.contains(item, tag),
            })
            .map(|entry| entry.display.first_item().unwrap_or(-1))
            .collect()
    }

    /// The items a recipe property set accepts (`RecipePropertySet.test`);
    /// none for a set the server has not sent.
    pub fn recipe_property_set(&self, set: &str) -> Arc<BTreeSet<i32>> {
        self.recipe_property_sets
            .get(set)
            .cloned()
            .unwrap_or_default()
    }
}
