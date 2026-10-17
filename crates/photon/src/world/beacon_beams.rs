//! The beams of the beacons (`BeaconRenderer`): drawn out to the render distance, whether the
//! beacon is in view or not, and as tall as the fog lets them be seen.

use photon_net::connection::PlaySession;
use photon_render::block_entity::beacon::{self, BeamVertex, faces_camera, repeats};
use photon_render::entity::textures::{EntityTextures, TextureRect, uv};
use photon_render::joml::Pose;
use photon_render::scene::{Vertex, entity_colour};
use photon_world::BeaconView;

use crate::overlays::FrameView;

/// How far above the camera the last stretch of a beam is drawn, beyond the render distance: the
/// fog has hidden it by then.
const BEYOND_FOG: f64 = 64.0;

/// The beams the level's beacons shine.
pub fn beacon_beams(
    session: &PlaySession,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(rect) = textures.rect(beacon::TEXTURE) else {
        return;
    };
    let camera = [0, 1, 2].map(|axis| f64::from(view.origin[axis]) + f64::from(view.camera[axis]));
    for beacon in session.world().beacon_views(view.partial as f32) {
        if !beacon.sections.is_empty() {
            one_beam(&beacon, (rect, camera), view, vertices);
        }
    }
}

/// One beacon's beam, unless it is too far off along the ground to be drawn.
fn one_beam(
    beacon: &BeaconView,
    (rect, camera): (TextureRect, [f64; 3]),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let reach = f64::from(view.render_distance) * 16.0;
    let (x, y, z) = beacon.position;
    // `BeaconRenderer.shouldRender`: by how far off along the ground, in any direction.
    let (dx, dz) = (
        f64::from(x) + 0.5 - camera[0],
        f64::from(z) + 0.5 - camera[2],
    );
    let near = dx * dx + dz * dz;
    if near >= reach * reach {
        return;
    }
    // `BeaconRenderer.extract`: the beam is fatter the farther off it is.
    let scale = ((near.sqrt() as f32) / 96.0).max(1.0);
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let sections: Vec<(u32, i32)> = beacon
        .sections
        .iter()
        .map(|section| (section.color, section.height))
        .collect();
    let end = (camera[1] + reach + BEYOND_FOG - f64::from(y)).ceil() as i32;
    let axis = (
        (x - view.origin[0]) as f32 + 0.5,
        (z - view.origin[2]) as f32 + 0.5,
    );
    for layer in beacon::beam_layers(&base, &sections, (beacon.animation_time, scale), end) {
        let light = if layer.translucent {
            [-1.0, 32.0 / 255.0]
        } else {
            [-1.0, -1.0]
        };
        for quad in layer
            .quads
            .iter()
            .filter(|quad| faces_camera(quad, axis, view.camera))
        {
            for piece in repeats(quad) {
                for index in [0, 1, 2, 0, 2, 3] {
                    vertices.push(beam_vertex(&piece[index], rect, light));
                }
            }
        }
    }
}

/// One corner of a beam's quad as the entity shader takes it: lit by nothing, its colour the beam's
/// and its alpha the layer's.
fn beam_vertex(vertex: &BeamVertex, rect: TextureRect, light: [f32; 2]) -> Vertex {
    Vertex {
        position: vertex.position,
        uv: uv(rect, vertex.uv),
        shade: 0.0,
        tint: [0.0, 0.0, 0.0, entity_colour(vertex.color & 0x00FF_FFFF)],
        light,
    }
}
