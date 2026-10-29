use super::*;
use photon_core::ids::BlockStateId;

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
    assert_eq!(world.block_state_at(-16, -64, 32), Some(BlockStateId(11)));
    assert_eq!(world.block_state_at(-1, -64, 32), Some(BlockStateId(12)));
    assert_eq!(world.block_state_at(-16, -64, 33), Some(BlockStateId(13)));
    assert_eq!(world.block_state_at(-1, -64, 47), Some(BlockStateId(14)));
    assert_eq!(world.block_state_at(-16, -63, 32), Some(BlockStateId(15)));
    assert_eq!(world.block_state_at(-1, -49, 47), Some(BlockStateId(16)));
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
    assert_eq!(world.block_state_at(0, -64, 0), Some(BlockStateId(11)));
    assert_eq!(world.section_counts_at(0, -4, 0), Some((6, 1)));
    assert_eq!(world.biome_at(0, -64, 0), Some(21));
    assert!(world.update_block_state(0, -64, 0, BlockStateId(86)));

    world.update_chunk_cache_center(8, 0);
    assert_eq!(world.loaded_chunk_count(), 1);
    assert!(!world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), None);
    assert_eq!(world.section_counts_at(0, -4, 0), None);
    assert_eq!(world.biome_at(0, -64, 0), None);
    assert!(!world.update_block_state(0, -64, 0, BlockStateId(87)));

    world.update_chunk_cache_center(0, 0);
    assert!(world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), Some(BlockStateId(86)));
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

    assert_eq!(world.block_state_at(-1, -64, 32), Some(BlockStateId(12)));
    assert!(world.update_block_state(-1, -64, 32, BlockStateId(86)));
    assert_eq!(world.block_state_at(-1, -64, 32), Some(BlockStateId(86)));
    assert!(!world.update_block_state(-17, -64, 32, BlockStateId(86)));
    assert!(!world.update_block_state(-1, -48, 32, BlockStateId(86)));
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
        Some(&crate::ClientBlockEntity::from_wire(7, Some(tag.to_vec())))
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
fn replacing_and_unloading_a_chunk_updates_the_section_cache() {
    let (first_blocks, first_biomes) = section(7, 8);
    let (mut second_blocks, mut second_biomes) = section(17, 18);
    second_blocks[0] = 31;
    second_biomes[0] = 32;
    let mut world = ClientWorld::default();
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &first_blocks,
            biomes: &first_biomes,
        }],
    );
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 2,
            fluid_count: 0,
            block_states: &second_blocks,
            biomes: &second_biomes,
        }],
    );

    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.loaded_section_count(), 1);
    assert_eq!(world.block_state_at(0, -64, 0), Some(BlockStateId(31)));
    assert_eq!(world.biome_at(0, -64, 0), Some(32));
    assert!(world.unload_chunk(0, 0));
    assert!(!world.contains_chunk(0, 0));
    assert_eq!(world.loaded_section_count(), 0);
    assert_eq!(world.block_state_at(0, -64, 0), None);
    assert!(!world.unload_chunk(0, 0));
}

#[test]
fn client_world_applies_block_update_tags_to_state_registry() {
    let mut world = ClientWorld::default();
    let packet = ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        Identifier::parse("minecraft:block").unwrap(),
        vec![
            TagEntry::new(
                Identifier::parse("minecraft:climbable").unwrap(),
                vec![
                    crate::tests::block_index("minecraft:ladder"),
                    crate::tests::block_index("minecraft:vine"),
                ],
            ),
            TagEntry::new(
                Identifier::parse("minecraft:can_glide_through").unwrap(),
                vec![crate::tests::block_index("minecraft:vine")],
            ),
        ],
    )]);

    world.apply_update_tags_packet(&packet);
    assert!(
        world
            .block_state_registry()
            .is_climbable_state(crate::tests::state_of("minecraft:ladder", 1))
    );
    assert!(
        world
            .block_state_registry()
            .is_climbable_state(crate::tests::state_of("minecraft:vine", 31))
    );
    assert!(
        world
            .block_state_registry()
            .is_glide_through_state(crate::tests::state_of("minecraft:vine", 31))
    );
    assert!(
        !world
            .block_state_registry()
            .is_glide_through_state(crate::tests::state_of("minecraft:ladder", 1))
    );

    let replacement = ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        Identifier::parse("minecraft:block").unwrap(),
        vec![TagEntry::new(
            Identifier::parse("minecraft:climbable").unwrap(),
            vec![crate::tests::block_index("minecraft:ladder")],
        )],
    )]);
    world.apply_update_tags_packet(&replacement);
    assert!(
        world
            .block_state_registry()
            .is_climbable_state(crate::tests::state_of("minecraft:ladder", 1))
    );
    assert!(
        !world
            .block_state_registry()
            .is_climbable_state(crate::tests::state_of("minecraft:vine", 31))
    );
    assert!(
        !world
            .block_state_registry()
            .is_glide_through_state(crate::tests::state_of("minecraft:vine", 31))
    );
}

#[test]
fn client_world_updates_stair_shape_from_loaded_neighbors() {
    let mut target_blocks = vec![0; 4_096];
    target_blocks[0] = crate::tests::state_of("minecraft:oak_stairs", 11).0;
    let mut north_blocks = vec![0; 4_096];
    north_blocks[15 * 16] = crate::tests::state_of("minecraft:oak_stairs", 51).0;
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &target_blocks,
            biomes: &biomes,
        }],
    );
    world.replace_chunk(
        0,
        -1,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &north_blocks,
            biomes: &biomes,
        }],
    );

    assert!(world.apply_stair_neighbor_shape_update(0, 0, 0));
    assert_eq!(
        world.block_state_at(0, 0, 0),
        Some(crate::tests::state_of("minecraft:oak_stairs", 17))
    );
    assert_eq!(
        world
            .block_state_registry()
            .stair_shape_name(crate::tests::state_of("minecraft:oak_stairs", 17)),
        Some("outer_left")
    );
    assert_eq!(
        world
            .block_state_registry()
            .collision_shape(crate::tests::state_of("minecraft:oak_stairs", 17))
            .to_aabbs(),
        vec![
            Aabb::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            Aabb::new(0.0, 0.5, 0.0, 0.5, 1.0, 0.5),
        ]
    );
}

#[test]
fn a_copied_region_holds_the_light_the_getters_answer() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    for (chunk_x, chunk_z) in [(0, 0), (1, 0)] {
        world.replace_chunk(
            chunk_x,
            chunk_z,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        // Every nibble differs from its neighbours, so a misplaced run shows.
        let pattern = |seed: usize| -> Vec<u8> {
            (0..2_048)
                .map(|index| ((index * 7 + seed * 31) % 251) as u8)
                .collect()
        };
        let (sky, block) = (pattern(chunk_x as usize + 1), pattern(chunk_x as usize + 5));
        world.apply_chunk_light(
            chunk_x,
            chunk_z,
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
    }
    world.flush_light();

    // A region over the seam between the chunks, below, in and above the lit sections.
    let min = [-3, -65, -2];
    let size = [24, 20, 6];
    let copy = world.copy_region(min, size, [-1, -17, -1], [6, 5, 2]);
    for y in min[1]..min[1] + size[1] {
        for z in min[2]..min[2] + size[2] {
            for x in min[0]..min[0] + size[0] {
                assert_eq!(
                    copy.light_at(x, y, z),
                    (
                        world.block_brightness(x, y, z),
                        world.sky_brightness(x, y, z)
                    ),
                    "at {x} {y} {z}"
                );
            }
        }
    }
}
