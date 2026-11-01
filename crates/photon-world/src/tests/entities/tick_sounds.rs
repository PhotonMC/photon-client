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
