//! The skins of the faces of the friends overlay: looked up on a thread of their own and put
//! into the GUI atlas, where the overlay draws them from.

use std::collections::HashMap;
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
    /// The atlas texture of each skin that has a place.
    placed: HashMap<Uuid, String>,
}

impl Default for Faces {
    fn default() -> Self {
        let (sender, answers) = channel();
        Self {
            asks: None,
            answers,
            sender,
            placed: HashMap::new(),
        }
    }
}

impl Faces {
    /// One lookup asked for; the thread starts with the first.
    fn ask(&mut self, id: Uuid) {
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

    /// What came back is put into the atlas: its texture, and whether the atlas changed.
    fn place(&mut self, (atlas, image): (&mut GuiAtlas, &mut DecodedImage)) -> bool {
        let mut changed = false;
        while let Ok((id, rgba)) = self.answers.try_recv() {
            let skin = DecodedImage {
                width: 64,
                height: 64,
                rgba: rgba.to_vec(),
            };
            let name = format!("friend_face/{:016x}{:016x}", id.most, id.least);
            if atlas.set_favicon(image, &name, Some(&skin)) {
                self.placed.insert(id, name);
                changed = true;
            }
        }
        changed
    }
}

impl Controls {
    /// The overlay's faces: the players it shows are looked up, and the skins that came are put
    /// into the atlas and handed to it. Whether the atlas is to be sent to the GPU again.
    pub(crate) fn sync_faces(&mut self, atlas: (&mut GuiAtlas, &mut DecodedImage)) -> bool {
        let Some(overlay) = self.friends.as_mut() else {
            return false;
        };
        for id in overlay.faces_wanted() {
            match self.faces.placed.get(&id) {
                Some(name) => overlay.set_face(id, name.clone()),
                None => self.faces.ask(id),
            }
        }
        let changed = self.faces.place(atlas);
        for (id, name) in &self.faces.placed {
            overlay.set_face(*id, name.clone());
        }
        changed
    }
}
