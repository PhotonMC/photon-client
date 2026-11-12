use super::*;

use photon_core::binary::BlockPos;
use photon_core::ids::EntityId;
use photon_data::block::registry::block;

/// A block put in the test chunk: its cell, its name and its properties.
type Placed<'a> = (usize, usize, usize, &'a str, &'a [(&'a str, &'a str)]);

/// A world with one chunk whose blocks are given by position: `(x, y, z, name, properties)`.
fn world_with(blocks: &[Placed<'_>]) -> ClientWorld {
    let mut states = vec![0; 4_096];
    for (x, y, z, name, properties) in blocks {
        let block = block(name).unwrap();
        let chosen = properties
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        let state = block.state_id(&chosen).unwrap();
        states[y * 256 + z * 16 + x] = state.0;
    }
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    assert!(world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: blocks.len() as i16,
            fluid_count: 0,
            block_states: &states,
            biomes: &biomes,
        }],
    ));
    world
}

fn at(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

/// A bed lifts the sleeper by its shape's top (9/16) and the 1/8 the sleeper is set above.
#[test]
fn a_bed_sleeps_at_the_top_of_its_shape() {
    let world = world_with(&[(1, 1, 1, "minecraft:red_bed", &[("part", "head")])]);
    let (x, y, z) = crate::block::bed::sleeping_position(&world, at(1, 1, 1)).unwrap();
    assert_eq!((x, z), (1.5, 1.5));
    assert!((y - 1.6875).abs() < 1e-9, "{y}");
}

/// A straw bed's foot half is lower than a bed, and its head half answers with the foot half it
/// is joined to; a head with no foot has none.
#[test]
fn a_straw_bed_head_sleeps_at_the_height_of_its_foot() {
    let foot = [("part", "foot"), ("facing", "north")];
    let head = [("part", "head"), ("facing", "north")];
    // A north-facing head lies north of its foot.
    let world = world_with(&[
        (1, 1, 2, "minecraft:straw_bed", &foot),
        (1, 1, 1, "minecraft:straw_bed", &head),
        (5, 1, 5, "minecraft:straw_bed", &head),
    ]);
    let foot_y = crate::block::bed::sleeping_position(&world, at(1, 1, 2))
        .unwrap()
        .1;
    let head_y = crate::block::bed::sleeping_position(&world, at(1, 1, 1))
        .unwrap()
        .1;
    assert!((foot_y - 1.375).abs() < 1e-9, "{foot_y}");
    assert_eq!(foot_y, head_y);
    assert!(crate::block::bed::sleeping_position(&world, at(5, 1, 5)).is_none());
}

/// Nothing but a bed gives a sleeping height.
#[test]
fn a_block_that_is_no_bed_gives_no_sleeping_position() {
    let world = world_with(&[(1, 1, 1, "minecraft:stone", &[])]);
    assert!(crate::block::bed::sleeping_position(&world, at(1, 1, 1)).is_none());
    assert!(crate::block::bed::sleeping_position(&world, at(2, 1, 1)).is_none());
}

/// `getHeightForCamera`: a source block under a block with a sturdy underside is a whole block
/// for the camera and the eyes test; a flowing or open one keeps its own height.
#[test]
fn a_roofed_source_is_full_for_the_camera() {
    let world = world_with(&[
        (1, 1, 1, "minecraft:water", &[("level", "0")]),
        (1, 2, 1, "minecraft:stone", &[]),
        (3, 1, 1, "minecraft:water", &[("level", "0")]),
        (5, 1, 1, "minecraft:water", &[("level", "0")]),
        (5, 2, 1, "minecraft:water", &[("level", "0")]),
    ]);
    let (_, roofed) = world.camera_fluid_height(1, 1, 1);
    let (_, open) = world.camera_fluid_height(3, 1, 1);
    let (_, stacked) = world.camera_fluid_height(5, 1, 1);
    assert_eq!(roofed, 1.0);
    assert!((open - 8.0 / 9.0).abs() < 1e-6, "{open}");
    assert_eq!(stacked, 1.0);
}

/// A remote sleeper whose block is no bed stands up on its first tick (26.3); one in a bed lies
/// over it.
#[test]
fn a_sleeper_over_no_bed_stands_up_on_its_first_tick() {
    let mut world = world_with(&[(1, 1, 1, "minecraft:red_bed", &[("part", "head")])]);
    for (id, bed) in [(9, at(1, 1, 1)), (10, at(5, 1, 5))] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: photon_core::binary::Uuid::new(1, id as u64),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 0.5,
            y: 1.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let mut entity = world.entities_mut().at_mut(EntityId(id)).unwrap();
        entity.vitals.sleeping_pos = Some(bed);
        entity.status.pose = ClientEntityPose::Sleeping;
        entity.ticking = true;
    }
    world.tick_entities(false);
    let sleeper = world.entities().at(EntityId(9)).unwrap();
    assert_eq!(sleeper.sleeping_pos(), Some(at(1, 1, 1)));
    let stood = world.entities().at(EntityId(10)).unwrap();
    assert_eq!(stood.sleeping_pos(), None);
    assert_eq!(stood.pose(), ClientEntityPose::Standing);
}
