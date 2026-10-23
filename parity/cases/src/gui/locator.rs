//! The locator bar and name tags.

use serde_json::{Value, json};

use super::gui;
use crate::bits::{f32_bits, f64_bits, hex, varint};
use crate::line::float_text;

/// What a waypoint is tracked by: a name or a UUID's two halves.
#[derive(Clone, Copy)]
enum Ident {
    Name(&'static str),
    Uuid(i64, i64),
}

/// Where a waypoint is: nothing, a block, a chunk or an azimuth.
enum Place {
    Empty,
    Block(i64, i64, i64),
    Chunk(i64, i64),
    Azimuth(f32),
}

fn utf(text: &str) -> Vec<u8> {
    [varint(text.len() as i64), text.as_bytes().to_vec()].concat()
}

/// A `ClientboundTrackedWaypointPacket` body as hex: the operation, the
/// identifier, the icon, the kind and its contents.
fn waypoint(operation: i64, ident: Ident, place: &Place, (style, color): (&str, Option<[u8; 3]>)) -> String {
    let mut body = varint(operation);
    match ident {
        Ident::Uuid(most, least) => {
            body.push(1);
            body.extend((most as u64).to_be_bytes());
            body.extend((least as u64).to_be_bytes());
        }
        Ident::Name(name) => {
            body.push(0);
            body.extend(utf(name));
        }
    }
    body.extend(utf(style));
    match color {
        None => body.push(0),
        Some(color) => {
            body.push(1);
            body.extend(color);
        }
    }
    match place {
        Place::Empty => body.extend(varint(0)),
        Place::Block(x, y, z) => body.extend([varint(1), varint(*x), varint(*y), varint(*z)].concat()),
        Place::Chunk(x, z) => body.extend([varint(2), varint(*x), varint(*z)].concat()),
        Place::Azimuth(angle) => {
            body.extend(varint(3));
            body.extend(angle.to_be_bytes());
        }
    }
    hex(&body)
}

const DEFAULT_STYLE: (&str, Option<[u8; 3]>) = ("minecraft:default", None);
const ME: (i64, i64) = (0x1234_5678_90ab_cdef, -0x1234_5678_90ab_cdef);

fn block(name: &'static str, x: i64, y: i64, z: i64) -> String {
    waypoint(0, Ident::Name(name), &Place::Block(x, y, z), DEFAULT_STYLE)
}

fn styled(name: &'static str, (x, y, z): (i64, i64, i64), style: (&str, Option<[u8; 3]>)) -> String {
    waypoint(0, Ident::Name(name), &Place::Block(x, y, z), style)
}

fn tracked(id: i64, place: Place) -> String {
    waypoint(0, Ident::Uuid(id, id), &place, DEFAULT_STYLE)
}

/// A camera at `eye`, turned by `yaw` and `pitch`.
struct Camera {
    yaw: f64,
    pitch: f64,
    size: (i64, i64),
    eye: [f64; 3],
    fov: f64,
}

const CAMERA: Camera = Camera { yaw: 0.0, pitch: 0.0, size: (427, 240), eye: [0.5, 65.62, 0.5], fov: 70.0 };

fn bar(label: &str, packets: Vec<String>, camera: Camera) -> Value {
    let eye = camera.eye;
    json!({
        "label": label, "width": camera.size.0, "height": camera.size.1, "packets": packets,
        "camera": {"yaw": camera.yaw, "pitch": camera.pitch, "pos": eye, "fov": camera.fov, "aspect": 16.0 / 9.0, "far": 768.0},
        "player": {"pos": [eye[0], eye[1] - 1.62, eye[2]], "uuid": [ME.0, ME.1]},
    })
}

/// A whole number when the value is one, as `%g` writes it.
fn short(value: f64) -> String {
    if value.fract() == 0.0 { format!("{}", value as i64) } else { float_text(value) }
}

fn distance_rows() -> Vec<Value> {
    let mut rows = vec![bar("empty", vec![], CAMERA)];
    // Ahead is +z at yaw 0; the bar spans sixty degrees either way.
    for blocks in [10, 60, 127, 128, 160, 200, 250, 331, 332, 400, 3000] {
        rows.push(bar(&format!("distance-{blocks}"), vec![block("ahead", 0, 64, blocks)], CAMERA));
    }
    for degrees in [-70, -61, -60, -59, -45, -30, -10, -1, 0, 1, 10, 30, 45, 59, 60, 61, 90, 135, 180] {
        let radians = f64::from(degrees).to_radians();
        let (x, z) = (-radians.sin() * 50.0, radians.cos() * 50.0);
        let place = block("w", x.round_ties_even() as i64, 64, z.round_ties_even() as i64);
        rows.push(bar(&format!("angle-{degrees}"), vec![place], CAMERA));
    }
    let heights = [
        ("above", "up", 160, 20),
        ("below", "down", -60, 20),
        ("level", "level", 66, 20),
        ("behind-above", "up", 160, -20),
        ("behind-below", "down", -60, -20),
        ("behind-level", "level", 66, -20),
        ("overhead", "up", 300, 1),
    ];
    for (label, name, y, z) in heights {
        rows.push(bar(label, vec![block(name, 0, y, z)], CAMERA));
    }
    rows
}

fn icon_rows() -> Vec<Value> {
    let colour = |color: [u8; 3]| ("minecraft:default", Some(color));
    let bowtie = ("minecraft:bowtie", None);
    vec![
        bar(
            "colours",
            vec![
                styled("red", (-20, 64, 50), colour([255, 0, 0])),
                styled("green", (0, 64, 50), colour([0, 255, 0])),
                block("hashed-a", 20, 64, 50),
                block("hashed-b", -40, 64, 50),
                block("hashed-c", 40, 64, 50),
            ],
            CAMERA,
        ),
        bar(
            "grey-and-dark",
            vec![
                styled("grey", (-20, 64, 50), colour([128, 128, 128])),
                styled("black", (0, 64, 50), colour([0, 0, 0])),
                styled("white", (20, 64, 50), colour([255, 255, 255])),
            ],
            CAMERA,
        ),
        bar(
            "bowtie",
            vec![
                styled("near", (-20, 64, 30), bowtie),
                styled("mid", (0, 64, 80), bowtie),
                styled("far", (20, 64, 300), bowtie),
                styled("end", (40, 64, 500), bowtie),
            ],
            CAMERA,
        ),
        bar(
            "unknown-style",
            vec![
                styled("near", (-20, 64, 30), ("minecraft:nope", None)),
                styled("far", (20, 64, 300), ("minecraft:nope", None)),
            ],
            CAMERA,
        ),
    ]
}

fn kind_rows() -> Vec<Value> {
    let chunks = vec![
        tracked(1, Place::Chunk(0, 4)),
        tracked(2, Place::Chunk(-3, 6)),
        tracked(3, Place::Chunk(3, 6)),
        tracked(4, Place::Chunk(0, -5)),
    ];
    let azimuths = vec![
        tracked(5, Place::Azimuth(0.0)),
        tracked(6, Place::Azimuth(0.5)),
        tracked(7, Place::Azimuth(-0.7)),
        tracked(8, Place::Azimuth(3.0)),
    ];
    let own = waypoint(0, Ident::Uuid(ME.0, ME.1), &Place::Azimuth(0.0), DEFAULT_STYLE);
    let overlap = vec![block("far", 0, 64, 300), block("mid", 1, 64, 150), block("near", 2, 64, 20)];
    vec![
        bar("chunks", chunks, CAMERA),
        bar("azimuths", azimuths, CAMERA),
        bar("azimuth-looking-up", vec![tracked(5, Place::Azimuth(0.3))], Camera { pitch: -60.0, ..CAMERA }),
        bar("azimuth-looking-down", vec![tracked(5, Place::Azimuth(0.3))], Camera { pitch: 60.0, ..CAMERA }),
        bar("chunk-looking-up", vec![tracked(1, Place::Chunk(0, 4))], Camera { pitch: -80.0, ..CAMERA }),
        bar("empty-waypoint", vec![tracked(9, Place::Empty)], CAMERA),
        bar("own-waypoint", vec![own, block("other", 5, 64, 50)], CAMERA),
        bar("overlap", overlap, CAMERA),
    ]
}

fn camera_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for yaw in [90.0, -90.0, 180.0, 370.0, -190.0, 45.5] {
        let compass = vec![
            block("north", 0, 64, -50),
            block("east", 50, 64, 0),
            block("west", -50, 64, 0),
            block("south", 0, 64, 50),
        ];
        rows.push(bar(&format!("yaw-{}", short(yaw)), compass, Camera { yaw, ..CAMERA }));
    }
    for pitch in [-90.0, -45.0, 45.0, 90.0] {
        let pair = vec![block("ahead", 0, 70, 30), block("low", 0, 55, 30)];
        rows.push(bar(&format!("pitch-{}", short(pitch)), pair, Camera { pitch, ..CAMERA }));
    }
    let moved = waypoint(2, Ident::Name("a"), &Place::Block(-30, 64, 50), DEFAULT_STYLE);
    let removed = waypoint(1, Ident::Name("a"), &Place::Block(0, 0, 0), DEFAULT_STYLE);
    rows.extend([
        bar("wide-fov", vec![block("up", 0, 100, 30)], Camera { fov: 110.0, ..CAMERA }),
        bar("narrow-fov", vec![block("up", 0, 80, 30)], Camera { fov: 30.0, ..CAMERA }),
        bar("far-corner", vec![block("a", -30000, 64, 30000)], Camera { eye: [-29990.5, 70.0, 29900.5], ..CAMERA }),
        bar("updated", vec![block("a", 0, 64, 50), block("b", 20, 64, 50), moved], CAMERA),
        bar("untracked", vec![block("a", 0, 64, 50), block("b", 20, 64, 50), removed], CAMERA),
        bar("retracked", vec![block("a", 0, 64, 50), block("a", 20, 64, 80)], CAMERA),
        bar("small-screen", vec![block("a", 0, 64, 50), block("b", 30, 90, 50)], Camera { size: (320, 180), ..CAMERA }),
        bar("tall-screen", vec![block("a", 0, 64, 50)], Camera { size: (213, 240), ..CAMERA }),
    ]);
    rows
}

/// The locator bar: where a dot sits for each heading off the camera's, the
/// sprite each distance picks, the arrows for a waypoint above or below,
/// colours, chunk and azimuth waypoints, the viewer's own waypoint left out,
/// the farthest drawn first, a turned and a pitched camera, and the
/// operations that move and remove waypoints.
pub fn locator_bar() -> Value {
    gui("locator_bar", "locator_bar", [distance_rows(), icon_rows(), kind_rows(), camera_rows()].concat())
}

/// A remote player's name tag scenario.
#[derive(Clone, Copy)]
pub(super) struct Tag {
    pub(super) name: &'static str,
    pub(super) position: [f64; 3],
    pub(super) y_rot: f64,
    pub(super) crouching: bool,
    pub(super) pose: Option<&'static str>,
    pub(super) scale: Option<f64>,
    pub(super) shift: bool,
    pub(super) invisible: bool,
    pub(super) vehicle: bool,
    pub(super) camera_rot: (f64, f64),
    pub(super) spectator: bool,
    pub(super) hud_hidden: bool,
    pub(super) light: i64,
    pub(super) chat_only: bool,
    pub(super) opacity: f64,
}

pub(super) const TAG: Tag = Tag {
    name: "Steve",
    position: [10.5, 64.0, 5.5],
    y_rot: 0.0,
    crouching: false,
    pose: None,
    scale: None,
    shift: false,
    invisible: false,
    vehicle: false,
    camera_rot: (0.0, 0.0),
    spectator: false,
    hud_hidden: false,
    light: 15728880,
    chat_only: true,
    opacity: 0.25,
};

pub(super) fn tag(label: &str, tag: Tag) -> Value {
    let camera = [0.5, 65.62, 0.5];
    let mut row = json!({
        "label": label, "blocks": [], "chunk_radius": 0, "name": tag.name,
        "x": f64_bits(tag.position[0]), "y": f64_bits(tag.position[1]), "z": f64_bits(tag.position[2]),
        "y_rot": f32_bits(tag.y_rot), "crouching": tag.crouching, "shift_key_down": tag.shift,
        "invisible": tag.invisible, "vehicle": tag.vehicle, "camera_x": f64_bits(camera[0]),
        "camera_y": f64_bits(camera[1]), "camera_z": f64_bits(camera[2]), "camera_y_rot": f32_bits(tag.camera_rot.0),
        "camera_x_rot": f32_bits(tag.camera_rot.1), "local_spectator": tag.spectator, "hud_hidden": tag.hud_hidden,
        "light": tag.light, "background_for_chat_only": tag.chat_only, "background_opacity": f32_bits(tag.opacity),
    });
    if let Some(pose) = tag.pose {
        row["pose"] = json!(pose);
    }
    if let Some(scale) = tag.scale {
        row["scale"] = json!(f64_bits(scale));
    }
    row
}

/// A remote player's name tag as the reference's own renderer shows and
/// submits it: the rules (distance, crouching, invisible, spectating,
/// riding, the HUD) and the pose of the text for cameras that look in
/// different directions.
pub fn name_tags() -> Value {
    let mut rows = vec![tag("standing", TAG)];
    let looks = [
        ("look-south", 0.0, 0.0),
        ("look-east", -90.0, 0.0),
        ("look-north-down", 180.0, 45.0),
        ("look-up-left", 37.5, -60.0),
        ("odd-angles", 123.456, 17.25),
        ("past-a-turn", 400.0, -10.0),
    ];
    for (label, yaw, pitch) in looks {
        rows.push(tag(label, Tag { camera_rot: (yaw, pitch), ..TAG }));
    }
    for distance in [1.0, 20.0, 31.9, 32.1, 40.0, 63.9, 64.1] {
        let position = [0.5 + distance, 64.0, 0.5];
        rows.push(tag(&format!("standing-{}", float_text(distance)), Tag { position, ..TAG }));
        rows.push(tag(&format!("sneaking-{}", float_text(distance)), Tag { position, shift: true, ..TAG }));
    }
    rows.extend(rule_tags());
    rows.extend(super::mob_tags::rows());
    gui("name-tags", "name_tags", rows)
}

fn rule_tags() -> Vec<Value> {
    vec![
        tag("sleeping-pose", Tag { pose: Some("sleeping"), ..TAG }),
        tag("swimming-pose", Tag { pose: Some("swimming"), ..TAG }),
        tag("gliding-pose", Tag { pose: Some("fall_flying"), ..TAG }),
        tag("scaled-player", Tag { scale: Some(2.0), ..TAG }),
        tag("scaled-sleeping-player", Tag { pose: Some("sleeping"), scale: Some(2.0), ..TAG }),
        tag("crouching-pose", Tag { crouching: true, shift: true, ..TAG }),
        tag("crouching-pose-no-key", Tag { crouching: true, ..TAG }),
        tag("invisible", Tag { invisible: true, ..TAG }),
        tag("invisible-spectating", Tag { invisible: true, spectator: true, ..TAG }),
        tag("spectating", Tag { spectator: true, ..TAG }),
        tag("riding", Tag { vehicle: true, ..TAG }),
        tag("hud-hidden", Tag { hud_hidden: true, ..TAG }),
        tag("deadmau5", Tag { name: "deadmau5", ..TAG }),
        tag("wide-name", Tag { name: "WWWWWWWWWWWWWWWW", ..TAG }),
        tag("short-name", Tag { name: "I", ..TAG }),
        tag("dark-cave", Tag { light: 0, ..TAG }),
        tag("lit-by-torch", Tag { light: 15 << 4, ..TAG }),
        tag("sky-lit", Tag { light: 15 << 20, ..TAG }),
        tag("opacity-from-options", Tag { chat_only: false, opacity: 0.5, ..TAG }),
        tag("opacity-zero", Tag { chat_only: false, opacity: 0.0, ..TAG }),
        tag("opacity-full", Tag { chat_only: false, opacity: 1.0, ..TAG }),
        tag("facing-away", Tag { y_rot: 180.0, ..TAG }),
        tag("turned", Tag { y_rot: -73.5, ..TAG }),
        tag("below-the-camera", Tag { position: [10.5, 40.0, 5.5], ..TAG }),
        tag("far-below-spectating-sneak", Tag { position: [30.0, 50.0, 30.0], shift: true, spectator: true, ..TAG }),
    ]
}
