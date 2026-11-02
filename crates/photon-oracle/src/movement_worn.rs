//! The items a row's entities wear and hold.

use serde_json::Value;

use super::*;

/// The `armored` (body slot, wolf armor unless it names an item), `saddled` (a saddle unless it
/// names an item), `offhand` and `mainhand` flags.
pub(super) fn apply_worn_items(
    world: &mut photon_world::ClientWorld,
    id: EntityId,
    described: &Value,
) -> Result<(), String> {
    use photon_world::EquipmentSlot;
    let worn = [
        ("armored", EquipmentSlot::Body, "minecraft:wolf_armor"),
        ("saddled", EquipmentSlot::Saddle, "minecraft:saddle"),
        ("offhand", EquipmentSlot::OffHand, ""),
        ("mainhand", EquipmentSlot::MainHand, ""),
    ];
    for (key, slot, default) in worn {
        let Some(flag) = described.get(key) else {
            continue;
        };
        let name = flag.as_str().unwrap_or(default);
        let item = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        let mut stack = photon_world::ClientItemStackState::new(1, item);
        if let (EquipmentSlot::Body, Some(hex)) = (slot, described["armor_patch"].as_str()) {
            stack.set_patch(&super::movement_gear::wire_patch(hex)?);
        }
        world.entities_mut().apply_equipment(id, slot, Some(stack));
    }
    Ok(())
}
