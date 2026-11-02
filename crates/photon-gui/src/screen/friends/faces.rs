//! The faces of the entries, asked for through the host.

use photon_core::binary::Uuid;

use super::{Entry, FriendsOverlay, Listing};

impl FriendsOverlay {
    /// The players whose skins are not asked for yet; each is named once.
    pub fn faces_wanted(&mut self) -> Vec<Uuid> {
        let mut ids = Vec::new();
        for listing in [&self.view.friends, &self.view.pending] {
            if let Listing::Entries(entries) = listing {
                ids.extend(entries.iter().map(|Entry { id, .. }| *id));
            }
        }
        self.faces.wanted(ids)
    }

    /// The texture a player's skin is drawn from.
    pub fn set_face(&mut self, id: Uuid, texture: String) {
        self.faces.set(id, texture);
    }
}
