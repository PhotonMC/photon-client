//! Entity ids as the cases give them.

use photon_core::ids::EntityId;
use serde_json::Value;

use super::i32_field;

/// A case's field that names an entity.
pub(super) fn entity_id_field(value: &Value, name: &str) -> Result<EntityId, String> {
    i32_field(value, name).map(EntityId)
}
