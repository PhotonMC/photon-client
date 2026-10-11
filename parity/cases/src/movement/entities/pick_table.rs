//! What the crosshair needs of every entity type, as a new entity of it
//! answers: whether it can be aimed at, how much its box grows for the aim,
//! and its size (a baby's too, for the types that have babies).

use serde_json::{Value, json};

use crate::case::case;

pub fn entity_pick_table() -> Value {
    let rows = json!([{"label": "all", "blocks": [], "chunk_radius": 0}]);
    case("movement/entity-pick-table", "movement", "entity_pick_table", json!({ "rows": rows }))
}
