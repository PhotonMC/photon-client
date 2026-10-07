use super::*;

impl ClientWorld {
    /// The items a recipe property set accepts (`RecipePropertySet.test`);
    /// none for a set the server has not sent.
    pub fn recipe_property_set(&self, set: &str) -> Arc<BTreeSet<i32>> {
        self.recipe_property_sets
            .get(set)
            .cloned()
            .unwrap_or_default()
    }

    pub fn apply_local_player_carried_item(&mut self, item: Option<&ClientItemStackState>) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if let Some(container) = player.open_container.as_mut() {
            container.carried_item = item.cloned();
        } else {
            player.carried_item = item.cloned();
        }
        true
    }
}
