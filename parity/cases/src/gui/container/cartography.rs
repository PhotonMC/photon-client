//! The cartography table with a map in its first slot: the map's picture and decorations drawn
//! over the table's, the error mark, and the picture for what the second slot holds.

use serde_json::{Value, json};

use super::{Screen, screen};
use crate::components::{MAP_ID, patch, vi};

/// A map of four bands of colour with the decorations that show on a frame (a frame's own, a
/// mansion, a banner) and one that does not (a player).
fn map(id: i64, scale: i64, locked: bool, decorated: bool) -> Value {
    let runs = json!([[0, 4096, 5], [4096, 4096, 29], [8192, 4096, 45], [12288, 4096, 107], [100, 40, 18]]);
    let decorations = if decorated {
        json!([
            {"type": "frame", "x": 20, "y": -30, "rot": 4},
            {"type": "mansion", "x": -64, "y": 50, "rot": 8},
            {"type": "banner_red", "x": 90, "y": 100, "rot": 15},
            {"type": "player", "x": 0, "y": 0, "rot": 0},
        ])
    } else {
        json!([])
    };
    json!({"id": id, "scale": scale, "locked": locked, "runs": runs, "decorations": decorations})
}

/// A table holding the item `first` with the map id `id` in its first slot and `second` in its
/// second, and the map described.
fn table(first: &'static str, second: Option<&'static str>, described: Option<Value>) -> Value {
    let mut slots = vec![(0, first, 1)];
    let mut sync_slot = None;
    if let Some(second) = second {
        slots.push((1, second, 1));
        sync_slot = Some(1);
    }
    let held = Screen {
        slots,
        sync_slot,
        patches: vec![(0, patch(&[(MAP_ID, vi(1))]))],
        ..screen("cartography_table", "container.cartography_table")
    };
    let mut row = held.row("");
    if let Some(described) = described {
        row["map"] = described;
    }
    row
}

fn labelled(label: &str, mut row: Value) -> Value {
    row["label"] = json!(label);
    row
}

pub(super) fn rows() -> Vec<Value> {
    let filled = "filled_map";
    vec![
        labelled("cartography-map-plain", table(filled, None, Some(map(1, 0, false, false)))),
        labelled("cartography-map-no-data", table(filled, Some("paper"), None)),
        labelled("cartography-map-scaled", table(filled, Some("paper"), Some(map(1, 1, false, true)))),
        labelled("cartography-map-scaled-largest", table(filled, Some("paper"), Some(map(1, 4, false, true)))),
        labelled("cartography-map-scaled-locked", table(filled, Some("paper"), Some(map(1, 1, true, false)))),
        labelled("cartography-map-scaled-locked-largest", table(filled, Some("paper"), Some(map(1, 4, true, false)))),
        // Only a filled map can be made larger, from 26.3 on; the stick holds a map id as any item may.
        labelled("cartography-map-scaled-other-item", table("stick", Some("paper"), Some(map(1, 1, false, false)))),
        labelled("cartography-map-duplicated", table(filled, Some("map"), Some(map(1, 0, false, true)))),
        labelled("cartography-map-duplicated-locked", table(filled, Some("map"), Some(map(1, 0, true, false)))),
        labelled("cartography-map-locking", table(filled, Some("glass_pane"), Some(map(1, 2, false, true)))),
        labelled("cartography-map-locking-locked", table(filled, Some("glass_pane"), Some(map(1, 2, true, false)))),
        labelled("cartography-map-other-item", table(filled, Some("stick"), Some(map(1, 0, false, true)))),
    ]
}
