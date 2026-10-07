use super::*;

/// Clicks in the player's own inventory menu, as the client predicts them
/// and sends them.
pub(super) fn container_clicks_result(request: &Value) -> Result<Value, String> {
    use voxel_game::menus::container::{ClickType, ContainerMenu, Player};
    use voxel_world::ClientItemStackState;
    let stack = |described: &Value| -> Result<ClientItemStackState, String> {
        let name = described["item"].as_str().ok_or("item must be a string")?;
        let id = voxel_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        let count = described["count"]
            .as_i64()
            .ok_or("count must be a number")? as i32;
        Ok(ClientItemStackState::new(count, id))
    };
    let describe = |stack: Option<&ClientItemStackState>| -> Value {
        match stack {
            None => Value::Null,
            Some(stack) => json!({
                "item": voxel_data::item::registry::ITEMS[stack.item_registry_id() as usize].name,
                "count": stack.item_count(),
            }),
        }
    };
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = row["label"].as_str().ok_or("label must be a string")?;
        let mut menu = match row.get("menu").and_then(Value::as_str) {
            None => ContainerMenu::default(),
            Some(name) => menu_named(name, row)?,
        };
        for slot in row["slots"].as_array().ok_or("slots must be an array")? {
            let index = slot["slot"].as_u64().ok_or("slot must be a number")? as usize;
            menu.slots[index] = Some(stack(slot)?);
        }
        if row.get("carried").is_some() {
            menu.carried = Some(stack(&row["carried"])?);
        }
        if menu.kind == voxel_game::menus::container::MenuKind::Merchant {
            // Setting the payments made the result, as the reference's slots do.
            menu.update_sell_item();
            if let Some(trade) = row.get("select").and_then(Value::as_u64) {
                menu.selection_hint = trade as usize;
                menu.update_sell_item();
                menu.try_move_items(trade as usize);
            }
        }
        menu.state_id = row["state_id"]
            .as_i64()
            .ok_or("state_id must be a number")? as i32;
        let selected = row["selected"]
            .as_u64()
            .ok_or("selected must be a number")? as usize;
        let player = Player {
            selected,
            infinite_materials: row["game_mode"].as_str() == Some("creative"),
        };
        let mut clicks = Vec::new();
        for click in row["clicks"].as_array().ok_or("clicks must be an array")? {
            let kind = ClickType::from_name(click["type"].as_str().ok_or("type must be a string")?)
                .ok_or("unknown click type")?;
            let (swings, body) = menu.handle_click(
                click["slot"].as_i64().ok_or("slot must be a number")? as i32,
                click["button"].as_i64().ok_or("button must be a number")? as i32,
                kind,
                player,
            );
            let mut sent = Vec::new();
            for _ in 0..swings {
                sent.push(json!({"type": "minecraft:swing", "bytes": "00"}));
            }
            sent.push(json!({"type": "minecraft:container_click", "bytes": hex(&body)}));
            let slots: Vec<Value> = menu
                .slots
                .iter()
                .map(|slot| describe(slot.as_ref()))
                .collect();
            let mut described = json!({"sent": sent, "slots": slots, "selected": selected});
            if menu.carried.is_some() {
                described["carried"] = describe(menu.carried.as_ref());
            }
            clicks.push(described);
        }
        out.push(json!({"label": label, "outcome": "clicked", "clicks": clicks}));
    }
    Ok(json!({"rows": out}))
}
