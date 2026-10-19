//! The debug overlay F3 shows (`DebugScreenOverlay`), laid out from the camera entity.

use super::*;
use photon_gui::hud::debug::{self, CameraEntity, FooterKeys, Input};

/// The two columns of the overlay's lines, or none when nothing is shown: no entry is on, or the
/// interface is hidden with no screen over it.
pub(super) type DebugColumns = (Vec<String>, Vec<String>);

impl Gui {
    pub(super) fn debug_columns(
        session: &PlaySession,
        controls: &Controls,
    ) -> Option<DebugColumns> {
        // `Options.reducedDebugInfo` is not an option here yet.
        let reduced = session.reduced_debug_info();
        let list = &controls.debug.entries;
        if list.enabled(reduced).is_empty() || controls.hud_hidden && !controls.any_screen_open() {
            return None;
        }
        let camera = session.player().map(|player| {
            let position = player.position();
            let (yaw, pitch) = player.rotation();
            CameraEntity {
                position: (position.x, position.y, position.z),
                yaw,
                pitch,
                dimension: session.world().dimension_key().to_owned(),
            }
        });
        let input = Input {
            version: {
                let version = photon_net::resources::KnownPack::VANILLA_VERSION;
                debug::version_line(version, version)
            },
            camera,
            frame: controls.debug.frame,
        };
        let footer = list.is_overlay_visible().then(|| {
            debug::footer(
                &FooterKeys::defaults(),
                (false, false, false, false),
                (false, false),
            )
        });
        Some(debug::lines(list, reduced, &input, footer))
    }
}
