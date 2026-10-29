use super::*;

/// `witch_model`: the witch's model posed and drawn.
pub(super) fn witch_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{MobModel, QuadrupedState, WitchState};
    let layer = photon_data::entity_model::layer("minecraft:witch#main").ok_or("no witch layer")?;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let mut model = MobModel::bake(layer);
        model.setup_witch(&WitchState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            entity_id: row["entity_id"].as_i64().ok_or("entity_id")? as i32,
            holding_item: row["holding_item"].as_bool().ok_or("holding_item")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `creeper_render`: the scale a swelling creeper's renderer applies, and how
/// far it flashes white.
pub(super) fn creeper_render_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{creeper_scale, creeper_white_overlay};
    use photon_render::joml::Pose;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let swelling = u32::from_str_radix(row["swelling"].as_str().ok_or("swelling")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let [x, y, z] = creeper_scale(swelling);
        let mut pose = Pose::default();
        pose.scale(x, y, z);
        let matrix: Vec<String> = (0..4)
            .flat_map(|column| (0..4).map(move |row| (column, row)))
            .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
            .collect();
        out.push(json!({
            "label": row["label"],
            "scale": matrix,
            "overlay": format!("{:08x}", creeper_white_overlay(swelling).to_bits()),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `sheep_model`: a sheep's models (the body, the undercoat, the wool) posed
/// and drawn.
pub(super) fn sheep_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{MobModel, QuadrupedState, SheepState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = photon_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let mut model = MobModel::bake(layer);
        model.setup_sheep(&SheepState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            head_eat_position_scale: bits("eat_position")?,
            head_eat_angle_scale: bits("eat_angle")?,
            age_scale: bits("age_scale")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `chicken_model`: a chicken's model posed with its flap and drawn.
pub(super) fn chicken_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{ChickenState, MobModel};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = photon_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let mut model = MobModel::bake(layer);
        model.setup_chicken(&ChickenState {
            head_y_rot: bits("head_y_rot")?,
            x_rot: bits("x_rot")?,
            walk_position: bits("walk_position")?,
            walk_speed: bits("walk_speed")?,
            flap: bits("flap")?,
            flap_speed: bits("flap_speed")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `sheep_colors`: what a sheep's wool is multiplied by, for each dye and for
/// a `jeb_` sheep at points of its cycle.
pub(super) fn sheep_colors_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::wool::{jeb_colour, wool_colour};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        // The reference answers a signed int with the alpha in its top byte.
        if let Some(dye) = row["texture_dye"].as_u64() {
            let colour = photon_data::entity_type::dye_texture_colour(dye as usize).ok_or("dye")?;
            out.push(json!({"label": row["label"], "color": colour as i32}));
            continue;
        }
        let colour = if let Some(dye) = row["dye"].as_u64() {
            wool_colour(dye as usize)
        } else {
            let text = row["tick"].as_str().ok_or("tick")?;
            let tick = u32::from_str_radix(text, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())?;
            jeb_colour(tick)
        };
        out.push(json!({"label": row["label"], "color": (colour | 0xFF00_0000) as i32}));
    }
    Ok(json!({"rows": out}))
}

/// `sheep_eat`: how far a sheep's head is down and how it is tipped.
pub(super) fn sheep_eat_result(request: &Value) -> Result<Value, String> {
    use photon_world::mobs::animal::{head_eat_angle_scale, head_eat_position_scale};
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let tick = row["eat_tick"].as_i64().ok_or("eat_tick")? as i32;
        let partial = bits("partial")?;
        out.push(json!({
            "label": row["label"],
            "position": hex(head_eat_position_scale(tick, partial)),
            "angle": hex(head_eat_angle_scale(tick, partial, bits("x_rot")?)),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `held_item_pose`: a humanoid model posed with its arms' poses, and the
/// pose each held item is drawn from (`ItemInHandLayer`), before the item's
/// own display transform.
/// The items that are kinetic weapons, with the numbers a spear is posed and timed by.
pub(super) fn spear_table_result() -> Value {
    let rows: Vec<Value> = crate::case_ids::items()
        .filter(|item| item.kinetic_weapon)
        .filter_map(|item| photon_data::spear::spear_of(item.name))
        .map(|spear| {
            json!({
                "name": spear.name,
                "swing": "STAB",
                "swing_ticks": spear.swing_ticks,
                "delay_ticks": spear.delay_ticks,
                "dismount_ticks": spear.dismount_ticks,
                "knockback_ticks": spear.knockback_ticks,
                "damage_ticks": spear.damage_ticks,
                "forward_movement": format!(
                    "{:08x}",
                    photon_data::spear::FORWARD_MOVEMENT.to_bits()
                ),
            })
        })
        .collect();
    json!({ "rows": rows })
}
