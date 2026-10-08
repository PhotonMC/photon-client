use super::super::*;

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
            photon_core::binary::BlockPos::new(0, 0, 0),
            photon_core::binary::BlockPos::new(1, 0, 0),
            photon_core::binary::BlockPos::new(0, 1, 0),
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
