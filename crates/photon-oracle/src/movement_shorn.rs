//! The shorn flags of a movement row's entity.

use serde_json::Value;

/// A shorn row's metadata: a sheep's wool byte (the dye in the low four bits, sheared in bit
/// four) or a bogged's sheared flag.
pub(crate) fn data(name: &str, described: &Value) -> Option<[u8; 3]> {
    let sheared = described.get("sheared").is_some();
    let dye = described.get("dye").and_then(Value::as_u64);
    if name == "minecraft:bogged" {
        return sheared.then_some([photon_world::BOGGED_SHEARED_DATA_ID, 8, 1]);
    }
    let wool = dye.unwrap_or(0) as u8 | if sheared { 16 } else { 0 };
    (sheared || dye.is_some()).then_some([photon_world::SHEEP_WOOL_DATA_ID, 0, wool])
}
