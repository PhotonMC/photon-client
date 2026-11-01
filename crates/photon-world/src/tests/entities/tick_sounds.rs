use super::super::*;
use crate::local_player::LocalSoundSource;
use photon_core::ids::EntityId;

fn world_with(name: &str) -> ClientWorld {
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
    world.entities.at_mut(EntityId(9)).unwrap().ticking = true;
    world
}

/// The sounds of `ticks` ticks, with the tick each came on.
fn ticked(world: &mut ClientWorld, ticks: i32) -> Vec<(i32, crate::local_player::LocalSound)> {
    let mut heard = Vec::new();
    for tick in 1..=ticks {
        // The entity's chunk is not loaded here, which stops it ticking: tick it all the same.
        world.entities.at_mut(EntityId(9)).unwrap().ticking = true;
        world.tick_entities(false);
        heard.extend(world.take_sounds().into_iter().map(|sound| (tick, sound)));
    }
    heard
}

#[test]
fn a_blaze_crackles_about_once_in_twenty_four_ticks_and_a_phantom_beats_its_wings() {
    let mut world = world_with("minecraft:blaze");
    world.fix_entity_sound_seed(7);
    let heard = ticked(&mut world, 2400);
    assert!((60..=140).contains(&heard.len()), "{}", heard.len());
    let (_, first) = &heard[0];
    assert_eq!(first.event, "minecraft:entity.blaze.burn");
    assert_eq!(first.source, LocalSoundSource::Hostile);
    assert!((1.0..2.0).contains(&first.volume));
    assert!((0.3..1.0).contains(&first.pitch));
    assert_eq!(first.position, (1.0, 64.5, 1.0));

    // A wing turns 7.448451 degrees a tick: the cosine falls through zero once in 48.3 ticks.
    let mut world = world_with("minecraft:phantom");
    let heard = ticked(&mut world, 200);
    assert_eq!(heard.len(), 4, "{heard:?}");
    assert!(heard.iter().all(|(_, sound)| {
        sound.event == "minecraft:entity.phantom.flap"
            && (0.95..1.0).contains(&sound.volume)
            && (0.95..1.0).contains(&sound.pitch)
    }));
}

#[test]
fn a_warden_s_heart_beats_every_forty_ticks_and_a_silent_one_is_still() {
    let mut world = world_with("minecraft:warden");
    let heard = ticked(&mut world, 100);
    assert_eq!(
        heard.iter().map(|(tick, _)| *tick).collect::<Vec<_>>(),
        [40, 80]
    );
    let sound = &heard[0].1;
    assert_eq!(sound.event, "minecraft:entity.warden.heartbeat");
    assert_eq!(sound.volume, 5.0);
    assert!((0.8..1.2).contains(&sound.pitch));
}

#[test]
fn a_boat_above_a_bubble_column_splashes_now_and_then() {
    let air = photon_data::block::registry::default_state_named("minecraft:air").0;
    let column = photon_data::block::registry::default_state_named("minecraft:bubble_column").0;
    let mut blocks = vec![air; 4_096];
    // The block the boat stands in: x 0, y 5, z 0.
    blocks[5 * 256] = column;
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 1,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::id("minecraft:oak_boat").unwrap(),
        x: 0.5,
        y: 5.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let heard = ticked(&mut world, 1000);
    assert!((5..=60).contains(&heard.len()), "{}", heard.len());
    let (_, sound) = &heard[0];
    assert_eq!(sound.event, "minecraft:entity.generic.splash");
    assert_eq!(sound.source, LocalSoundSource::Neutral);
    assert_eq!(sound.volume, 1.0);
    assert!((0.8..1.2).contains(&sound.pitch));
    assert_eq!(sound.position, (0.5, 5.0, 0.5));
}

#[test]
fn an_enderman_stares_when_its_creepy_flag_is_sent_while_stared_at() {
    let mut world = world_with("minecraft:enderman");
    // A creepy flag with nothing stared at is silent.
    assert!(world.apply_entity_metadata(EntityId(9), &[17, 8, 1, 255]));
    assert!(ticked(&mut world, 1).is_empty());
    // The stared-at flag alone is no creepy update.
    assert!(world.apply_entity_metadata(EntityId(9), &[18, 8, 1, 255]));
    assert!(ticked(&mut world, 1).is_empty());
    // A creepy update now plays it, from the eyes, whichever way the flag turns.
    assert!(world.apply_entity_metadata(EntityId(9), &[17, 8, 0, 255]));
    let sounds: Vec<_> = ticked(&mut world, 1)
        .into_iter()
        .map(|(_, sound)| sound)
        .collect();
    assert_eq!(sounds.len(), 1);
    assert_eq!(sounds[0].event, "minecraft:entity.enderman.stare");
    assert_eq!((sounds[0].volume, sounds[0].pitch), (2.5, 1.0));
    assert!((sounds[0].position.1 - 64.0 - 2.55).abs() < 0.01);
    // The next one is inside the four hundred ticks.
    assert!(world.apply_entity_metadata(EntityId(9), &[17, 8, 1, 255]));
    assert!(ticked(&mut world, 1).is_empty());
}

#[test]
fn an_interaction_is_as_big_as_its_data_says() {
    let mut world = world_with("minecraft:interaction");
    let entity = world.entities().at(EntityId(9)).unwrap();
    assert_eq!(
        (entity.bb_height(), entity.dimensions().width()),
        (1.0, 1.0)
    );
    let mut data = vec![8, 3];
    data.extend(2.5_f32.to_be_bytes());
    data.extend([9, 3]);
    data.extend(0.5_f32.to_be_bytes());
    data.push(255);
    assert!(world.apply_entity_metadata(EntityId(9), &data));
    let entity = world.entities().at(EntityId(9)).unwrap();
    assert_eq!(
        (entity.bb_height(), entity.dimensions().width()),
        (0.5, 2.5)
    );
}
