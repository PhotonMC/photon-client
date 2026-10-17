//! The beacon's beam (`BeaconRenderer`): the quads of each stretch of it, as the reference works
//! them out, and the cut that lets a texture that does not repeat in the atlas draw as one that
//! does.

use crate::joml::{Pose, Quaternion};
use photon_core::math::{floor_f32, frac_f32};

/// The texture under `textures/` (`BeaconRenderer.BEAM_LOCATION`).
pub const TEXTURE: &str = "entity/beacon/beacon_beam";

/// `BeaconRenderer.SOLID_BEAM_RADIUS`.
const SOLID_RADIUS: f32 = 0.2;
/// `BeaconRenderer.BEAM_GLOW_RADIUS`.
const GLOW_RADIUS: f32 = 0.25;
/// `BeaconRenderer.MAX_RENDER_Y`: how tall the last stretch is.
pub const LAST_SECTION_HEIGHT: i32 = 2048;
/// The light of every vertex: full (`15728880`).
pub const FULL_LIGHT: i32 = 15_728_880;

/// One corner of a beam's quad as the reference writes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    /// ARGB.
    pub color: u32,
    pub normal: [f32; 3],
}

/// One quad of the beam: four corners, the end of the beam first.
pub type BeamQuad = [BeamVertex; 4];

/// The quads of one layer of one stretch of the beam: the solid core, or the glow around it.
#[derive(Debug, Clone, PartialEq)]
pub struct BeamLayer {
    pub translucent: bool,
    pub quads: Vec<BeamQuad>,
}

/// `BeaconRenderer.submit`: every layer of every stretch, in the order they are submitted. `base`
/// is the pose of the beacon's block, `sections` each stretch's colour (ARGB) and height (the
/// last is `LAST_SECTION_HEIGHT` tall whatever it says), `last_end` how far up the last stretch
/// is worked out to when it is not the whole of it.
pub fn beam_layers(
    base: &Pose,
    sections: &[(u32, i32)],
    (animation_time, radius_scale): (f32, f32),
    last_end: i32,
) -> Vec<BeamLayer> {
    let mut layers = Vec::new();
    let mut start = 0;
    for (index, (color, height)) in sections.iter().enumerate() {
        let last = index + 1 == sections.len();
        let height = if last {
            last_end.min(LAST_SECTION_HEIGHT)
        } else {
            *height
        };
        layers.extend(beacon_beam(
            base,
            animation_time,
            (start, height),
            (
                *color,
                SOLID_RADIUS * radius_scale,
                GLOW_RADIUS * radius_scale,
            ),
        ));
        start += sections[index].1;
        if last {
            break;
        }
    }
    layers
}

/// `BeaconRenderer.submitBeaconBeam` with the texture's own scale of one.
fn beacon_beam(
    base: &Pose,
    animation_time: f32,
    (start, height): (i32, i32),
    (color, solid_radius, glow_radius): (u32, f32, f32),
) -> [BeamLayer; 2] {
    let end = start + height;
    let mut pose = *base;
    pose.translate(0.5, 0.0, 0.5);
    let scroll = if height < 0 {
        animation_time
    } else {
        -animation_time
    };
    let v_offset = frac_f32(scroll * 0.2 - floor_f32(scroll * 0.1) as f32);
    // The solid core, turned as the beam's clock turns it.
    let mut turned = pose;
    turned.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        animation_time * 2.25 - 45.0,
    )));
    let r = solid_radius;
    let v2 = -1.0 + v_offset;
    let v1 = height as f32 * 1.0 * (0.5 / r) + v2;
    let solid = BeamLayer {
        translucent: false,
        quads: part(
            &turned,
            color,
            (start, end),
            [(0.0, r), (r, 0.0), (-r, 0.0), (0.0, -r)],
            (v1, v2),
        ),
    };
    let g = glow_radius;
    let v1 = height as f32 * 1.0 + v2;
    let glow = BeamLayer {
        translucent: true,
        quads: part(
            &pose,
            (32 << 24) | (color & 0x00FF_FFFF),
            (start, end),
            [(-g, -g), (g, -g), (-g, g), (g, g)],
            (v1, v2),
        ),
    };
    [solid, glow]
}

/// `BeaconRenderer.renderPart`: the four faces from the corners (west-north, east-north,
/// west-south, east-south, as `(x, z)`).
fn part(
    pose: &Pose,
    color: u32,
    (start, end): (i32, i32),
    [wn, en, ws, es]: [(f32, f32); 4],
    (v1, v2): (f32, f32),
) -> Vec<BeamQuad> {
    [(wn, en), (es, ws), (en, es), (ws, wn)]
        .into_iter()
        .map(|(from, to)| quad(pose, color, (start, end), (from, to), (v1, v2)))
        .collect()
}

/// `BeaconRenderer.renderQuad`: a face from one corner to the next, the end of the beam first.
fn quad(
    pose: &Pose,
    color: u32,
    (start, end): (i32, i32),
    (from, to): ((f32, f32), (f32, f32)),
    (v1, v2): (f32, f32),
) -> BeamQuad {
    let vertex = |y: i32, (x, z): (f32, f32), u: f32, v: f32| BeamVertex {
        position: pose.pose.transform_position(x, y as f32, z),
        uv: [u, v],
        color,
        normal: pose.transform_normal([0.0, 1.0, 0.0]),
    };
    [
        vertex(end, from, 1.0, v1),
        vertex(start, from, 1.0, v2),
        vertex(start, to, 0.0, v2),
        vertex(end, to, 0.0, v1),
    ]
}

/// A quad cut at every whole `v` it crosses, each piece's `v` made to start from its own repeat:
/// the beam's texture repeats along it, which the packed image does not. A piece keeps the corners'
/// order, with its `v` between zero and one.
pub fn repeats(quad: &BeamQuad) -> Vec<BeamQuad> {
    let [end_from, start_from, start_to, end_to] = *quad;
    let (low, high) = (start_from.uv[1], end_from.uv[1]);
    if high <= low {
        return vec![*quad];
    }
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32| {
        std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t)
    };
    let at = |v: f32| (v - low) / (high - low);
    let mut pieces = Vec::new();
    let mut cut = floor_f32(low);
    while (cut as f32) < high {
        let from_v = low.max(cut as f32);
        let to_v = high.min((cut + 1) as f32);
        let (t0, t1) = (at(from_v), at(to_v));
        let corner = |a: BeamVertex, b: BeamVertex, t: f32, v: f32| BeamVertex {
            position: lerp(a.position, b.position, t),
            uv: [a.uv[0], v - cut as f32],
            ..a
        };
        pieces.push([
            corner(start_from, end_from, t1, to_v),
            corner(start_from, end_from, t0, from_v),
            corner(start_to, end_to, t0, from_v),
            corner(start_to, end_to, t1, to_v),
        ]);
        cut += 1;
    }
    pieces
}

/// Whether a face of the beam shows from the camera: the reference culls the back of every quad,
/// and the quads' fronts look out of the beam.
pub fn faces_camera(quad: &BeamQuad, axis: (f32, f32), camera: [f32; 3]) -> bool {
    let centre: [f32; 3] =
        std::array::from_fn(|i| quad.iter().map(|v| v.position[i]).sum::<f32>() / 4.0);
    let outward = (centre[0] - axis.0, centre[2] - axis.1);
    (camera[0] - centre[0]) * outward.0 + (camera[2] - centre[2]) * outward.1 > 0.0
}
