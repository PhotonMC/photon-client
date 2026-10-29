//! Player skins: what a profile's `textures` property says (`MinecraftSessionService`
//! `unpackTextures`), the keys that sign it, and, later, the files it names.

mod cape;
mod download;
mod keys;
mod legacy;
mod lookup;
mod unpack;

pub use cape::{CAPE_HEIGHT, CAPE_WIDTH, fit_cape};
pub use download::{fetch_texture, local_copy};
pub use keys::{fetch_certificate_keys, fetch_property_keys, parse_property_keys};
pub use legacy::process_legacy_skin;
pub use lookup::{FetchedProfile, fetch_by_id, fetch_by_name};
pub use photon_protocol::play::ProfileProperty;
pub use unpack::{ProfileTexture, ProfileTextures, SignatureState, unpack_textures};
