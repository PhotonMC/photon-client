//! Where a sleeper lies over a bed (`LivingEntity.setPosToBed`): the newest version asks the bed
//! block for its sleep height, the top of its shape, and a straw bed's head half answers with the
//! foot half it is joined to.

use serde_json::{Value, json};

use crate::case::case;

/// A block at a place with block state `properties`.
fn put_with(x: i64, y: i64, z: i64, block: &str, properties: &[(&str, &str)]) -> Value {
    let properties: serde_json::Map<String, Value> =
        properties.iter().map(|(name, value)| ((*name).to_owned(), json!(value))).collect();
    json!({"x": x, "y": y, "z": z, "block": block, "properties": properties})
}

/// A bed half of a kind, facing the way its foot looks toward its head.
fn half(pos: (i64, i64, i64), kind: &str, facing: &str, part: &str) -> Value {
    let properties = [("facing", facing), ("part", part), ("occupied", "false")];
    put_with(pos.0, pos.1, pos.2, kind, &properties)
}

/// Where the head half of a bed lies from its foot, along the direction the bed faces.
fn head_of(foot: (i64, i64, i64), facing: &str) -> (i64, i64, i64) {
    let (dx, dz) = match facing {
        "east" => (1, 0),
        "west" => (-1, 0),
        "south" => (0, 1),
        _ => (0, -1),
    };
    (foot.0 + dx, foot.1, foot.2 + dz)
}

/// Both halves of a bed laid out along a direction from the foot.
fn bed(foot: (i64, i64, i64), kind: &str, facing: &str) -> Vec<Value> {
    vec![half(foot, kind, facing, "foot"), half(head_of(foot, facing), kind, facing, "head")]
}

pub fn bed_sleeping_position() -> Vec<Value> {
    if crate::components::is_26_2() {
        return Vec::new();
    }
    let (white, straw) = ("minecraft:white_bed", "minecraft:straw_bed");
    let mut blocks = Vec::new();
    let mut rows = Vec::new();
    let mut ask = |label: &str, pos: (i64, i64, i64)| {
        rows.push(json!({"label": label, "x": pos.0, "y": pos.1, "z": pos.2}));
    };
    for (lane, facing) in ["east", "west", "south", "north"].into_iter().enumerate() {
        let z = lane as i64 * 4;
        for (offset, (name, kind)) in [("bed", white), ("straw-bed", straw)].into_iter().enumerate() {
            let x = offset as i64 * 8;
            let foot = (x, 64, z);
            blocks.extend(bed(foot, kind, facing));
            ask(&format!("{name}-foot-{facing}"), foot);
            ask(&format!("{name}-head-{facing}"), head_of(foot, facing));
        }
    }
    // A straw bed half with nothing joined to it: the foot lies by itself, the head has no height.
    blocks.push(half((0, 64, 20), straw, "east", "foot"));
    blocks.push(half((8, 64, 20), straw, "east", "head"));
    // A head whose foot is another kind of bed, or not a foot.
    blocks.push(half((0, 64, 24), white, "east", "foot"));
    blocks.push(half((1, 64, 24), straw, "east", "head"));
    blocks.push(half((8, 64, 24), straw, "east", "head"));
    blocks.push(half((7, 64, 24), straw, "east", "head"));
    // A bed with a block over it and a bed on a floor of stone, and what is no bed.
    blocks.push(put_with(0, 64, 28, "minecraft:stone", &[]));
    blocks.extend(bed((8, 64, 28), straw, "east"));
    blocks.push(put_with(8, 65, 28, "minecraft:stone", &[]));
    ask("lone-straw-foot", (0, 64, 20));
    ask("lone-straw-head", (8, 64, 20));
    ask("straw-head-over-white-foot", (1, 64, 24));
    ask("straw-head-over-straw-head", (8, 64, 24));
    ask("stone", (0, 64, 28));
    ask("air", (0, 70, 28));
    ask("straw-foot-under-stone", (8, 64, 28));
    let mut newer = case(
        "entity/bed-sleeping-position",
        "entity",
        "bed_sleeping_position",
        json!({"blocks": blocks, "rows": rows}),
    );
    newer["version"] = json!("26.3");
    vec![newer]
}
