use super::*;

pub(super) fn player_model_case(request: &Value) -> Result<Value, String> {
    use photon_render::entity::player_model::{Arm, PlayerModel, PlayerRenderState};
    let rows = request["rows"]
        .as_array()
        .ok_or("entity rows must be an array")?;
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let flag = |row: &Value, key: &str| row[key].as_bool().unwrap_or(false);
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut model = PlayerModel::new(flag(row, "slim"));
        model.setup_anim(&PlayerRenderState {
            head_y_rot: bits(row, "head_y_rot")?,
            x_rot: bits(row, "x_rot")?,
            walk_position: bits(row, "walk_position")?,
            walk_speed: bits(row, "walk_speed")?,
            age_in_ticks: bits(row, "age_in_ticks")?,
            attack_time: bits(row, "attack_time")?,
            attack_arm: if row["attack_arm"].as_str() == Some("left") {
                Arm::Left
            } else {
                Arm::Right
            },
            swim_amount: bits(row, "swim_amount")?,
            crouching: flag(row, "crouching"),
            spectator: flag(row, "spectator"),
            ..Default::default()
        });
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let mut quads: Vec<String> = model
            .render(&Default::default())
            .into_iter()
            .map(|(_, corners)| {
                corners
                    .iter()
                    .map(|vertex| {
                        vertex
                            .position
                            .iter()
                            .chain(&vertex.uv)
                            .chain(&vertex.normal)
                            .map(|value| hex(*value))
                            .collect::<String>()
                    })
                    .collect::<String>()
            })
            .collect();
        quads.sort();
        out.push(json!({"label": row["label"], "quads": quads}));
    }
    Ok(json!({"rows": out}))
}
