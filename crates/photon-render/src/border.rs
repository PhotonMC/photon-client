//! The world border overlay: `WorldBorderRenderer.extract`,
//! `WorldBorderRenderState.closestBorder` and the vertex math
//! `rebuildWorldBorderBuffer` lays out.
//!
//! The reference caches its quads in a GPU buffer and only rebuilds them
//! when the border itself changes size, translating the cached geometry
//! toward the camera in the shader between rebuilds (`ModelOffset`). This
//! renderer has no such cache: it lays the same quads out fresh in world
//! space every frame from the current camera, which only differs from the
//! reference when the camera has moved since its last rebuild (crossing a
//! section) without the border changing -- a narrower view of a wall that is
//! already far past the fog, not a different border.

use crate::scene::Vertex;
use photon_core::math::{ceil_f64, clamp_f64, fdlibm_pow, frac_f64, lfloor_f64};
use photon_world::ClientWorldBorder;

/// `WorldBorderRenderState`'s `alpha` and `tint` fields, as `extract` fills
/// them in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderExtract {
    pub alpha: f64,
    /// `ARGB.red/green/blue` of `BorderStatus.getColor()`, each already
    /// divided by 255.
    pub tint: [f32; 3],
}

/// `WorldBorderRenderer.extract`: whether the camera sits close enough to
/// any edge of the border (within `render_distance`, against the edges at
/// `partial_tick`) while still inside its bounding box widened by that same
/// distance; if so, how transparent the overlay is (the border's own
/// distance to its nearest edge, normalised and raised to the fourth power)
/// and which status colour tints it.
#[expect(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "transcribed from the reference's own `!(a < b)` conditions, whose NaN handling \
              differs from the `a >= b` clippy suggests"
)]
pub fn extract(
    border: &ClientWorldBorder,
    partial_tick: f32,
    (camera_x, camera_z): (f64, f64),
    render_distance: f64,
) -> BorderExtract {
    let (min_x, max_x) = (border.min_x(partial_tick), border.max_x(partial_tick));
    let (min_z, max_z) = (border.min_z(partial_tick), border.max_z(partial_tick));
    let near_an_edge = !(camera_x < max_x - render_distance)
        || !(camera_x > min_x + render_distance)
        || !(camera_z < max_z - render_distance)
        || !(camera_z > min_z + render_distance);
    let within_bounds = !(camera_x < min_x - render_distance)
        && !(camera_x > max_x + render_distance)
        && !(camera_z < min_z - render_distance)
        && !(camera_z > max_z + render_distance);
    if !(near_an_edge && within_bounds) {
        return BorderExtract {
            alpha: 0.0,
            tint: [0.0, 0.0, 0.0],
        };
    }
    let distance = border.distance_to_border(camera_x, camera_z);
    let alpha = clamp_f64(fdlibm_pow(1.0 - distance / render_distance, 4.0), 0.0, 1.0);
    let status_color = border.status().color();
    let tint = [
        ((status_color >> 16) & 0xFF) as f32 / 255.0,
        ((status_color >> 8) & 0xFF) as f32 / 255.0,
        (status_color & 0xFF) as f32 / 255.0,
    ];
    BorderExtract { alpha, tint }
}

/// `Direction.get2DDataValue()` for the four horizontal directions, the way
/// `WorldBorderRenderState.closestBorder` and the renderer's quad order
/// agree on them: south, west, north, east.
pub const SOUTH: u8 = 0;
pub const WEST: u8 = 1;
pub const NORTH: u8 = 2;
pub const EAST: u8 = 3;

/// `WorldBorderRenderState.closestBorder`: the four sides, nearest first (a
/// stable sort, so a tie keeps north/south/west/east order), each tagged by
/// its `get2DDataValue()` and its signed distance from the camera.
pub fn closest_sides(
    min_x: f64,
    max_x: f64,
    min_z: f64,
    max_z: f64,
    x: f64,
    z: f64,
) -> [(u8, f64); 4] {
    let mut sides = [
        (NORTH, z - min_z),
        (SOUTH, max_z - z),
        (WEST, x - min_x),
        (EAST, max_x - x),
    ];
    sides.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("border distances are finite"));
    sides
}

/// `WorldBorderRenderer.rebuildWorldBorderBuffer` and `render`'s per-vertex
/// math, placed directly in world space (minus the mesh `origin`) and only
/// for the sides `closest_sides` finds within `render_distance`, nearest
/// first -- the reference draws in that same order, though its blend mode
/// (`BlendFunction.OVERLAY`) makes the order invisible on screen.
#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the reference method's own parameter list"
)]
pub fn quads(
    border_min_x: f64,
    border_max_x: f64,
    border_min_z: f64,
    border_max_z: f64,
    camera: [f64; 3],
    origin: [i32; 3],
    render_distance: f64,
    half_height_y: f32,
    time_millis: i64,
    tint: [f32; 3],
    alpha: f32,
) -> Vec<(u8, [Vertex; 4])> {
    let [camera_x, camera_y, camera_z] = camera;
    let min_z = (lfloor_f64(camera_z - render_distance) as f64).max(border_min_z);
    let max_z = f64::from(ceil_f64(camera_z + render_distance)).min(border_max_z);
    let min_x = (lfloor_f64(camera_x - render_distance) as f64).max(border_min_x);
    let max_x = f64::from(ceil_f64(camera_x + render_distance)).min(border_max_x);
    let u0z = (lfloor_f64(min_z) & 1) as f32 * 0.5;
    let u1z = ((max_z - min_z) / 2.0) as f32;
    let u0x = (lfloor_f64(min_x) & 1) as f32 * 0.5;
    let u1x = ((max_x - min_x) / 2.0) as f32;
    let scroll = (time_millis.rem_euclid(3000) as f32) / 3000.0;
    let v0 = -(frac_f64(camera_y * 0.5) as f32);
    let v1 = v0 + half_height_y;
    // The reference's `ModelOffset.y` is `-cameraPos.y`, which cancels the
    // camera term out of the local `+/-halfHeightY` vertex: the wall sits at
    // the same absolute height (minus the mesh origin) no matter where the
    // camera stands, only its texture (`v0`/`v1`, above) tracks the camera.
    let origin_y = f64::from(origin[1]);
    let y0 = -half_height_y - origin_y as f32;
    let y1 = half_height_y - origin_y as f32;
    let ox = f64::from(origin[0]);
    let oz = f64::from(origin[2]);
    let sx = |value: f64| (value - ox) as f32;
    let sz = |value: f64| (value - oz) as f32;
    let corner = |position: [f32; 3], uv: [f32; 2]| Vertex {
        position,
        uv: [uv[0] + scroll, uv[1] + scroll],
        shade: 1.0,
        tint: [tint[0], tint[1], tint[2], alpha],
        light: [240.0, 240.0],
    };
    let quad_for = |side: u8| -> [Vertex; 4] {
        match side {
            SOUTH => [
                corner([sx(min_x), y0, sz(border_max_z)], [u0x, v1]),
                corner([sx(max_x), y0, sz(border_max_z)], [u1x + u0x, v1]),
                corner([sx(max_x), y1, sz(border_max_z)], [u1x + u0x, v0]),
                corner([sx(min_x), y1, sz(border_max_z)], [u0x, v0]),
            ],
            WEST => [
                corner([sx(min_x), y0, sz(min_z)], [u0z, v1]),
                corner([sx(min_x), y0, sz(max_z)], [u1z + u0z, v1]),
                corner([sx(min_x), y1, sz(max_z)], [u1z + u0z, v0]),
                corner([sx(min_x), y1, sz(min_z)], [u0z, v0]),
            ],
            NORTH => [
                corner([sx(max_x), y0, sz(min_z)], [u0x, v1]),
                corner([sx(min_x), y0, sz(min_z)], [u1x + u0x, v1]),
                corner([sx(min_x), y1, sz(min_z)], [u1x + u0x, v0]),
                corner([sx(max_x), y1, sz(min_z)], [u0x, v0]),
            ],
            _ => [
                corner([sx(border_max_x), y0, sz(max_z)], [u0z, v1]),
                corner([sx(border_max_x), y0, sz(min_z)], [u1z + u0z, v1]),
                corner([sx(border_max_x), y1, sz(min_z)], [u1z + u0z, v0]),
                corner([sx(border_max_x), y1, sz(max_z)], [u0z, v0]),
            ],
        }
    };
    let mut out = Vec::new();
    for (side, distance) in closest_sides(
        border_min_x,
        border_max_x,
        border_min_z,
        border_max_z,
        camera_x,
        camera_z,
    ) {
        if distance < render_distance {
            out.push((side, quad_for(side)));
        }
    }
    out
}

/// Two triangles from a quad's four corners, in the reference's own winding
/// (`0,1,2,2,3,0`); the pipeline draws triangles, the reference draws quads.
pub fn triangulate(corners: [Vertex; 4]) -> [Vertex; 6] {
    [
        corners[0], corners[1], corners[2], corners[2], corners[3], corners[0],
    ]
}
