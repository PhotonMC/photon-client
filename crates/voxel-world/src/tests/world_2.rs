use super::*;

#[test]
fn block_collision_collection_follows_cursor_order_and_overlap() {
    let mut blocks = vec![0; 4_096];
    let set_block = |blocks: &mut [i32], x: usize, y: usize, z: usize, state_id: i32| {
        blocks[y * 256 + z * 16 + x] = state_id;
    };
    set_block(&mut blocks, 0, 0, 0, 1);
    set_block(&mut blocks, 1, 0, 0, 2);
    set_block(&mut blocks, 0, 1, 0, 3);
    set_block(&mut blocks, 0, 0, 1, 99);
    set_block(&mut blocks, 1, 0, 1, 4);

    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 5,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let shapes = BTreeMap::from([
        (1, Shapes::block()),
        (2, Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 0.5, 1.0).unwrap()),
        (3, Shapes::box_shape(0.0, 0.5, 0.0, 1.0, 1.0, 1.0).unwrap()),
        (4, Shapes::empty()),
    ]);

    let collisions = world
        .collect_block_collision_shapes(Aabb::new(0.25, 0.25, 0.25, 1.75, 1.75, 1.75), &shapes);
    assert_eq!(
        collisions
            .iter()
            .map(|collision| collision.position)
            .collect::<Vec<_>>(),
        vec![
            voxel_core::binary::BlockPos::new(0, 0, 0),
            voxel_core::binary::BlockPos::new(1, 0, 0),
            voxel_core::binary::BlockPos::new(0, 1, 0),
        ]
    );
    assert_eq!(
        collisions[1].shape.to_aabbs(),
        vec![Aabb::new(1.0, 0.0, 0.0, 2.0, 0.5, 1.0)]
    );
    assert!(
        world
            .collect_block_collision_shapes(Aabb::new(2.0, 0.25, 0.25, 3.0, 0.75, 0.75), &shapes,)
            .is_empty()
    );
}

#[test]
fn chunk_sections_are_queryable_with_java_local_order() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    world.replace_chunk(
        -1,
        2,
        [SectionInput {
            non_empty_block_count: 6,
            fluid_count: 1,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );

    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.loaded_section_count(), 1);
    assert_eq!(world.section_counts_at(-1, -4, 2), Some((6, 1)));
    assert_eq!(world.block_state_at(-16, -64, 32), Some(11));
    assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
    assert_eq!(world.block_state_at(-16, -64, 33), Some(13));
    assert_eq!(world.block_state_at(-1, -64, 47), Some(14));
    assert_eq!(world.block_state_at(-16, -63, 32), Some(15));
    assert_eq!(world.block_state_at(-1, -49, 47), Some(16));
    assert_eq!(world.block_state_at(-17, -64, 32), None);
    assert_eq!(world.biome_at(-16, -64, 32), Some(21));
    assert_eq!(world.biome_at(-4, -64, 32), Some(22));
    assert_eq!(world.biome_at(-16, -64, 36), Some(23));
    assert_eq!(world.biome_at(-1, -49, 47), Some(24));
}

#[test]
fn chunk_cache_hides_stale_slots_from_world_queries() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    world.configure_chunk_cache(0);
    assert!(world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 6,
            fluid_count: 1,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ));

    assert!(world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), Some(11));
    assert_eq!(world.section_counts_at(0, -4, 0), Some((6, 1)));
    assert_eq!(world.biome_at(0, -64, 0), Some(21));
    assert!(world.update_block_state(0, -64, 0, 86));

    world.update_chunk_cache_center(8, 0);
    assert_eq!(world.loaded_chunk_count(), 1);
    assert!(!world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), None);
    assert_eq!(world.section_counts_at(0, -4, 0), None);
    assert_eq!(world.biome_at(0, -64, 0), None);
    assert!(!world.update_block_state(0, -64, 0, 87));

    world.update_chunk_cache_center(0, 0);
    assert!(world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), Some(86));
}

#[test]
fn server_block_update_replaces_loaded_state() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    world.replace_chunk(
        -1,
        2,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );

    assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
    assert!(world.update_block_state(-1, -64, 32, 86));
    assert_eq!(world.block_state_at(-1, -64, 32), Some(86));
    assert!(!world.update_block_state(-17, -64, 32, 86));
    assert!(!world.update_block_state(-1, -48, 32, 86));
}

#[test]
fn chunk_light_stores_java_nibble_order_and_empty_sections() {
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

    let mut sky = vec![0; 2_048];
    sky[0] = 0xc3;
    sky[8] = 0x05;
    sky[128] = 0x09;
    let mut block = vec![0; 2_048];
    block[0] = 0xa6;
    let application = world.apply_chunk_light(
        0,
        0,
        LightUpdateInput {
            sky: LightLayerInput {
                mask: &BitSet::from_bits(&[1]),
                empty_mask: &BitSet::from_bits(&[2]),
                updates: &[sky],
            },
            block: LightLayerInput {
                mask: &BitSet::from_bits(&[1]),
                empty_mask: &BitSet::from_bits(&[2]),
                updates: &[block],
            },
        },
    );

    assert_eq!(application.applied_sky_sections, 2);
    assert_eq!(application.applied_block_sections, 2);
    assert_eq!(application.invalid_sky_updates, 0);
    assert_eq!(application.invalid_block_updates, 0);
    // Nothing reaches the engine until a frame runs the queued task.
    assert_eq!(world.sky_light_at(0, -64, 0), None);
    world.flush_light();
    // The filled section and its neighbours above and below hold light.
    assert_eq!(world.loaded_light_section_count(), 3);
    assert_eq!(world.sky_light_at(0, -64, 0), Some(3));
    assert_eq!(world.sky_light_at(1, -64, 0), Some(12));
    assert_eq!(world.sky_light_at(0, -64, 1), Some(5));
    assert_eq!(world.sky_light_at(0, -63, 0), Some(9));
    assert_eq!(world.block_light_at(0, -64, 0), Some(6));
    assert_eq!(world.block_light_at(1, -64, 0), Some(10));
    assert_eq!(world.sky_light_at(0, -48, 0), Some(0));
    assert_eq!(world.block_light_at(0, -48, 0), Some(0));

    assert!(world.unload_chunk(0, 0));
    world.flush_light();
    assert_eq!(world.sky_light_at(0, -64, 0), None);
    assert_eq!(world.loaded_light_section_count(), 0);
}

#[test]
fn light_updates_for_unloaded_chunks_stay_queued_in_the_engine() {
    let mut world = ClientWorld::default();
    let (first, second) = (vec![sky_update(0x05)], vec![sky_update(0x0a)]);
    assert!(world.apply_light_updates(3, -2, sky_only(&first)).queued);
    assert!(world.apply_light_updates(3, -2, sky_only(&second)).queued);
    assert_eq!(world.pending_light_update_count(), 2);
    assert_eq!(world.sky_light_at(48, -64, -32), None);

    // The later layer replaces the earlier one in the engine's queue, which
    // the getters read before any section stores it.
    world.flush_light();
    assert_eq!(world.pending_light_update_count(), 0);
    assert_eq!(world.sky_light_at(48, -64, -32), Some(10));

    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        3,
        -2,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert_eq!(world.sky_light_at(48, -64, -32), Some(10));
    assert_eq!(world.loaded_light_section_count(), 0);
}

#[test]
fn unloading_a_chunk_removes_its_queued_light() {
    let mut world = ClientWorld::default();
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        3,
        -2,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let sky = vec![sky_update(0x0f)];
    world.apply_light_updates(3, -2, sky_only(&sky));
    assert!(world.unload_chunk(3, -2));
    assert_eq!(world.pending_light_update_count(), 2);
    world.flush_light();
    assert_eq!(world.sky_light_at(48, -64, -32), None);
}

#[test]
fn block_entities_are_queryable_and_removed_with_their_chunk() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    world.replace_chunk(
        -1,
        2,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let tag = [10, 0];

    assert_eq!(
        world.apply_block_entities(
            -1,
            2,
            [BlockEntityInput {
                x: -1,
                y: -63,
                z: 32,
                type_id: 7,
                tag: Some(&tag),
            }],
        ),
        1
    );
    assert_eq!(world.loaded_block_entity_count(), 1);
    assert_eq!(
        world.block_entity_at(-1, -63, 32),
        Some(&crate::ClientBlockEntity {
            type_id: 7,
            tag: Some(tag.to_vec()),
            spawner_state: None,
        })
    );

    let replacement = [10, 0, 0];
    assert!(world.apply_block_entity_update(BlockEntityInput {
        x: -1,
        y: -63,
        z: 32,
        type_id: 8,
        tag: Some(&replacement),
    }));
    assert_eq!(world.block_entity_at(-1, -63, 32).unwrap().type_id, 8);
    assert_eq!(
        world.block_entity_at(-1, -63, 32).unwrap().tag,
        Some(replacement.to_vec())
    );
    assert!(world.unload_chunk(-1, 2));
    assert_eq!(world.loaded_block_entity_count(), 0);
    assert_eq!(world.block_entity_at(-1, -63, 32), None);
}

#[test]
fn player_entities_keep_ticking_after_their_chunk_unloads() {
    let mut world = ClientWorld::default();
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: 42,
        uuid: Uuid::new(5, 6),
        type_id: crate::VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(
        world
            .entity_at(42)
            .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
    );

    assert!(world.unload_chunk(0, -3));
    assert!(
        world
            .entity_at(42)
            .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
    );
    world.tick();
    assert_eq!(
        world.entity_at(42).map(crate::ClientEntity::tick_count),
        Some(1)
    );
}
