use super::super::*;
use photon_core::ids::EntityId;
use photon_core::random::RandomSource;

fn world_with(name: &str, silent: bool) -> ClientWorld {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::id(name).unwrap(),
        x: 0.5,
        y: 64.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    if silent {
        assert!(world.apply_entity_metadata(EntityId(9), &[4, 8, 1, 255]));
    }
    world
}

/// Source-derived (the oracle has no level of entities to take from): 26.3's
/// `handleTakeItemEntity` plays nothing for a silent entity and draws nothing for its pitch;
/// the orb stays for the server to remove and the pickup still goes on.
#[test]
fn a_silent_entity_taken_makes_no_sound_and_draws_nothing() {
    for name in ["minecraft:item", "minecraft:experience_orb"] {
        let mut world = world_with(name, true);
        let mut random = RandomSource::create(5);
        let untouched = RandomSource::create(5).next_long();
        let taken = world.take_item_entity(EntityId(9), 1, &mut random);
        assert!(taken.is_some(), "{name}");
        assert!(world.take_sounds().is_empty(), "{name}");
        assert_eq!(random.next_long(), untouched, "{name}");
    }
}

#[test]
fn an_entity_that_speaks_is_taken_with_a_pitch_from_two_draws() {
    let mut world = world_with("minecraft:item", false);
    let mut random = RandomSource::create(5);
    let mut expected = RandomSource::create(5);
    let pitch = (expected.next_float() - expected.next_float()) * 1.4 + 2.0;
    assert!(
        world
            .take_item_entity(EntityId(9), 1, &mut random)
            .is_some()
    );
    let sounds = world.take_sounds();
    assert_eq!(sounds.len(), 1);
    assert_eq!(sounds[0].event, "minecraft:entity.item.pickup");
    assert_eq!((sounds[0].volume, sounds[0].pitch), (0.2, pitch));
    assert_eq!(random.next_long(), expected.next_long());
}
