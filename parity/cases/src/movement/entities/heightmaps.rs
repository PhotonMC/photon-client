//! The heightmaps a client chunk keeps: the packet's values, as sent (and where they disagree
//! with the blocks), kept up as block updates change the chunk.

use crate::movement::entities::prelude::*;

const BITS: u32 = 9;
const PER_LONG: usize = 64 / BITS as usize;
const TYPES: [&str; 3] = ["WORLD_SURFACE", "MOTION_BLOCKING", "MOTION_BLOCKING_NO_LEAVES"];

/// A map's 256 stored values (the first free height less the world's bottom, -64) packed as the
/// packet packs a world 384 high: nine bits to a value, seven to a long, none across two.
fn packed(values: &[u32; 256]) -> Vec<Value> {
    values
        .chunks(PER_LONG)
        .map(|group| {
            let word = group
                .iter()
                .enumerate()
                .fold(0_u64, |word, (index, value)| word | u64::from(*value) << (index as u32 * BITS));
            json!(format!("{word:x}"))
        })
        .collect()
}

/// Every column at the first free height `y`, but for `columns` at their own.
fn level(y: i64, columns: &[((usize, usize), i64)]) -> [u32; 256] {
    let mut values = [(y + 64) as u32; 256];
    for ((x, z), height) in columns {
        values[z * 16 + x] = (height + 64) as u32;
    }
    values
}

/// The three maps sent with these values.
fn sent(values: &[u32; 256]) -> Vec<Value> {
    TYPES.iter().map(|kind| json!({"chunk": [0, 0], "type": kind, "raw": packed(values)})).collect()
}

fn probes(columns: &[(i64, i64)]) -> Vec<Value> {
    columns.iter().map(|(x, z)| json!({"x": x, "z": z})).collect()
}

struct Row {
    label: &'static str,
    blocks: Vec<Value>,
    heightmaps: Vec<Value>,
    edits: Vec<Value>,
    probes: Vec<Value>,
}

impl Row {
    fn value(self) -> Value {
        json!({
            "label": self.label, "blocks": self.blocks, "chunk_radius": 0,
            "heightmaps": self.heightmaps, "edits": self.edits, "probes": self.probes,
        })
    }
}

fn row(label: &'static str, blocks: Vec<Value>, heights: &[u32; 256], edits: Vec<Value>, at: &[(i64, i64)]) -> Row {
    Row { label, blocks, heightmaps: sent(heights), edits, probes: probes(at) }
}

pub fn rows() -> Vec<Value> {
    cat![update_rows(), removal_rows(), kind_rows(), unpacking_rows(), mismatched_rows()]
}

/// Where a block update lands against the packet's value.
fn update_rows() -> Vec<Value> {
    let at = |x: usize, z: usize| (x, z);
    let rows = vec![
        // The packet agrees with the blocks: one stone at 64 gives a free height of 65.
        row(
            "agrees-with-blocks",
            vec![put(1, 64, 1, "minecraft:stone")],
            &level(-64, &[(at(1, 1), 65)]),
            vec![],
            &[(1, 1), (0, 0)],
        ),
        // The packet says 100 where the blocks say 65.
        row(
            "stale-high-place-above",
            vec![put(2, 64, 2, "minecraft:stone")],
            &level(-64, &[(at(2, 2), 100)]),
            vec![put(2, 101 - 1, 2, "minecraft:stone")],
            &[(2, 2)],
        ),
        row(
            "stale-high-place-two-under",
            vec![put(2, 64, 2, "minecraft:stone")],
            &level(-64, &[(at(2, 2), 100)]),
            vec![put(2, 98, 2, "minecraft:stone")],
            &[(2, 2)],
        ),
        row(
            "stale-high-place-one-under",
            vec![put(2, 64, 2, "minecraft:stone")],
            &level(-64, &[(at(2, 2), 100)]),
            vec![put(2, 99, 2, "minecraft:stone")],
            &[(2, 2)],
        ),
        row(
            "stale-high-place-at-top",
            vec![put(2, 64, 2, "minecraft:stone")],
            &level(-64, &[(at(2, 2), 100)]),
            vec![put(2, 100, 2, "minecraft:stone")],
            &[(2, 2)],
        ),
    ];
    rows.into_iter().map(Row::value).collect()
}

/// A block goes: the scan finds the next one down, or the bottom of the world.
fn removal_rows() -> Vec<Value> {
    let at = |x: usize, z: usize| (x, z);
    let rows = vec![
        row(
            "remove-top-scans-down",
            vec![put(3, 64, 3, "minecraft:stone"), put(3, 40, 3, "minecraft:stone")],
            &level(-64, &[(at(3, 3), 65)]),
            vec![put(3, 64, 3, "minecraft:air")],
            &[(3, 3)],
        ),
        row(
            "remove-only-block",
            vec![put(4, 64, 4, "minecraft:stone")],
            &level(-64, &[(at(4, 4), 65)]),
            vec![put(4, 64, 4, "minecraft:air")],
            &[(4, 4)],
        ),
        // Air over air changes nothing, whatever the packet says is the top.
        row(
            "air-over-air-at-stale-top",
            vec![put(5, 64, 5, "minecraft:stone")],
            &level(-64, &[(at(5, 5), 80)]),
            vec![put(5, 79, 5, "minecraft:air")],
            &[(5, 5)],
        ),
        // A column the packet and the blocks both say is empty, then built on.
        row("build-on-empty", vec![], &level(-64, &[]), vec![put(6, -64, 6, "minecraft:stone")], &[(6, 6)]),
    ];
    rows.into_iter().map(Row::value).collect()
}

/// What each map counts: leaves, water, glass, a block with no collision, air that is not plain.
fn kind_rows() -> Vec<Value> {
    let at = |x: usize, z: usize| (x, z);
    let rows = vec![
        row(
            "kinds-leaves",
            vec![put(7, 64, 7, "minecraft:stone")],
            &level(-64, &[(at(7, 7), 65)]),
            vec![put(7, 80, 7, "minecraft:oak_leaves")],
            &[(7, 7)],
        ),
        row(
            "kinds-water",
            vec![put(8, 64, 8, "minecraft:stone")],
            &level(-64, &[(at(8, 8), 65)]),
            vec![put(8, 80, 8, "minecraft:water")],
            &[(8, 8)],
        ),
        row(
            "kinds-glass",
            vec![put(9, 64, 9, "minecraft:stone")],
            &level(-64, &[(at(9, 9), 65)]),
            vec![put(9, 80, 9, "minecraft:glass")],
            &[(9, 9)],
        ),
        row(
            "kinds-light-block",
            vec![put(10, 64, 10, "minecraft:stone")],
            &level(-64, &[(at(10, 10), 65)]),
            vec![put(10, 80, 10, "minecraft:light")],
            &[(10, 10)],
        ),
        row(
            "kinds-cave-air",
            vec![put(11, 64, 11, "minecraft:stone")],
            &level(-64, &[(at(11, 11), 65)]),
            vec![put(11, 80, 11, "minecraft:cave_air")],
            &[(11, 11)],
        ),
        // Leaves at the top come out: the leaf-free map keeps its stone.
        row(
            "remove-leaves",
            vec![put(12, 64, 12, "minecraft:stone"), put(12, 70, 12, "minecraft:oak_leaves")],
            &level(-64, &[(at(12, 12), 71)]),
            vec![put(12, 70, 12, "minecraft:air")],
            &[(12, 12)],
        ),
    ];
    rows.into_iter().map(Row::value).collect()
}

/// A value in every column, the last in a long, the first of the next and the last column.
fn unpacking_rows() -> Vec<Value> {
    let mut values = [0_u32; 256];
    for (index, value) in values.iter_mut().enumerate() {
        *value = (index as u32 * 37 + 5) % 385;
    }
    let corners = [(0, 0), (6, 0), (7, 0), (13, 0), (14, 0), (15, 0), (0, 15), (15, 15), (3, 9)];
    vec![row("every-column", vec![], &values, vec![], &corners).value()]
}

/// A map of the wrong length is not taken: the chunk's blocks give it.
fn mismatched_rows() -> Vec<Value> {
    let mut heightmaps = sent(&level(-64, &[]));
    heightmaps[1]["raw"] = json!(vec!["7fffffffffffffff"; 36]);
    let short = Row {
        label: "wrong-length-is-rebuilt",
        blocks: vec![put(2, 64, 2, "minecraft:stone"), put(2, 70, 2, "minecraft:oak_leaves")],
        heightmaps,
        edits: vec![],
        probes: probes(&[(2, 2), (3, 3)]),
    };
    let mut heightmaps = sent(&level(-64, &[]));
    heightmaps[0]["raw"] = json!(vec!["0"; 38]);
    let long = Row {
        label: "wrong-length-too-long",
        blocks: vec![put(4, 64, 4, "minecraft:stone")],
        heightmaps,
        edits: vec![],
        probes: probes(&[(4, 4)]),
    };
    vec![short.value(), long.value()]
}
