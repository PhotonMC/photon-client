//! What a skin takes before it can be drawn, done on a thread of its own: the keys that
//! sign it, its textures property unpacked, its file downloaded and made ready.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use photon_net::skins;
use photon_net::skins::ProfileProperty;

/// How long after a failed request for the signing keys another is made
/// (`YggdrasilServicesKeyInfo`'s first backoff).
const KEY_RETRY: Duration = Duration::from_secs(5 * 60);
const KEY_TIMEOUT: Duration = Duration::from_secs(20);

/// A skin ready for the textures.
#[derive(Debug, Clone)]
pub(super) struct Loaded {
    /// The texture's name in the services, which names its place in the cache.
    pub hash: String,
    pub slim: bool,
    /// The property was signed by a key of the services.
    pub secure: bool,
    /// 64 by 64 pixels, four bytes each.
    pub rgba: std::sync::Arc<Vec<u8>>,
}

/// The keys of the services, asked for once and again after a failure.
#[derive(Default)]
pub(super) struct KeyCell(Mutex<KeyState>);

#[derive(Default)]
struct KeyState {
    keys: Option<Vec<Vec<u8>>>,
    failed: Option<Instant>,
}

impl KeyCell {
    /// The keys, fetched by the first caller while the others wait; empty where they could not
    /// be had (every signature is then invalid, as with the reference's empty key set).
    fn get(&self) -> Vec<Vec<u8>> {
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(keys) = &state.keys {
            return keys.clone();
        }
        if state.failed.is_some_and(|at| at.elapsed() < KEY_RETRY) {
            return Vec::new();
        }
        let fetched = skins::fetch_property_keys(KEY_TIMEOUT);
        state.failed = fetched.is_none().then(Instant::now);
        state.keys.clone_from(&fetched);
        fetched.unwrap_or_default()
    }
}

/// `SkinManager`'s load of one property: the skin it names, or why there is none.
pub(super) fn load(property: &ProfileProperty, keys: &KeyCell, cache: &Path) -> Option<Loaded> {
    // An unsigned property is never secure, so no key is needed to say so.
    let known = if property.signature.is_some() {
        keys.get()
    } else {
        Vec::new()
    };
    let textures = skins::unpack_textures(property, &known);
    let skin = textures.skin.as_ref()?;
    let bytes = skins::fetch_texture(&skin.url, &skins::local_copy(cache, &skin.hash))
        .inspect_err(|error| log::warn!("skin {}: {error}", skin.hash))
        .ok()?;
    let image = photon_render::texture::png::decode_rgba(&bytes)
        .inspect_err(|error| log::warn!("skin {}: {error:?}", skin.hash))
        .ok()?;
    let rgba = skins::process_legacy_skin(&image.rgba, image.width as usize, image.height as usize)
        .inspect_err(|error| log::warn!("{error}: {}", skin.hash))
        .ok()?;
    Some(Loaded {
        hash: skin.hash.clone(),
        slim: skin.is_slim(),
        secure: textures.secure(),
        rgba: std::sync::Arc::new(rgba),
    })
}
