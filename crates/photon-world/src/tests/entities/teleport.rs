use super::super::*;
use photon_core::ids::EntityId;

/// `LivingEntity.handleEntityEvent(46)`: a living entity the server teleports gives off 128
/// portal particles in a box twice its width and as high as it, each drifting a little.
#[test]
fn a_teleported_living_entity_gives_off_a_hundred_and_twenty_eight_portal_particles() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 0.5,
        y: 80.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.throw_teleport_particles(EntityId(9));
    let portal = photon_data::particle_type::id("portal").expect("the portal particle");
    let thrown = world.take_effect_particles();
    assert_eq!(thrown.len(), 128);
    for particle in &thrown {
        assert_eq!(particle.type_id, portal);
        assert_eq!(particle.count, 0);
        // A player is 0.6 wide and 1.8 high: the box reaches 0.6 either side, and the drift is
        // within a tenth.
        assert!((particle.at.0 - 0.5).abs() <= 0.6 && (particle.at.2 - 0.5).abs() <= 0.6);
        assert!((80.0..=81.8).contains(&particle.at.1));
        assert!(particle.spread.0.abs() <= 0.1 && particle.spread.1.abs() <= 0.1);
    }
    // An entity the world does not know has nothing to show.
    world.throw_teleport_particles(EntityId(77));
    assert!(world.take_effect_particles().is_empty());
}
