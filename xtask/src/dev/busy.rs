//! The `perf-busy` scene: a few hundred mixed entities with names, text and item displays, armor
//! stands holding items, block entities and particles in front of the camera, at fixed
//! coordinates, so the route looks like a real server and not like one empty or one-type scene.

/// The chunks the scene stands in, forceloaded so it exists while the player is far away.
pub const FORCELOAD: &str = "forceload add 0 -32 63 31";

/// Every entity of the scene carries this tag, so one command takes them all away.
pub const TAG: &str = "busy";

/// The camera stands at the summit's centre looking east (see `perf::STILL_SPOT`); the scene is a
/// grid in front of it: columns this far apart in x from `FIRST_X`, rows this far apart in z.
const FIRST_X: i32 = 5;
const COLUMNS: i32 = 11;
const COLUMN_STEP: i32 = 4;
const ROWS: i32 = 17;
const ROW_STEP: i32 = 3;
const FIRST_Z: i32 = -24;

/// What stands in a cell, in turn: `SummonKind`s with the NBT after the type.
const KINDS: [(&str, &str); 10] = [
    (
        "minecraft:villager",
        "NoAI:1b,PersistenceRequired:1b,CustomName:\"Trader\",CustomNameVisible:1b",
    ),
    (
        "minecraft:cow",
        "NoAI:1b,PersistenceRequired:1b,CustomName:\"Bessie\",CustomNameVisible:1b",
    ),
    ("minecraft:iron_golem", "NoAI:1b,PersistenceRequired:1b"),
    (
        "minecraft:armor_stand",
        "equipment:{head:{id:\"minecraft:diamond_helmet\",count:1},chest:{id:\"minecraft:diamond_chestplate\",count:1},mainhand:{id:\"minecraft:bow\",count:1}},CustomName:\"Guard\",CustomNameVisible:1b",
    ),
    (
        "minecraft:text_display",
        "text:\"Shop: 5 gold\",billboard:\"center\",background:1073741824",
    ),
    (
        "minecraft:item_display",
        "item:{id:\"minecraft:diamond_sword\",count:1},billboard:\"fixed\"",
    ),
    (
        "minecraft:block_display",
        "block_state:{Name:\"minecraft:gold_block\"}",
    ),
    ("minecraft:sheep", "NoAI:1b,PersistenceRequired:1b"),
    (
        "minecraft:pig",
        "NoAI:1b,PersistenceRequired:1b,equipment:{saddle:{id:\"minecraft:saddle\",count:1}}",
    ),
    (
        "minecraft:item_frame",
        "Item:{id:\"minecraft:filled_map\",count:1},Facing:1b",
    ),
];

/// The block entities that stand beside the entities, in turn.
const BLOCKS: [&str; 4] = [
    "minecraft:chest",
    "minecraft:oak_sign",
    "minecraft:white_banner",
    "minecraft:skeleton_skull",
];

/// The cells of the grid as (x, z).
fn cells() -> impl Iterator<Item = (i32, i32)> {
    (0..COLUMNS).flat_map(|column| {
        (0..ROWS).map(move |row| (FIRST_X + column * COLUMN_STEP, FIRST_Z + row * ROW_STEP))
    })
}

/// The commands that build the scene: two entities in every second cell, one in the rest, a
/// block entity beside every third cell.
pub fn summon() -> Vec<String> {
    let mut commands = vec![FORCELOAD.to_owned()];
    for (index, (x, z)) in cells().enumerate() {
        let on_ground = |body: String| {
            format!("execute positioned {x} 100 {z} positioned over world_surface run {body}")
        };
        let (kind, nbt) = KINDS[index % KINDS.len()];
        commands.push(on_ground(format!(
            "summon {kind} ~ ~ ~ {{Tags:[\"{TAG}\"],{nbt}}}"
        )));
        if index % 2 == 0 {
            let (kind, nbt) = KINDS[(index * 3 + 1) % KINDS.len()];
            commands.push(on_ground(format!(
                "summon {kind} ~1.2 ~ ~1.2 {{Tags:[\"{TAG}\"],{nbt}}}"
            )));
        }
        if index % 3 == 0 {
            let block = BLOCKS[(index / 3) % BLOCKS.len()];
            commands.push(on_ground(format!("setblock ~-1 ~ ~-1 {block}")));
        }
    }
    commands
}

/// Every second, the scene's particles: smoke, flames and sparkles over the grid.
pub const PARTICLES: [&str; 4] = [
    "particle minecraft:campfire_cosy_smoke 20 200 -8 4 1 4 0.01 12",
    "particle minecraft:flame 28 199 10 5 1 5 0.01 30",
    "particle minecraft:happy_villager 12 200 4 4 1 4 0 20",
    "particle minecraft:end_rod 36 201 -12 6 2 6 0.02 20",
];

/// The commands that take the scene away: the tagged entities and the block entities. Blocks go
/// with `fill ... air replace` over the scene's area in bands, each under the fill limit
/// ([`FILL_LIMIT`]); the terrain there stays within the bands' height.
pub fn clear() -> Vec<String> {
    let mut commands = vec![
        format!("kill @e[tag={TAG}]"),
        "kill @e[type=minecraft:item]".to_owned(),
    ];
    for block in BLOCKS {
        for (low, high) in HALVES {
            for bottom in (BOTTOM..BOTTOM + BANDS * BAND_HEIGHT).step_by(BAND_HEIGHT as usize) {
                let top = bottom + BAND_HEIGHT - 1;
                commands.push(format!(
                    "fill 0 {bottom} {low} 60 {top} {high} minecraft:air replace {block}"
                ));
            }
        }
    }
    commands
}

/// What one `fill` may change.
#[cfg(test)]
const FILL_LIMIT: i32 = 32768;
/// The area is cleared in two halves in z, and bands of this many layers from `BOTTOM` up.
const HALVES: [(i32, i32); 2] = [(-32, -1), (0, 31)];
const BOTTOM: i32 = 170;
const BAND_HEIGHT: i32 = 12;
const BANDS: i32 = 5;

/// How many entities the commands of [`summon`] make.
pub fn expected_entities() -> usize {
    cells()
        .enumerate()
        .map(|(index, _)| 1 + usize::from(index % 2 == 0))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_is_a_few_hundred_entities_in_fixed_places() {
        assert!((200..400).contains(&expected_entities()));
        assert_eq!(summon(), summon());
        assert!(summon().len() > expected_entities());
    }

    #[test]
    fn the_clear_commands_stay_under_the_fill_limit() {
        for (low, high) in HALVES {
            assert!(61 * BAND_HEIGHT * (high - low + 1) <= FILL_LIMIT);
        }
        assert!(
            clear()
                .iter()
                .any(|command| command.contains("minecraft:chest"))
        );
    }
}
