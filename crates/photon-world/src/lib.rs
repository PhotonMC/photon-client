//! Queryable client-side world state built from decoded chunk sections.

use photon_core::ids::BlockStateId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crate::input::{
    Input, KeyboardInput, LocalPlayerHandleRelativeFrictionParameters, LocalPlayerInputModifiers,
    LocalPlayerSprintTriggerConditions, LocalPlayerSprintTriggerInput,
    LocalPlayerTravelDispatchParameters, LocalPlayerTravelDispatchResult,
    LocalPlayerTravelFlyingParameters, LocalPlayerTravelInAirParameters,
    LocalPlayerTravelInLavaParameters, LocalPlayerTravelInWaterParameters,
    LocalPlayerUpdateFallFlyingMovementParameters, advance_local_player_sprint_trigger,
};
use photon_core::ids::{EntityId, EntityTypeId, ItemId};
use photon_core::java_float::{DVec3, JavaF32, JavaF64};
use photon_core::math::{Vec2, Vec3, abs_max_f64, mth_cos_f64, mth_sin_f64};
use photon_core::{
    aabb::Aabb,
    binary::BlockPos,
    shape::{BooleanOp, Shapes, VoxelShape},
};

pub mod animation_state;
pub mod block;
pub use ride::boat::{RideState, Status};
pub use ride::mount::{Controller, MountKind, MountState};
pub use ride::vehicle::{VehicleKind, VehicleView};
mod border;
mod chunk;
mod chunk_store;
pub mod cloak;
mod collision;
mod constants;
mod container;
mod data_ids;
mod dimension;
pub mod elytra;
pub mod end_flash;
mod entity;
pub(crate) mod entity_kind;
mod entity_random;
mod fluid;
mod heightmap;
pub mod item_components;
mod item_physics;
mod java_math;
mod level;
pub mod light;
pub mod living;
pub mod map;
mod metadata;
pub mod mobs;
pub mod objects;
mod paletted;
pub mod particles;
mod pick;
mod player;
mod prediction;
mod recipes;
mod region_copy;
mod ride;
mod shadow;
pub mod tick_rate;

pub use block::chest::{
    BannerView, BrushableView, CampfireView, ChestView, CopperStatueView, DYES, PortalView,
    PotView, ShelfView, ShulkerBoxView, SignView, SkullBlockView, VaultView,
};
pub use block::entity::MovingPiston;
pub use block::entity::*;
pub use block::registry::*;
pub use border::*;
pub use chunk::*;
pub(crate) use chunk_store::*;
pub use collision::*;
pub use constants::*;
pub use container::*;
pub use data_ids::DATA_IDS;
pub use dimension::*;
pub use entity::*;
pub use heightmap::HeightmapType;
pub(crate) use level::*;
pub use metadata::find_metadata_field as entity_metadata_field;
pub(crate) use metadata::*;
pub use pick::EntityHit;
pub use player::*;
pub use prediction::{DestroyProgress, SyncedBlock};
pub use recipes::{
    BookTypeSettings, Ingredient, RecipeBook, RecipeDisplay, RecipeDisplayEntry, SlotDisplay,
    StonecutterEntry,
};
pub use region_copy::RegionCopy;
pub use shadow::{ShadowInput, ShadowPiece};
pub use world::{
    BeaconView, CommandBlockMode, CommandBlockView, CommandMinecartView, ConduitView,
    EnchantTableView, Entities, GatewayBeamView, JigsawBlockView, LevelEvent, RocketHand,
    TestBlockView, block_sound_of,
};

/// The client-side copy of the currently loaded world.
#[derive(Debug)]
pub struct ClientWorld {
    level: Level,
    weather: Weather,
    block_state_registry: ClientBlockStateRegistry,
    /// The items each recipe property set accepts, as the server sent them.
    recipe_property_sets: BTreeMap<String, Arc<BTreeSet<i32>>>,
    /// Whether the server enabled `minecart_improvements`: the minecarts made from now on use the
    /// new behaviour (`AbstractMinecart.useExperimentalMovement`, read as one is made).
    minecart_improvements: bool,
    /// The stonecutter's recipes, as the server sent them.
    stonecutter: Arc<Vec<StonecutterEntry>>,
    /// The recipe book the server keeps the client in step with.
    recipe_book: RecipeBook,
    chunks: Chunks,
    /// The client level's light engine and the light tasks waiting for a
    /// frame, as `ClientLevel.queueLightUpdate` keeps them.
    light: light::LevelLightEngine,
    light_tasks: VecDeque<LightTask>,
    /// Sections the light tasks asked to redraw, besides the engine's own.
    light_dirty: Vec<light::SectionKey>,
    entities: Entities,
    /// The maps the server has sent, by id (`ClientLevel.getMapData`).
    maps: BTreeMap<i32, map::MapData>,
    local_player: Option<ClientPlayerState>,
    /// The sounds the world's own events made, until they are taken.
    sounds: local_player::SoundQueue,
    /// Where rain rather than snow or nothing falls, by the biomes (`Biome.getPrecipitationAt`).
    precipitation: world::rain::Precipitation,
    /// The source a sound made for an entity's click draws its pitch from (the reference keeps
    /// one per entity, unseeded).
    entity_sound_random: entity_random::EntityRandom,
    /// `Level.random`, which a block's use sound draws its pitch from (unseeded in the reference).
    level_random: entity_random::EntityRandom,
    /// The swirls the living entities' effects threw (`tickEffects`), until the particle engine
    /// takes them, and the source they draw from (the reference's own per entity, unseeded).
    effect_particles: (Vec<particles::LevelParticles>, entity_random::EntityRandom),
    /// The level events, until the particle engine takes them.
    level_events: world::LevelEventQueue,
    /// The blocks the player is breaking and the face it aims at, for each tick it swings at one
    /// (`Minecraft.continueAttack`'s `addBreakingBlockEffect`), until taken.
    breaking_effects: Vec<((i32, i32, i32), photon_core::direction::Direction)>,
    world_border: ClientWorldBorder,
    block_animations: block::chest::BlockAnimations,
    /// The chunk the client's own player stands in, as the session last said
    /// (`Minecraft.player.chunkPosition()`); unknown until it does.
    local_chunk: Option<(i32, i32)>,
    /// The paddles each ridden boat's tick told the server of, until the session sends them.
    paddle_packets: Vec<(bool, bool)>,
    /// What the player on a mount steers it with, as of its last tick.
    rider_controls: ride::mount::Controller,
    predictions: prediction::BlockPredictions,
    destroying_blocks: BTreeMap<EntityId, DestroyProgress>,
    /// `BaseCommandBlock.isTrackOutput` of the command block minecarts the player set; a client
    /// copy not in it tracks, as one just made does.
    minecart_track_output: BTreeMap<EntityId, bool>,
    /// The block entities the level ticks, in the order they were set up.
    block_tickers: world::BlockTickers,
    /// The mobs spawners show (`BaseSpawner.displayEntity`): made on their own, never added to the
    /// level, so nothing ticks, picks or counts them.
    spawner_displays: Vec<world::SpawnerDisplayEntry>,
    /// The spawners the renderer has drawn: only those make their mob (`getOrCreateDisplayEntity`).
    spawners_extracted: std::sync::Mutex<std::collections::HashSet<block::piston::Pos>>,
    /// What the enchanting tables' ticks read of the level.
    tables: world::Tables,
}

impl Default for ClientWorld {
    fn default() -> Self {
        Self::new(DimensionBounds::default())
    }
}

/// An open container's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenContainerMenu {
    pub container_id: i32,
    /// The `MenuType` registry id.
    pub menu_type_id: i32,
    /// Every slot, the player's inventory at the end.
    pub slots: Vec<Option<ClientItemStackState>>,
    /// The stack the cursor carries.
    pub carried: Option<ClientItemStackState>,
    pub state_id: i32,
}

/// `ItemEntity.bobOffs` is drawn from an unseeded random source; this one is
/// drawn from the entity's uuid.
fn bob_offset(uuid: photon_core::binary::Uuid) -> JavaF32 {
    let folded = uuid.most ^ uuid.least;
    let mut random = photon_core::random::RandomSource::create(folded as i64);
    JavaF32::new(random.next_float() * std::f32::consts::PI * 2.0)
}

pub mod input;
pub mod local_player;

#[cfg(test)]
mod tests;

/// Whether the `MOTION_BLOCKING` heightmap counts a state: it blocks
/// motion or holds a fluid.
/// The bed an entity data update names for the sleeper: none when it holds no such field, an
/// empty place when it gets up.
pub fn entity_sleeping_pos(metadata: &[u8]) -> Option<Option<photon_core::binary::BlockPos>> {
    metadata::sleeping_pos_from_metadata(metadata)
}

/// The living entity flags an entity data update carries: none when it holds no such field.
pub fn entity_living_flags(metadata: &[u8]) -> Option<u8> {
    metadata::living_flags_from_metadata(metadata)
}

/// `LivingEntity.setPosToBed`: where a sleeper lies, over the bed's middle.
pub(crate) fn bed_position(bed: photon_core::binary::BlockPos) -> (f64, f64, f64) {
    (
        f64::from(bed.x) + 0.5,
        f64::from(bed.y) + 0.6875,
        f64::from(bed.z) + 0.5,
    )
}

pub fn motion_blocking(state: BlockStateId) -> bool {
    let state = block::state_or_air(state);
    state.blocks_motion || !state.fluid.is_empty()
}

/// A light task waiting for a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LightTask {
    Data {
        update: PendingLightUpdate,
        schedule_rebuild: bool,
        enable_chunk: bool,
    },
    Removal {
        chunk_x: i32,
        chunk_z: i32,
    },
}

/// `FireworkRocketEntity.DATA_SHOT_AT_ANGLE`'s data id: the item and the attached target come first.
pub const FIREWORK_SHOT_AT_ANGLE_DATA_ID: u8 = 10;

/// `FireworkRocketEntity.DATA_ATTACHED_TO_TARGET`'s data id, an optional unsigned int.
const FIREWORK_ATTACHED_TO_TARGET_DATA_ID: u8 = 9;

/// The data of a display (`Display`) and of an item frame's turn, which this entity may have.
fn apply_display_and_frame_data(entity: &mut ClientEntity, metadata: &[u8]) {
    if let Some(display) = entity.kind.display_mut() {
        display.apply_metadata(metadata);
        match photon_data::entity_type::name(entity.type_id) {
            Some("minecraft:block_display") => display.apply_block_state(metadata),
            Some("minecraft:item_display") => display.apply_item_context(metadata),
            Some("minecraft:text_display") => display.apply_text(metadata),
            _ => {}
        }
    }
    if entity.is_item_frame()
        && let Some(rotation) = metadata_int_value(metadata, constants::ITEM_FRAME_ROTATION_DATA_ID)
    {
        // `ItemFrame.setRotation`'s data holds the turn below eight.
        entity.item_frame_rotation = rotation;
    }
}

/// A firework's data: whether a crossbow shot it, and the entity it flies with.
fn apply_firework_data(entity: &mut ClientEntity, metadata: &[u8]) {
    if let Some(angled) = metadata_boolean_value(metadata, FIREWORK_SHOT_AT_ANGLE_DATA_ID) {
        entity.shot_at_angle = angled;
    }
    if let Some(target) =
        metadata_optional_unsigned_int_value(metadata, FIREWORK_ATTACHED_TO_TARGET_DATA_ID)
    {
        entity.attached_target = target.map(EntityId);
    }
}

/// What the data of a sniffer, an armadillo, an arrow, a minecart, an end crystal, a wither, a
/// creaking and an ender dragon says of it (each when it holds that kind's fields).
fn apply_special_data(entity: &mut ClientEntity, metadata: &[u8]) {
    if photon_data::entity_type::name(entity.type_id) == Some("minecraft:firework_rocket") {
        apply_firework_data(entity, metadata);
    }
    if let Some(mode) = sniffer_state_from_metadata(entity.type_id, metadata) {
        entity.set_sniffer_mode(mode);
    }
    if let Some(mode) = armadillo_state_from_metadata(entity.type_id, metadata) {
        entity.armadillo_mut().set_mode(mode);
    }
    if entity.is_wither() {
        wither_data_from_metadata(metadata).apply(entity.wither_mut());
    }
    if entity.is_abstract_arrow() {
        let ticked = entity.tick_count > 0;
        if let Some(in_ground) = metadata_boolean_value(metadata, objects::arrow::IN_GROUND_DATA_ID)
        {
            entity.arrow_mut().set_in_ground(in_ground, ticked);
        }
        if let Some(color) = metadata_int_value(metadata, objects::arrow::EFFECT_COLOR_DATA_ID) {
            entity.arrow_mut().set_color(color);
        }
    }
    if entity.is_primed_tnt() {
        let state = entity.primed_tnt_mut();
        if let Some(fuse) = metadata_int_value(metadata, objects::primed_tnt::FUSE_DATA_ID) {
            state.set_fuse(fuse);
        }
        if let Some((objects::primed_tnt::BLOCK_STATE_SERIALIZER_ID, mut value)) =
            find_metadata_field(metadata, objects::primed_tnt::BLOCK_STATE_DATA_ID)
            && let Some(id) = photon_core::wire::read_varint(&mut value)
                .ok()
                .filter(|id| *id >= 0)
        {
            state.set_block_state(BlockStateId(id));
        }
    }
    if entity.is_minecart() {
        let (data, hurt) = (
            minecart_data_from_metadata(metadata),
            boat_data_from_metadata(metadata),
        );
        let state = entity.minecart_mut();
        state.hurt_time = hurt.hurt_time.unwrap_or(state.hurt_time);
        state.hurt_dir = hurt.hurt_dir.unwrap_or(state.hurt_dir);
        state.damage = hurt.damage.unwrap_or(state.damage);
        data.apply(state);
    }
    apply_display_and_frame_data(entity, metadata);
    if entity.is_end_crystal() {
        let (beam_target, show_bottom) = end_crystal_data_from_metadata(metadata);
        let state = entity.end_crystal_mut();
        state.beam_target = beam_target.unwrap_or(state.beam_target);
        state.show_bottom = show_bottom.unwrap_or(state.show_bottom);
    }
    if entity.is_ender_dragon()
        && let Some(id) = metadata_int_value(metadata, mobs::ender_dragon::PHASE_DATA_ID)
    {
        entity
            .ender_dragon_mut()
            .set_phase(mobs::ender_dragon::DragonPhase::from_id(id));
    }
    if entity.is_creaking() {
        creaking_data_from_metadata(metadata).apply(entity.creaking_mut());
    }
    if let Some(anger) = warden_anger_from_metadata(entity.type_id, metadata) {
        entity.warden_mut().anger = anger;
    }
    if let Some(size) = cube_mob_size_from_metadata(entity.type_id, metadata) {
        entity.set_cube_size(size);
    }
    if let Some(fuse) = sulfur_cube_fuse_from_metadata(entity.type_id, metadata) {
        entity.living.cube.set_fuse(fuse);
    }
    let (shulker_face, shulker_peek, shulker_color) =
        shulker_data_from_metadata(entity.type_id, metadata);
    if let Some(face) = shulker_face {
        entity.living.shulker_face = face;
    }
    if let Some(peek) = shulker_peek {
        entity.living.shulker_raw_peek = peek as i8;
    }
    if let Some(color) = shulker_color {
        entity.living.shulker_color = (color <= 15).then_some(color);
    }
    let (guardian_moving, guardian_target) = guardian_data_from_metadata(entity.type_id, metadata);
    if guardian_moving.is_some() || guardian_target.is_some() {
        entity.set_guardian_data(guardian_moving, guardian_target);
    }
    let (axolotl_variant, playing_dead) = axolotl_data_from_metadata(entity.type_id, metadata);
    if let Some(variant) = axolotl_variant {
        entity.living.axolotl_variant = variant;
    }
    if let Some(playing_dead) = playing_dead {
        entity.living.axolotl_playing_dead = playing_dead;
    }
}

/// What the data of a boat, a parrot, a bat, a vex, an allay, a ghast, a phantom, a
/// guardian or an axolotl says of it (each when it holds that species' fields).
fn apply_species_data(entity: &mut ClientEntity, metadata: &[u8]) {
    entity.recentre_hanging();
    if entity.is_boat() {
        entity.set_boat_data(boat_data_from_metadata(metadata));
    }
    if let Some(variant) = parrot_variant_from_metadata(entity.type_id, metadata) {
        entity.living.parrot_variant = variant;
    }
    let (bat_flags, vex_flags, allay_dancing) = flier_data_from_metadata(entity.type_id, metadata);
    if let Some(flags) = bat_flags {
        entity.living.bat_resting = flags & 1 != 0;
    }
    if let Some(flags) = vex_flags {
        entity.living.vex_charging = flags & 1 != 0;
    }
    if let Some(dancing) = allay_dancing {
        entity.living.allay_dancing = dancing;
    }
    if let Some(charging) = ghast_charging_from_metadata(entity.type_id, metadata) {
        entity.living.ghast_charging = charging;
    }
    if let Some(dangerous) = wither_skull_dangerous_from_metadata(entity.type_id, metadata) {
        entity.living.wither_skull_dangerous = dangerous;
    }
    if let Some(size) = phantom_size_from_metadata(entity.type_id, metadata) {
        entity.living.phantom_size = size;
    }
    apply_special_data(entity, metadata);
}

fn new_light_engine(dimension: DimensionBounds, has_sky_light: bool) -> light::LevelLightEngine {
    light::LevelLightEngine::new(
        dimension.min_y,
        dimension.section_count() as i32,
        has_sky_light,
    )
}

/// The level as the light engine reads it: `getChunkForLighting` finds only
/// a chunk the cache holds.
struct LightView<'a> {
    chunks: &'a Chunks,
}

impl LightView<'_> {
    fn chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        self.chunks.get(chunk_x, chunk_z)
    }
}

impl light::LightChunks for LightView<'_> {
    fn has_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk(chunk_x, chunk_z).is_some()
    }

    fn block_state(&self, x: i32, y: i32, z: i32) -> BlockStateId {
        self.chunk(x >> 4, z >> 4)
            .and_then(|chunk| chunk.sections.get(&(y >> 4)))
            .and_then(|section| {
                section
                    .block_states
                    .get(((y & 15) * 256 + (z & 15) * 16 + (x & 15)) as usize)
            })
            .unwrap_or(constants::VANILLA_AIR_BLOCK_STATE_ID)
    }

    fn sky_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<&light::ChunkSkyLightSources> {
        self.chunk(chunk_x, chunk_z)?.sky_sources.as_ref()
    }
}

/// `BlockAttachedEntity.setPos` and the `recalculateBoundingBox` of what hangs on a block: the
/// packet carries the block, and the entity stands where its kind puts it (a leash knot,
/// above the middle of a fence). The others (paintings, item frames) depend on their size and
/// turn, and are placed where they are drawn.
fn block_attached_position(type_id: EntityTypeId, (x, y, z): (f64, f64, f64)) -> (f64, f64, f64) {
    if photon_data::entity_type::name(type_id) == Some("minecraft:leash_knot") {
        (
            photon_core::math::floor_f64(x) as f64 + 0.5,
            photon_core::math::floor_f64(y) as f64 + 0.375,
            photon_core::math::floor_f64(z) as f64 + 0.5,
        )
    } else {
        (x, y, z)
    }
}

/// What a projectile does of its own in a tick: the orb ticks either way and takes the blocked path
/// as input; the others only advance while their path stays open.
fn tick_projectile(entity: &mut ClientEntity, kind: VanillaProjectileKind, path_is_open: bool) {
    match kind {
        VanillaProjectileKind::ExperienceOrb => entity.tick_experience_orb(path_is_open),
        _ if !path_is_open => {}
        VanillaProjectileKind::Snowball => entity.tick_snowball(),
        VanillaProjectileKind::ExperienceBottle => entity.tick_experience_bottle(),
        VanillaProjectileKind::Arrow => entity.tick_arrow(),
        VanillaProjectileKind::PrimedTnt => entity.tick_primed_tnt(),
        VanillaProjectileKind::FallingBlock => entity.tick_falling_block(),
        VanillaProjectileKind::Firework => entity.tick_firework(),
    }
}
mod world;

#[cfg(test)]
fn _assert_send_sync<T: Send + Sync + 'static>() {}
#[cfg(test)]
#[test]
fn entity_is_send_sync() {
    _assert_send_sync::<ClientEntity>();
}
