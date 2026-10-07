use super::*;

impl PlaySession {
    /// `ServerboundPlaceRecipePacket`: the recipe book asks the server to
    /// move a recipe's ingredients into the open menu's grid.
    pub fn place_recipe(&mut self, recipe: i32, shift: bool) -> Result<(), ConnectionError> {
        let container_id = self.open_container_id();
        self.send(PLAY_SERVERBOUND_PLACE_RECIPE_PACKET_ID, |output| {
            voxel_core::wire::write_varint(container_id, output);
            voxel_core::wire::write_varint(recipe, output);
            output.push(u8::from(shift));
            Ok(())
        })
    }

    /// `ServerboundRecipeBookChangeSettingsPacket`: `book_type` is the
    /// `RecipeBookType` ordinal.
    pub fn recipe_book_settings(
        &mut self,
        book_type: i32,
        open: bool,
        filtering: bool,
    ) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(book_type, output);
                output.push(u8::from(open));
                output.push(u8::from(filtering));
                Ok(())
            },
        )
    }

    /// `RecipeBookComponent.setVisible` and the filter button: the book's
    /// settings change here at once and the server is told.
    pub fn set_recipe_book_settings(
        &mut self,
        book_type: i32,
        open: bool,
        filtering: bool,
    ) -> Result<(), ConnectionError> {
        let book = self.trace.world.recipe_book_mut();
        if let Some(settings) = book.settings.get_mut(book_type as usize) {
            settings.open = open;
            settings.filtering = filtering;
            book.revision += 1;
        }
        self.recipe_book_settings(book_type, open, filtering)
    }

    /// `GhostSlots.clear`.
    pub fn clear_recipe_ghost(&mut self) {
        self.trace.world.recipe_book_mut().ghost = None;
    }

    /// `LocalPlayer.removeRecipeHighlight`: a new recipe that showed on a page
    /// stops being new, and the server is told.
    pub fn mark_recipe_seen(&mut self, recipe: i32) -> Result<(), ConnectionError> {
        if self.trace.world.recipe_book_mut().highlight.remove(&recipe) {
            self.recipe_seen(recipe)?;
        }
        Ok(())
    }

    /// `ServerboundRecipeBookSeenRecipePacket`: a highlighted recipe was seen.
    pub fn recipe_seen(&mut self, recipe: i32) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(recipe, output);
                Ok(())
            },
        )
    }
}
