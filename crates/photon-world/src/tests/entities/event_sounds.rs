use super::super::*;
use crate::EquipmentSlot;
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
    world
}

#[test]
fn an_armadillo_peeks_and_an_armor_stand_is_hit_with_their_own_volumes() {
    let mut world = world_with("minecraft:armadillo");
    world.play_entity_event_sound(EntityId(9), 64);
    let sounds = world.take_sounds();
    assert_eq!(sounds.len(), 1);
    assert_eq!(sounds[0].event, "minecraft:entity.armadillo.peek");
    assert_eq!(sounds[0].source, LocalSoundSource::Neutral);
    assert_eq!((sounds[0].volume, sounds[0].pitch), (1.0, 1.0));
    assert_eq!(sounds[0].position, (0.5, 64.0, 0.5));
    // Another event on it is quiet.
    world.play_entity_event_sound(EntityId(9), 32);
    assert!(world.take_sounds().is_empty());

    let mut world = world_with("minecraft:armor_stand");
    world.play_entity_event_sound(EntityId(9), 32);
    let sounds = world.take_sounds();
    assert_eq!(sounds[0].event, "minecraft:entity.armor_stand.hit");
    assert_eq!((sounds[0].volume, sounds[0].pitch), (0.3, 1.0));
}

#[test]
fn a_cured_zombie_villager_and_biting_fangs_sound_unless_silenced() {
    let mut world = world_with("minecraft:zombie_villager");
    world.play_entity_event_sound(EntityId(9), 16);
    let sounds = world.take_sounds();
    assert_eq!(sounds[0].event, "minecraft:entity.zombie_villager.cure");
    assert_eq!(sounds[0].source, LocalSoundSource::Hostile);
    assert!((1.0..2.0).contains(&sounds[0].volume));
    assert!((0.3..1.0).contains(&sounds[0].pitch));
    // It sounds at the eyes.
    assert!(sounds[0].position.1 > 64.0);

    let mut world = world_with("minecraft:evoker_fangs");
    world.play_entity_event_sound(EntityId(9), 4);
    let sounds = world.take_sounds();
    assert_eq!(sounds[0].event, "minecraft:entity.evoker_fangs.attack");
    assert!((0.85..1.05).contains(&sounds[0].pitch));
}

#[test]
fn a_broken_item_sounds_by_its_own_break_sound_and_a_silent_wearer_makes_none() {
    let mut world = world_with("minecraft:zombie");
    let item = |name: &str| {
        ClientItemStackState::new(
            1,
            photon_core::ids::ItemId(photon_data::item::registry::index(name).unwrap() as i32),
        )
    };
    let wear = |world: &mut ClientWorld, slot, name: &str| {
        assert!(
            world
                .entities_mut()
                .apply_equipment(EntityId(9), slot, Some(item(name)))
        );
    };
    wear(
        &mut world,
        EquipmentSlot::MainHand,
        "minecraft:diamond_sword",
    );
    wear(&mut world, EquipmentSlot::OffHand, "minecraft:shield");
    world.play_entity_event_sound(EntityId(9), 47);
    world.play_entity_event_sound(EntityId(9), 48);
    // An empty slot breaks nothing.
    world.play_entity_event_sound(EntityId(9), 49);
    let sounds = world.take_sounds();
    assert_eq!(sounds.len(), 2);
    assert_eq!(sounds[0].event, "minecraft:entity.item.break");
    assert_eq!(sounds[1].event, "minecraft:item.shield.break");
    for sound in &sounds {
        assert_eq!(sound.source, LocalSoundSource::Hostile);
        assert_eq!(sound.volume, 0.8);
        assert!((0.8..1.2).contains(&sound.pitch));
    }
}
