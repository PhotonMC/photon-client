//! The entities `AvatarRenderer` draws, players and mannequins: which they are, the skin each
//! wears, and the name its outline is coloured by.

use photon_net::connection::PlaySession;

/// `AvatarRenderer`'s entities: players, and mannequins.
pub(super) fn is_avatar(entity: &photon_world::ClientEntity) -> bool {
    entity.is_player()
        || photon_data::entity_type::name(entity.type_id) == Some("minecraft:mannequin")
}

/// The skin an avatar wears: a player's from its profile in the tab list; a mannequin's
/// `ClientMannequin.DEFAULT_SKIN`, the default skin of an empty profile.
pub(super) fn avatar_look(
    session: &PlaySession,
    (skins, textures): (
        &mut crate::skins::Skins,
        &mut photon_render::entity::textures::EntityTextures,
    ),
    entity: &photon_world::ClientEntity,
) -> Option<crate::skins::Resolved> {
    if entity.is_player() {
        return skins.of_player(session, textures, entity.uuid);
    }
    let empty = crate::skins::Who {
        id: photon_core::binary::Uuid::new(0, 0),
        properties: &[],
        require_secure: false,
    };
    skins.resolve(textures, &empty)
}

/// The scoreboard name of an avatar: a player's profile name, a mannequin's UUID.
fn scoreboard_name(session: &PlaySession, entity: &photon_world::ClientEntity) -> Option<String> {
    match session.player_name(entity.uuid) {
        Some(name) => Some(name.to_owned()),
        None => (!entity.is_player()).then(|| entity.uuid.to_string_java()),
    }
}

/// The name a glowing avatar's outline is coloured by.
pub(super) fn outline_name(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
) -> Option<String> {
    let glows = crate::mobs::outline::player_glows(entity.is_glowing());
    scoreboard_name(session, entity).filter(|_| glows)
}
