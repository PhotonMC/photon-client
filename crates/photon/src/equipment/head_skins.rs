//! The skins of the player heads worn (`CustomHeadLayer` with a `minecraft:profile`): found a
//! frame after a head is first drawn, which wears the default skin meanwhile.

use std::collections::HashMap;

use photon_render::entity::textures::{EntityTextures, TextureRect};

use crate::skins::profile::StaticProfile;
use crate::skins::{Skins, Who};

/// What tells one profile's skin from another's: the id and the packed textures.
type Key = (u64, u64, String);

fn key_of(profile: &StaticProfile) -> Key {
    let textures = profile
        .properties
        .iter()
        .find(|property| property.name == "textures")
        .map_or_else(String::new, |property| property.value.clone());
    (profile.id.most, profile.id.least, textures)
}

#[derive(Default)]
pub struct HeadSkins {
    found: HashMap<Key, TextureRect>,
    /// The profiles of the heads drawn since the skins were last found.
    wanted: HashMap<Key, StaticProfile>,
}

impl HeadSkins {
    /// The skin found for a head with this profile, which is asked for again for the next frame.
    pub fn rect(&mut self, profile: StaticProfile) -> Option<TextureRect> {
        let key = key_of(&profile);
        let rect = self.found.get(&key).copied();
        self.wanted.insert(key, profile);
        rect
    }

    /// Find the skins of the heads drawn since, before the frame's meshes use them. A skin that
    /// moves to another place in the textures is found there.
    pub fn resolve(&mut self, textures: &mut EntityTextures, skins: &mut Skins) {
        self.found.clear();
        for (key, profile) in self.wanted.drain() {
            let who = Who {
                id: profile.id,
                properties: &profile.properties,
                require_secure: false,
            };
            if let Some(resolved) = skins.resolve(textures, &who) {
                self.found.insert(key, resolved.rect);
            }
        }
    }
}
