use photon_game::hotbar::HotbarManager;
use photon_game::text::{Component, Lang};

use super::*;

impl Controls {
    /// The saved hotbars, read from `hotbar.nbt` on first use.
    fn hotbars(&mut self) -> &mut HotbarManager {
        self.hotbars
            .get_or_insert_with(|| HotbarManager::new(&game_directory()))
    }

    /// `selectTab` for the hotbar tab reads the saved hotbars: an open creative screen gets them.
    pub(super) fn show_saved_hotbars(&mut self) {
        let saved = self.hotbars().saved();
        if let Some(creative) = self.screen.as_mut().and_then(|open| open.as_creative_mut()) {
            creative.set_saved_hotbars(saved);
        }
    }

    /// `KeyMapping.getTranslatedKeyMessage` of a mapping.
    fn key_message(&self, mapping: &str, lang: &Lang) -> Component {
        self.options.key_binding(mapping).map_or_else(
            || Component::literal(""),
            |key| photon_game::options::key_display_name(key, keybinds::system_key_name(key), lang),
        )
    }

    /// The activator keys held for the toolbar, as `held_key` sees them.
    pub(super) fn toolbar_activator_key(&mut self, name: &str, down: bool) -> bool {
        let held = down && self.captured;
        match name {
            "key.saveToolbarActivator" => self.save_hotbar_held = held,
            "key.loadToolbarActivator" => self.load_hotbar_held = held,
            _ => return false,
        }
        true
    }

    /// `Minecraft.handleKeybinds` for a `key.hotbar.N` press: a creative player with an activator
    /// held loads (first) or saves that hotbar, anyone else selects the slot.
    pub(super) fn hotbar_key(
        &mut self,
        name: &str,
        session: &mut PlaySession,
        spectating: bool,
        lang: &Lang,
    ) {
        let (load, save) = (self.load_hotbar_held, self.save_hotbar_held);
        if spectating || !session.has_infinite_materials() || !(load || save) {
            select_hotbar(session, name, spectating);
            return;
        }
        let Some(id) = name["key.hotbar.".len()..]
            .parse::<usize>()
            .ok()
            .and_then(|number| number.checked_sub(1))
            .filter(|id| *id < photon_game::hotbar::HOTBARS)
        else {
            return;
        };
        if load {
            let stacks = self.hotbars().get(id).load();
            let _ = session.load_hotbar(&stacks);
        } else {
            let stacks = session.hotbar_stacks();
            self.hotbars().store(id, &stacks);
            session.show_overlay(Component::translatable_with(
                "inventory.hotbarSaved",
                vec![
                    self.key_message("key.loadToolbarActivator", lang),
                    self.key_message(name, lang),
                ],
            ));
        }
    }
}

/// A `key.hotbar.N` press: the slot is selected, but not by a spectator.
fn select_hotbar(session: &mut PlaySession, name: &str, spectating: bool) {
    let slot = name["key.hotbar.".len()..]
        .parse::<i32>()
        .map(|slot| slot - 1);
    if let (false, Ok(slot)) = (spectating, slot) {
        session.select_slot(slot);
    }
}
