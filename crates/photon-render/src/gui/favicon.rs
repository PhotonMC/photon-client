//! A server's icon in the GUI atlas (`FaviconTexture` and the list entry's `uploadServerIcon`).

use crate::gui::atlas::GuiAtlas;
use crate::texture::png::{DecodedImage, decode_rgba};
use photon_core::sha1::hash_unencoded_chars;

/// `FaviconTexture.forServer`: the texture a server of `address` has its icon registered as.
pub fn texture_name(address: &str) -> String {
    format!("servers/{}/icon", hash_unencoded_chars(address))
}

/// `NativeImage.read` and the size check of `FaviconTexture.upload`: the image of a 64 by 64 PNG.
fn decode_icon(bytes: &[u8]) -> Option<DecodedImage> {
    decode_rgba(bytes)
        .ok()
        .filter(|image| image.width == 64 && image.height == 64)
}

/// What a draw of a list entry made of its server's icon.
#[derive(Debug, PartialEq, Eq)]
pub struct IconSync {
    /// The texture the entry draws the icon from; none draws the unknown server.
    pub texture: Option<String>,
    /// Whether the atlas image changed and is to be sent to the GPU again.
    pub atlas_changed: bool,
    /// Whether the icon could not be used, which the entry answers by forgetting it.
    pub rejected: bool,
}

/// The entry's check of `bytes`, the server's icon, against `last`, the ones it showed: none when
/// they are the same, otherwise the icon uploaded or the texture cleared.
pub fn sync_icon(
    atlas: &mut GuiAtlas,
    atlas_image: &mut DecodedImage,
    address: &str,
    bytes: Option<&[u8]>,
    last: &mut Option<Vec<u8>>,
) -> Option<IconSync> {
    if bytes == last.as_deref() {
        return None;
    }
    let name = texture_name(address);
    let icon = bytes.map(decode_icon);
    let image = icon.as_ref().and_then(Option::as_ref);
    let rejected = bytes.is_some() && image.is_none();
    // An icon that fails to upload leaves the texture as it was: the entry's own is empty unless
    // it showed an icon before, which the cleared bytes then take out.
    let placed = if rejected && last.is_none() {
        false
    } else {
        atlas.set_favicon(atlas_image, &name, image)
    };
    *last = if rejected {
        None
    } else {
        bytes.map(<[u8]>::to_vec)
    };
    Some(IconSync {
        texture: (placed && image.is_some()).then_some(name),
        atlas_changed: placed,
        rejected,
    })
}

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
