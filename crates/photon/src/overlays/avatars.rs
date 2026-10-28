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
    let (id, properties, patch) = profile.map_or(
        (
            photon_core::binary::Uuid::new(0, 0),
            Vec::new(),
            SkinPatch::default(),
        ),
        |(found, patch)| (found.id, found.properties, patch),
    );
    let who = crate::skins::Who {
        id,
        properties: &properties,
        require_secure: false,
    };
    let resolved = skins.resolve(textures, &who)?;
    Some(patched(resolved, &patch, textures))
}

use crate::skins::item_profile::SkinPatch;

/// `PlayerSkin.with(patch)`: what the profile's patch names replaces the skin's (a texture only
/// where it is among the entity textures, the model always).
fn patched(
    mut resolved: crate::skins::Resolved,
    patch: &SkinPatch,
    textures: &photon_render::entity::textures::EntityTextures,
) -> crate::skins::Resolved {
    let rect = |id: &Option<String>| {
        let id = id.as_deref()?;
        textures.rect(id.strip_prefix("minecraft:").unwrap_or(id))
    };
    if let Some(body) = rect(&patch.body) {
        resolved.rect = body;
    }
    if let Some(cape) = rect(&patch.cape) {
        resolved.cape = Some(cape);
    }
    if let Some(slim) = patch.slim {
        resolved.slim = slim;
    }
    resolved
}

type Remembered = (crate::skins::profile::StaticProfile, SkinPatch);

thread_local! {
    /// The profile each mannequin last had a skin from (`ClientMannequin.skin`), with its patch, by
    /// its UUID (entity ids come back after a restart); the mannequins gone are dropped each frame.
    static LAST_SKINS: std::cell::RefCell<std::collections::HashMap<(u64, u64), Remembered>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// `ClientMannequin.updateSkin`: the profile's skin once known (`RenderInfo`, with the patch).
/// While a name or an id is looked up, or when the lookup finds nothing, the mannequin keeps the
/// skin it had (`ifPresent`), at first `DEFAULT_SKIN`, the empty profile's, unpatched.
fn mannequin_profile(
    session: &PlaySession,
    skins: &mut crate::skins::Skins,
    entity: &photon_world::ClientEntity,
) -> Option<Remembered> {
    let id = (entity.uuid.most, entity.uuid.least);
    let last = || LAST_SKINS.with(|last| last.borrow().get(&id).cloned());
    let Some((resolvable, patch)) = entity
        .mannequin_profile()
        .and_then(crate::skins::item_profile::wire_profile)
    else {
        return last();
    };
    let looked_up = matches!(resolvable, crate::skins::dynamic::Resolvable::Dynamic(_));
    let found = skins.lookups.profile(resolvable, session);
    if looked_up && found.properties.is_empty() {
        return last();
    }
    let remembered = (found, patch);
    LAST_SKINS.with(|last| last.borrow_mut().insert(id, remembered.clone()));
    Some(remembered)
}

/// The skin layers the local player's own data names (the server's echo of what Photon sent),
/// set once a frame for the model and the inventory doll; all until the server says.
static LOCAL_LAYERS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0x7f);

pub fn set_local_layers(world: &photon_world::ClientWorld) {
    let layers = world
        .local_player()
        .map_or(0x7f, |state| state.model_customisation());
    LOCAL_LAYERS.store(layers, std::sync::atomic::Ordering::Relaxed);
}

/// The local player's main arm is the left (`Player.getMainArm`), set once a frame.
static LOCAL_MAIN_LEFT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_local_main_arm(left: bool) {
    LOCAL_MAIN_LEFT.store(left, std::sync::atomic::Ordering::Relaxed);
}

pub fn local_main_arm_left() -> bool {
    LOCAL_MAIN_LEFT.load(std::sync::atomic::Ordering::Relaxed)
}

/// The local player's hidden skin layers, as `PlayerRenderState.hidden_layers` takes them.
pub fn local_hidden_layers() -> u8 {
    !LOCAL_LAYERS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Forget the skins of mannequins no longer in the world.
pub fn forget_gone_mannequins(world: &photon_world::ClientWorld) {
    LAST_SKINS.with(|last| {
        let mut last = last.borrow_mut();
        if last.is_empty() {
            return;
        }
        let present: std::collections::HashSet<(u64, u64)> = world
            .entities()
            .drawn()
            .filter(|entity| entity.is_avatar() && !entity.is_player())
            .map(|entity| (entity.uuid.most, entity.uuid.least))
            .collect();
        last.retain(|id, _| present.contains(id));
    });
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
