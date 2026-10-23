use super::*;
use photon_world::block::rules::{self, PlaceContext, Rule};
use photon_world::block::tags::BlockTags;

/// `placement_rules`: each block of a row used on a small scripted world, answered by the rules
/// the client predicts a placement with. A block whose rule is not written is an error, so a
/// case never lists one.
pub(super) fn placement_rules(request: &Value) -> Result<Value, String> {
    let packet = vanilla_block_tags()?;
    let tags = BlockTags::from_registries(&packet.registries).ok_or("no block tags")?;
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let rows: Result<Vec<Value>, String> =
        rows.iter().map(|row| placement_row(row, &tags)).collect();
    Ok(json!({"rows": rows?}))
}

fn placement_row(row: &Value, tags: &BlockTags) -> Result<Value, String> {
    let mut world = std::collections::HashMap::new();
    for entry in row["world"].as_array().ok_or("world must be an array")? {
        world.insert(position(&entry["pos"])?, named_state(entry)?);
    }
    let air = named_state(&json!({"block": "minecraft:air"}))?;
    let state_at = |x: i32, y: i32, z: i32| world.get(&(x, y, z)).copied().unwrap_or(air);
    let face = Direction::from_name(string(row, "face")?).ok_or("unknown face")?;
    let location = row["location"]
        .as_array()
        .ok_or("location must be an array")?;
    let coordinate = |index: usize| f64_field(&json!({"c": location[index]}), "c");
    let yaw = f32_field(row, "yaw")?;
    let pitch = f32_field(row, "pitch")?;
    let mut placements = Vec::new();
    for name in row["blocks"].as_array().ok_or("blocks must be an array")? {
        let name = name.as_str().ok_or("block names are strings")?;
        let block = photon_data::block::registry::block(name)
            .ok_or_else(|| format!("unknown block {name}"))?;
        let aimed = (
            position(&row["hit"])?,
            face,
            (coordinate(0)?, coordinate(1)?, coordinate(2)?),
        );
        let item = photon_game::item::id_of(block.item_name());
        let mut ctx = PlaceContext::new(&state_at, aimed, item, (yaw, pitch), tags);
        ctx.secondary_use = row["sneak"].as_bool().unwrap_or(false);
        ctx.inside = row["inside"].as_bool().unwrap_or(false);
        ctx.game_master = row["game_master"].as_bool().unwrap_or(false);
        placements.push(place(name, &ctx)?);
    }
    Ok(json!({
        "label": string(row, "label")?,
        "outcome": "listed",
        "placements": placements,
    }))
}

fn position(value: &Value) -> Result<(i32, i32, i32), String> {
    let at = |index: usize| {
        value[index]
            .as_i64()
            .and_then(|coordinate| i32::try_from(coordinate).ok())
            .ok_or_else(|| "positions are three integers".to_owned())
    };
    Ok((at(0)?, at(1)?, at(2)?))
}

/// The three questions of one block, in the order `BlockItem.place` asks them.
fn place(name: &str, ctx: &PlaceContext<'_>) -> Result<Value, String> {
    let unported = || format!("{name}: a placement rule is not written");
    let block = photon_data::block::registry::block_index(name)
        .ok_or_else(|| format!("unknown block {name}"))?;
    let mut ctx = PlaceContext {
        replace_clicked: true,
        ..*ctx
    };
    let clicked = (ctx.state_at)(ctx.hit_pos.0, ctx.hit_pos.1, ctx.hit_pos.2);
    let Rule::Known(replace_clicked) = rules::can_be_replaced(clicked, &ctx) else {
        return Err(unported());
    };
    ctx.replace_clicked = replace_clicked;
    let pos = ctx.clicked_pos();
    let can_place = if replace_clicked {
        true
    } else {
        match rules::can_be_replaced((ctx.state_at)(pos.0, pos.1, pos.2), &ctx) {
            Rule::Known(replaced) => replaced,
            Rule::Unported => return Err(unported()),
        }
    };
    let Rule::Known(state) = rules::state_for_placement(block, &ctx) else {
        return Err(unported());
    };
    let mut described = json!({
        "block": name,
        "replace_clicked": replace_clicked,
        "can_place": can_place,
    });
    match state {
        None => {}
        Some(state) => {
            described["state"] = json!(rules::serialize_state(state));
            let Rule::Known(survives) = rules::can_survive(state, pos, &ctx) else {
                return Err(unported());
            };
            described["survives"] = json!(survives);
        }
    }
    item_placement(block, &ctx, &mut described, unported)?;
    Ok(described)
}

/// What the item classes beyond `BlockItem` add: the state the item chooses and the blocks it
/// sets, in order.
fn item_placement(
    block: usize,
    ctx: &PlaceContext<'_>,
    described: &mut Value,
    unported: impl Fn() -> String,
) -> Result<(), String> {
    let Some(kind) = ctx.item.and_then(rules::item_placement) else {
        return Ok(());
    };
    if kind == rules::ItemPlacement::Plain {
        return Ok(());
    }
    let Rule::Known(updated) = rules::update_context(kind, ctx) else {
        return Err(unported());
    };
    let Some(ctx) = updated else {
        return Ok(());
    };
    let ctx = &ctx;
    let Rule::Known(chosen) = rules::placement_state(kind, block, ctx) else {
        return Err(unported());
    };
    let Some(state) = chosen else {
        return Ok(());
    };
    described["item_state"] = json!(rules::serialize_state(state));
    let writes = rules::place_writes(kind, state, ctx.clicked_pos(), ctx);
    described["writes"] = writes
        .in_order()
        .map(|write| {
            json!({
                "pos": [write.pos.0, write.pos.1, write.pos.2],
                "state": rules::serialize_state(write.state),
                "flags": write.flags,
            })
        })
        .collect();
    Ok(())
}
