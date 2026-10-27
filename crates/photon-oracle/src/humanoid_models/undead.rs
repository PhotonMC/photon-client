//! The skeleton, piglin and zombie model operations.

use super::*;

/// `skeleton_model`: a skeleton family model posed and drawn.
pub(crate) fn skeleton_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{MobModel, SkeletonState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = photon_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_skeleton(&SkeletonState {
            humanoid: humanoid_state(row)?,
            aggressive: row["aggressive"].as_bool().ok_or("aggressive")?,
            holding_bow: row["holding_bow"].as_bool().ok_or("holding_bow")?,
        });
        model.set_bogged_sheared(row["sheared"].as_bool().unwrap_or(false));
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `piglin_model`: a piglin family model posed and drawn.
pub(crate) fn piglin_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{MobModel, PiglinState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = photon_data::entity_model::layer(id).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_piglin(&PiglinState {
            humanoid: photon_render::entity::player_model::PlayerRenderState {
                main_arm: if row["main_arm"].as_str() == Some("left") {
                    photon_render::entity::player_model::Arm::Left
                } else {
                    photon_render::entity::player_model::Arm::Right
                },
                ticks_using_item: f32_field(row, "ticks_using_item")?,
                max_crossbow_charge_duration: f32_field(row, "max_crossbow_charge")?,
                ..humanoid_state(row)?
            },
            aggressive: row["aggressive"].as_bool().ok_or("aggressive")?,
            zombified: id.starts_with("minecraft:zombified_piglin"),
            baby: id.contains("_baby"),
            arm_pose: photon_render::entity::mob_model::PiglinArmPose::named(
                row["arm_pose"].as_str().ok_or("arm_pose")?,
            )
            .ok_or("unknown piglin arm pose")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `zombie_model`: a zombie family model posed and drawn.
pub(crate) fn zombie_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{MobModel, ZombieState};
    use photon_render::entity::player_model::PlayerRenderState;
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = photon_data::entity_model::layer(id).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_zombie(&ZombieState {
            humanoid: PlayerRenderState {
                head_y_rot: bits(row, "head_y_rot")?,
                x_rot: bits(row, "x_rot")?,
                walk_position: bits(row, "walk_position")?,
                walk_speed: bits(row, "walk_speed")?,
                age_in_ticks: bits(row, "age_in_ticks")?,
                attack_time: bits(row, "attack_time")?,
                swim_amount: bits(row, "swim_amount")?,
                ..Default::default()
            },
            aggressive: row["aggressive"].as_bool().ok_or("aggressive")?,
            drowned: id.starts_with("minecraft:drowned"),
        });
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
