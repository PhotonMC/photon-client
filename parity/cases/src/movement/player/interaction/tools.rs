//! The axe, the hoe and the shovel used on blocks: the classes' own tables before 26.3, the
//! `block_transformer` component of the stack from 26.3 (which also lets any stack a patch gives
//! the component work like a tool, and strips poplar).

use serde_json::Value;

use super::item_on::north;
use super::placing::uses;
use super::{aim_at, using};
use crate::components::{BLOCK_TRANSFORMER, DAMAGE, MAX_DAMAGE, patch, patch_of, vi};
use crate::movement::world::{put, put_with};

const AROUND: [(i64, i64, i64); 4] = [(0, 64, 1), (0, 64, 2), (0, 65, 2), (0, 64, 3)];

/// A tool used on the block at (0, 64, 2) through the given face, with what the row holds.
struct Use {
    label: String,
    world: Vec<Value>,
    aim: Value,
    held: &'static str,
    sneaking: bool,
}

fn tool(label: &str, world: Vec<Value>, held: &'static str) -> Use {
    Use { label: label.to_owned(), world, aim: north(), held, sneaking: false }
}

impl Use {
    fn aim(self, aim: Value) -> Self {
        Self { aim, ..self }
    }

    fn sneaking(self) -> Self {
        Self { sneaking: true, ..self }
    }

    /// The row, ready for the hands and game mode.
    fn row(self) -> super::Interaction {
        let keys: &'static [&'static str] = if self.sneaking { &["sneak"] } else { &[] };
        // The crouch takes hold a tick after its key goes down, so the click comes after two
        // ticks of it.
        let lead = if self.sneaking { uses(2, None, &AROUND).held(keys).go() } else { vec![] };
        let ticks = [
            lead,
            uses(1, Some(&self.aim), &AROUND).clicks(1).held(keys).go(),
            uses(3, Some(&self.aim), &AROUND).held(keys).go(),
        ]
        .concat();
        using(&self.label, self.world, ticks).main_hand(self.held).count(1)
    }
}

fn log(block: &str, axis: &str) -> Vec<Value> {
    vec![put_with(0, 64, 2, block, &[("axis", axis)])]
}

fn down() -> Value {
    aim_at((0, 64, 2), "down", (0.5, 64.0, 2.5))
}

fn top() -> Value {
    aim_at((0, 64, 2), "up", (0.5, 65.0, 2.5))
}

fn covered(block: &str) -> Vec<Value> {
    vec![put(0, 64, 2, block), put(0, 65, 2, "minecraft:stone")]
}

fn plain(block: &str) -> Vec<Value> {
    vec![put(0, 64, 2, block)]
}

fn axe() -> Vec<Value> {
    let axe = |label: &str, world: Vec<Value>| tool(&format!("tool-axe-{label}"), world, "minecraft:iron_axe");
    let mut rows = vec![
        axe("strip-log", log("minecraft:oak_log", "x")).row().row(),
        axe("strip-wood", log("minecraft:oak_wood", "z")).row().row(),
        axe("strip-bamboo", log("minecraft:bamboo_block", "y")).row().row(),
        axe("strip-pale-oak", log("minecraft:pale_oak_log", "y")).row().row(),
        axe("strip-stem", log("minecraft:crimson_stem", "x")).row().row(),
        axe("strip-hyphae", log("minecraft:warped_hyphae", "z")).row().row(),
        axe("strip-top-face", log("minecraft:oak_log", "y")).aim(top()).row().row(),
        axe("strip-down-face", log("minecraft:oak_log", "y")).aim(down()).row().row(),
        axe("strip-creative", log("minecraft:birch_log", "y")).row().game_mode("creative").row(),
        axe("strip-stripped", log("minecraft:stripped_oak_log", "y")).row().row(),
        axe("strip-planks", plain("minecraft:oak_planks")).row().row(),
        axe("strip-shield", log("minecraft:oak_log", "y")).row().off_hand("minecraft:shield").row(),
        axe("strip-shield-sneaking", log("minecraft:oak_log", "y")).sneaking().row().off_hand("minecraft:shield").row(),
        axe("scrape-down-face", plain("minecraft:weathered_copper")).aim(down()).row().row(),
        axe("scrape-shield", plain("minecraft:weathered_copper")).row().off_hand("minecraft:shield").row(),
    ];
    // Poplar came with 26.3.
    if !crate::components::is_26_2() {
        rows.push(axe("strip-poplar-log", log("minecraft:poplar_log", "y")).row().row());
        rows.push(axe("strip-poplar-wood", log("minecraft:poplar_wood", "x")).row().row());
    }
    rows
}

fn chest(kind: &str, block: &str, x: i64) -> Value {
    put_with(x, 64, 2, block, &[("facing", "south"), ("type", kind), ("waterlogged", "false")])
}

/// Copper chests, which the transformer treats on their own (a double chest's other half is told).
fn copper_chests() -> Vec<Value> {
    let axe = |label: &str, world: Vec<Value>| tool(&format!("tool-axe-{label}"), world, "minecraft:iron_axe");
    vec![
        axe("scrape-chest", vec![chest("single", "minecraft:oxidized_copper_chest", 0)]).row().row(),
        axe(
            "scrape-chest-double",
            vec![
                chest("left", "minecraft:weathered_copper_chest", 0),
                chest("right", "minecraft:weathered_copper_chest", 1),
            ],
        )
        .row()
        .row(),
        axe("wax-off-chest", vec![chest("single", "minecraft:waxed_exposed_copper_chest", 0)]).row().row(),
        axe(
            "wax-off-chest-double",
            vec![chest("left", "minecraft:waxed_copper_chest", 0), chest("right", "minecraft:waxed_copper_chest", 1)],
        )
        .row()
        .row(),
    ]
}

fn hoe() -> Vec<Value> {
    let hoe = |label: &str, world: Vec<Value>| tool(&format!("tool-hoe-{label}"), world, "minecraft:iron_hoe");
    vec![
        hoe("grass", plain("minecraft:grass_block")).row().row(),
        hoe("dirt", plain("minecraft:dirt")).row().row(),
        hoe("dirt-path", plain("minecraft:dirt_path")).row().row(),
        hoe("coarse-dirt", plain("minecraft:coarse_dirt")).row().row(),
        hoe("rooted-dirt", plain("minecraft:rooted_dirt")).row().row(),
        hoe("podzol", plain("minecraft:podzol")).row().row(),
        hoe("grass-top-face", plain("minecraft:grass_block")).aim(top()).row().row(),
        hoe("grass-down-face", plain("minecraft:grass_block")).aim(down()).row().row(),
        hoe("rooted-dirt-down-face", plain("minecraft:rooted_dirt")).aim(down()).row().row(),
        hoe("grass-covered", covered("minecraft:grass_block")).row().row(),
        hoe("coarse-dirt-covered", covered("minecraft:coarse_dirt")).row().row(),
        hoe("rooted-dirt-covered", covered("minecraft:rooted_dirt")).row().row(),
        hoe("grass-creative", plain("minecraft:grass_block")).row().game_mode("creative").row(),
        hoe("grass-shield", plain("minecraft:grass_block")).row().off_hand("minecraft:shield").row(),
        hoe("grass-shield-sneaking", plain("minecraft:grass_block"))
            .sneaking()
            .row()
            .off_hand("minecraft:shield")
            .row(),
        hoe("grass-adventure", plain("minecraft:grass_block")).row().game_mode("adventure").row(),
    ]
}

fn campfire(lit: &str, block: &str) -> Vec<Value> {
    vec![put_with(
        0,
        64,
        2,
        block,
        &[("facing", "north"), ("lit", lit), ("signal_fire", "false"), ("waterlogged", "false")],
    )]
}

fn shovel() -> Vec<Value> {
    let shovel = |label: &str, world: Vec<Value>| tool(&format!("tool-shovel-{label}"), world, "minecraft:iron_shovel");
    vec![
        shovel("grass", plain("minecraft:grass_block")).row().row(),
        shovel("dirt", plain("minecraft:dirt")).row().row(),
        shovel("podzol", plain("minecraft:podzol")).row().row(),
        shovel("mycelium", plain("minecraft:mycelium")).row().row(),
        shovel("coarse-dirt", plain("minecraft:coarse_dirt")).row().row(),
        shovel("rooted-dirt", plain("minecraft:rooted_dirt")).row().row(),
        shovel("dirt-path", plain("minecraft:dirt_path")).row().row(),
        shovel("stone", plain("minecraft:stone")).row().row(),
        shovel("grass-top-face", plain("minecraft:grass_block")).aim(top()).row().row(),
        shovel("grass-down-face", plain("minecraft:grass_block")).aim(down()).row().row(),
        shovel("grass-covered", covered("minecraft:grass_block")).row().row(),
        shovel("grass-creative", plain("minecraft:grass_block")).row().game_mode("creative").row(),
        shovel("grass-shield", plain("minecraft:grass_block")).row().off_hand("minecraft:shield").row(),
        shovel("campfire-lit", campfire("true", "minecraft:campfire")).row().row(),
        shovel("campfire-lit-top-face", campfire("true", "minecraft:campfire")).aim(top()).row().row(),
        shovel("campfire-lit-down-face", campfire("true", "minecraft:campfire")).aim(down()).row().row(),
        shovel("campfire-lit-sneaking", campfire("true", "minecraft:campfire")).sneaking().row().row(),
        shovel("campfire-lit-shield", campfire("true", "minecraft:campfire")).row().off_hand("minecraft:shield").row(),
        shovel("campfire-unlit", campfire("false", "minecraft:campfire")).row().row(),
        shovel("soul-campfire-lit", campfire("true", "minecraft:soul_campfire")).row().row(),
    ]
}

/// Stacks that are not tools but carry the component, and tools whose component a patch changes.
fn component() -> Vec<Value> {
    // The vanilla entries in the order the registry holds them.
    let (axe, hoe, shovel) = (0, 1, 2);
    let carrying = |entry: i64| patch(&[(BLOCK_TRANSFORMER, vi(entry))]);
    let stick = |label: &str, world: Vec<Value>, entry: i64, count: i64| {
        tool(&format!("tool-component-{label}"), world, "minecraft:stick").row().count(count).patch(carrying(entry))
    };
    let stick_down = tool("tool-component-stick-hoe-down-face", plain("minecraft:grass_block"), "minecraft:stick")
        .aim(down())
        .row()
        .count(3)
        .patch(carrying(hoe))
        .row();
    // A stackable stack with room to be damaged, damaged: it no longer stacks.
    let damaged = patch(&[(MAX_DAMAGE, vi(10)), (DAMAGE, vi(3)), (BLOCK_TRANSFORMER, vi(axe))]);
    let undamaged = patch(&[(MAX_DAMAGE, vi(10)), (BLOCK_TRANSFORMER, vi(axe))]);
    vec![
        stick("stick-axe", log("minecraft:oak_log", "x"), axe, 3).row(),
        stick("stick-axe-creative", log("minecraft:oak_log", "x"), axe, 3).game_mode("creative").row(),
        stick("stick-axe-last", log("minecraft:oak_log", "x"), axe, 1).row(),
        stick("stick-axe-grass", plain("minecraft:grass_block"), axe, 3).row(),
        stick("stick-hoe", plain("minecraft:grass_block"), hoe, 3).row(),
        stick_down,
        stick("stick-shovel", plain("minecraft:grass_block"), shovel, 3).row(),
        stick("stick-shovel-shield", plain("minecraft:grass_block"), shovel, 3).off_hand("minecraft:shield").row(),
        stick("stick-shovel-campfire", campfire("true", "minecraft:campfire"), shovel, 3).row(),
        tool("tool-component-stick-damaged", log("minecraft:oak_log", "x"), "minecraft:stick")
            .row()
            .count(3)
            .patch(damaged)
            .row(),
        tool("tool-component-stick-undamaged", log("minecraft:oak_log", "x"), "minecraft:stick")
            .row()
            .count(3)
            .patch(undamaged)
            .row(),
        // An item with the component removed does not work; one with another entry works as that.
        tool("tool-component-axe-removed", log("minecraft:oak_log", "x"), "minecraft:iron_axe")
            .row()
            .patch(patch_of(&[], &[BLOCK_TRANSFORMER]))
            .row(),
        tool("tool-component-axe-as-shovel-grass", plain("minecraft:grass_block"), "minecraft:iron_axe")
            .row()
            .patch(carrying(shovel))
            .row(),
        tool("tool-component-axe-as-shovel-log", log("minecraft:oak_log", "x"), "minecraft:iron_axe")
            .row()
            .patch(carrying(shovel))
            .row(),
        tool("tool-component-shovel-as-hoe", plain("minecraft:grass_block"), "minecraft:iron_shovel")
            .row()
            .patch(carrying(hoe))
            .row(),
    ]
}

pub(super) fn rows() -> Vec<Value> {
    let mut rows = [axe(), copper_chests(), hoe(), shovel()].concat();
    // The component is 26.3's.
    if !crate::components::is_26_2() {
        rows.extend(component());
    }
    rows
}
