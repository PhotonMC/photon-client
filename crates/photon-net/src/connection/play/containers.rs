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
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        let value = i16::from(!enabled);
        self.world
            .apply_local_player_container_data(container_id, slot as i16, value);
        if let Some(entry) = self
            .menus
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

    /// `BundleItem.playInsertSound` and the like: `LocalPlayer.playSound`
    /// where the player is, pitched by the level's random but for a failure.
    fn play_bundle_sound(&mut self, event: &'static str) {
        use photon_game::menus::container::BUNDLE_INSERT_FAIL;
        use photon_world::local_player::{LocalSound, LocalSoundSource};
        let Some(position) = self.world.local_player().map(|player| player.position()) else {
            return;
        };
        let (volume, pitch) = if event == BUNDLE_INSERT_FAIL {
            (1.0, 1.0)
        } else {
            (0.8, 0.8 + self.effects.level_random.next_float() * 0.4)
        };
        self.world.push_sound(LocalSound {
            event,
            source: LocalSoundSource::Players,
            volume,
            pitch,
            position,
        });
    }

    /// The language the anvil compares item names in.
    pub fn set_lang(&mut self, lang: std::sync::Arc<photon_game::text::Lang>) {
        self.social.lang = lang;
    }

    /// What menus read of the enchantments (curses, armour locks), which
    /// the data pack says and the session cannot read itself.
    pub fn set_enchantment_traits(
        &mut self,
        traits: Vec<photon_game::menus::container::EnchantmentTraits>,
    ) {
        self.menus.enchantment_traits = std::sync::Arc::new(traits);
    }

    /// What a click reads of the player: the selected hotbar slot, and whether materials are
    /// infinite.
    fn click_player(&self) -> photon_game::menus::container::Player {
        photon_game::menus::container::Player {
            selected: self.local.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.local.game_mode
                == photon_world::ClientPlayerGameMode::Creative,
        }
    }

    /// `LocalPlayer.swing`: the arm, then the packet.
    fn swing_hand(&mut self) -> Result<(), ConnectionError> {
        if let Some(player) = self.local.player.as_mut() {
            player.swing();
        }
        self.local.swings += 1;
        self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
            output.push(0);
            Ok(())
        })
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
        let player = self.click_player();
        let opened = self
            .world
            .local_player()
            .is_some_and(|player| player.open_container().is_some());
        // `BundleMouseActions.onSlotClicked`: a shift click or a number key on a bundle selects
        // none of its items first.
        if matches!(
            kind,
            photon_game::menus::container::ClickType::QuickMove
                | photon_game::menus::container::ClickType::Swap
        ) && usize::try_from(slot).is_ok_and(|index| self.active_menu().is_bundle_slot(index))
        {
            self.select_bundle_item(slot, -1)?;
        }
        let sounds: Vec<&'static str>;
        let (swings, body) = if opened {
            // Synced before the click; `None` when the container is of a kind
            // with no menu here, which no screen sends clicks for.
            if self.open_container_menu().is_none() {
                return Ok(());
            }
            let menu = self.menus.open_menu.as_mut().expect("synced above");
            let clicked = menu.handle_click(slot, button, kind, player);
            sounds = menu.sounds().to_vec();
            let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
            self.world.set_local_player_container_menu(&slots, carried);
            clicked
        } else {
            let Some((slots, carried, state_id)) = self.world.local_player_menu() else {
                return Ok(());
            };
            self.menus.container_menu.load(slots, carried, state_id);
            self.menus.container_menu.enchantments = self.enchantment_names();
            self.menus.container_menu.enchantment_traits = self.menus.enchantment_traits.clone();
            self.menus.container_menu.lang = self.social.lang.clone();
            let clicked = self
                .menus
                .container_menu
                .handle_click(slot, button, kind, player);
            sounds = self.menus.container_menu.sounds().to_vec();
            let menu = &self.menus.container_menu;
            self.world
                .set_local_player_menu(&menu.slots, menu.carried.clone());
            clicked
        };
        for event in sounds {
            self.play_bundle_sound(event);
        }
        for _ in 0..swings {
            self.swing_hand()?;
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
        let Some((slots, carried, state_id)) = self.world.local_player_menu() else {
            return Ok(());
        };
        self.menus.container_menu.load(slots, carried, state_id);
        let player = self.click_player();
        // `Player.canDropItems`: under the throttler's threshold.
        let can_drop = self.local.drop_spam < 1280;
        let effects =
            creative.slot_clicked(click, &mut self.menus.container_menu, player, can_drop);
        let menu = &self.menus.container_menu;
        self.world
            .set_local_player_menu(&menu.slots, menu.carried.clone());
        if !player.infinite_materials {
            return Ok(());
        }
        for effect in effects {
            match effect {
                Effect::Swing => self.swing_hand()?,
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
                    self.local.drop_spam += 20;
                }
            }
        }
        Ok(())
    }

    /// `LocalPlayer.closeContainer` for the inventory: the close packet,
    /// then `ContainerMenu.removed`, which empties the result slot.
    pub fn close_inventory(&mut self) -> Result<(), ConnectionError> {
        let opened = self
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
            self.world.apply_local_player_container_close(id);
            self.menus.open_menu = None;
            return Ok(());
        }
        if let Some((mut slots, carried, _)) = self.world.local_player_menu() {
            slots[0] = None;
            self.world.set_local_player_menu(&slots, carried);
        }
        Ok(())
    }

    /// The hands whose stack a use spent since the last call.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        self.local.game_mode_state.take_items_used()
    }

    /// Whether the server has opened a container this client has no menu for.
    pub fn open_container_unsupported(&self) -> bool {
        self.world
            .local_player()
            .and_then(|player| player.open_container())
            .is_some_and(|container| {
                self.menu_kind(container.container_id(), container.menu_type_id())
                    .is_none()
            })
    }

    /// `MultiPlayerGameMode.handleInventoryButtonClick`: a button of the open
    /// menu (a stonecutter recipe, a loom pattern, an enchantment) pressed.
    pub fn container_button(&mut self, button: i32) -> Result<(), ConnectionError> {
        let Some(container_id) = self
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        // `StonecutterMenu.clickMenuButton`: the entry is chosen, and the
        // result, which only the server can make, is emptied.
        let stonecutter =
            self.menus.open_menu.as_ref().is_some_and(|menu| {
                menu.kind == photon_game::menus::container::MenuKind::Stonecutter
            });
        let loom = self
            .menus
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == photon_game::menus::container::MenuKind::Loom);
        if loom {
            // `LoomMenu.clickMenuButton`: the pattern is chosen.
            self.world
                .apply_local_player_container_data(container_id, 0, button as i16);
        }
        if stonecutter {
            self.world
                .apply_local_player_container_data(container_id, 0, button as i16);
            if let Some(photon_world::OpenContainerMenu {
                mut slots, carried, ..
            }) = self.world.local_player_container_menu()
            {
                slots[1] = None;
                self.world.set_local_player_container_menu(&slots, carried);
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
        self.world
            .local_player()
            .and_then(|player| player.open_container())
            .map_or(0, |container| container.container_id())
    }

    /// `ServerboundChangeGameModePacket`: the game mode switcher picked a mode.
    pub fn change_game_mode(&mut self, mode: ClientPlayerGameMode) -> Result<(), ConnectionError> {
        let id = match mode {
            ClientPlayerGameMode::Survival => 0,
            ClientPlayerGameMode::Creative => 1,
            ClientPlayerGameMode::Adventure => 2,
            ClientPlayerGameMode::Spectator => 3,
        };
        self.send(PLAY_SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID, |output| {
            photon_core::wire::write_varint(id, output);
            Ok(())
        })
    }

    /// `ServerboundRenameItemPacket`: the anvil's name box changed.
    pub fn rename_item(&mut self, name: &str) -> Result<(), ConnectionError> {
        // `AnvilScreen.onNameChanged`: the menu works its result out again.
        if let Some(menu) = self
            .menus
            .open_menu
            .as_mut()
            .filter(|menu| menu.kind == photon_game::menus::container::MenuKind::Anvil)
        {
            menu.set_item_name(name);
        }
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
            .menus
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
        self.world.set_local_player_container_menu(&slots, carried);
        self.send(PLAY_SERVERBOUND_SELECT_TRADE_PACKET_ID, |output| {
            output.extend_from_slice(&encode_select_trade(trade));
            Ok(())
        })
    }

    /// The breaking state, for drawing the crack and reporting.
    pub fn game_mode_state(&self) -> &photon_game::game_mode::GameMode {
        &self.local.game_mode_state
    }

    /// The player's inventory menu as the client holds it now.
    pub fn container_menu(&mut self) -> &photon_game::menus::container::ContainerMenu {
        if let Some((slots, carried, state_id)) = self.world.local_player_menu() {
            self.menus.container_menu.load(slots, carried, state_id);
        }
        self.menus.container_menu.item_tags = self.world.block_state_registry().item_tags().clone();
        &self.menus.container_menu
    }

    /// `BundleMouseActions.toggleSelectedBundleItem`: the bundle in a slot of the open menu
    /// selects another of its items (or none, for -1), and the server is told
    /// (`ServerboundSelectBundleItemPacket`).
    pub fn select_bundle_item(&mut self, slot: i32, selected: i32) -> Result<(), ConnectionError> {
        let Ok(index) = usize::try_from(slot) else {
            return Ok(());
        };
        let opened = self
            .world
            .local_player()
            .is_some_and(|player| player.open_container().is_some());
        let changed = if opened {
            if self.open_container_menu().is_none() {
                return Ok(());
            }
            let menu = self.menus.open_menu.as_mut().expect("synced above");
            let changed = menu.select_bundle_item(index, selected);
            let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
            self.world.set_local_player_container_menu(&slots, carried);
            changed
        } else {
            self.container_menu();
            let menu = &mut self.menus.container_menu;
            let changed = menu.select_bundle_item(index, selected);
            let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
            self.world.set_local_player_menu(&slots, carried);
            changed
        };
        if !changed {
            return Ok(());
        }
        self.send(PLAY_SERVERBOUND_SELECT_BUNDLE_ITEM_PACKET_ID, |output| {
            photon_core::wire::write_varint(slot, output);
            photon_core::wire::write_varint(selected, output);
            Ok(())
        })
    }

    /// The container the server opened, as a screen needs to know it: its id,
    /// the menu it is and its title as NBT. `None` with none open, or one of
    /// a kind not modelled yet.
    pub fn open_container_info(
        &self,
    ) -> Option<(i32, photon_game::menus::container::MenuKind, Vec<u8>)> {
        let container = self.world.local_player()?.open_container()?;
        let kind = self.menu_kind(container.container_id(), container.menu_type_id())?;
        // A screen opens once the server has sent what is in the container.
        self.world.local_player_container_menu()?;
        Some((
            container.container_id(),
            kind,
            container.title_nbt().to_vec(),
        ))
    }
}
