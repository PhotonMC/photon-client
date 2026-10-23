//! Blocks whose state follows the block placed beside or below them (`updateShape`).

use super::shapes::{SIDES, neighbour_row};
use super::*;

/// Beds, campfires and doors, whose state follows the block placed beside or below them.
pub fn paired_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (side, at) in SIDES {
        for occupied in ["true", "false"] {
            let world = vec![
                block([0, 63, 0], "stone", &[]),
                block(at, "red_bed", &[("part", "head"), ("facing", side), ("occupied", occupied)]),
                block([at[0], 63, at[2]], "end_stone", &[]),
            ];
            rows.push(neighbour_row(&format!("shape-bed-{side}-{occupied}"), world, &["red_bed"]));
        }
    }
    for (lit, signal) in [("true", "false"), ("false", "true")] {
        let world = vec![
            block([0, 63, 0], "stone", &[]),
            block([0, 65, 0], "campfire", &[("lit", lit), ("signal_fire", signal)]),
        ];
        let label = format!("shape-campfire-{lit}");
        rows.push(neighbour_row(&label, world, &["hay_block", "stone", "oak_fence"]));
    }
    let door = vec![
        block([0, 63, 0], "stone", &[]),
        block([0, 65, 0], "oak_door", &[("half", "lower"), ("open", "true"), ("hinge", "right")]),
        block([0, 66, 0], "oak_door", &[("half", "upper"), ("open", "true"), ("hinge", "right")]),
    ];
    rows.push(neighbour_row("shape-door-over", door, &["stone", "glass", "oak_fence", "snow_block"]));
    rows
}

/// Attached stems: a neighbour other than the fruit in the way they face turns them back.
pub fn stem_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (stem, fruit) in [("attached_melon_stem", "melon"), ("attached_pumpkin_stem", "pumpkin")] {
        for ((side, at), back) in SIDES.into_iter().zip(["south", "west", "north", "east"]) {
            let world = vec![
                block([0, 63, 0], "stone", &[]),
                block(at, stem, &[("facing", back)]),
                block([at[0], 63, at[2]], "end_stone", &[]),
            ];
            rows.push(neighbour_row(&format!("shape-{stem}-{side}"), world, &["stone", fruit]));
        }
    }
    rows
}

/// Concrete powder beside water turns to concrete when something is placed against it.
pub fn powder_rows() -> Vec<Value> {
    let stone = block([0, 63, 0], "stone", &[]);
    let placed = ["stone", "glass", "oak_slab"];
    let powder = |label: &str, extra: Vec<Value>| {
        let mut world = vec![stone.clone(), block([1, 64, 0], "white_concrete_powder", &[])];
        world.extend(extra);
        neighbour_row(label, world, &placed)
    };
    vec![
        powder("shape-powder-dry", vec![]),
        powder("shape-powder-east", vec![block([2, 64, 0], "water", &[])]),
        powder("shape-powder-north", vec![block([1, 64, -1], "water", &[])]),
        powder("shape-powder-above", vec![block([1, 65, 0], "water", &[])]),
        powder("shape-powder-below", vec![block([1, 63, 0], "water", &[])]),
        powder(
            "shape-powder-held",
            vec![block([2, 64, 0], "oak_slab", &[("waterlogged", "true"), ("type", "double")])],
        ),
    ]
}

fn carpet<'a>(bottom: &'a str, west: &'a str, north: &'a str) -> [(&'a str, &'a str); 5] {
    [("bottom", bottom), ("west", west), ("north", north), ("east", "none"), ("south", "none")]
}

/// Lichen, sculk veins and the pale moss carpet, which keep only the faces and sides their
/// neighbours can still hold.
pub fn face_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let placed = ["stone", "glass", "oak_fence", "oak_slab"];
    for kind in ["glow_lichen", "sculk_vein"] {
        for (label, props) in
            [("alone", vec![("west", "true")]), ("with-floor", vec![("west", "true"), ("down", "true")])]
        {
            let world = vec![
                block([0, 63, 0], "stone", &[]),
                block([1, 64, 0], kind, &props),
                block([1, 63, 0], "end_stone", &[]),
            ];
            rows.push(neighbour_row(&format!("shape-{kind}-{label}"), world, &placed));
        }
    }
    let ground = block([1, 63, 0], "end_stone", &[]);
    let wall = block([1, 64, -1], "end_stone", &[]);
    let base = |at: [i32; 3], north: &str| block(at, "pale_moss_carpet", &carpet("true", "none", north));
    let on_stone = vec![
        block([0, 63, 0], "stone", &[]),
        ground.clone(),
        wall.clone(),
        block([1, 64, 0], "pale_moss_carpet", &carpet("true", "low", "low")),
    ];
    rows.push(neighbour_row("shape-carpet-base", on_stone, &placed));
    for (label, below_north) in [("held", "low"), ("dropped", "none")] {
        let world = vec![
            block([0, 63, 0], "stone", &[]),
            block([1, 62, 0], "end_stone", &[]),
            base([1, 63, 0], below_north),
            wall.clone(),
            block([1, 64, 0], "pale_moss_carpet", &carpet("false", "low", "low")),
        ];
        rows.push(neighbour_row(&format!("shape-carpet-upper-{label}"), world, &placed));
    }
    let tall = vec![
        block([0, 63, 0], "stone", &[]),
        ground,
        block([1, 64, 0], "pale_moss_carpet", &carpet("true", "low", "none")),
        block([1, 65, 0], "pale_moss_carpet", &carpet("false", "low", "none")),
    ];
    rows.push(neighbour_row("shape-carpet-tall", tall, &placed));
    let kept = vec![
        block([0, 63, 0], "stone", &[]),
        block([1, 62, 0], "end_stone", &[]),
        block([1, 63, 0], "pale_moss_carpet", &carpet("true", "low", "low")),
        wall,
        block([1, 64, 0], "pale_moss_carpet", &carpet("false", "tall", "low")),
    ];
    rows.push(neighbour_row("shape-carpet-kept", kept, &placed));
    rows
}

/// Bamboo: a sapling under bamboo becomes bamboo. A stalk under an older stalk cannot be reached
/// by placing (bamboo takes its age from the stalk below), so its rows only show the stalk stays.
pub fn bamboo_rows() -> Vec<Value> {
    let ground = block([0, 62, 0], "dirt", &[]);
    let sapling = vec![ground.clone(), block([0, 63, 0], "bamboo_sapling", &[])];
    let mut rows = vec![neighbour_row("shape-bamboo-sapling", sapling, &["bamboo", "stone"])];
    let beside = vec![
        block([0, 62, 0], "dirt", &[]),
        block([0, 63, 0], "bamboo", &[]),
        block([1, 63, 0], "dirt", &[]),
        block([1, 64, 0], "bamboo_sapling", &[]),
    ];
    rows.push(neighbour_row("shape-bamboo-sapling-beside", beside, &["bamboo", "stone"]));
    for age in ["0", "1"] {
        let stalk = vec![ground.clone(), block([0, 63, 0], "bamboo", &[("age", age)])];
        rows.push(neighbour_row(&format!("shape-bamboo-stalk-{age}"), stalk, &["bamboo", "stone"]));
    }
    rows
}

/// Bells hung from walls, along both axes: a double bell loses a side that stops holding it,
/// a single one gains the side opposite its support.
pub fn bell_rows() -> Vec<Value> {
    let placed = ["stone", "oak_fence", "oak_stairs", "glass", "oak_slab"];
    // (axis, bell, far support, name of the way to the far support, and of the placed side)
    let axes = [("x", [1, 64, 0], [2, 64, 0], "east", "west"), ("z", [0, 64, 1], [0, 64, 2], "south", "north")];
    let mut rows = Vec::new();
    for (axis, at, far, toward_far, toward_placed) in axes {
        let walls =
            [("double", toward_far), ("double", toward_placed), ("single", toward_far), ("single", toward_placed)];
        for (attachment, facing) in walls {
            let kind = format!("{attachment}_wall");
            let world = vec![
                block(at, "bell", &[("attachment", kind.as_str()), ("facing", facing)]),
                block(far, "end_stone", &[]),
                block([0, 63, 0], "stone", &[]),
            ];
            let label = format!("shape-bell-{axis}-{attachment}-{facing}");
            rows.push(neighbour_row(&label, world, &placed));
        }
    }
    rows
}
