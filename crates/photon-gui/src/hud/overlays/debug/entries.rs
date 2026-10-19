//! The lines the entries add (`DebugScreenEntries`), from what the camera entity and the
//! connection say.

use super::format::fixed;
use super::layout::Line;
use photon_core::direction::Direction;
use photon_core::math::{floor_f64, wrap_degrees_f32};

/// The camera entity (`Minecraft.getCameraEntity`) and the level it is in.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraEntity {
    pub position: (f64, f64, f64),
    pub yaw: f32,
    pub pitch: f32,
    /// The dimension's id, such as `minecraft:overworld`.
    pub dimension: String,
}

/// What the entries read.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    /// `Minecraft <version> (<launched version>/<brand>)`.
    pub version: String,
    pub camera: Option<CameraEntity>,
}

/// The lines of one entry's `display`.
pub fn display(id: &str, input: &Input) -> Vec<Line> {
    match id {
        "game_version" => vec![Line::Priority(input.version.clone())],
        "player_position" => input.camera.as_ref().map(position).unwrap_or_default(),
        "player_section_position" => input.camera.as_ref().map(section).unwrap_or_default(),
        _ => Vec::new(),
    }
}

const POSITION_GROUP: &str = "minecraft:position";

fn feet(camera: &CameraEntity) -> (i32, i32, i32) {
    let (x, y, z) = camera.position;
    (floor_f64(x), floor_f64(y), floor_f64(z))
}

/// `DebugEntryPosition`: the coordinates, the block, the chunk and its region file, the facing
/// and the dimension with its forced chunks (none on a client level).
fn position(camera: &CameraEntity) -> Vec<Line> {
    let (x, y, z) = feet(camera);
    let (chunk_x, chunk_z) = (x >> 4, z >> 4);
    let direction = Direction::from_y_rot(f64::from(camera.yaw));
    let (name, face) = match direction {
        Direction::North => ("north", "Towards negative Z"),
        Direction::South => ("south", "Towards positive Z"),
        Direction::West => ("west", "Towards negative X"),
        Direction::East => ("east", "Towards positive X"),
        Direction::Up => ("up", "Invalid"),
        Direction::Down => ("down", "Invalid"),
    };
    let (px, py, pz) = camera.position;
    vec![Line::Group(
        POSITION_GROUP,
        vec![
            format!(
                "XYZ: {} / {} / {}",
                fixed(px, 3),
                fixed(py, 5),
                fixed(pz, 3)
            ),
            format!("Block: {x} {y} {z}"),
            format!(
                "Chunk: {chunk_x} {} {chunk_z} [{} {} in r.{}.{}.mca]",
                y >> 4,
                chunk_x & 31,
                chunk_z & 31,
                chunk_x >> 5,
                chunk_z >> 5
            ),
            format!(
                "Facing: {name} ({face}) ({} / {})",
                fixed(f64::from(wrap_degrees_f32(camera.yaw)), 1),
                fixed(f64::from(wrap_degrees_f32(camera.pitch)), 1)
            ),
            format!("{} FC: 0", camera.dimension),
        ],
    )]
}

/// `DebugEntrySectionPosition`: the block's place in its 16-block section.
fn section(camera: &CameraEntity) -> Vec<Line> {
    let (x, y, z) = feet(camera);
    vec![Line::Group(
        POSITION_GROUP,
        vec![format!(
            "Section-relative: {:02} {:02} {:02}",
            x & 15,
            y & 15,
            z & 15
        )],
    )]
}
