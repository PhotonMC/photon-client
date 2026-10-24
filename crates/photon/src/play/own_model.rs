use super::*;

impl Play {
    /// A frame begins for the skins, the local player's among them.
    pub(super) fn begin_skins(&mut self) {
        self.skins.begin_frame();
        self.wear_own_skin();
    }

    /// A skin written into the textures is there before the frame draws.
    pub(super) fn upload_skins(&mut self) -> Result<()> {
        let written = self.skins.take_dirty();
        if !written.is_empty() {
            self.backend.update_entity_texture(&written)?;
        }
        Ok(())
    }

    /// `PlayerInfo.createSkinLookup` for the local player, which needs no signature: the skin
    /// its model, hands and inventory figure are drawn in.
    pub(super) fn wear_own_skin(&mut self) {
        let Some(id) = self.session.profile_id() else {
            return;
        };
        let who = crate::skins::Who {
            id,
            properties: self.session.player_properties(id),
            require_secure: false,
        };
        let Some(crate::skins::Resolved { rect, slim, .. }) =
            self.skins.resolve(&mut self.entity_textures, &who)
        else {
            return;
        };
        if self.player_model.slim != slim {
            self.player_model = photon_render::entity::player_model::PlayerModel::new(slim);
        }
        if self.skin_rect != rect {
            self.skin_rect = rect;
            self.gui.set_player_skin(rect);
        }
    }

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
