//! The surfaces of the end portals and end gateways (`AbstractEndPortalRenderer`): the cube of
//! each, with the faces that show, drawn by the portal pipeline over where it is on the screen.

use photon_core::direction::Direction;
use photon_net::connection::PlaySession;
use photon_render::block_entity::end_portal::{cube_quads, portal_pose};
use photon_render::joml::Pose;
use photon_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::FrameView;

/// How far from the camera, in chunks, the sweep for portals reaches: the gateway's view distance
/// of 256 blocks.
const RADIUS_CHUNKS: i32 = 16;

/// `TheEndGatewayRenderer.getViewDistance`.
const GATEWAY_VIEW_DISTANCE: f64 = 256.0;

/// How many layers a portal and a gateway lay over the sky (`PORTAL_LAYERS` of their pipelines).
pub(crate) const PORTAL_LAYERS: f32 = 15.0;
pub(crate) const GATEWAY_LAYERS: f32 = 16.0;

/// The triangles of every portal and gateway surface in view, kept in `models` for the frame.
pub fn end_portals(session: &PlaySession, models: &mut MobModels, view: FrameView) {
    let world = session.world();
    models.portal_time = (world.game_time(), view.partial as f32);
    models.portals.clear();
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    let camera = [0, 1, 2].map(|axis| f64::from(view.origin[axis]) + f64::from(view.camera[axis]));
    for (x, y, z) in world.chest_positions_near(centre, RADIUS_CHUNKS) {
        let Some(surface) = world.portal_view((x, y, z)) else {
            continue;
        };
        // `BlockEntityRenderer.shouldRender`: the block's centre within the view distance, which a
        // gateway sets to 256 and a portal leaves at 64.
        let reach = if surface.gateway {
            GATEWAY_VIEW_DISTANCE
        } else {
            64.0
        };
        let centre = [x, y, z].map(|axis| f64::from(axis) + 0.5);
        let distance: f64 = (0..3)
            .map(|axis| (centre[axis] - camera[axis]).powi(2))
            .sum();
        if distance >= reach * reach {
            continue;
        }
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let pose = if surface.gateway {
            base
        } else {
            portal_pose(&base)
        };
        let layers = if surface.gateway {
            GATEWAY_LAYERS
        } else {
            PORTAL_LAYERS
        };
        push_cube(models, (&pose, layers), &surface.faces);
    }
}

/// The faces of a portal or gateway cube, drawn by the portal pipeline with `layers` over the sky.
pub(crate) fn push_cube(models: &mut MobModels, (pose, layers): (&Pose, f32), faces: &[Direction]) {
    for quad in cube_quads(pose, faces) {
        for index in [0, 1, 2, 0, 2, 3] {
            models.portals.push(Vertex {
                position: quad[index],
                uv: [0.0; 2],
                shade: 1.0,
                tint: [layers, 0.0, 0.0, 0.0],
                light: [0.0; 2],
            });
        }
    }
}
