use photon_game::spectator::{Item, SpectatorMenu};
use photon_game::text::{Component, Style};
use photon_gui::hud::spectator::{Icon, SpectatorHud, SpectatorSlot};

use super::*;

/// A slot's icon.
fn icon(item: &Item) -> Option<Icon> {
    Some(match item {
        Item::TeleportToPlayer(_) => Icon::Sprite("spectator/teleport_to_player"),
        Item::TeleportToTeam(_) => Icon::Sprite("spectator/teleport_to_team"),
        Item::Team(team) => Icon::Team(team.colour),
        Item::Player(_) => Icon::Player,
        Item::Close => Icon::Sprite("spectator/close"),
        Item::Scroll { direction, .. } if *direction < 0 => Icon::Sprite("spectator/scroll_left"),
        Item::Scroll { .. } => Icon::Sprite("spectator/scroll_right"),
        Item::Empty => return None,
    })
}

impl Gui {
    /// The spectator menu as this frame draws it, when it is open and not yet faded out.
    pub(super) fn spectator_hud(lang: &Lang, controls: &Controls) -> Option<SpectatorHud> {
        let gui = &controls.spectator;
        let menu: &SpectatorMenu = gui.menu()?;
        let alpha = gui.alpha(crate::controls::spectator::now_ms());
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let slots = (0..9)
            .map(|slot| {
                let item = menu.item(slot);
                let key = controls.key_message(&format!("key.hotbar.{}", slot + 1), lang);
                icon(&item).map(|icon| SpectatorSlot {
                    icon,
                    enabled: item.is_enabled(),
                    key: runs(&key),
                })
            })
            .collect();
        let selected = menu.selected_item();
        let action = if selected == Item::Empty {
            Component::translatable(menu.prompt())
        } else {
            selected.name()
        };
        Some(SpectatorHud {
            alpha,
            selected: menu.selected_slot(),
            slots,
            action: runs(&action),
        })
    }
}
