//! The skins of the player heads worn (`CustomHeadLayer` with a `minecraft:profile`): found a
//! frame after a head is first drawn, which wears the default skin meanwhile.

use std::collections::HashMap;

use photon_render::entity::textures::{EntityTextures, TextureRect};

use photon_net::connection::PlaySession;

use crate::skins::dynamic::{Dynamic, Resolvable};
use crate::skins::{Skins, Who};

/// What tells one profile's skin from another's: the id and the packed textures.
pub type Key = (u64, u64, String);

pub fn key_of(profile: &Resolvable) -> Key {
    match profile {
        Resolvable::Static(profile) => {
            let textures = profile
                .properties
                .iter()
                .find(|property| property.name == "textures")
                .map_or_else(String::new, |property| property.value.clone());
            (profile.id.most, profile.id.least, textures)
        }
        Resolvable::Dynamic(Dynamic::Name(name)) => (0, 0, format!("name:{name}")),
        Resolvable::Dynamic(Dynamic::Id(id)) => (id.most, id.least, "id".to_owned()),
    }
}

#[derive(Default)]
pub struct HeadSkins {
    found: HashMap<Key, TextureRect>,
    /// The profiles of the heads drawn since the skins were last found.
    wanted: HashMap<Key, Resolvable>,
}

impl HeadSkins {
    /// The skins found for the frame, for the drawing that has no access to these.
    pub fn found(&self) -> HashMap<Key, TextureRect> {
        self.found.clone()
    }

    /// Heads that were drawn elsewhere, to be found for the next frame.
    pub fn want(&mut self, profiles: Vec<Resolvable>) {
        for profile in profiles {
            self.wanted.insert(key_of(&profile), profile);
        }
    }

    /// The skin found for a head with this profile, which is asked for again for the next frame.
    pub fn rect(&mut self, profile: Resolvable) -> Option<TextureRect> {
        let key = key_of(&profile);
        let rect = self.found.get(&key).copied();
        self.wanted.insert(key, profile);
        rect
    }

    /// Find the skins of the heads drawn since, before the frame's meshes use them. A skin that
    /// moves to another place in the textures is found there.
    pub fn resolve(
        &mut self,
        (textures, skins): (&mut EntityTextures, &mut Skins),
        session: &PlaySession,
    ) {
        self.found.clear();
        for (key, wanted) in self.wanted.drain() {
            let profile = skins.lookups.profile(wanted, session);
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
