use super::super::*;
use crate::{DimensionBounds, SectionInput};
use photon_core::ids::EntityId;

/// An area effect cloud at (8.5, 64, 8.5), in a loaded chunk (an entity ticks only in one) with the data a lingering potion sends: a radius, a
/// waiting flag and a particle, as the entity data lays them out (`DATA_RADIUS`, `DATA_WAITING`,
/// `DATA_PARTICLE`).
fn cloud(radius: f32, waiting: bool, particle: &[u8]) -> ClientWorld {
    let (blocks, biomes) = (vec![0; 4_096], vec![0; 64]);
    let mut world = ClientWorld::new(DimensionBounds::new(0, 128));
    assert!(world.replace_chunk(
        0,
        0,
        (0..8).map(|_| SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }),
    ));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(5),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::id("minecraft:area_effect_cloud").unwrap(),
        x: 8.5,
        y: 64.0,
        z: 8.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let mut data = vec![crate::AREA_EFFECT_CLOUD_RADIUS_DATA_ID, 3];
    data.extend(radius.to_be_bytes());
    data.extend([
        crate::AREA_EFFECT_CLOUD_WAITING_DATA_ID,
        8,
        u8::from(waiting),
        crate::AREA_EFFECT_CLOUD_PARTICLE_DATA_ID,
        16,
    ]);
    data.extend(particle);
    data.push(255);
    assert!(world.apply_entity_metadata(EntityId(5), &data));
    world
}

/// `AreaEffectCloud.clientTick` while the cloud works: a speck for each unit of its area, over
/// its disc, standing still when they are the entity effect's swirl.
#[test]
fn a_lingering_potions_cloud_scatters_swirls_over_its_disc() {
    // The entity effect (type 28) in an ARGB colour.
    let mut swirl = vec![28];
    swirl.extend(0xFF33_66CC_u32.to_be_bytes());
    let mut world = cloud(2.0, false, &swirl);
    world.tick_entities();
    let specks = world.take_effect_particles();
    // ceil(pi * 2 * 2).
    assert_eq!(specks.len(), 13);
    for speck in &specks {
        assert_eq!(speck.type_id, 28);
        assert_eq!(
            speck.data,
            crate::particles::ParticleData::Color(0xFF33_66CC)
        );
        assert!(speck.always_show);
        assert_eq!(speck.spread, (0.0, 0.0, 0.0));
        assert_eq!(speck.at.1, 64.0);
        assert!((speck.at.0 - 8.5).hypot(speck.at.2 - 8.5) <= 2.0 + 1e-6);
    }
}

/// Any other particle drifts a little upward, and a cloud that waits shows at most two.
#[test]
fn another_particle_drifts_and_a_waiting_cloud_shows_little() {
    let villager = photon_data::particle_type::id("happy_villager").unwrap();
    let mut world = cloud(2.0, false, &[villager as u8]);
    world.tick_entities();
    let specks = world.take_effect_particles();
    assert_eq!(specks.len(), 13);
    assert!(specks.iter().all(|speck| speck.spread.1 == 0.01));

    let mut waiting = cloud(2.0, true, &[villager as u8]);
    for _ in 0..20 {
        waiting.tick_entities();
        assert!(waiting.take_effect_particles().len() <= 2);
    }
}
