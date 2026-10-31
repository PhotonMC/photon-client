//! The skins of the faces of the friends overlay: looked up on a thread of their own and put
//! into the GUI atlas, where the overlay draws them from.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use photon_core::binary::Uuid;
use photon_render::gui::GuiAtlas;
use photon_render::texture::png::DecodedImage;

use super::Controls;

/// The lookups asked for and what they answered.
pub(crate) struct Faces {
    asks: Option<Sender<Uuid>>,
    answers: Receiver<(Uuid, Arc<Vec<u8>>)>,
    sender: Sender<(Uuid, Arc<Vec<u8>>)>,
    /// The players asked for once, so that a reopened overlay or a failure asks nothing again.
    requested: HashSet<Uuid>,
    /// The skins that came, kept while the atlas does not hold them.
    ready: HashMap<Uuid, Arc<Vec<u8>>>,
    /// The atlas texture of each skin that has a place there, while the overlay is open.
    placed: HashMap<Uuid, String>,
}

impl Default for Faces {
    fn default() -> Self {
        let (sender, answers) = channel();
        Self {
            asks: None,
            answers,
            sender,
            requested: HashSet::new(),
            ready: HashMap::new(),
            placed: HashMap::new(),
        }
    }
}

impl Faces {
    /// One lookup asked for, once; the thread starts with the first.
    fn ask(&mut self, id: Uuid) {
        if !self.requested.insert(id) {
            return;
        }
        if self.asks.is_none() {
            let (asks, wanted) = channel::<Uuid>();
            let answers = self.sender.clone();
            let spawned = std::thread::Builder::new()
                .name("Face Lookup".to_owned())
                .spawn(move || {
                    for id in wanted {
                        if let Some(skin) = crate::skins::face::skin_of(id) {
                            let _ = answers.send((id, skin));
                        }
                    }
                });
            if spawned.is_err() {
                return;
            }
            self.asks = Some(asks);
        }
        if let Some(asks) = &self.asks {
            let _ = asks.send(id);
        }
    }

    /// The skins that came are put into the atlas: whether it changed.
    fn place(&mut self, (atlas, image): (&mut GuiAtlas, &mut DecodedImage)) -> bool {
        while let Ok((id, rgba)) = self.answers.try_recv() {
            self.ready.insert(id, rgba);
        }
        let mut changed = false;
        for (id, rgba) in &self.ready {
            if self.placed.contains_key(id) {
                continue;
            }
            let skin = DecodedImage {
                width: 64,
                height: 64,
                rgba: rgba.to_vec(),
            };
            let name = format!("friend_face/{:016x}{:016x}", id.most, id.least);
            if atlas.set_favicon(image, &name, Some(&skin)) {
                self.placed.insert(*id, name);
                changed = true;
            }
        }
        changed
    }

    /// The overlay is closed: the places the skins took in the atlas go back to the other icons.
    fn free(&mut self, (atlas, image): (&mut GuiAtlas, &mut DecodedImage)) {
        for (_, name) in self.placed.drain() {
            atlas.set_favicon(image, &name, None);
        }
    }
}

impl Controls {
    /// The overlay's faces: the players it shows are looked up, and the skins that came are put
    /// into the atlas and handed to it. Whether the atlas is to be sent to the GPU again.
    pub(crate) fn sync_faces(&mut self, atlas: (&mut GuiAtlas, &mut DecodedImage)) -> bool {
        let Some(overlay) = self.friends.as_mut() else {
            self.faces.free(atlas);
            return false;
        };
        for id in overlay.faces_wanted() {
            self.faces.ask(id);
        }
        let changed = self.faces.place(atlas);
        for (id, name) in &self.faces.placed {
            overlay.set_face(*id, name.clone());
        }
        changed
    }
}
