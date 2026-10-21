//! The lines the entries add (`DebugScreenEntries`), from what the camera entity and the
//! connection say.

mod impactors;
mod memory;
mod system;
pub use impactors::{Clouds, Filtering, Impactors};
pub use memory::MemoryUsage;
pub use system::{DeviceType, SystemSpecs, vendor_name};

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
    /// The level's lowest block row and its height (`isInsideBuildHeight`).
    pub build_height: (i32, i32),
    /// The block the view ends on within 20 blocks, fluids looked through.
    pub target: Option<TargetedBlock>,
    /// The sky and block light at the feet's block (`Level.getBrightness`).
    pub light: (i32, i32),
    /// The biome at the feet's quart position, `minecraft:plains` where the chunk is not there.
    pub biome: String,
}

/// A block position and what the level holds there (`DebugEntryLookingAt`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetedBlock {
    pub position: (i32, i32, i32),
    /// The block's registered name, such as `minecraft:oak_stairs`.
    pub name: String,
    /// Each property with its value's name, in the order the block lists them.
    pub properties: Vec<(String, String)>,
}

impl TargetedBlock {
    /// The block a state number belongs to; none for a number the registry lacks.
    pub fn of(position: (i32, i32, i32), state: photon_core::ids::BlockStateId) -> Option<Self> {
        let (block, offset) = photon_data::block::registry::block_for_state(state)?;
        Some(Self {
            position,
            name: block.name.to_owned(),
            properties: block
                .properties_at(offset)?
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value.to_owned()))
                .collect(),
        })
    }
}

/// How the window surface presents (`GpuSurface.PresentMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

impl PresentMode {
    /// From `VkPresentModeKHR`'s value: immediate, mailbox, fifo and fifo relaxed are 0 to 3.
    pub fn from_vulkan(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::Immediate),
            1 => Some(Self::Mailbox),
            2 => Some(Self::Fifo),
            3 => Some(Self::FifoRelaxed),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Immediate => " (immediate)",
            Self::Mailbox => " (mailbox)",
            Self::Fifo => " (fifo)",
            Self::FifoRelaxed => " (fifo relaxed)",
        }
    }
}

/// What the frame loop says of itself.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameStats {
    /// `Minecraft.getFps`.
    pub fps: i32,
    /// `FramerateLimitTracker.getFramerateLimit`; 260 is no limit.
    pub limit: i32,
    /// The surface's mode, none without a configuration.
    pub present_mode: Option<PresentMode>,
}

/// What the connection says of the server.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerInfo {
    /// `ClientPacketListener.serverBrand`, none until the server has said it.
    pub brand: Option<String>,
    /// The tick rate manager: stepping forward, frozen.
    pub stepping: bool,
    pub frozen: bool,
    /// `Connection.getAverageSentPackets` and `getAverageReceivedPackets`.
    pub sent: f32,
    pub received: f32,
}

/// What the entries read.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    /// `Minecraft <version> (<launched version>/<brand>)`.
    pub version: String,
    pub camera: Option<CameraEntity>,
    pub frame: FrameStats,
    pub server: Option<ServerInfo>,
    /// The options `simple_performance_impactors` reads, none without them.
    pub impactors: Option<Impactors>,
    /// The machine `system_specs` describes, none without it.
    pub system: Option<SystemSpecs>,
    /// What `memory` shows, none without it.
    pub memory: Option<MemoryUsage>,
    /// `Minecraft.getGpuUtilization`, none where the frame loop does not measure it.
    pub gpu_utilization: Option<f64>,
    /// The registered name of the entity type under the crosshair (`crosshairPickEntity`).
    pub crosshair_entity: Option<String>,
    /// The overworld day timeline's period count, none without the timeline in the registry.
    pub day_count: Option<i32>,
}

/// The lines of one entry's `display`.
pub fn display(id: &str, input: &Input) -> Vec<Line> {
    match id {
        "fps" => vec![Line::Priority(fps_line(&input.frame))],
        "tps" => input.server.as_ref().map(tps).unwrap_or_default(),
        "simple_performance_impactors" => input
            .impactors
            .as_ref()
            .map(impactors::display)
            .unwrap_or_default(),
        "memory" => input
            .memory
            .as_ref()
            .map(memory::display)
            .unwrap_or_default(),
        "system_specs" => input
            .system
            .as_ref()
            .map(system::display)
            .unwrap_or_default(),
        "game_version" => vec![Line::Priority(input.version.clone())],
        "day_count" => day(input.day_count),
        "gpu_utilization" => gpu(input.gpu_utilization),
        "looking_at_entity" => looking_at_entity(input.crosshair_entity.as_deref()),
        _ => input
            .camera
            .as_ref()
            .map(|camera| at_camera(id, camera))
            .unwrap_or_default(),
    }
}

/// The entries that describe where the camera entity is.
fn at_camera(id: &str, camera: &CameraEntity) -> Vec<Line> {
    match id {
        "player_position" => position(camera),
        "player_section_position" => section(camera),
        "biome" => biome(camera).into_iter().collect(),
        "light_levels" => light(camera),
        "looking_at_block_state" => looking_at_block(camera),
        _ => Vec::new(),
    }
}

/// `DebugEntryLookingAtEntity`: the entity type under the crosshair; the group stays when none is.
fn looking_at_entity(name: Option<&str>) -> Vec<Line> {
    let lines = name
        .map(|name| vec!["\u{a7}nTargeted Entity".to_owned(), name.to_owned()])
        .unwrap_or_default();
    vec![Line::Group(LOOKING_AT_ENTITY_GROUP, lines)]
}

/// `DebugEntryGpuUtilization`: the share of the frame the GPU was busy, in red from the second the
/// GPU is past a full frame (`Math.round` of the share otherwise).
fn gpu(utilization: Option<f64>) -> Vec<Line> {
    let Some(share) = utilization else {
        return Vec::new();
    };
    let shown = if share > 100.0 {
        "\u{a7}c100%".to_owned()
    } else {
        format!("{}%", java_round(share))
    };
    vec![Line::Regular(format!("GPU: {shown}"))]
}

/// `Math.round(double)`: the nearest whole number, halves up, NaN zero.
fn java_round(value: f64) -> i64 {
    let floor = value.floor();
    (if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    }) as i64
}

/// `DebugEntryDayCount`: the days the overworld clock has run.
fn day(count: Option<i32>) -> Vec<Line> {
    count
        .map(|day| Line::Regular(format!("Day #{day}")))
        .into_iter()
        .collect()
}

/// `DebugEntryLookingAt.BlockStateInfo`: where the view lands and the block's state, a boolean
/// property green when true and red when false.
fn looking_at_block(camera: &CameraEntity) -> Vec<Line> {
    let Some(target) = &camera.target else {
        return vec![Line::Group(LOOKING_AT_BLOCK_GROUP, Vec::new())];
    };
    let (x, y, z) = target.position;
    let mut lines = vec![
        format!("\u{a7}nTargeted Block: {x}, {y}, {z}"),
        target.name.clone(),
    ];
    lines.extend(
        target
            .properties
            .iter()
            .map(|(name, value)| match value.as_str() {
                "true" => format!("{name}: \u{a7}atrue"),
                "false" => format!("{name}: \u{a7}cfalse"),
                _ => format!("{name}: {value}"),
            }),
    );
    vec![Line::Group(LOOKING_AT_BLOCK_GROUP, lines)]
}

/// `DebugEntryLight`: the light at the feet, the brighter of the two layers first.
fn light(camera: &CameraEntity) -> Vec<Line> {
    let (sky, block) = camera.light;
    vec![Line::Group(
        LIGHT_GROUP,
        vec![format!(
            "Client Light: {} ({sky} sky, {block} block)",
            sky.max(block)
        )],
    )]
}

/// `DebugEntryBiome`: the biome at the feet, while they are inside the build height.
fn biome(camera: &CameraEntity) -> Option<Line> {
    let (_, y, _) = feet(camera);
    let (min_y, height) = camera.build_height;
    (min_y..min_y + height)
        .contains(&y)
        .then(|| Line::Regular(format!("Biome: {}", camera.biome)))
}

/// `DebugEntryFps`: the frames a second, the limit (`inf` from 260) and how the surface presents.
fn fps_line(frame: &FrameStats) -> String {
    let limit = if frame.limit == 260 {
        "inf".to_owned()
    } else {
        frame.limit.to_string()
    };
    let mode = frame.present_mode.map_or("", PresentMode::name);
    format!("{} fps T: {limit}{mode}", frame.fps)
}

/// `DebugEntryTps` for a server that is not the integrated one: its brand, whether the tick rate is
/// frozen, and the packets a second each way.
fn tps(server: &ServerInfo) -> Vec<Line> {
    let status = if server.stepping {
        " (frozen - stepping)"
    } else if server.frozen {
        " (frozen)"
    } else {
        ""
    };
    vec![Line::Regular(format!(
        "\"{}\" server{status}, {} tx, {} rx",
        server.brand.as_deref().unwrap_or("null"),
        fixed(f64::from(server.sent), 0),
        fixed(f64::from(server.received), 0)
    ))]
}

const POSITION_GROUP: &str = "minecraft:position";
const LIGHT_GROUP: &str = "minecraft:light";
const LOOKING_AT_ENTITY_GROUP: &str = "minecraft:looking_at_entity";
const LOOKING_AT_BLOCK_GROUP: &str = "minecraft:looking_at_block";

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
