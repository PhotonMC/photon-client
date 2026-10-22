use super::*;
use photon_core::ids::BlockStateId;
use photon_core::nbt::{Field, FieldValue};

/// A trial spawner tag the way the update packet sends it: the mob it will spawn.
fn trial_tag(entity: &str) -> Vec<u8> {
    let id = Field {
        key: "id".into(),
        value: FieldValue::Str(entity.into()),
    };
    let data = Field {
        key: "entity".into(),
        value: FieldValue::Compound(vec![id]),
    };
    let spawn_data = Field {
        key: "spawn_data".into(),
        value: FieldValue::Compound(vec![data]),
    };
    let mut bytes = Vec::new();
    photon_core::nbt::write_nbt(Some(&[spawn_data]), &mut bytes);
    bytes
}

fn place_trial_spawner(world: &mut ClientWorld, entity: &str, waiting: BlockStateId) {
    let player = photon_core::math::Vec3::new(0.0, 0.0, 0.0);
    assert!(world.set_block_predicted(1, -60, 3, waiting, player));
    let type_id = photon_data::block_entity_type::id("minecraft:trial_spawner").unwrap();
    let tag = trial_tag(entity);
    assert!(world.apply_block_entity_update(BlockEntityInput {
        x: 1,
        y: -60,
        z: 3,
        type_id,
        tag: Some(&tag),
    }));
}

fn shown_type(world: &ClientWorld) -> Option<&str> {
    world.spawner_display_type_at((1, -60, 3))
}

#[test]
fn a_trial_spawner_keeps_its_mob_only_while_its_block_entity_lives() {
    let (blocks, biomes) = section(0, 0);
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let block = photon_data::block::registry::block("minecraft:trial_spawner").unwrap();
    let waiting = crate::block::rules::with_property(
        block.default_state_id(),
        "trial_spawner_state",
        "waiting_for_players",
    )
    .unwrap();
    let air = BlockStateId(0);
    place_trial_spawner(&mut world, "minecraft:zombie", waiting);
    world.tick_block_entities(None).unwrap();
    assert_eq!(shown_type(&world), Some("minecraft:zombie"));

    // The update tag changes: the reference keeps the mob it made first.
    let tag = trial_tag("minecraft:skeleton");
    let type_id = photon_data::block_entity_type::id("minecraft:trial_spawner").unwrap();
    world.apply_block_entity_update(BlockEntityInput {
        x: 1,
        y: -60,
        z: 3,
        type_id,
        tag: Some(&tag),
    });
    world.tick_block_entities(None).unwrap();
    assert_eq!(shown_type(&world), Some("minecraft:zombie"));

    // A new block entity makes a new mob.
    let player = photon_core::math::Vec3::new(0.0, 0.0, 0.0);
    assert!(world.set_block_predicted(1, -60, 3, air, player));
    place_trial_spawner(&mut world, "minecraft:skeleton", waiting);
    world.tick_block_entities(None).unwrap();
    assert_eq!(shown_type(&world), Some("minecraft:skeleton"));
}

/// A world of one chunk whose section holds a spawner at 1, 1, 1 from the start (as a chunk
/// packet brings it), with the update tag that names its mob.
fn world_with_spawner() -> ClientWorld {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let spawner = photon_data::block::registry::block("minecraft:spawner")
        .unwrap()
        .default_state_id();
    let mut blocks = vec![0; 4_096];
    blocks[256 + 16 + 1] = spawner.0;
    let biomes = vec![0; 64];
    assert!(world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ));
    update_spawner(&mut world);
    world
}

/// The update packet of the spawner at 1, 1, 1: the zombie it shows.
fn update_spawner(world: &mut ClientWorld) {
    let spawn_data = Field {
        key: "SpawnData".into(),
        value: FieldValue::Compound(vec![Field {
            key: "entity".into(),
            value: FieldValue::Compound(vec![Field {
                key: "id".into(),
                value: FieldValue::Str("minecraft:zombie".into()),
            }]),
        }]),
    };
    let mut bytes = Vec::new();
    photon_core::nbt::write_nbt(Some(&[spawn_data]), &mut bytes);
    let type_id = photon_data::block_entity_type::id("minecraft:mob_spawner").unwrap();
    assert!(world.apply_block_entity_update(BlockEntityInput {
        x: 1,
        y: 1,
        z: 1,
        type_id,
        tag: Some(&bytes),
    }));
}

#[test]
fn a_spawner_makes_its_mob_once_the_renderer_has_drawn_it() {
    let mut world = world_with_spawner();
    world.tick_block_entities(None).unwrap();
    assert_eq!(world.spawner_display_type_at((1, 1, 1)), None);
    assert!(world.extract_spawner((1, 1, 1)));
    world.tick_block_entities(None).unwrap();
    assert_eq!(
        world.spawner_display_type_at((1, 1, 1)),
        Some("minecraft:zombie")
    );
    // Nothing but a spawner is drawn as one.
    assert!(!world.extract_spawner((2, 1, 1)));
    // `BaseSpawner.load` forgets the mob: it is made again when the renderer next draws it.
    update_spawner(&mut world);
    world.tick_block_entities(None).unwrap();
    assert_eq!(world.spawner_display_type_at((1, 1, 1)), None);
    world.extract_spawner((1, 1, 1));
    world.tick_block_entities(None).unwrap();
    assert_eq!(
        world.spawner_display_type_at((1, 1, 1)),
        Some("minecraft:zombie")
    );
}

#[test]
fn a_spawner_that_came_with_its_chunk_is_found_by_the_renderer() {
    let world = world_with_spawner();
    assert!(world.chest_positions_near((0, 0), 1).contains(&(1, 1, 1)));
}

#[test]
fn a_new_spawner_block_has_to_be_drawn_again() {
    let mut world = world_with_spawner();
    assert!(world.extract_spawner((1, 1, 1)));
    let player = photon_core::math::Vec3::new(0.0, 0.0, 0.0);
    assert!(world.set_block_predicted(1, 1, 1, BlockStateId(0), player));
    let spawner = photon_data::block::registry::block("minecraft:spawner")
        .unwrap()
        .default_state_id();
    assert!(world.set_block_predicted(1, 1, 1, spawner, player));
    update_spawner(&mut world);
    world.tick_block_entities(None).unwrap();
    assert_eq!(world.spawner_display_type_at((1, 1, 1)), None);
}
