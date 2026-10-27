//! Player head items held in a hand that carry a profile: drawn with the entity pictures, in the
//! skin the profile names, where every other item is drawn from the item atlas.

use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_render::scene::Vertex;

use crate::equipment::held_items::{Carried, HeldItem, held_item_vertices};
use crate::equipment::item_layers::ItemLayerCache;
use crate::skins::{Skins, Who, item_profile};

/// The picture an item model gives a player head, which carries no skin of its own.
const DEFAULT_SKIN: &str = "minecraft:entity/player/slim/steve";

/// A texture coordinate moved from one picture's place to another's.
fn moved(uv: [f32; 2], from: [f32; 4], to: TextureRect) -> [f32; 2] {
    let along = |value: f32, low: usize| {
        (value - from[low]) / (from[low + 2] - from[low]) * (to[low + 2] - to[low]) + to[low]
    };
    [along(uv[0], 0), along(uv[1], 1)]
}

/// The vertices of the held items that are player heads with a profile, in the entity pictures;
/// they leave `held`, whose rest the item pass draws.
pub fn skinned_head_vertices(
    held: &mut Vec<HeldItem>,
    layers: &mut [ItemLayerCache; 4],
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    (textures, skins): (&mut EntityTextures, &mut Skins),
) -> Vec<Vertex> {
    let head = photon_data::item::registry::named("minecraft:player_head");
    let Some(default) = renderer.atlas.sprites.get(DEFAULT_SKIN).copied() else {
        return Vec::new();
    };
    let mut vertices = Vec::new();
    let mut rest = Vec::new();
    for item in held.drain(..) {
        let profile = match &item.stack {
            Carried::Stack(stack) if stack.item_registry_id() == head => {
                item_profile::stack_profile(stack)
            }
            _ => None,
        };
        let resolved = profile.and_then(|profile| {
            let who = Who {
                id: profile.id,
                properties: &profile.properties,
                require_secure: false,
            };
            skins.resolve(textures, &who)
        });
        let Some(resolved) = resolved else {
            rest.push(item);
            continue;
        };
        let drawn = held_item_vertices(std::slice::from_ref(&item), layers, (renderer, tints));
        vertices.extend(drawn.into_iter().map(|mut vertex| {
            vertex.uv = moved(vertex.uv, default, resolved.rect);
            vertex
        }));
    }
    *held = rest;
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_coordinate_keeps_its_place_in_the_picture() {
        let uv = moved([0.15, 0.3], [0.1, 0.2, 0.2, 0.4], [0.5, 0.0, 0.7, 1.0]);
        assert!((uv[0] - 0.6).abs() < 1e-6);
        assert!((uv[1] - 0.5).abs() < 1e-6);
    }
}
