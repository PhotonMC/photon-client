//! The particle values a packet carries: the options of a type, an explosion, a particle batch.

use photon_core::binary::BlockPos;
use photon_core::ids::{BlockStateId, EntityId, ItemId};

/// What a particle type carries after its id (`ParticleOptions`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ParticleData {
    /// `SimpleParticleType`: nothing.
    #[default]
    Plain,
    /// `BlockParticleOption`: a block state.
    Block(BlockStateId),
    /// `ColorParticleOption`: an ARGB colour.
    Color(u32),
    /// `SpellParticleOption`: a colour and how fast it goes.
    Spell { color: u32, power: f32 },
    /// `DustParticleOptions`: an RGB colour and a size (held between 0.01 and 4).
    Dust { color: u32, scale: f32 },
    /// `DustColorTransitionOptions`: from one colour to another, and a size.
    DustTransition { from: u32, to: u32, scale: f32 },
    /// `SculkChargeParticleOptions`: how far the quad is turned.
    Roll(f32),
    /// `ItemParticleOption`: the item, by its registry id (its components are not read).
    Item(ItemId),
    /// `VibrationParticleOption`: where it is headed, and how many ticks until it arrives.
    Vibration {
        destination: PositionSource,
        arrival_in_ticks: i32,
    },
    /// `TrailParticleOption`: where it is headed, its colour, and how long it takes.
    Trail {
        target: (f64, f64, f64),
        color: u32,
        duration: i32,
    },
    /// `ShriekParticleOption`: how many ticks before it starts ageing.
    Shriek { delay: i32 },
    /// What a firework's starter (or a spark with a trail) makes a `firework` spark with.
    FireworkSpark(SparkOptions),
    /// `PowerParticleOption`: how hard it is thrown.
    Power(f32),
    /// `GeyserParticleOptions`: how many blocks of water the geyser rises through.
    Geyser { water_blocks: i32 },
    /// `GeyserBaseParticleOptions`: the same, and how hard the spray at its foot is thrown.
    GeyserBase { water_blocks: i32, burst: f32 },
}

/// `world.level.gameevent.PositionSource`: a fixed block (`BlockPositionSource`), or an entity
/// tracked by id with a vertical offset (`EntityPositionSource`, resolved each tick the way
/// `ClientLevel.getEntity` does).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionSource {
    Block(BlockPos),
    Entity { id: EntityId, y_offset: f32 },
}

impl ParticleData {
    /// The block state of a type that carries one.
    pub fn block_state(self) -> Option<BlockStateId> {
        match self {
            Self::Block(state) => Some(state),
            _ => None,
        }
    }
}

/// What a spark is made with (`SparkParticle`'s state besides its motion), and what a spark that
/// leaves a trail hands the one it leaves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SparkOptions {
    pub trail: bool,
    pub twinkle: bool,
    /// The tint, as the particle holds it.
    pub color: [f32; 3],
    /// The tint it fades toward.
    pub fade: Option<[f32; 3]>,
    /// Born half through its life (a spark left by a trail).
    pub half_aged: bool,
}

impl Default for SparkOptions {
    fn default() -> Self {
        Self {
            trail: false,
            twinkle: false,
            color: [1.0; 3],
            fade: None,
            half_aged: false,
        }
    }
}

/// One kind of particle an explosion scatters (`ExplosionParticleInfo`), with its weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionParticle {
    pub type_id: i32,
    pub data: ParticleData,
    pub scaling: f32,
    pub speed: f32,
    pub weight: i32,
}

/// An explosion the server reports.
#[derive(Debug, Clone, PartialEq)]
pub struct Explosion {
    pub center: (f64, f64, f64),
    pub radius: f32,
    pub block_count: i32,
    /// The particle at its centre.
    pub particle: (i32, ParticleData),
    /// Its sound; none for one the client does not know by name.
    pub sound: Option<&'static str>,
    pub block_particles: Vec<ExplosionParticle>,
}

/// `ClientboundLevelParticlesPacket`, of a type that is only its id or a block state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelParticles {
    pub type_id: i32,
    /// What the type carries.
    pub data: ParticleData,
    pub override_limiter: bool,
    pub always_show: bool,
    pub at: (f64, f64, f64),
    pub spread: (f32, f32, f32),
    pub max_speed: f32,
    pub count: i32,
}

/// `FireworkExplosion.Shape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireworkShape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

impl FireworkShape {
    /// `Shape.BY_ID`: an id out of range is the first.
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::LargeBall,
            2 => Self::Star,
            3 => Self::Creeper,
            4 => Self::Burst,
            _ => Self::SmallBall,
        }
    }
}

/// `FireworkExplosion`: one burst of a rocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkExplosion {
    pub shape: FireworkShape,
    pub colors: Vec<i32>,
    pub fade_colors: Vec<i32>,
    pub trail: bool,
    pub twinkle: bool,
}
