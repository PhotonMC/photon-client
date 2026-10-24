//! `YggdrasilMinecraftSessionService.unpackTextures`: the `textures` property of a profile, its
//! signature, and the textures it names.

use photon_core::json::JsonValue;
use photon_core::{base64, json};
use photon_protocol::crypto::verify_sha1_rsa;
use photon_protocol::play::ProfileProperty;

/// `SignatureState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureState {
    Unsigned,
    Invalid,
    Signed,
}

/// A `MinecraftProfileTexture`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileTexture {
    pub url: String,
    /// `getHash`: the file name of the url's path without its extension.
    pub hash: String,
    /// The `model` of its metadata.
    pub model: Option<String>,
}

impl ProfileTexture {
    /// `PlayerModelType.byLegacyServicesName`: `slim`, otherwise the wide model.
    pub fn is_slim(&self) -> bool {
        self.model.as_deref() == Some("slim")
    }
}

/// `MinecraftProfileTextures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileTextures {
    pub skin: Option<ProfileTexture>,
    pub cape: Option<ProfileTexture>,
    pub elytra: Option<ProfileTexture>,
    pub signature: SignatureState,
}

impl ProfileTextures {
    /// `MinecraftProfileTextures.EMPTY`.
    pub const EMPTY: Self = Self {
        skin: None,
        cape: None,
        elytra: None,
        signature: SignatureState::Unsigned,
    };
}

/// `getPropertySignatureState`: signed when any of the `keys` verifies the value.
fn signature_state(property: &ProfileProperty, keys: &[Vec<u8>]) -> SignatureState {
    let Some(signature) = &property.signature else {
        return SignatureState::Unsigned;
    };
    let Some(signature) = base64::decode(signature) else {
        return SignatureState::Invalid;
    };
    if keys
        .iter()
        .any(|key| verify_sha1_rsa(key, property.value.as_bytes(), &signature))
    {
        SignatureState::Signed
    } else {
        SignatureState::Invalid
    }
}

/// `TextureUrlChecker.isAllowedTextureDomain`: http or https, to `textures.minecraft.net` in
/// lower case.
fn is_allowed_texture_domain(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !matches!(scheme, "http" | "https")
        || url.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return false;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = match host_port.rsplit_once(':') {
        Some((host, port)) if port.bytes().all(|b| b.is_ascii_digit()) => host,
        _ => host_port,
    };
    host == "textures.minecraft.net"
}

/// `MinecraftProfileTexture.getHash`: the last segment of the path, without its extension.
fn texture_hash(url: &str) -> String {
    let path = url.split_once("://").map_or("", |(_, rest)| {
        let from_path = rest.find('/').map_or("", |at| &rest[at..]);
        from_path.split(['?', '#']).next().unwrap_or_default()
    });
    let name = path.rsplit('/').next().unwrap_or_default();
    name.rsplit_once('.')
        .map_or(name, |(base, _)| base)
        .to_owned()
}

/// One entry of the payload's `textures` map; none where its url is missing or not allowed.
fn texture_of(entry: &JsonValue) -> Option<ProfileTexture> {
    let JsonValue::Object(entry) = entry else {
        return None;
    };
    let url = entry.text("url")?;
    if !is_allowed_texture_domain(&url) {
        return None;
    }
    let model = entry.object("metadata").and_then(|meta| meta.text("model"));
    Some(ProfileTexture {
        hash: texture_hash(&url),
        url,
        model,
    })
}

/// `unpackTextures`: the textures a property names, with how it is signed. A payload that
/// cannot be read, has no textures, or names a url outside the allowed domain is empty.
pub fn unpack_textures(property: &ProfileProperty, keys: &[Vec<u8>]) -> ProfileTextures {
    let signature = signature_state(property, keys);
    let Some(payload) = base64::decode(&property.value)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|text| json::parse_object(&text))
    else {
        return ProfileTextures::EMPTY;
    };
    let Some(textures) = payload.object("textures").filter(|map| !map.is_empty()) else {
        return ProfileTextures::EMPTY;
    };
    let mut unpacked = ProfileTextures {
        signature,
        ..ProfileTextures::EMPTY
    };
    for (name, entry) in textures.entries() {
        let Some(texture) = texture_of(entry) else {
            return ProfileTextures::EMPTY;
        };
        match String::from_utf16_lossy(name).as_str() {
            "SKIN" => unpacked.skin = Some(texture),
            "CAPE" => unpacked.cape = Some(texture),
            "ELYTRA" => unpacked.elytra = Some(texture),
            _ => {}
        }
    }
    unpacked
}

impl ProfileTextures {
    /// `PlayerSkin.secure`: the property is signed by a known key.
    pub fn secure(&self) -> bool {
        self.signature == SignatureState::Signed
    }
}

#[cfg(test)]
mod tests;
