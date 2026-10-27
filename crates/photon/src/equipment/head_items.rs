//! Player head items in the GUI that carry a profile: drawn in the skin it names, from the entity
//! pictures, where every other item is drawn from the item atlas.

use std::collections::HashMap;

use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::textures::TextureRect;
use photon_render::gui::GuiItemVertex;
use photon_world::ClientItemStackState;

use super::head_skins::{Key, key_of};
use crate::skins::dynamic::Resolvable;
use crate::skins::item_profile;

/// The picture an item model gives a player head, which carries no skin of its own.
const DEFAULT_SKIN: &str = "minecraft:entity/player/slim/steve";

#[derive(Default)]
pub struct GuiHeads {
    /// The skins found for the frame, which the frame before asked for.
    found: HashMap<Key, TextureRect>,
    /// The profiles of the heads drawn since they were last taken.
    wanted: Vec<Resolvable>,
}

/// A texture coordinate moved from one picture's place to another's.
fn moved(uv: [f32; 2], from: [f32; 4], to: TextureRect) -> [f32; 2] {
    let along = |value: f32, low: usize| {
        (value - from[low]) / (from[low + 2] - from[low]) * (to[low + 2] - to[low]) + to[low]
    };
    [along(uv[0], 0), along(uv[1], 1)]
}

impl GuiHeads {
    /// The skins found for the heads of the frame.
    pub fn set_found(&mut self, found: HashMap<Key, TextureRect>) {
        self.found = found;
    }

    /// The profiles of the heads drawn, to be found for the next frame.
    pub fn take_wanted(&mut self) -> Vec<Resolvable> {
        std::mem::take(&mut self.wanted)
    }

    /// A head's triangles moved into its skin, when the stack is a player head whose skin is
    /// found; whether they were.
    pub fn skin(
        &mut self,
        stack: &ClientItemStackState,
        renderer: &BlockRenderer,
        vertices: &mut [GuiItemVertex],
    ) -> bool {
        if stack.item_registry_id() != photon_data::item::registry::named("minecraft:player_head") {
            return false;
        }
        let Some(profile) = item_profile::stack_profile(stack) else {
            return false;
        };
        let key = key_of(&profile);
        self.wanted.push(profile);
        let (Some(rect), Some(default)) = (
            self.found.get(&key).copied(),
            renderer.atlas.sprites.get(DEFAULT_SKIN).copied(),
        ) else {
            return false;
        };
        for vertex in vertices {
            vertex.uv = moved(vertex.uv, default, rect);
        }
        true
    }
}
