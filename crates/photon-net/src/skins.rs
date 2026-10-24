//! Player skins: what a profile's `textures` property says (`MinecraftSessionService`
//! `unpackTextures`), the keys that sign it, and, later, the files it names.

mod download;
mod keys;
mod legacy;
mod unpack;

pub use download::{fetch_texture, local_copy};
pub use keys::{fetch_property_keys, parse_property_keys};
pub use legacy::process_legacy_skin;
pub use unpack::{ProfileTexture, ProfileTextures, SignatureState, unpack_textures};
