//! `ClientWorld` integration tests shared across the world submodules.

use super::{
    BlockEntityInput, BoatInput, ClientAttributeModifierState, ClientAttributeState,
    ClientEntityCollisionCandidate, ClientEntityDimensions, ClientEntityMovementState,
    ClientEntityPose, ClientItemStackState, ClientPlayerAbilities, ClientPlayerGameMode,
    ClientPlayerPoseInput, ClientWorld, DimensionBounds, DimensionEnvironment, DimensionInput,
    ENTITY_DATA_SERIALIZER_BOOLEAN_ID, ENTITY_NO_GRAVITY_DATA_ID, ENTITY_RELATIVE_DELTA_X_FLAG,
    ENTITY_RELATIVE_DELTA_Y_FLAG, ENTITY_RELATIVE_DELTA_Z_FLAG, ENTITY_RELATIVE_ROTATE_DELTA_FLAG,
    ENTITY_RELATIVE_X_FLAG, ENTITY_RELATIVE_X_ROT_FLAG, ENTITY_RELATIVE_Y_FLAG,
    ENTITY_RELATIVE_Y_ROT_FLAG, ENTITY_RELATIVE_Z_FLAG, EntityInput, EntityMotionInput,
    EntityMoveInput, EntityPositionInput, EntityPositionUpdateMode, EntityRotationInput,
    EntityTeleportInput, LightLayerInput, LightUpdateInput,
    PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS, PLAYER_RESPAWN_KEEP_ENTITY_DATA,
    PROJECTILE_DEGREES_PER_RADIAN, SNOWBALL_GRAVITY, SectionInput, VANILLA_ARROW_ENTITY_TYPE_ID,
    VANILLA_BOAT_PASSENGER_MAX_ROTATION, VANILLA_BREEZE_ENTITY_TYPE_ID, VANILLA_COW_ENTITY_TYPE_ID,
    VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID, VANILLA_FALLING_BLOCK_AIR_FRICTION,
    VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID, VANILLA_FALLING_BLOCK_GRAVITY,
    VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID, VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
    VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION, VANILLA_LIVING_ENTITY_TYPE_IDS,
    VANILLA_MINECART_ENTITY_TYPE_IDS, VANILLA_PLAYER_ENTITY_TYPE_ID,
    VANILLA_PRIMED_TNT_AIR_FRICTION, VANILLA_PRIMED_TNT_ENTITY_TYPE_ID, VANILLA_PRIMED_TNT_GRAVITY,
    VANILLA_SNOWBALL_ENTITY_TYPE_ID, apply_entity_movement_state,
    collect_candidate_step_up_heights, collect_entity_collision_shapes, collide_entity_movement,
    collide_entity_movement_state, is_minor_horizontal_collision, living_entity_dimensions,
    menu_top_slot_count, refresh_entity_dimensions, select_step_up_movement,
};

use crate::input::{Input, LocalPlayerInputModifiers, LocalPlayerSprintTriggerConditions};

use std::collections::BTreeMap;

use photon_core::aabb::Aabb;

use photon_core::binary::{BitSet, Uuid};

use photon_core::identifier::Identifier;

use photon_core::math::{
    Vec2, Vec3, mth_atan2, mth_cos_f64, mth_sin_f64, projectile_lerp_rotation_f32,
};

use photon_core::shape::Shapes;

use photon_core::tags::{ClientboundUpdateTagsPacket, RegistryTags, TagEntry};

fn section(block_value: i32, biome_value: i32) -> (Vec<i32>, Vec<i32>) {
    let mut blocks = vec![block_value; 4_096];
    blocks[0] = 11;
    blocks[15] = 12;
    blocks[16] = 13;
    blocks[255] = 14;
    blocks[256] = 15;
    blocks[4_095] = 16;

    let mut biomes = vec![biome_value; 64];
    biomes[0] = 21;
    biomes[3] = 22;
    biomes[4] = 23;
    biomes[63] = 24;
    (blocks, biomes)
}

/// The server's climbing tags for the ladder and the vine, as a
/// vanilla server sends them.
fn climbing_tags() -> ClientboundUpdateTagsPacket {
    ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        Identifier::parse("minecraft:block").unwrap(),
        vec![
            TagEntry::new(
                Identifier::parse("minecraft:climbable").unwrap(),
                vec![
                    block_index("minecraft:ladder"),
                    block_index("minecraft:vine"),
                ],
            ),
            TagEntry::new(
                Identifier::parse("minecraft:can_glide_through").unwrap(),
                vec![block_index("minecraft:vine")],
            ),
        ],
    )])
}

fn sky_update(value: u8) -> Vec<u8> {
    let mut sky = vec![0; 2_048];
    sky[0] = value;
    sky
}

fn sky_only(updates: &[Vec<u8>]) -> LightUpdateInput<'_> {
    static MASK: std::sync::OnceLock<BitSet> = std::sync::OnceLock::new();
    static NONE: std::sync::OnceLock<BitSet> = std::sync::OnceLock::new();
    LightUpdateInput {
        sky: LightLayerInput {
            mask: MASK.get_or_init(|| BitSet::from_bits(&[1])),
            empty_mask: NONE.get_or_init(BitSet::new),
            updates,
        },
        block: LightLayerInput {
            mask: NONE.get_or_init(BitSet::new),
            empty_mask: NONE.get_or_init(BitSet::new),
            updates: &[],
        },
    }
}

mod block_light;
mod campfires;
mod chunks;
mod entities;
mod lids;
mod local_player;
mod map_data;
mod ore_steps;
mod passengers;
mod presentation;
mod spawners;

/// The state of a block at an offset in its own states, found by name so a test does not
/// depend on where the registry puts the block.
pub(crate) fn state_of(name: &str, offset: u32) -> photon_core::ids::BlockStateId {
    photon_data::block::registry::block(name)
        .expect(name)
        .state_at(offset)
}

/// The id a new world's version puts on the wire for a block, which is how a tag packet names it.
pub(crate) fn block_index(name: &str) -> i32 {
    let canonical = photon_data::block::blocks::index_of(name) as i32;
    photon_data::version_registries::VersionRegistries::of(photon_core::version::Version::NEWEST)
        .blocks
        .wire_or_unknown(canonical)
}

/// The id a new world's version puts on the wire for a particle type, as metadata carries it.
pub(crate) fn wire_particle(name: &str) -> i32 {
    let canonical = photon_data::particle_type::id(name).expect(name);
    photon_data::version_registries::VersionRegistries::of(photon_core::version::Version::NEWEST)
        .particle_types
        .wire_or_unknown(canonical)
}
