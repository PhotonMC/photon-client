use super::*;

impl PlaySession {
    /// `CrafterScreen.updateSlotState`: a slot of the crafter's grid is switched on or off, in the
    /// menu at once and by `ServerboundContainerSlotStateChangedPacket`.
    pub fn set_crafter_slot_state(
        &mut self,
        slot: i32,
        enabled: bool,
    ) -> Result<(), ConnectionError> {
        let Some(container_id) = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        let value = i16::from(!enabled);
        self.trace
            .world
            .apply_local_player_container_data(container_id, slot as i16, value);
        if let Some(entry) = self
            .open_menu
            .as_mut()
            .and_then(|menu| menu.data.get_mut(slot as usize))
        {
            *entry = value;
        }
        self.send(
            PLAY_SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID,
            |output| {
                output.extend_from_slice(&encode_container_slot_state(slot, container_id, enabled));
                Ok(())
            },
        )
    }

    /// `ServerboundSetBeaconPacket`: the effects the beacon's screen chose, by registry id.
    pub fn set_beacon(
        &mut self,
        primary: Option<i32>,
        secondary: Option<i32>,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_BEACON_PACKET_ID, |output| {
            output.extend_from_slice(&encode_set_beacon(primary, secondary));
            Ok(())
        })
    }

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
                let name = usize::try_from(pattern.item_registry_id())
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

    /// `BundleItem.playInsertSound` and the like: `LocalPlayer.playSound`
    /// where the player is, pitched by the level's random but for a failure.
    fn play_bundle_sound(&mut self, event: &'static str) {
        use photon_game::menus::container::BUNDLE_INSERT_FAIL;
        use photon_world::local_player::{LocalSound, LocalSoundSource};
        let Some(position) = self
            .trace
            .world
            .local_player()
            .map(|player| player.position())
        else {
            return;
        };
        let (volume, pitch) = if event == BUNDLE_INSERT_FAIL {
            (1.0, 1.0)
        } else {
            (0.8, 0.8 + self.level_random.next_float() * 0.4)
        };
        self.trace.world.push_sound(LocalSound {
            event,
            source: LocalSoundSource::Players,
            volume,
            pitch,
            position,
        });
    }

    /// The enchantments (network ids) with `prevent_armor_change`, which the
    /// data pack says and the session cannot read itself.
    pub fn set_locking_enchantments(&mut self, ids: Vec<i32>) {
        self.locking_enchantments = std::sync::Arc::new(ids);
    }

    /// The enchantment registry's entries, which a click's component hashes name.
    fn enchantment_names(&self) -> std::sync::Arc<Vec<String>> {
        std::sync::Arc::new(self.registry_names("minecraft:enchantment").to_vec())
    }

    /// `AbstractContainerScreen.slotClicked` into the open menu: the player's
    /// own, or the container the server opened. The click as the client
    /// predicts it, the arm swings its drops cause, then the click packet.
    pub fn container_click(
        &mut self,
        slot: i32,
        button: i32,
        kind: photon_game::menus::container::ClickType,
    ) -> Result<(), ConnectionError> {
        let player = photon_game::menus::container::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == photon_world::ClientPlayerGameMode::Creative,
        };
        let opened = self
            .trace
            .world
            .local_player()
            .is_some_and(|player| player.open_container().is_some());
        let sounds: Vec<&'static str>;
        let (swings, body) = if opened {
            // Synced before the click; `None` when the container is of a kind
            // with no menu here, which no screen sends clicks for.
            if self.open_container_menu().is_none() {
                return Ok(());
            }
            let menu = self.open_menu.as_mut().expect("synced above");
            let clicked = menu.handle_click(slot, button, kind, player);
            sounds = menu.sounds().to_vec();
            let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
            self.trace
                .world
                .set_local_player_container_menu(&slots, carried);
            clicked
        } else {
            let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
                return Ok(());
            };
            self.container_menu.load(slots, carried, state_id);
            self.container_menu.enchantments = self.enchantment_names();
            self.container_menu.locking_enchantments = self.locking_enchantments.clone();
            let clicked = self.container_menu.handle_click(slot, button, kind, player);
            sounds = self.container_menu.sounds().to_vec();
            let menu = &self.container_menu;
            self.trace
                .world
                .set_local_player_menu(&menu.slots, menu.carried.clone());
            clicked
        };
        for event in sounds {
            self.play_bundle_sound(event);
        }
        for _ in 0..swings {
            // `LocalPlayer.swing`: the arm, then the packet.
            if let Some(player) = self.player.as_mut() {
                player.swing();
            }
            self.swings += 1;
            self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
                output.push(0);
                Ok(())
            })?;
        }
        self.send(PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID, |output| {
            output.extend_from_slice(&body);
            Ok(())
        })
    }

    /// `CreativeModeInventoryScreen.slotClicked` into the player's own menu:
    /// the click as the client predicts it, then what it sends for it.
    pub fn creative_click(
        &mut self,
        creative: &mut photon_game::menus::creative::CreativeMenu,
        click: photon_game::menus::creative::Click,
    ) -> Result<(), ConnectionError> {
        use photon_game::menus::creative::Effect;
        let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
            return Ok(());
        };
        self.container_menu.load(slots, carried, state_id);
        let player = photon_game::menus::container::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == photon_world::ClientPlayerGameMode::Creative,
        };
        // `Player.canDropItems`: under the throttler's threshold.
        let can_drop = self.drop_spam < 1280;
        let effects = creative.slot_clicked(click, &mut self.container_menu, player, can_drop);
        let menu = &self.container_menu;
        self.trace
            .world
            .set_local_player_menu(&menu.slots, menu.carried.clone());
        if !player.infinite_materials {
            return Ok(());
        }
        for effect in effects {
            match effect {
                Effect::Swing => {
                    if let Some(player) = self.player.as_mut() {
                        player.swing();
                    }
                    self.swings += 1;
                    self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
                        output.push(0);
                        Ok(())
                    })?;
                }
                Effect::SetSlot(slot, stack) => {
                    self.send(
                        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                        |output| {
                            encode_creative_slot(output, slot, stack.as_ref());
                            Ok(())
                        },
                    )?;
                }
                Effect::Drop(stack) => {
                    self.send(
                        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                        |output| {
                            encode_creative_slot(output, -1, Some(&stack));
                            Ok(())
                        },
                    )?;
                    self.drop_spam += 20;
                }
            }
        }
        Ok(())
    }

    /// `LocalPlayer.closeContainer` for the inventory: the close packet,
    /// then `ContainerMenu.removed`, which empties the result slot.
    pub fn close_inventory(&mut self) -> Result<(), ConnectionError> {
        let opened = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id());
        let id = opened.unwrap_or(0);
        self.send(PLAY_SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, |output| {
            photon_core::wire::write_varint(id, output);
            Ok(())
        })?;
        if opened.is_some() {
            // `LocalPlayer.clientSideCloseContainer`: back to the player's menu.
            self.trace.world.apply_local_player_container_close(id);
            self.open_menu = None;
            return Ok(());
        }
        if let Some((mut slots, carried, _)) = self.trace.world.local_player_menu() {
            slots[0] = None;
            self.trace.world.set_local_player_menu(&slots, carried);
        }
        Ok(())
    }

    /// The hands whose stack a use spent since the last call.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        self.game_mode_state.take_items_used()
    }

    /// Whether the server has opened a container this client has no menu for.
    pub fn open_container_unsupported(&self) -> bool {
        self.trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .is_some_and(|container| {
                photon_game::menus::container::MenuKind::from_menu_type(container.menu_type_id())
                    .is_none()
            })
    }

    /// The menu an open screen is over: the container the server opened, or
    /// the player's own.
    pub fn active_menu(&mut self) -> &photon_game::menus::container::ContainerMenu {
        if self.open_container_menu().is_some() {
            return self.open_menu.as_ref().expect("synced above");
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
        } = self.trace.world.local_player_container_menu()?;
        let kind = photon_game::menus::container::MenuKind::from_menu_type(menu_type_id)?;
        match self.open_menu.as_mut() {
            Some(menu) if menu.container_id == container_id && menu.kind == kind => {
                menu.load(slots, carried, state_id);
            }
            _ => {
                self.open_menu = Some(photon_game::menus::container::ContainerMenu::for_menu(
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
        if let Some(menu) = self.open_menu.as_mut() {
            menu.accepted = menu
                .kind
                .input_sets()
                .iter()
                .map(|set| self.trace.world.recipe_property_set(set))
                .collect();
            if menu.kind == photon_game::menus::container::MenuKind::Stonecutter {
                menu.accepted = vec![std::sync::Arc::new(self.trace.world.stonecutter_inputs())];
                menu.options = menu
                    .slots
                    .first()
                    .and_then(Option::as_ref)
                    .map(|input| {
                        self.trace
                            .world
                            .stonecutter_options(input.item_registry_id())
                    })
                    .unwrap_or_default();
            }
            if menu.kind == photon_game::menus::container::MenuKind::Loom {
                let (patterns, layers) = Self::loom_patterns(&self.trace.world, menu);
                menu.patterns = patterns;
                menu.banner_layers = layers;
            }
            menu.item_tags = self.trace.world.block_state_registry().item_tags().clone();
            menu.enchantments = enchantments;
            menu.locking_enchantments = self.locking_enchantments.clone();
            menu.xp_level = self.status.experience_level;
            if menu.kind == photon_game::menus::container::MenuKind::Merchant {
                // The merchant's offers; the result is the client's own to make.
                if let Some((_, state)) = self
                    .merchant
                    .as_ref()
                    .filter(|(id, _)| *id == menu.container_id)
                {
                    menu.offers = std::sync::Arc::new(state.offers.clone());
                    menu.trader = state.trader;
                }
                menu.update_sell_item();
            }
            if let Some(container) = self
                .trace
                .world
                .local_player()
                .and_then(|player| player.open_container())
            {
                menu.data = (0..10).map(|index| container.data(index)).collect();
            }
        }
        self.open_menu.as_ref()
    }

    /// `MultiPlayerGameMode.handleInventoryButtonClick`: a button of the open
    /// menu (a stonecutter recipe, a loom pattern, an enchantment) pressed.
    pub fn container_button(&mut self, button: i32) -> Result<(), ConnectionError> {
        let Some(container_id) = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        // `StonecutterMenu.clickMenuButton`: the entry is chosen, and the
        // result, which only the server can make, is emptied.
        let stonecutter = self
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == photon_game::menus::container::MenuKind::Stonecutter);
        let loom = self
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == photon_game::menus::container::MenuKind::Loom);
        if loom {
            // `LoomMenu.clickMenuButton`: the pattern is chosen.
            self.trace
                .world
                .apply_local_player_container_data(container_id, 0, button as i16);
        }
        if stonecutter {
            self.trace
                .world
                .apply_local_player_container_data(container_id, 0, button as i16);
            if let Some(photon_world::OpenContainerMenu {
                mut slots, carried, ..
            }) = self.trace.world.local_player_container_menu()
            {
                slots[1] = None;
                self.trace
                    .world
                    .set_local_player_container_menu(&slots, carried);
            }
        }
        self.send(
            PLAY_SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID,
            |output| {
                photon_core::wire::write_varint(container_id, output);
                photon_core::wire::write_varint(button, output);
                Ok(())
            },
        )
    }

    /// The open menu's container id; 0 is the player's own.
    pub(super) fn open_container_id(&self) -> i32 {
        self.trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map_or(0, |container| container.container_id())
    }

    /// `ServerboundRenameItemPacket`: the anvil's name box changed.
    pub fn rename_item(&mut self, name: &str) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_RENAME_ITEM_PACKET_ID, |output| {
            photon_core::wire::write_varint(name.len() as i32, output);
            output.extend_from_slice(name.as_bytes());
            Ok(())
        })
    }

    /// `MerchantScreen.postButtonClick`: the chosen trade is tried first and its payments moved
    /// in, and the server is told.
    pub fn select_trade(&mut self, trade: i32) -> Result<(), ConnectionError> {
        let Some(menu) = self
            .open_menu
            .as_mut()
            .filter(|menu| menu.kind == photon_game::menus::container::MenuKind::Merchant)
        else {
            return Ok(());
        };
        let index = usize::try_from(trade).unwrap_or(0);
        menu.selection_hint = index;
        menu.update_sell_item();
        menu.try_move_items(index);
        let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
        self.trace
            .world
            .set_local_player_container_menu(&slots, carried);
        self.send(PLAY_SERVERBOUND_SELECT_TRADE_PACKET_ID, |output| {
            output.extend_from_slice(&encode_select_trade(trade));
            Ok(())
        })
    }

    /// The breaking state, for drawing the crack and reporting.
    pub fn game_mode_state(&self) -> &photon_game::game_mode::GameMode {
        &self.game_mode_state
    }

    /// The player's inventory menu as the client holds it now.
    pub fn container_menu(&mut self) -> &photon_game::menus::container::ContainerMenu {
        if let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() {
            self.container_menu.load(slots, carried, state_id);
        }
        &self.container_menu
    }

    /// The container the server opened, as a screen needs to know it: its id,
    /// the menu it is and its title as NBT. `None` with none open, or one of
    /// a kind not modelled yet.
    pub fn open_container_info(
        &self,
    ) -> Option<(i32, photon_game::menus::container::MenuKind, Vec<u8>)> {
        let container = self.trace.world.local_player()?.open_container()?;
        let kind =
            photon_game::menus::container::MenuKind::from_menu_type(container.menu_type_id())?;
        // A screen opens once the server has sent what is in the container.
        self.trace.world.local_player_container_menu()?;
        Some((
            container.container_id(),
            kind,
            container.title_nbt().to_vec(),
        ))
    }
}
