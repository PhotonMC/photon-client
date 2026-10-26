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

/// `FireworkRocketEntity.tick` throws its trail whether or not the entity it is attached to is
/// found: a rocket attached to one the client does not know leaves the trail where it is, once
/// each time the level boosts its rockets, and none from the level's own tick.
#[test]
fn an_attached_rocket_with_a_missing_target_still_leaves_its_trail() {
    use crate::local_player::LocalPlayer;
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
    // Attached to entity 77, which does not exist (an optional unsigned int holds one more).
    assert!(world.apply_entity_metadata(
        EntityId(5),
        &[
            crate::FIREWORK_ATTACHED_TO_TARGET_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_OPTIONAL_UNSIGNED_INT_ID as u8,
            78,
            255,
        ],
    ));
    world.tick_entities();
    assert!(world.take_effect_particles().is_empty());

    let mut player = LocalPlayer::new(
        EntityId(1),
        photon_core::math::Vec3::new(0.0, 64.0, 0.0),
        0.0,
        0.0,
        crate::ClientPlayerGameMode::Survival,
        true,
    );
    world.boost_from_rockets(&mut player, crate::RocketHand::default());
    assert_eq!(world.take_effect_particles().len(), 1);
}
