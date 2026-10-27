use super::*;

/// `ClientLevel.tick`: a chest the server opened with a block event swings its lid open over the
/// ticks that follow, and shuts it again when the count drops to none.
#[test]
fn a_chest_lid_follows_the_ticks_after_its_block_event() {
    let chest = photon_data::block::registry::block("minecraft:chest")
        .unwrap()
        .default_state_id();
    let mut blocks = vec![0; 4_096];
    blocks[256 + 16 + 1] = chest.0;
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
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
    let pos = (1, 1, 1);
    world.apply_block_event(pos, 1, 1);
    assert_eq!(world.chest_openness(pos, None, 1.0), 0.0);
    for _ in 0..5 {
        world.tick_after_entities(None).unwrap();
    }
    assert!((world.chest_openness(pos, None, 1.0) - 0.5).abs() < 1e-6);
    world.apply_block_event(pos, 1, 0);
    for _ in 0..5 {
        world.tick_after_entities(None).unwrap();
    }
    assert!(world.chest_openness(pos, None, 1.0) < 1e-6);
}
