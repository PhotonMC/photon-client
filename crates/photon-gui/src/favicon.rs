//! A saved server's icon in the GUI atlas (`ServerData.getIconBytes`).

use photon_render::gui::GuiAtlas;
use photon_render::gui::favicon::{IconSync, sync_icon};
use photon_render::texture::png::DecodedImage;

/// `sync_icon` of a saved server's icon; one the entry cannot use is taken from the server,
/// which the caller then saves (`setIconBytes(null)`, `updateServerList`).
pub fn sync_server_icon(
    atlas: &mut GuiAtlas,
    atlas_image: &mut DecodedImage,
    server: &mut photon_game::servers::ServerData,
    last: &mut Option<Vec<u8>>,
) -> Option<IconSync> {
    let sync = sync_icon(
        atlas,
        atlas_image,
        &server.ip,
        server.icon_bytes().as_deref(),
        last,
    )?;
    if sync.rejected {
        server.set_icon_bytes(None);
    }
    Some(sync)
}
