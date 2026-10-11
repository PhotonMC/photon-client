//! The movement family: entities: items, living entities, projectiles and
//! shadows.

mod animation;
mod damage;
mod events;
mod items;
mod living;
mod pick_table;
mod prelude;
mod projectiles;
mod scene;
mod shadows;
pub(crate) mod world;

use serde_json::{Value, json};

use crate::case::case;

pub fn cases() -> Vec<Value> {
    let ticks = |id: &str, operation: &str, rows: Vec<Value>| case(id, "movement", operation, json!({ "rows": rows }));
    vec![
        ticks("movement/item-entity-ticks", "item_entity_ticks", items::rows()),
        ticks("movement/living-entity-ticks", "living_entity_ticks", damage::rows()),
        ticks("movement/living-entity-animations", "living_entity_ticks", animation::rows()),
        pick_table::entity_pick_table(),
        ticks("movement/projectile-ticks", "projectile_ticks", projectiles::rows()),
        ticks("movement/entity-shadows", "entity_shadows", shadows::rows()),
    ]
}
