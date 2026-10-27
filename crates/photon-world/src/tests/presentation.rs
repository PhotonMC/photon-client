use super::*;
use photon_core::ids::EntityId;
use photon_core::ids::ItemId;

#[test]
fn a_walking_player_swings_its_limbs_and_turns_its_body_after_the_way_it_walks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 0.5,
        y: 0.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        // Facing west, the body and the head.
        rotation: (90.0, 0.0, 90.0),
        data: 0,
    }));
    let snapshot = |world: &ClientWorld| world.entity_snapshot(EntityId(9), 1.0).unwrap();
    assert_eq!(snapshot(&world).walk_speed, 0.0);
    assert_eq!(snapshot(&world).body_yaw, 90.0);
    // Standing still moves nothing.
    world.tick_entities();
    assert_eq!(snapshot(&world).walk_speed, 0.0);
    // Walking south (towards +z), 0.2158 a tick, as the server steps it.
    for _ in 0..12 {
        assert!(world.apply_entity_move(EntityMoveInput {
            entity_id: EntityId(9),
            delta_x: 0,
            delta_y: 0,
            delta_z: 884,
            has_position: true,
            y_rot: None,
            x_rot: None,
        }));
        world.tick_entities();
    }
    let walking = snapshot(&world);
    assert!(walking.walk_speed > 0.5, "{walking:?}");
    assert!(walking.walk_position > 3.0, "{walking:?}");
    // The body went round towards the way it walks (yaw 0), but the head
    // (looking west) held it within fifty degrees.
    assert!(
        walking.body_yaw < 90.0 && walking.body_yaw >= 40.0,
        "{walking:?}"
    );
    // Between two ticks the limbs are read part of the way.
    let mid = world.entity_snapshot(EntityId(9), 0.5).unwrap();
    assert!(mid.walk_position < walking.walk_position);
    assert!(mid.walk_position > walking.walk_position - walking.walk_speed);
    // It stops, and the swing dies away.
    for _ in 0..30 {
        world.tick_entities();
    }
    assert!(snapshot(&world).walk_speed < 0.001);
}

#[test]
fn a_walking_mob_eases_to_each_step_and_faces_where_it_goes() {
    let mut world = ClientWorld::default();
    let (blocks, biomes) = ([0_i32; 4096], [0_i32; 64]);
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    // A cow, looking west, its body and head.
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(3, 4),
        type_id: VANILLA_COW_ENTITY_TYPE_ID,
        x: 4.5,
        y: 0.0,
        z: 4.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (90.0, 0.0, 90.0),
        data: 0,
    }));
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: EntityId(9),
        delta_x: 0,
        delta_y: 0,
        delta_z: 4096,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    // The step is not taken at once: three ticks bring the cow to it.
    let z_after = |world: &ClientWorld| world.entities().at(EntityId(9)).unwrap().position().2;
    assert_eq!(z_after(&world), 4.5);
    let mut seen = Vec::new();
    for _ in 0..3 {
        world.tick_entities();
        seen.push(z_after(&world));
    }
    assert!(
        seen[0] > 4.5 && seen[0] < seen[1] && seen[1] < seen[2],
        "{seen:?}"
    );
    assert!((seen[2] - 5.5).abs() < 1e-9, "{seen:?}");
    // It moved, so its legs swing and its body turned to where it looks (west).
    let moving = world.entity_snapshot(EntityId(9), 1.0).unwrap();
    assert!(moving.walk_speed > 0.0, "{moving:?}");
    assert_eq!(moving.body_yaw, 90.0);
}

#[test]
fn held_items_follow_the_equipment_slots_and_the_main_arm() {
    use crate::entity::{ClientEntity, EquipmentSlot};
    let mut world = ClientWorld::default();
    let player = photon_data::entity_type::id("minecraft:player").expect("the player type");
    let mob = photon_data::entity_type::id("minecraft:zombie").expect("the zombie type");
    for (entity_id, type_id) in [(5, player), (6, mob)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(1, entity_id as u64),
            type_id,
            x: 0.0,
            y: 80.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let main = ClientItemStackState::new(1, photon_core::ids::ItemId(10));
        let off = ClientItemStackState::new(2, photon_core::ids::ItemId(11));
        assert!(world.entities_mut().apply_equipment(
            EntityId(entity_id),
            EquipmentSlot::MainHand,
            Some(main)
        ));
        assert!(world.entities_mut().apply_equipment(
            EntityId(entity_id),
            EquipmentSlot::OffHand,
            Some(off)
        ));
    }
    let id_of = |entity: &ClientEntity, left: bool| {
        entity
            .item_held_by_arm(left)
            .map(|stack| stack.item_registry_id())
    };
    // Right-handed: the main hand is the right arm's.
    let entity = world.entities().at(EntityId(5)).expect("the player");
    assert_eq!(
        (id_of(entity, false), id_of(entity, true)),
        (Some(ItemId(10)), Some(ItemId(11)))
    );
    // The avatar's data names the left arm (`HumanoidArm` id zero).
    assert!(world.apply_entity_metadata(
        EntityId(5),
        &[
            crate::AVATAR_MAIN_HAND_DATA_ID,
            crate::HUMANOID_ARM_SERIALIZER_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entities().at(EntityId(5)).expect("the player");
    assert!(entity.main_arm_is_left());
    assert_eq!(
        (id_of(entity, false), id_of(entity, true)),
        (Some(ItemId(11)), Some(ItemId(10)))
    );
    // A mob is left-handed by its flag.
    assert!(world.apply_entity_metadata(
        EntityId(6),
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b010,
            255,
        ],
    ));
    let entity = world.entities().at(EntityId(6)).expect("the mob");
    assert_eq!(
        (id_of(entity, false), id_of(entity, true)),
        (Some(ItemId(11)), Some(ItemId(10)))
    );
    // An empty slot clears it.
    assert!(
        world
            .entities_mut()
            .apply_equipment(EntityId(6), EquipmentSlot::OffHand, None)
    );
    assert_eq!(
        id_of(world.entities().at(EntityId(6)).expect("the mob"), false),
        None
    );
    assert!(
        !world
            .entities_mut()
            .apply_equipment(EntityId(99), EquipmentSlot::Head, None)
    );
}

#[test]
fn a_cat_lies_on_top_of_a_player_who_sleeps_within_two_blocks_of_it() {
    let cat = photon_data::entity_type::id("minecraft:cat").expect("a cat");
    let mut world = ClientWorld::default();
    let input = |entity_id, type_id, x| EntityInput {
        entity_id,
        uuid: Uuid::new(1, entity_id.0 as u64),
        type_id,
        x,
        y: 80.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    };
    assert!(world.apply_entity(input(EntityId(1), cat, 0.5)));
    assert!(world.apply_entity(input(EntityId(2), VANILLA_PLAYER_ENTITY_TYPE_ID, 2.5)));
    // The cat lies down (`IS_LYING`); the player stands near it, then sleeps.
    assert!(world.apply_entity_metadata(EntityId(1), &[crate::CAT_LYING_DATA_ID, 8, 1, 255]));
    let lies_on_player = |world: &mut ClientWorld| {
        // No chunk is loaded here, which would leave the cat idle.
        world.entities.at_mut(EntityId(1)).unwrap().ticking = true;
        world.tick_entities();
        world
            .entities()
            .at(EntityId(1))
            .unwrap()
            .cat()
            .is_lying_on_top_of_sleeping_player()
    };
    assert!(!lies_on_player(&mut world));
    let mut asleep = vec![
        crate::ENTITY_SLEEPING_POS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    photon_core::binary::write_blockpos(photon_core::binary::BlockPos::new(2, 80, 0), &mut asleep);
    // The sleeping pose (`Pose.SLEEPING`) makes the player's box small.
    asleep.extend([6, 20, 2, 255]);
    assert!(world.apply_entity_metadata(EntityId(2), &asleep));
    assert!(lies_on_player(&mut world));
    // One that sleeps beyond the box the cat looks in does not count.
    assert!(world.apply_entity(input(EntityId(3), VANILLA_PLAYER_ENTITY_TYPE_ID, 9.5)));
    assert!(world.apply_entity_metadata(EntityId(2), &[14, 11, 0, 255]));
    assert!(world.apply_entity_metadata(EntityId(3), &asleep));
    assert!(!lies_on_player(&mut world));
    // And a cat that does not lie never lies on anyone.
    assert!(world.apply_entity_metadata(EntityId(2), &asleep));
    assert!(world.apply_entity_metadata(EntityId(1), &[crate::CAT_LYING_DATA_ID, 8, 0, 255]));
    assert!(!lies_on_player(&mut world));
}

#[test]
fn a_guardian_turns_its_head_to_the_local_player_it_aims_at() {
    let guardian = photon_data::entity_type::id("minecraft:guardian").expect("a guardian");
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(1),
        uuid: Uuid::new(1, 1),
        type_id: guardian,
        x: 0.5,
        y: 80.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    // The local player stands six blocks east: the guardian looks along -x.
    world.set_local_player_id(EntityId(2));
    assert!(world.apply_local_player_position((6.5, 80.0, 0.5), (0.0, 0.0, 0.0), (0.0, 0.0)));
    let mut data = vec![crate::GUARDIAN_ATTACK_TARGET_DATA_ID, 1, 2, 255];
    assert!(world.apply_entity_metadata(EntityId(1), &data));
    // No chunk is loaded here, which would leave the guardian idle.
    world.entities.at_mut(EntityId(1)).unwrap().ticking = true;
    world.tick_entities();
    let (y_rot, x_rot, head) = world.entities().at(EntityId(1)).unwrap().rotation();
    assert_eq!((head, y_rot), (-90.0, -90.0));
    // The player's eyes are 1.195 blocks above the guardian's, six away.
    assert!(
        (x_rot + (1.195_f32 / 6.0).atan().to_degrees()).abs() < 0.01,
        "{x_rot}"
    );
    // Aiming at nothing the world holds does not look: head and pitch stay.
    let looked = world.entities().at(EntityId(1)).unwrap().rotation();
    data[2] = 9;
    assert!(world.apply_entity_metadata(EntityId(1), &data));
    world.tick_entities();
    assert_eq!(world.entities().at(EntityId(1)).unwrap().rotation(), looked);
}

#[test]
fn a_leash_knot_stands_above_the_middle_of_the_block_the_packet_names() {
    let knot = photon_data::entity_type::id("minecraft:leash_knot").expect("leash knot");
    assert_eq!(
        crate::block_attached_position(knot, (10.0, 64.0, -3.0)),
        (10.5, 64.375, -2.5)
    );
    // What does not hang on a block keeps the position it was sent.
    let pig = photon_data::entity_type::id("minecraft:pig").expect("pig");
    assert_eq!(
        crate::block_attached_position(pig, (10.25, 64.0, -3.75)),
        (10.25, 64.0, -3.75)
    );
}

#[test]
fn a_block_broken_makes_its_break_sound_at_its_middle_and_a_hit_a_softer_one() {
    let mut world = ClientWorld::default();
    let stone = photon_data::block::registry::block("minecraft:stone")
        .expect("stone")
        .default_state_id();
    let broken = crate::block_sound_of(
        (10, 64, -3),
        stone,
        photon_data::block_sound::SoundTypeRow::break_sound,
        (|volume| (volume + 1.0) / 2.0, 0.8),
    )
    .expect("stone has a sound");
    assert_eq!(broken.event, "minecraft:block.stone.break");
    assert_eq!((broken.volume, broken.pitch), (1.0, 0.8));
    assert_eq!(broken.position, (10.5, 64.5, -2.5));
    world.play_hit_sound((0, 0, 0), stone);
    let hit = world.take_sounds();
    assert_eq!(hit[0].event, "minecraft:block.stone.hit");
    assert_eq!((hit[0].volume, hit[0].pitch), (0.25, 0.5));
    assert!(world.take_sounds().is_empty());
}

#[test]
fn a_swing_lasts_as_long_as_the_held_spears_and_a_kinetic_hit_kicks_for_ten_ticks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 0.5,
        y: 0.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let swinging = |world: &ClientWorld| {
        world
            .entities()
            .all()
            .find(|entity| entity.id() == EntityId(9))
            .unwrap()
            .swing_state()
            .is_swinging()
    };
    // An empty hand swings over six ticks, the first of which only starts it.
    world.entities_mut().apply_animate(EntityId(9), 0);
    for _ in 0..6 {
        world.tick_entities();
    }
    assert!(swinging(&world));
    world.tick_entities();
    assert!(!swinging(&world));
    // An iron spear's stab lasts nineteen.
    let iron_spear = photon_data::item::items::ITEMS
        .iter()
        .position(|item| item.name == "minecraft:iron_spear")
        .unwrap();
    world.entities_mut().apply_equipment(
        EntityId(9),
        crate::EquipmentSlot::MainHand,
        Some(ClientItemStackState::from_wire(
            1,
            photon_core::ids::ItemId(iron_spear as i32),
            vec![],
            vec![],
        )),
    );
    world.entities_mut().apply_animate(EntityId(9), 0);
    for _ in 0..19 {
        world.tick_entities();
    }
    assert!(swinging(&world));
    world.tick_entities();
    assert!(!swinging(&world));

    // The kick of a hit starts at the game time of the event and is not restarted within ten ticks.
    let since = |world: &ClientWorld| {
        world
            .entities()
            .all()
            .find(|entity| entity.id() == EntityId(9))
            .unwrap()
            .ticks_since_kinetic_hit(world.game_time(), 0.5)
    };
    assert_eq!(since(&world), 0.0);
    world.set_game_time(100);
    assert!(world.apply_entity_event(EntityId(9), 2));
    world.set_game_time(104);
    assert_eq!(since(&world), 4.5);
    assert!(world.apply_entity_event(EntityId(9), 2));
    assert_eq!(since(&world), 4.5);
    world.set_game_time(111);
    assert!(world.apply_entity_event(EntityId(9), 2));
    assert_eq!(since(&world), 0.5);
}

/// An item frame's turn comes from its data (after the direction `HangingEntity` has), and only an
/// item frame keeps it.
#[test]
fn an_item_frame_keeps_the_turn_of_its_item() {
    let mut world = ClientWorld::new(DimensionBounds::default());
    for (id, name) in [
        (1, "minecraft:item_frame"),
        (2, "minecraft:glow_item_frame"),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: Uuid::new(0, id as u64),
            type_id: photon_data::entity_type::id(name).expect("an item frame"),
            x: 3.0,
            y: 64.0,
            z: 3.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 2,
        }));
        assert!(world.apply_entity_metadata(EntityId(id), &[10, 1, 5, 255]));
        let frame = world.entities().at(EntityId(id)).expect("the frame");
        assert!(frame.is_item_frame());
        assert_eq!(frame.item_frame_rotation(), 5);
    }
}

#[test]
fn the_level_events_wait_for_the_particle_engine_in_order() {
    let mut world = ClientWorld::default();
    let stone = photon_data::block::registry::block("minecraft:stone")
        .expect("stone")
        .default_state_id();
    world.apply_level_event(2001, (10, 64, -3), stone.0);
    world.apply_level_event(1000, (0, 0, 0), 0);
    let event = |event, pos, data| crate::LevelEvent::Level { event, pos, data };
    assert_eq!(
        world.take_level_events(),
        vec![
            event(2001, (10, 64, -3), stone.0),
            event(1000, (0, 0, 0), 0)
        ]
    );
    assert!(world.take_level_events().is_empty());
}

#[test]
fn a_wake_up_animation_empties_a_remote_players_sleeping_position() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 0.5,
        y: 0.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let bed = photon_core::binary::BlockPos { x: 1, y: 64, z: 2 };
    world
        .entities_mut()
        .at_mut(EntityId(9))
        .unwrap()
        .sleeping_pos = Some(bed);
    assert!(world.entities_mut().apply_animate(EntityId(9), 2));
    let player = world.entities().at(EntityId(9)).unwrap();
    assert_eq!(player.sleeping_pos(), None);
}

#[test]
fn love_and_taming_events_queue_a_burst_around_the_animal() {
    let mut world = ClientWorld::default();
    let pig = photon_data::entity_type::named("minecraft:pig");
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(5),
        uuid: Uuid::new(3, 4),
        type_id: pig,
        x: 1.5,
        y: 70.0,
        z: -2.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_event(EntityId(5), 18));
    let events = world.take_level_events();
    let [crate::LevelEvent::EntityBurst(burst)] = events.as_slice() else {
        panic!("one burst expected, got {events:?}");
    };
    assert_eq!((burst.particle, burst.at), ("heart", (1.5, 70.0, -2.5)));
    // A pig cannot be tamed: the event 7 is not its own.
    assert!(!world.apply_entity_event(EntityId(5), 7));
    assert!(world.take_level_events().is_empty());
}

#[test]
fn an_ominous_item_spawner_ages_with_each_tick() {
    let mut world = ClientWorld::default();
    let spawner = photon_data::entity_type::id("minecraft:ominous_item_spawner").unwrap();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(4),
        uuid: Uuid::new(3, 4),
        type_id: spawner,
        x: 0.5,
        y: 64.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    // As in a loaded chunk.
    world.entities.at_mut(EntityId(4)).unwrap().ticking = true;
    for _ in 0..3 {
        world.tick_entities();
    }
    assert_eq!(world.entities().at(EntityId(4)).unwrap().tick_count(), 3);
}
