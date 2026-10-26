use super::super::*;
use photon_core::ids::EntityId;

/// `FireworkRocketEntity.tick`: a rocket in flight leaves a white spark at its place each tick,
/// drifting down with its climb.
#[test]
fn a_rocket_in_flight_leaves_a_trail_of_sparks() {
    let mut world = super::clouds::world_with_chunk();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(5),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::id("minecraft:firework_rocket").unwrap(),
        x: 8.5,
        y: 70.0,
        z: 8.5,
        movement: (0.0, 0.5, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let spark = photon_data::particle_type::id("firework").unwrap();
    for _ in 0..3 {
        world.tick_entities();
        let sparks = world.take_effect_particles();
        assert_eq!(sparks.len(), 1);
        assert_eq!(sparks[0].type_id, spark);
        assert_eq!(sparks[0].data, crate::particles::ParticleData::Plain);
        assert!((sparks[0].at.0 - 8.5).abs() < 1e-9 && sparks[0].at.1 > 70.0);
        // Down by half the climb, and a little sideways.
        assert!(sparks[0].spread.1 < 0.0);
        assert!(sparks[0].spread.0.abs() < 0.5 && sparks[0].spread.2.abs() < 0.5);
    }
}
