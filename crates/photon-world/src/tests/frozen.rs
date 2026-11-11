use super::*;
use photon_core::ids::EntityId;

/// An entity of the given type, added as in a loaded chunk.
fn add_ticking(world: &mut ClientWorld, id: i32, type_name: &str) {
    let type_id = photon_data::entity_type::id(type_name).unwrap();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(id),
        uuid: Uuid::new(3, id as u64),
        type_id,
        x: 0.5,
        y: 64.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.entities.at_mut(EntityId(id)).unwrap().ticking = true;
}

#[test]
fn a_frozen_level_ticks_only_players_and_what_carries_one() {
    let mut world = ClientWorld::default();
    let spawner = "minecraft:ominous_item_spawner";
    add_ticking(&mut world, 1, spawner);
    add_ticking(&mut world, 2, spawner);
    add_ticking(&mut world, 3, spawner);
    add_ticking(&mut world, 4, "minecraft:player");
    // 2 carries 3 carries the player 4: the whole stack still ticks.
    assert!(world.apply_entity_passengers(EntityId(2), &[EntityId(3)]));
    assert!(world.apply_entity_passengers(EntityId(3), &[EntityId(4)]));
    world.tick_entities(true);
    let ticks = |id| world.entities().at(EntityId(id)).unwrap().tick_count();
    assert_eq!([ticks(1), ticks(2), ticks(3), ticks(4)], [0, 1, 1, 1]);
}

#[test]
fn a_frozen_bolt_does_not_hold_the_sky_flash() {
    let mut world = ClientWorld::default();
    add_ticking(&mut world, 1, "minecraft:lightning_bolt");
    world.tick_entities(false);
    assert_eq!(world.weather().sky_flash_time, 2);
    world.tick_sky_flash();
    world.tick_entities(true);
    assert_eq!(world.weather().sky_flash_time, 1);
}

#[test]
fn a_crack_nobody_updates_goes_after_four_hundred_ticks() {
    let mut world = ClientWorld::default();
    world.set_game_time(100);
    world.destroy_block_progress(EntityId(7), 1, 2, 3, 4);
    world.set_game_time(500);
    world.expire_destroy_progress();
    assert_eq!(world.destroying_blocks().len(), 1);
    world.set_game_time(501);
    world.expire_destroy_progress();
    assert_eq!(world.destroying_blocks().len(), 1);
    world.set_game_time(520);
    world.expire_destroy_progress();
    assert!(world.destroying_blocks().is_empty());
}

/// `LightningBolt.tick` (26.3): a silent bolt strikes without its thunder and impact.
#[test]
fn a_silent_bolt_strikes_without_a_sound() {
    let strike = |silent: bool| {
        let mut world = ClientWorld::default();
        add_ticking(&mut world, 1, "minecraft:lightning_bolt");
        world.entities.at_mut(EntityId(1)).unwrap().status.silent = silent;
        world.tick_entities(false);
        world.take_sounds().len()
    };
    assert_eq!(strike(false), 2);
    assert_eq!(strike(true), 0);
}
