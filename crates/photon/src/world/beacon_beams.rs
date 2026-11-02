//! The beams of the beacons (`BeaconRenderer`): drawn out to the render distance, whether the
//! beacon is in view or not, and as tall as the fog lets them be seen.

use photon_net::connection::PlaySession;
use photon_render::block_entity::beacon::{self, BeamVertex, faces_camera, repeats};
use photon_render::entity::textures::{EntityTextures, TextureRect, uv};
use photon_render::joml::Pose;
use photon_render::scene::{Vertex, entity_colour};
use photon_world::{BeaconView, GatewayBeamView};

use crate::overlays::FrameView;

/// How far above the camera the last stretch of a beam is drawn, beyond the render distance: the
/// fog has hidden it by then.
const BEYOND_FOG: f64 = 64.0;

/// Which layers of the beams are drawn: the opaque inner ones with the entities, and the
/// translucent outer ones (`BEACON_BEAM_TRANSLUCENT`: blended, tested but not written to the
/// depth) with the shapes that are drawn that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamLayers {
    Opaque,
    Translucent,
}

/// The beams the level's beacons shine.
pub fn beacon_beams(
    session: &PlaySession,
    textures: &EntityTextures,
    view: FrameView,
    (layers, vertices): (BeamLayers, &mut Vec<Vertex>),
) {
    let camera = [0, 1, 2].map(|axis| f64::from(view.origin[axis]) + f64::from(view.camera[axis]));
    gateway_beams(session, textures, (view, camera), (layers, vertices));
    let Some(rect) = textures.rect(beacon::TEXTURE) else {
        return;
    };
    let scoping = session.player().is_some_and(|player| player.is_scoping());
    for beacon in session.world().beacon_views(view.partial as f32) {
        if !beacon.sections.is_empty() {
            one_beam(&beacon, (rect, camera), (view, scoping), (layers, vertices));
        }
    }
}

/// One beacon's beam, unless it is too far off along the ground to be drawn.
fn one_beam(
    beacon: &BeaconView,
    (rect, camera): (TextureRect, [f64; 3]),
    (view, scoping): (FrameView, bool),
    (layers, vertices): (BeamLayers, &mut Vec<Vertex>),
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
    let scale = if scoping {
        1.0
    } else {
        ((near.sqrt() as f32) / 96.0).max(1.0)
    };
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
    let quads = beacon::beam_layers(&base, &sections, (beacon.animation_time, scale), end);
    push_layers(&quads, (rect, axis, view.camera), (layers, vertices));
}

/// The quads of a beam's layers that face the camera, cut where their texture repeats.
fn push_layers(
    layers: &[beacon::BeamLayer],
    (rect, axis, camera): (TextureRect, (f32, f32), [f32; 3]),
    (drawn, vertices): (BeamLayers, &mut Vec<Vertex>),
) {
    for layer in layers {
        if layer.translucent != (drawn == BeamLayers::Translucent) {
            continue;
        }
        let light = if layer.translucent {
            [-1.0, 32.0 / 255.0]
        } else {
            [-1.0, -1.0]
        };
        for quad in layer
            .quads
            .iter()
            .filter(|quad| faces_camera(quad, axis, camera))
        {
            for piece in repeats(quad) {
                for index in [0, 1, 2, 0, 2, 3] {
                    vertices.push(beam_vertex(&piece[index], rect, (light, layer.translucent)));
                }
            }
        }
    }
}

/// `TheEndGatewayRenderer.getViewDistance`.
const GATEWAY_VIEW_DISTANCE: f64 = 256.0;

/// The beams the level's end gateways shine, for those within the renderer's view distance.
fn gateway_beams(
    session: &PlaySession,
    textures: &EntityTextures,
    (view, camera): (FrameView, [f64; 3]),
    (layers, vertices): (BeamLayers, &mut Vec<Vertex>),
) {
    let Some(rect) = textures.rect(beacon::GATEWAY_TEXTURE) else {
        return;
    };
    for gateway in session.world().gateway_beam_views(view.partial as f32) {
        let (x, y, z) = gateway.position;
        let offset = [
            f64::from(x) + 0.5 - camera[0],
            f64::from(y) + 0.5 - camera[1],
            f64::from(z) + 0.5 - camera[2],
        ];
        // `Vec3.closerThan`: by how far off in any direction.
        if offset.iter().map(|d| d * d).sum::<f64>() >= GATEWAY_VIEW_DISTANCE.powi(2) {
            continue;
        }
        one_gateway_beam(&gateway, (rect, view), (layers, vertices));
    }
}

fn one_gateway_beam(
    gateway: &GatewayBeamView,
    (rect, view): (TextureRect, FrameView),
    (layers, vertices): (BeamLayers, &mut Vec<Vertex>),
) {
    let (x, y, z) = gateway.position;
    let mut base = Pose::default();
    base.translate(
        (x - view.origin[0]) as f32,
        (y - view.origin[1]) as f32,
        (z - view.origin[2]) as f32,
    );
    let axis = (
        (x - view.origin[0]) as f32 + 0.5,
        (z - view.origin[2]) as f32 + 0.5,
    );
    let quads = beacon::gateway_beam_layers(
        &base,
        (gateway.scale, gateway.animation_time),
        gateway.height,
        gateway.color,
    );
    push_layers(&quads, (rect, axis, view.camera), (layers, vertices));
}

/// One corner of a beam's quad as the shaders take it: lit by nothing, its colour the beam's. An
/// opaque layer goes to the entity shader, which takes the colour packed and the alpha in the
/// light; a translucent one to the shadow shader, whose tint is the colour and alpha as they
/// are.
fn beam_vertex(
    vertex: &BeamVertex,
    rect: TextureRect,
    (light, translucent): ([f32; 2], bool),
) -> Vertex {
    let rgb = vertex.color & 0x00FF_FFFF;
    let tint = if translucent {
        let channel = |shift: u32| ((rgb >> shift) & 0xFF) as f32 / 255.0;
        [channel(16), channel(8), channel(0), light[1]]
    } else {
        [0.0, 0.0, 0.0, entity_colour(rgb)]
    };
    Vertex {
        position: vertex.position,
        uv: uv(rect, vertex.uv),
        shade: 0.0,
        tint,
        light,
    }
}
