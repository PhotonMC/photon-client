use super::*;
use photon_core::aabb::Aabb;

fn block(name: &str, properties: &[(&str, &str)]) -> i32 {
    let properties: Vec<(String, String)> = properties
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    crate::block::named_state::state_named(name, &properties)
        .unwrap()
        .0
}

/// A chunk with the given blocks at positions inside it (x, y, z each below 16).
fn world_with(placed: &[((usize, usize, usize), i32)]) -> ClientWorld {
    let mut blocks = vec![0; 4_096];
    for ((x, y, z), state) in placed {
        blocks[y * 256 + z * 16 + x] = *state;
    }
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    assert!(world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: placed.len() as i16,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ));
    world
}

/// The box of a cushion standing with its feet at `y` in the cell above (1, 1, 1)'s column.
fn cushion_at(y: f64) -> Aabb {
    Aabb::new(1.0, y, 1.0, 2.0, y + 0.25, 2.0)
}

/// `Cushion.isAnchorBuried` with no entity asking, source-derived from `Cushion`,
/// `ScaffoldingBlock.getCollisionShape` (the empty context is above and not descending) and
/// `MovingPistonBlock.getCollisionShape` (what the block entity fills now).
#[test]
fn scaffolding_buries_by_its_stable_shape_in_any_state() {
    for (bottom, distance) in [("false", "0"), ("true", "3")] {
        let world = world_with(&[(
            (1, 1, 1),
            block(
                "minecraft:scaffolding",
                &[
                    ("bottom", bottom),
                    ("distance", distance),
                    ("waterlogged", "false"),
                ],
            ),
        )]);
        // Inside the top slab, which spans the top eighth of the block.
        assert!(
            world.cushion_is_buried(cushion_at(1.9)),
            "{bottom} {distance}"
        );
        // Between the posts below the slab, nothing covers the sliver.
        assert!(
            !world.cushion_is_buried(cushion_at(1.5)),
            "{bottom} {distance}"
        );
    }
}

#[test]
fn a_moving_piston_buries_what_its_block_entity_fills() {
    let piston = block(
        "minecraft:piston",
        &[("facing", "up"), ("extended", "false")],
    );
    let stone = block("minecraft:stone", &[]);
    let mut world = world_with(&[((1, 1, 1), piston), ((1, 2, 1), stone)]);
    world.set_game_time(1000);
    // Pushed up one block, the stone passes through the cell above the head's.
    assert!(!world.cushion_is_buried(cushion_at(3.0)));
    world.apply_block_event((1, 1, 1), 0, 1);
    world.tick_after_entities(None, true).unwrap();
    assert!(world.cushion_is_buried(cushion_at(3.0)));
}
