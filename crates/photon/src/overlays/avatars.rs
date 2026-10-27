//! The entities `AvatarRenderer` draws, players and mannequins: which they are, the skin each
//! wears, and the name its outline is coloured by.

use photon_net::connection::PlaySession;

/// `AvatarRenderer`'s entities: players, and mannequins.
pub(super) fn is_avatar(entity: &photon_world::ClientEntity) -> bool {
    entity.is_avatar()
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
    let profile = mannequin_profile(session, skins, entity);
    let (id, properties) = profile.map_or(
        (photon_core::binary::Uuid::new(0, 0), Vec::new()),
        |found| (found.id, found.properties),
    );
    let who = crate::skins::Who {
        id,
        properties: &properties,
        require_secure: false,
    };
    skins.resolve(textures, &who)
}

/// `ClientMannequin.updateSkin`: the profile's skin once known. A name or an id still being looked
/// up (or not found) leaves `DEFAULT_SKIN`, the empty profile's.
fn mannequin_profile(
    session: &PlaySession,
    skins: &mut crate::skins::Skins,
    entity: &photon_world::ClientEntity,
) -> Option<crate::skins::profile::StaticProfile> {
    let resolvable = crate::skins::item_profile::wire_profile(entity.mannequin_profile()?)?;
    let looked_up = matches!(resolvable, crate::skins::dynamic::Resolvable::Dynamic(_));
    let found = skins.lookups.profile(resolvable, session);
    (!looked_up || !found.properties.is_empty()).then_some(found)
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
