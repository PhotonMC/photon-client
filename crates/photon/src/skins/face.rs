//! The skin of a player named by id only, for the faces of the friends overlay.

use std::sync::Arc;
use std::time::Duration;

use photon_core::binary::Uuid;
use photon_net::skins::{ProfileProperty, fetch_by_id};

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
    // Another player's skin needs a property the services signed, as in the world.
    loaded.secure.then_some(loaded.rgba)
}

/// The 64 by 64 skin a profile's textures property names, as the tab list and the social screen
/// draw it: another player's skin needs a signature, the local player's does not.
pub(crate) fn skin_from(property: &ProfileProperty, local: bool) -> Option<Arc<Vec<u8>>> {
    let cache = crate::controls::game_directory().join("assets/skins");
    let loaded = super::worker::load(property, &super::shared_keys(), &cache)?;
    (local || loaded.secure).then_some(loaded.rgba)
}
