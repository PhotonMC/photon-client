//! The faces of the entries: a player's own skin once the host has put it where the GUI draws
//! from, the default skin of their id until then.

use std::collections::{HashMap, HashSet};

use photon_core::binary::Uuid;

use super::{Entry, FriendsOverlay, Listing};

/// What the overlay knows of the skins: those it asked for and the textures that came.
#[derive(Debug, Default)]
pub(super) struct Faces {
    textures: HashMap<Uuid, String>,
    asked: HashSet<Uuid>,
}

impl Faces {
    pub(super) fn texture(&self, id: Uuid) -> Option<&str> {
        self.textures.get(&id).map(String::as_str)
    }
}

impl FriendsOverlay {
    /// The players whose skins are not asked for yet; each is named once.
    pub fn faces_wanted(&mut self) -> Vec<Uuid> {
        let mut wanted = Vec::new();
        for listing in [&self.view.friends, &self.view.pending] {
            let Listing::Entries(entries) = listing else {
                continue;
            };
            for Entry { id, .. } in entries {
                if self.faces.asked.insert(*id) {
                    wanted.push(*id);
                }
            }
        }
        wanted
    }

    /// The texture a player's skin is drawn from.
    pub fn set_face(&mut self, id: Uuid, texture: String) {
        self.faces.textures.insert(id, texture);
    }
}
