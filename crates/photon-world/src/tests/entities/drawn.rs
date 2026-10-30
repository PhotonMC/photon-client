use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn the_drawn_entities_follow_the_out_of_view_set_through_spawns_and_removals() {
    let mut world = ClientWorld::default();
    let spawn = |world: &mut ClientWorld, id: i32| {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: Uuid::new(id as u64, 1),
            type_id: crate::VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 0.0,
            y: 80.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    };
    let drawn = |world: &ClientWorld| -> Vec<i32> {
        world
            .entities()
            .drawn_with_invisible()
            .map(|entity| entity.id().0)
            .collect()
    };
    for id in [5, 1, 3] {
        spawn(&mut world, id);
    }
    assert_eq!(drawn(&world), [1, 3, 5]);
    world
        .entities_mut()
        .set_out_of_view([EntityId(3)].into_iter().collect());
    assert_eq!(drawn(&world), [1, 5]);
    // A spawn between two others, and one of an id that is left out.
    spawn(&mut world, 2);
    spawn(&mut world, 3);
    assert_eq!(drawn(&world), [1, 2, 5]);
    world.entities_mut().remove(EntityId(1));
    world.entities_mut().remove(EntityId(3));
    assert_eq!(drawn(&world), [2, 5]);
    world.entities_mut().set_out_of_view(Default::default());
    assert_eq!(drawn(&world), [2, 5]);
}
