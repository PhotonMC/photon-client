//! Player head items held in a hand that carry a profile: drawn with the entity pictures, in the
//! skin the profile names, where every other item is drawn from the item atlas.

use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_render::scene::Vertex;

use crate::equipment::held_items::{Carried, HeldItem, held_item_vertices};
use crate::equipment::item_layers::ItemLayerCache;
use crate::skins::{Skins, Who, item_profile};
use photon_world::ClientItemStackState;

/// The picture an item model gives a player head, which carries no skin of its own.
const DEFAULT_SKIN: &str = "minecraft:entity/player/slim/steve";

/// A texture coordinate moved from one picture's place to another's.
fn moved(uv: [f32; 2], from: [f32; 4], to: TextureRect) -> [f32; 2] {
    let along = |value: f32, low: usize| {
        (value - from[low]) / (from[low + 2] - from[low]) * (to[low + 2] - to[low]) + to[low]
    };
    [along(uv[0], 0), along(uv[1], 1)]
}

/// An item mesh's vertices as the entity shader reads them: the entity shader lights a face from
/// the normal its tint carries and takes its shade as the hurt overlay, where the item shader
/// reads a shade the mesh already lit. The normal is the quad's, from its winding.
fn as_entity_vertices(vertices: &mut [Vertex]) {
    for quad in vertices.as_chunks_mut::<6>().0 {
        let [a, b, c] = [quad[0].position, quad[1].position, quad[2].position];
        let (u, v) = (
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
        );
        let normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let length = normal.iter().map(|part| part * part).sum::<f32>().sqrt();
        for vertex in quad {
            vertex.shade = 0.0;
            vertex.tint = if length > 0.0 {
                [
                    normal[0] / length,
                    normal[1] / length,
                    normal[2] / length,
                    0.0,
                ]
            } else {
                [0.0; 4]
            };
        }
    }
}

/// Where the skin of a held player head with a profile is, once it has loaded.
pub fn head_rect(
    stack: &ClientItemStackState,
    renderer: &BlockRenderer,
    (textures, skins): (&mut EntityTextures, &mut Skins),
    session: &photon_net::connection::PlaySession,
) -> Option<TextureRect> {
    let head = photon_data::item::registry::named("minecraft:player_head");
    if stack.item_registry_id() != head || !renderer.atlas.sprites.contains_key(DEFAULT_SKIN) {
        return None;
    }
    let profile = skins
        .lookups
        .profile(item_profile::stack_profile(stack)?, session);
    let who = Who {
        id: profile.id,
        properties: &profile.properties,
        require_secure: false,
    };
    Some(skins.resolve(textures, &who)?.rect)
}

/// A head's item mesh moved onto the skin at `rect`, in the vertices the entity pass reads;
/// `default` is the picture the item model gives it.
pub fn wear_skin_from(drawn: &mut [Vertex], default: [f32; 4], rect: TextureRect) {
    for vertex in drawn.iter_mut() {
        vertex.uv = moved(vertex.uv, default, rect);
    }
    as_entity_vertices(drawn);
}

/// [`wear_skin_from`] with the picture the renderer's item atlas has.
pub fn wear_skin(drawn: &mut [Vertex], renderer: &BlockRenderer, rect: TextureRect) {
    if let Some(default) = renderer.atlas.sprites.get(DEFAULT_SKIN).copied() {
        wear_skin_from(drawn, default, rect);
    }
}

/// The vertices of the held items that are player heads with a profile, in the entity pictures;
/// they leave `held`, whose rest the item pass draws.
pub fn skinned_head_vertices(
    held: &mut Vec<HeldItem>,
    layers: &mut [ItemLayerCache; 4],
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    (textures, skins): (&mut EntityTextures, &mut Skins),
    session: &photon_net::connection::PlaySession,
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let mut rest = Vec::new();
    for item in held.drain(..) {
        let rect = match &item.stack {
            Carried::Stack(stack) => head_rect(stack, renderer, (textures, skins), session),
            _ => None,
        };
        let Some(rect) = rect else {
            rest.push(item);
            continue;
        };
        let mut drawn = held_item_vertices(std::slice::from_ref(&item), layers, (renderer, tints));
        wear_skin(&mut drawn, renderer, rect);
        vertices.extend(drawn);
    }
    *held = rest;
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_meshs_normals_are_the_ones_a_model_gives_its_quads() {
        use photon_render::entity::mob_model::{MobModel, mob_vertices_posed};
        let layer = photon_data::entity_model::layer("minecraft:player_head#main").unwrap();
        let model = MobModel::bake(layer);
        let mut vertices = mob_vertices_posed(
            &model,
            &photon_render::joml::Pose::default(),
            [0.0, 0.0, 1.0, 1.0],
            (0, false),
        );
        let wanted: Vec<[f32; 4]> = vertices.iter().map(|vertex| vertex.tint).collect();
        for vertex in &mut vertices {
            vertex.tint = [1.0; 4];
            vertex.shade = 0.7;
        }
        as_entity_vertices(&mut vertices);
        for (vertex, wanted) in vertices.iter().zip(wanted) {
            for axis in 0..3 {
                assert!(
                    (vertex.tint[axis] - wanted[axis]).abs() < 1e-4,
                    "{vertex:?} {wanted:?}"
                );
            }
            assert_eq!(vertex.shade, 0.0);
        }
    }

    #[test]
    fn a_coordinate_keeps_its_place_in_the_picture() {
        let uv = moved([0.15, 0.3], [0.1, 0.2, 0.2, 0.4], [0.5, 0.0, 0.7, 1.0]);
        assert!((uv[0] - 0.6).abs() < 1e-6);
        assert!((uv[1] - 0.5).abs() < 1e-6);
    }
}
