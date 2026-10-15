//! The open menu kept in step with the container the server opened and what the world says of it
//! (`AbstractContainerMenu` as the client builds it).

use super::*;

impl PlaySession {
    /// `LoomMenu.slotsChanged`'s list of patterns: with a banner and a dye in,
    /// those that need no item, or those the pattern item in the loom gives.
    /// Nothing once the banner has six layers.
    pub(super) fn loom_patterns(
        world: &photon_world::ClientWorld,
        menu: &photon_game::menus::container::ContainerMenu,
    ) -> (Vec<i32>, usize) {
        let stack = |slot: usize| menu.slots.get(slot).and_then(Option::as_ref);
        let (Some(banner), Some(_dye)) = (stack(0), stack(1)) else {
            return (Vec::new(), 0);
        };
        let layers =
            photon_game::item::values::Values::new(banner.item_registry_id(), banner.patch_bytes())
                .banner_layers()
                .len();
        let tags = world.block_state_registry().banner_pattern_tags();
        let tag = match stack(2) {
            None => "minecraft:no_item_required".to_owned(),
            Some(pattern) => {
                // `provides_banner_patterns` names `pattern_item/<name>`.
                let name = usize::try_from(pattern.item_registry_id().0)
                    .ok()
                    .and_then(|id| photon_data::item::registry::ITEMS.get(id))
                    .map_or("", |item| item.name);
                let short = name
                    .trim_start_matches("minecraft:")
                    .trim_end_matches("_banner_pattern");
                format!("minecraft:pattern_item/{short}")
            }
        };
        (tags.members(&tag).collect(), layers)
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
            if menu.kind == photon_game::menus::container::MenuKind::Loom {
                let (patterns, layers) = Self::loom_patterns(&self.world, menu);
                menu.patterns = patterns;
                menu.banner_layers = layers;
            }
            menu.item_tags = self.world.block_state_registry().item_tags().clone();
            menu.enchantments = enchantments;
            menu.enchantment_traits = self.menus.enchantment_traits.clone();
            menu.lang = self.social.lang.clone();
            if let Some((_, _, entity)) = &mount {
                menu.mount = entity.clone();
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
