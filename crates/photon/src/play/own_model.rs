use super::*;

impl Play {
    /// The local player's own model with what it wears and holds (an invisible one keeps those,
    /// not its body).
    pub(super) fn own_model(
        &mut self,
        view: overlays::FrameView,
        held: &mut Vec<crate::equipment::held_items::HeldItem>,
        invisible: bool,
    ) -> Result<Vec<photon_render::scene::Vertex>> {
        let player = self.session.player().ok_or("no local player")?;
        // The selected hotbar slot and the off hand, the main arm being the right.
        let stack = |slot: i16| {
            self.session
                .world()
                .local_player()
                .and_then(|state| state.inventory_slot(slot))
                .filter(|stack| stack.item_count() > 0)
        };
        let selected = self.session.status().selected_slot as i16;
        Ok(overlays::player_model(
            (&self.session, player),
            &mut self.player_model,
            (view, held),
            (stack(36 + selected), stack(45)),
            (
                &mut self.armor,
                Worn {
                    head: stack(5),
                    chest: stack(6),
                    legs: stack(7),
                    feet: stack(8),
                },
                &self.entity_textures,
            ),
            invisible,
        ))
    }
}
