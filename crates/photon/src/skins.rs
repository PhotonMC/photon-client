//! Players' skins (`SkinManager`): what a player's profile says it wears, loaded on threads of
//! their own and drawn from places set aside in the entity textures. Until a skin is there, or
//! where there is none, a player wears its default skin (`DefaultPlayerSkin.get`).

pub(crate) mod dynamic;
pub(crate) mod item_profile;
pub(crate) mod profile;
mod slots;
mod worker;

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use photon_core::binary::Uuid;
use photon_net::skins::ProfileProperty;
use photon_render::entity::textures::{EntityTextures, TextureRect};

use slots::Slots;
use worker::{KeyCell, Loaded};

/// How many downloaded skins the entity textures have room for at once.
pub(crate) const SLOT_COUNT: usize = 64;
/// How many skins load at the same time.
pub(super) const MAX_LOADING: usize = 4;
/// How long a skin that failed to load is left alone.
pub(super) const RETRY: Duration = Duration::from_secs(5 * 60);
/// How many downloaded capes the entity textures have room for at once.
pub(crate) const CAPE_SLOT_COUNT: usize = 32;
/// A skin's width and height in the places; a cape has the same width and half the height.
const SIDE: usize = 64;

/// The name of a place in the entity textures.
pub(crate) fn slot_name(index: usize) -> String {
    format!("entity/skin_slot/{index}")
}

/// The name of a place for a cape in the entity textures.
pub(crate) fn cape_slot_name(index: usize) -> String {
    format!("entity/skin_slot/cape/{index}")
}

/// What a player wears: the place of its skin, whether its arms are slim, and the place of its
/// cape when it has one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Resolved {
    pub rect: TextureRect,
    pub slim: bool,
    pub cape: Option<TextureRect>,
    /// The elytra texture the profile names (`PlayerSkin.elytra`).
    pub elytra: Option<TextureRect>,
}

enum State {
    Loading,
    /// Nothing to show; asked for again after a while.
    Failed(Instant),
    Ready(Loaded),
}

struct Player {
    property: ProfileProperty,
    state: State,
    used: u64,
}

/// What a loaded skin belongs to: the profile's id and its packed textures, so two profiles that
/// share an id (the zero one of heads that name none) do not replace each other.
type PlayerKey = (u64, u64, String);

struct Done {
    id: PlayerKey,
    property: ProfileProperty,
    loaded: Option<Loaded>,
}

/// Who a skin is wanted for.
pub(crate) struct Who<'a> {
    pub id: Uuid,
    /// The profile's properties, among them the packed textures.
    pub properties: &'a [ProfileProperty],
    /// `PlayerInfo.createSkinLookup`: everyone but the local player needs a signed property.
    pub require_secure: bool,
}

type Loader = fn(&ProfileProperty, &KeyCell, &std::path::Path) -> Option<Loaded>;

pub(crate) struct Skins {
    slots: Slots,
    rects: Vec<TextureRect>,
    cape_slots: Slots,
    cape_rects: Vec<TextureRect>,
    players: HashMap<PlayerKey, Player>,
    queue: VecDeque<(PlayerKey, ProfileProperty)>,
    loading: usize,
    finished: Receiver<Done>,
    sender: Sender<Done>,
    keys: Arc<KeyCell>,
    cache: PathBuf,
    /// How a property is loaded, which a test replaces.
    load: Loader,
    frame: u64,
    /// Where the local player's cape is, if it has one: set by the frame that dresses it.
    pub(crate) own_cape: Option<TextureRect>,
    /// Where the local player's elytra texture is, if its profile names one.
    pub(crate) own_elytra: Option<TextureRect>,
    /// The lookups of heads that name their owner by name or id.
    pub(crate) lookups: dynamic::Lookups,
    /// The places written since the last call, for the GPU to take.
    dirty: Vec<photon_render::vulkan::AtlasRegion>,
}

impl Skins {
    pub(crate) fn new(textures: &EntityTextures, cache: PathBuf) -> Self {
        Self::with_loader(textures, cache, worker::load)
    }

    /// The skins of a game: downloads are kept under the game directory's assets.
    pub(crate) fn for_game(textures: &EntityTextures) -> Self {
        let skins = Self::new(
            textures,
            crate::controls::game_directory().join("assets/skins"),
        );
        // The signing keys are asked for while the game loads, not when the first skin waits.
        let keys = skins.keys.clone();
        let _ = std::thread::Builder::new()
            .name("Skin Keys".to_owned())
            .spawn(move || keys.warm());
        skins
    }

    fn with_loader(textures: &EntityTextures, cache: PathBuf, load: Loader) -> Self {
        let (sender, finished) = channel();
        let mut skins = Self {
            slots: Slots::new(0),
            rects: Vec::new(),
            cape_slots: Slots::new(0),
            cape_rects: Vec::new(),
            players: HashMap::new(),
            queue: VecDeque::new(),
            loading: 0,
            finished,
            sender,
            keys: shared_keys(),
            cache,
            load,
            frame: 0,
            own_cape: None,
            own_elytra: None,
            lookups: dynamic::Lookups::default(),
            dirty: Vec::new(),
        };
        skins.lay_out(textures);
        skins
    }

    /// The textures were built again (the packs reloaded): the places are elsewhere and empty.
    pub(crate) fn lay_out(&mut self, textures: &EntityTextures) {
        self.rects = (0..SLOT_COUNT)
            .map_while(|index| textures.rect(&slot_name(index)))
            .collect();
        self.slots = Slots::new(self.rects.len());
        self.cape_rects = (0..CAPE_SLOT_COUNT)
            .map_while(|index| textures.rect(&cape_slot_name(index)))
            .collect();
        self.cape_slots = Slots::new(self.cape_rects.len());
    }

    /// A frame begins: the skins that finished loading are taken in, and more are started.
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
        self.lookups.begin_frame();
        while let Ok(done) = self.finished.try_recv() {
            self.loading = self.loading.saturating_sub(1);
            if let Some(player) = self.players.get_mut(&done.id)
                && player.property == done.property
            {
                player.state = done
                    .loaded
                    .map_or_else(|| State::Failed(Instant::now()), State::Ready);
            }
        }
        while self.loading < MAX_LOADING
            && let Some((id, property)) = self.queue.pop_front()
        {
            self.start(id, property);
        }
        if self.frame.is_multiple_of(600) {
            let oldest = self.frame.saturating_sub(6000);
            self.players.retain(|_, player| player.used >= oldest);
        }
    }

    fn start(&mut self, id: PlayerKey, property: ProfileProperty) {
        self.loading += 1;
        let (sender, keys, cache) = (self.sender.clone(), self.keys.clone(), self.cache.clone());
        let load = self.load;
        let spawned = std::thread::Builder::new()
            .name("Skin Loader".to_owned())
            .spawn(move || {
                let loaded = load(&property, &keys, &cache);
                let _ = sender.send(Done {
                    id,
                    property,
                    loaded,
                });
            });
        if spawned.is_err() {
            self.loading -= 1;
        }
    }

    /// The places written (a skin was put into each) since the last call.
    pub(crate) fn take_dirty(&mut self) -> Vec<photon_render::vulkan::AtlasRegion> {
        std::mem::take(&mut self.dirty)
    }

    /// `PlayerInfo.createSkinLookup` of a player the server told of: everyone but the local
    /// player needs a signed property.
    pub(crate) fn of_player(
        &mut self,
        session: &photon_net::connection::PlaySession,
        textures: &mut EntityTextures,
        id: Uuid,
    ) -> Option<Resolved> {
        let who = Who {
            id,
            properties: session.player_properties(id),
            require_secure: session.profile_id() != Some(id),
        };
        self.resolve(textures, &who)
    }

    /// Where the skin of `who` is in `textures`, whether its arms are slim, and where its cape
    /// is: its own once it has loaded and may be shown, otherwise its default skin without a
    /// cape.
    pub(crate) fn resolve(&mut self, textures: &mut EntityTextures, who: &Who) -> Option<Resolved> {
        let default = photon_render::entity::player_renderer::default_skin(who.id);
        let fallback = || {
            textures.rect(default.texture).map(|rect| Resolved {
                rect,
                slim: default.slim,
                cape: None,
                elytra: None,
            })
        };
        let Some(property) = who.properties.iter().find(|p| p.name == "textures") else {
            return fallback();
        };
        let id = (who.id.most, who.id.least, property.value.clone());
        let frame = self.frame;
        let stale = self.players.get(&id).is_none_or(|player| {
            player.property != *property
                || matches!(player.state, State::Failed(at) if at.elapsed() > RETRY)
        });
        if stale {
            let player = Player {
                property: property.clone(),
                state: State::Loading,
                used: frame,
            };
            self.players.insert(id.clone(), player);
            self.queue.push_back((id.clone(), property.clone()));
        }
        let player = self.players.get_mut(&id)?;
        player.used = frame;
        let State::Ready(loaded) = &player.state else {
            return fallback();
        };
        if who.require_secure && !loaded.secure {
            return fallback();
        }
        let loaded = loaded.clone();
        let Some((index, fresh)) = self.slots.take(&loaded.hash, frame) else {
            return fallback();
        };
        let rect = self.rects[index];
        if fresh {
            blit(textures, rect, &loaded.rgba);
            self.dirty.push(region_of(textures, rect, &loaded.rgba));
        }
        let cape = self.place_cape(textures, loaded.cape.as_ref(), frame);
        let elytra = self.place_cape(textures, loaded.elytra.as_ref(), frame);
        Some(Resolved {
            rect,
            slim: loaded.slim,
            cape,
            elytra,
        })
    }

    /// The place of the cape (or elytra texture, which is cut the same) of a loaded skin, written
    /// when it was not there yet. None where there is none, or no place is free in this frame
    /// (the player goes without).
    fn place_cape(
        &mut self,
        textures: &mut EntityTextures,
        cape: Option<&worker::Cape>,
        frame: u64,
    ) -> Option<TextureRect> {
        let cape = cape?;
        let (index, fresh) = self.cape_slots.take(&cape.hash, frame)?;
        let rect = *self.cape_rects.get(index)?;
        if fresh {
            blit(textures, rect, &cape.rgba);
            self.dirty.push(region_of(textures, rect, &cape.rgba));
        }
        Some(rect)
    }
}

/// The signing keys, asked for once by the process and kept across games (the services' keys
/// belong to the launcher session in the reference, not to a level).
fn shared_keys() -> Arc<KeyCell> {
    static KEYS: std::sync::OnceLock<Arc<KeyCell>> = std::sync::OnceLock::new();
    KEYS.get_or_init(Arc::default).clone()
}

/// The place `rect` is in `textures`, with the pixels written into it.
fn region_of(
    textures: &EntityTextures,
    rect: TextureRect,
    rgba: &[u8],
) -> photon_render::vulkan::AtlasRegion {
    let image = &textures.image;
    photon_render::vulkan::AtlasRegion {
        level: 0,
        x: (rect[0] * image.width as f32).round() as u32,
        y: (rect[1] * image.height as f32).round() as u32,
        width: SIDE as u32,
        height: (rgba.len() / (SIDE * 4)) as u32,
        rgba: rgba.to_vec(),
    }
}

/// Write a skin's pixels into the textures where `rect` is.
fn blit(textures: &mut EntityTextures, rect: TextureRect, rgba: &[u8]) {
    let image = &mut textures.image;
    let (width, height) = (image.width as f32, image.height as f32);
    let (left, top) = (
        (rect[0] * width).round() as usize,
        (rect[1] * height).round() as usize,
    );
    for row in 0..rgba.len() / (SIDE * 4) {
        let to = ((top + row) * image.width as usize + left) * 4;
        image.rgba[to..to + SIDE * 4].copy_from_slice(&rgba[row * SIDE * 4..(row + 1) * SIDE * 4]);
    }
}

#[cfg(test)]
mod tests;
