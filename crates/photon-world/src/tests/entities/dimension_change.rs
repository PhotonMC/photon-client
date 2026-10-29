use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn changing_dimension_keeps_local_player_identity_until_respawn_replacement() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(17));
    let input = DimensionInput {
        key: "minecraft:the_nether",
        dimension_type_id: 3,
        seed: 776,
        is_debug: false,
        is_flat: false,
        sea_level: 32,
    };

    assert!(world.replace_dimension_with_environment(
        input,
        DimensionBounds::new(0, 256),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(
        world.local_player().map(|player| player.id()),
        Some(EntityId(17))
    );
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(0)
    );
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world.local_player().map(|player| player.id()),
        Some(EntityId(17))
    );
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(1)
    );
}

#[test]
fn changing_dimension_replaces_world_storage_and_builtin_bounds() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(5, 6),
        type_id: photon_data::entity_type::named("minecraft:item"),
        x: 0.0,
        y: 64.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.tick();

    let input = DimensionInput {
        key: "minecraft:the_nether",
        dimension_type_id: 1,
        seed: 776,
        is_debug: false,
        is_flat: false,
        sea_level: 32,
    };
    assert!(world.replace_dimension_with_environment(
        input,
        DimensionBounds::for_builtin_dimension(input.key),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.dimension_key(), "minecraft:the_nether");
    assert_eq!(world.dimension(), DimensionBounds::new(0, 256));
    assert_eq!(world.dimension_type_id(), 1);
    assert_eq!(world.seed(), 776);
    assert_eq!(world.sea_level(), 32);
    assert_eq!(world.loaded_chunk_count(), 0);
    assert_eq!(world.entities().count(), 0);
    assert_eq!(world.game_time(), 0);

    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(!world.replace_dimension_with_environment(
        input,
        DimensionBounds::for_builtin_dimension(input.key),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.entities().count(), 0);
}
