use super::super::*;
use crate::{DimensionBounds, SectionInput};
use photon_core::ids::EntityId;

/// `Entity.doWaterSplashEffect`: a living entity that goes into water bubbles and splashes at the
/// surface, but not one that appears in it, nor one that is out of it.
#[test]
fn a_living_entity_entering_water_throws_bubbles_and_splashes() {
    let water = photon_data::block::registry::default_state_named("minecraft:water").0;
    let mut layer = vec![0; 4_096];
    layer[15 * 256..].fill(water);
    let empty = vec![0; 4_096];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 128));
    assert!(world.replace_chunk(
        0,
        0,
        (0..8).map(|section| SectionInput {
            non_empty_block_count: if section == 3 { 256 } else { 0 },
            fluid_count: if section == 3 { 256 } else { 0 },
            block_states: if section == 3 { &layer } else { &empty },
            biomes: &[0; 64],
        }),
    ));
    // Water fills y = 63; the entity starts just above its surface.
    let spawn = |world: &mut ClientWorld, id: i32, y: f64| {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: Uuid::new(1, id as u64),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 8.5,
            y,
            z: 8.5,
            movement: (0.0, -0.1, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    };
    spawn(&mut world, 9, 64.5);
    // Appearing in water is no splash.
    spawn(&mut world, 10, 63.0);
    world.tick_entities();
    assert!(world.take_effect_particles().is_empty());

    // Sinking one block into it.
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: EntityId(9),
        delta_x: 0,
        delta_y: -4_096,
        delta_z: 0,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    let mut thrown = Vec::new();
    for _ in 0..6 {
        world.tick_entities();
        thrown.extend(world.take_effect_particles());
    }
    let bubble = photon_data::particle_type::id("bubble").unwrap();
    let splash = photon_data::particle_type::id("splash").unwrap();
    let count = |type_id| thrown.iter().filter(|p| p.type_id == type_id).count();
    // A player is 0.6 wide: 1 + 0.6 * 20 = 13 of each.
    assert_eq!((count(bubble), count(splash)), (13, 13));
    // At the surface of the block the feet are in: one above the floor of the water's y = 63.
    assert!(thrown.iter().all(|p| p.at.1 == 64.0 || p.at.1 == 63.0));
}
