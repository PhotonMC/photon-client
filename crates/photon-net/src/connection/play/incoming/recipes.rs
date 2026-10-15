use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
        typed::<RecipeBookAdd>,
    ),
    (
        PLAY_CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID,
        typed::<RecipeBookRemove>,
    ),
    (
        PLAY_CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID,
        typed::<RecipeBookSettings>,
    ),
    (
        PLAY_CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID,
        typed::<PlaceGhostRecipe>,
    ),
    (
        PLAY_CLIENTBOUND_UPDATE_RECIPES_PACKET_ID,
        typed::<UpdateRecipes>,
    ),
];

impl ClientboundPacket for RecipeBookAdd {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let (entries, replace) = self.0;
        session.trace.world.recipe_book_mut().add(entries, replace);
        Ok(())
    }
}

impl ClientboundPacket for RecipeBookRemove {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.trace.world.recipe_book_mut().remove(&self.0);
        Ok(())
    }
}

impl ClientboundPacket for RecipeBookSettings {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.trace.world.recipe_book_mut().set_settings(self.0);
        Ok(())
    }
}

impl ClientboundPacket for PlaceGhostRecipe {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.trace.world.recipe_book_mut().ghost = Some(self.0);
        Ok(())
    }
}

impl ClientboundPacket for UpdateRecipes {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let update = self.0;
        session.trace.world.apply_recipe_property_sets(update.sets);
        session
            .trace
            .world
            .apply_stonecutter_recipes(update.stonecutter);
        Ok(())
    }
}
