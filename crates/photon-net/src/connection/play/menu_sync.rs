//! The open menu kept in step with the container the server opened and what the world says of it
//! (`AbstractContainerMenu` as the client builds it).

use super::*;

impl PlaySession {
    /// The loom's pattern list, from the tags the server sent.
    fn sync_loom(
        world: &photon_world::ClientWorld,
        menu: &mut photon_game::menus::container::ContainerMenu,
    ) {
        if menu.kind == photon_game::menus::container::MenuKind::Loom {
            let tags = world.block_state_registry().banner_pattern_tags();
            (menu.patterns, menu.banner_layers) =
                photon_game::menus::container::loom_patterns(tags, menu);
        }
    }

    /// The enchantment registry's entries, which a click's component hashes name.
    pub(super) fn enchantment_names(&self) -> std::sync::Arc<Vec<String>> {
        std::sync::Arc::new(self.registry_names("minecraft:enchantment").to_vec())
    }

    /// The menu a container the server opened is: its type's, or the mount screen the server
    /// opened for this container.
    pub(super) fn menu_kind(
        &self,
        container_id: i32,
        menu_type_id: i32,
    ) -> Option<photon_game::menus::container::MenuKind> {
        use photon_game::menus::container::{MOUNT_MENU_TYPE, MenuKind};
        if menu_type_id == MOUNT_MENU_TYPE {
            return self
                .menus
                .mount
                .as_ref()
                .filter(|(id, ..)| *id == container_id)
                .map(|(_, mount, _)| MenuKind::Mount(*mount));
        }
        MenuKind::from_menu_type(menu_type_id)
    }

    /// The menu an open screen is over: the container the server opened, or
    /// the player's own.
    pub fn active_menu(&mut self) -> &photon_game::menus::container::ContainerMenu {
        if self.open_container_menu().is_some() {
            return self.menus.open_menu.as_ref().expect("synced above");
        }
        self.container_menu()
    }

    /// The menu of the container the server opened, as the client holds it
    /// now; `None` until its contents have come.
    pub fn open_container_menu(&mut self) -> Option<&photon_game::menus::container::ContainerMenu> {
        let photon_world::OpenContainerMenu {
            container_id,
            menu_type_id,
            slots,
            carried,
            state_id,
        } = self.world.local_player_container_menu()?;
        let mount = self
            .menus
            .mount
            .clone()
            .filter(|(id, ..)| *id == container_id)
            .filter(|_| menu_type_id == photon_game::menus::container::MOUNT_MENU_TYPE);
        let kind = self.menu_kind(container_id, menu_type_id)?;
        match self.menus.open_menu.as_mut() {
            Some(menu) if menu.container_id == container_id && menu.kind == kind => {
                menu.load(slots, carried, state_id);
            }
            _ => {
                self.menus.open_menu =
                    Some(photon_game::menus::container::ContainerMenu::for_menu(
                        kind,
                        container_id,
                        slots,
                        carried,
                        state_id,
                    ));
            }
        }
        // What a furnace menu reads besides its slots.
        let enchantments = self.enchantment_names();
        if let Some(menu) = self.menus.open_menu.as_mut() {
            menu.accepted = menu
                .kind
                .input_sets()
                .iter()
                .map(|set| self.world.recipe_property_set(set))
                .collect();
            if menu.kind == photon_game::menus::container::MenuKind::Stonecutter {
                menu.accepted = vec![std::sync::Arc::new(self.world.stonecutter_inputs())];
                menu.options = menu
                    .slots
                    .first()
                    .and_then(Option::as_ref)
                    .map(|input| self.world.stonecutter_options(input.item_registry_id()))
                    .unwrap_or_default();
            }
            Self::sync_loom(&self.world, menu);
            menu.item_tags = self.world.block_state_registry().item_tags().clone();
            menu.enchantments = enchantments;
            menu.enchantment_traits = self.menus.enchantment_traits.clone();
            menu.lang = self.social.lang.clone();
            if let Some((_, _, entity)) = &mount {
                menu.mount = Some(entity.clone());
            }
            menu.xp_level = self.local.status.experience_level;
            if menu.kind == photon_game::menus::container::MenuKind::Merchant {
                load_merchant(menu, self.menus.merchant.as_ref());
            }
            if let Some(container) = self
                .world
                .local_player()
                .and_then(|player| player.open_container())
            {
                menu.data = (0..10).map(|index| container.data(index)).collect();
            }
        }
        self.menus.open_menu.as_ref()
    }
}

/// The merchant's offers, for the menu whose container it is; the result is the client's own to
/// make.
fn load_merchant(
    menu: &mut photon_game::menus::container::ContainerMenu,
    merchant: Option<&(i32, photon_game::menus::merchant::MerchantState)>,
) {
    if let Some((_, state)) = merchant.filter(|(id, _)| *id == menu.container_id) {
        menu.offers = std::sync::Arc::new(state.offers.clone());
        menu.trader = state.trader;
    }
    menu.update_sell_item();
}
