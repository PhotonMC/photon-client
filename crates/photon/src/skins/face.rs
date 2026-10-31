//! The skin of a player named by id only, for the faces of the friends overlay.

use std::sync::Arc;
use std::time::Duration;

use photon_core::binary::Uuid;
use photon_net::skins::fetch_by_id;

/// The 64 by 64 skin of the player: the session server's profile, its textures property and the
/// skin file it names. None where any step fails; the face then stays the default skin.
pub(crate) fn skin_of(id: Uuid) -> Option<Arc<Vec<u8>>> {
    let profile = fetch_by_id(id, Duration::from_secs(10))?;
    let property = profile
        .properties
        .iter()
        .find(|property| property.name == "textures")?;
    let cache = crate::controls::game_directory().join("assets/skins");
    let loaded = super::worker::load(property, &super::shared_keys(), &cache)?;
    Some(loaded.rgba)
}
