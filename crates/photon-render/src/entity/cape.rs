//! A player's cape (`CapeLayer` and `PlayerCapeModel`): a slab on the back of the body, tipped
//! by how the player moves.

use crate::entity::player_model::{Part, PlayerModel};
use crate::entity::player_renderer::{Placement, placement_pose};
use crate::entity::textures::{TextureRect, uv};
use crate::joml::Quaternion;
use crate::scene::Vertex;

/// How the cape hangs, in degrees (`AvatarRenderState.capeLean`, `capeLean2`, `capeFlap`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CapeAngles {
    /// Swung back by the motion along the facing.
    pub lean: f32,
    /// Swung sideways by the motion across it.
    pub lean2: f32,
    /// Lifted by a rise and by the bob of the walk.
    pub flap: f32,
}

/// The cape cuboid on the back, in the layer `minecraft:player#cape`, as the mesh puts it.
fn mesh() -> &'static Part {
    static MESH: std::sync::OnceLock<Part> = std::sync::OnceLock::new();
    MESH.get_or_init(|| {
        let layer = photon_data::entity_model::layer("minecraft:player#cape")
            .expect("the cape layer is in the table");
        let definition = layer
            .root
            .children
            .iter()
            .find(|part| part.name == "body")
            .and_then(|body| body.children.iter().find(|part| part.name == "cape"))
            .expect("the cape layer has a cape on its body");
        Part::baked(definition, (layer.texture_width, layer.texture_height))
    })
}

/// `PlayerCapeModel.setupAnim`: the cape turned from where the mesh put it.
fn posed(angles: CapeAngles) -> Part {
    let radians = Quaternion::degrees_to_radians;
    let turn = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    }
    .rotate_y(-std::f32::consts::PI)
    .rotate_x(radians(6.0 + angles.lean / 2.0 + angles.flap))
    .rotate_z(radians(angles.lean2 / 2.0))
    .rotate_y(radians(180.0 - angles.lean2 / 2.0));
    let mut cape = mesh().clone();
    cape.rotate_by(&turn);
    cape
}

/// The cape of a player whose model `model` is posed, in the texture at `rect`. Chest armor
/// moves it out a little so it does not sink into the armor.
pub fn cape_vertices(
    (model, placement): (&PlayerModel, &Placement),
    (rect, angles): (TextureRect, CapeAngles),
    armored: bool,
) -> Vec<Vertex> {
    let cape = posed(angles);
    let mut pose = placement_pose(placement);
    if armored {
        pose.translate(0.0, -0.053_125, 0.068_75);
    }
    if let Some(body) = model
        .root
        .children()
        .iter()
        .find(|part| part.name() == "body")
    {
        body.translate_and_rotate(&mut pose);
    }
    let light = crate::block::light::light_uv(placement.light);
    let mut quads = Vec::new();
    cape.render_quads(&pose, &mut quads);
    let mut vertices = Vec::new();
    for corners in quads {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(rect, corner.uv),
                shade: crate::entity::mob_model::overlay_code(placement.red_overlay, 0.0),
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 0.0],
                light,
            });
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joml::Pose;

    /// Where the cape's own +z axis points after its pose.
    fn back_axis(angles: CapeAngles) -> [f32; 3] {
        let cape = posed(angles);
        let mut pose = Pose::default();
        cape.translate_and_rotate(&mut pose);
        let at = |z: f32| pose.pose.transform_position(0.0, 0.0, z);
        let (a, b) = (at(0.0), at(1.0));
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
    }

    fn close(a: [f32; 3], b: [f32; 3]) {
        assert!(
            a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-5),
            "{a:?} {b:?}"
        );
    }

    #[test]
    fn a_cape_at_rest_hangs_six_degrees_off_the_back() {
        // Turned half a circle about y (the mesh) and tipped six degrees about x, which with the
        // turn of the model carries the cape's z axis to (0, sin 6, -cos 6).
        let six = 6.0_f32.to_radians();
        close(
            back_axis(CapeAngles::default()),
            [0.0, six.sin(), -six.cos()],
        );
    }

    #[test]
    fn lean_and_flap_tip_the_cape_further() {
        // The tip about x is six degrees plus half the lean plus the flap.
        let angles = CapeAngles {
            lean: 40.0,
            lean2: 0.0,
            flap: 10.0,
        };
        let tip = 36.0_f32.to_radians();
        close(back_axis(angles), [0.0, tip.sin(), -tip.cos()]);
    }
}
