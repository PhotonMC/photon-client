use super::super::*;
use crate::EquipmentSlot;
use photon_core::ids::EntityId;

/// `Consumable.onUseTick` for a player the server says is eating: bread takes 32 ticks, so the
/// crumbs start with the 8th tick of the use and come five at a time every fourth tick after.
#[test]
fn a_player_the_server_says_is_eating_throws_crumbs_of_its_food() {
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
    let bread = photon_data::item::registry::named("minecraft:bread");
    assert!(world.entities_mut().apply_equipment(
        EntityId(9),
        EquipmentSlot::MainHand,
        Some(ClientItemStackState::new(1, bread))
    ));
    // Not eating yet: nothing.
    world.tick_entities();
    assert!(world.take_effect_particles().is_empty());

    assert!(world.apply_entity_metadata(
        EntityId(9),
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            1,
            255,
        ],
    ));
    let mut thrown = Vec::new();
    for tick in 0..32 {
        world.tick_entities();
        let crumbs = world.take_effect_particles();
        if !crumbs.is_empty() {
            thrown.push((tick, crumbs));
        }
    }
    // Used 8, 12, ... 28 ticks (the wait is 7 and the use ends at 32): the tick index is the
    // used count.
    let ticks: Vec<i32> = thrown.iter().map(|(tick, _)| *tick).collect();
    assert_eq!(ticks, [8, 12, 16, 20, 24, 28]);
    let (_, first) = &thrown[0];
    assert_eq!(first.len(), 5);
    assert_eq!(first[0].count, 0);
    assert_eq!(first[0].data, crate::particles::ParticleData::Item(bread));
    // In front of the face of a player at (0.5, 80, 0.5) facing south: 0.6 ahead, below the eyes.
    assert!((first[0].at.2 - 1.1).abs() < 1e-6);
    assert!(first[0].at.1 < 80.0 + 1.62);

    // A food that makes no crumbs, or a stick, throws none.
    assert!(world.entities_mut().apply_equipment(
        EntityId(9),
        EquipmentSlot::MainHand,
        Some(ClientItemStackState::new(
            1,
            photon_data::item::registry::named("minecraft:stick")
        ))
    ));
    for _ in 0..40 {
        world.tick_entities();
    }
    assert!(world.take_effect_particles().is_empty());
}
