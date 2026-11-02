//! The faces of the player lists: a player's own skin once the host has put it where the GUI
//! draws from, the default skin of their id until then.

use std::collections::{HashMap, HashSet};

use photon_core::binary::Uuid;

/// What a list knows of the skins: those it asked for and the textures that came.
#[derive(Debug, Default, Clone)]
pub(crate) struct Faces {
    textures: HashMap<Uuid, String>,
    asked: HashSet<Uuid>,
}

impl Faces {
    /// The texture a player's skin is drawn from, once it came.
    pub(crate) fn texture(&self, id: Uuid) -> Option<&str> {
        self.textures.get(&id).map(String::as_str)
    }

    /// The players of `ids` whose skins are not asked for yet; each is named once.
    pub(crate) fn wanted(&mut self, ids: impl IntoIterator<Item = Uuid>) -> Vec<Uuid> {
        ids.into_iter()
            .filter(|id| self.asked.insert(*id))
            .collect()
    }

    pub(crate) fn set(&mut self, id: Uuid, texture: String) {
        self.textures.insert(id, texture);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> Uuid {
        Uuid { most: n, least: 1 }
    }

    #[test]
    fn a_player_is_named_once_and_the_texture_is_kept() {
        let mut faces = Faces::default();
        assert_eq!(faces.wanted([id(1), id(2), id(1)]), vec![id(1), id(2)]);
        assert!(faces.wanted([id(2)]).is_empty());
        assert_eq!(faces.texture(id(1)), None);
        faces.set(id(1), "friend_face/a".to_owned());
        assert_eq!(faces.texture(id(1)), Some("friend_face/a"));
    }
}
