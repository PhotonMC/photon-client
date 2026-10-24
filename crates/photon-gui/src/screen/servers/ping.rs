//! What a ping of a saved server shows in its entry.

use photon_game::server_status::{PingState, Pinged};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;

use super::{ICON, JoinMultiplayerScreen, ROW_WIDTH};

/// A pinged server as its entry draws it (`OnlineServerEntry.refreshStatus` and its texts).
#[derive(Debug, Clone)]
pub(super) struct PingView {
    /// The description's first two lines, as wide as the entry leaves.
    pub(super) motd: Vec<Vec<Run>>,
    /// The player count, or the version in red for an incompatible server.
    pub(super) status: Vec<Run>,
    pub(super) icon: &'static str,
    pub(super) icon_tooltip: Vec<Run>,
    /// The sampled players, for a server that answered.
    pub(super) players: Option<Vec<Vec<Run>>>,
}

impl JoinMultiplayerScreen {
    /// The texture the icon of the server at `index` is drawn from, none for the unknown one.
    pub fn set_icon(&mut self, index: usize, texture: Option<String>) {
        if let Some(slot) = self.icons.get_mut(index) {
            *slot = texture;
        }
    }

    /// What a ping of the saved server at `index` found.
    pub fn set_pinged(&mut self, index: usize, pinged: &Pinged, font: &Font, lang: &Lang) {
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let mut motd = crate::chat::split(&runs(&pinged.motd), ROW_WIDTH - 4 - ICON - 2, font);
        motd.truncate(2);
        let players = Some(pinged.player_list.iter().map(runs).collect());
        let (icon, icon_tooltip, players) = match pinged.state {
            PingState::Incompatible => (
                "server_list/incompatible",
                runs(&Component::translatable("multiplayer.status.incompatible")),
                players,
            ),
            PingState::Unreachable => (
                "server_list/unreachable",
                runs(&Component::translatable("multiplayer.status.no_connection")),
                None,
            ),
            PingState::Successful => (
                match pinged.ping {
                    ..150 => "server_list/ping_5",
                    150..300 => "server_list/ping_4",
                    300..600 => "server_list/ping_3",
                    600..1000 => "server_list/ping_2",
                    _ => "server_list/ping_1",
                },
                runs(&Component::translatable_with(
                    "multiplayer.status.ping",
                    vec![Component::literal(pinged.ping.to_string())],
                )),
                players,
            ),
        };
        let status = if pinged.state == PingState::Incompatible {
            runs(&pinged.version.clone().with_colour("red"))
        } else {
            runs(&pinged.status)
        };
        if let Some(slot) = self.pings.get_mut(index) {
            *slot = Some(PingView {
                motd,
                status,
                icon,
                icon_tooltip,
                players,
            });
        }
    }
}
