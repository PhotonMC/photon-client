use super::*;
use crate::LevelEvent;

/// `CampfireBlock.getTicker`: a lit campfire's block entity ticks on a client and asks for its
/// smoke each tick; once the block is put out the ticker goes, and lighting it brings it back.
#[test]
fn only_a_lit_campfire_ticks_for_its_smoke() {
    let campfire = photon_data::block::registry::block("minecraft:campfire")
        .unwrap()
        .default_state_id();
    let with_lit = |lit: &str| crate::block::rules::with_property(campfire, "lit", lit).unwrap();
    let mut blocks = vec![0; 4_096];
    blocks[256 + 16 + 1] = with_lit("true").0;
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
    let type_id = photon_data::block_entity_type::id("minecraft:campfire").unwrap();
    assert!(world.apply_block_entity_update(BlockEntityInput {
        x: 1,
        y: 1,
        z: 1,
        type_id,
        tag: None,
    }));

    world.tick_block_entities(None).unwrap();
    let events = world.take_level_events();
    assert_eq!(events.len(), 1);
    let LevelEvent::CampfireTick(smoke) = events[0] else {
        panic!("not a campfire tick: {:?}", events[0]);
    };
    assert_eq!(
        (smoke.pos, smoke.signal, smoke.cooking),
        ((1, 1, 1), false, [false; 4])
    );

    // Put out: no more.
    let player = photon_core::math::Vec3::new(0.0, 0.0, 0.0);
    assert!(world.set_block_predicted(1, 1, 1, with_lit("false"), player));
    world.take_level_events();
    world.tick_block_entities(None).unwrap();
    assert!(world.take_level_events().is_empty());

    // Lit again: back.
    assert!(world.set_block_predicted(1, 1, 1, with_lit("true"), player));
    world.take_level_events();
    world.tick_block_entities(None).unwrap();
    assert_eq!(world.take_level_events().len(), 1);
}
