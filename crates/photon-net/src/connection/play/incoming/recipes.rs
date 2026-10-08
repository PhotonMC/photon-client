use super::*;

impl PlaySession {
    pub(super) fn on_recipe_book_add(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_recipe_book_add_packet(payload) {
            Ok((entries, replace)) => {
                self.trace.world.recipe_book_mut().add(entries, replace);
            }
            Err(error) => self.warn(format!("recipe book add: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_recipe_book_remove(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_recipe_book_remove_packet(payload) {
            Ok(ids) => self.trace.world.recipe_book_mut().remove(&ids),
            Err(error) => self.warn(format!("recipe book remove: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_recipe_book_settings(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        match parse_recipe_book_settings_packet(payload) {
            Ok(settings) => self.trace.world.recipe_book_mut().set_settings(settings),
            Err(error) => self.warn(format!("recipe book settings: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_place_ghost_recipe(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_place_ghost_recipe_packet(payload) {
            Ok(ghost) => self.trace.world.recipe_book_mut().ghost = Some(ghost),
            Err(error) => self.warn(format!("place ghost recipe: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_update_recipes(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_update_recipes_packet(payload) {
            Ok(update) => {
                self.trace.world.apply_recipe_property_sets(update.sets);
                self.trace
                    .world
                    .apply_stonecutter_recipes(update.stonecutter);
            }
            Err(error) => self.warn(format!("recipes: {error:?}")),
        }
        Ok(())
    }
}
