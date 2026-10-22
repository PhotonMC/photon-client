//! The block models the client builds in (`BuiltInBlockModels`): the operation by name.

use super::*;

/// The built-in block model operation; none for an operation of another kind.
pub(super) fn block_models_result(
    operation: &str,
    request: &Value,
) -> Option<Result<Value, String>> {
    match operation {
        "builtin_block_models" => Some(Ok(builtin_block_models_result())),
        "spawner_cage" => Some(spawner_cage_result(request)),
        _ => None,
    }
}

/// `spawner_cage`: the spin, scale and pose a spawner shows a mob through.
fn spawner_cage_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::SpawnerCage;
    use photon_world::SpawnerDisplayPose;
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = photon_data::entity_type::id(string(row, "type")?).ok_or("entity type")?;
        let (width, height, _) = photon_data::entity_type::pick(id).ok_or("pick")?.size;
        let o_spin = f64::from_bits(
            u64::from_str_radix(string(row, "o_spin")?, 16).map_err(|e| e.to_string())?,
        );
        let spin = f64::from_bits(
            u64::from_str_radix(string(row, "spin")?, 16).map_err(|e| e.to_string())?,
        );
        let partial = f32::from_bits(
            u32::from_str_radix(string(row, "partial")?, 16).map_err(|e| e.to_string())?,
        );
        let shown = SpawnerDisplayPose::of((0, 0, 0), (o_spin, spin), partial, (width, height));
        let cage = SpawnerCage {
            origin: [0.0; 3],
            spin: shown.spin,
            scale: shown.scale,
        };
        let matrix: Vec<String> = cage
            .pose()
            .pose
            .m
            .iter()
            .flatten()
            .map(|v| bits(*v))
            .collect();
        rows.push(serde_json::json!({
            "label": row["label"], "spin": bits(shown.spin), "scale": bits(shown.scale), "matrix": matrix,
        }));
    }
    Ok(serde_json::json!({ "rows": rows }))
}

/// `BuiltInBlockModels.createBlockModels`: every state that has a model of its own, by the name
/// of its serialized state.
fn builtin_block_models_result() -> Value {
    use photon_core::ids::BlockStateId;
    use photon_render::block_entity::special_model::SpecialModel;
    use photon_render::joml::Pose;
    let mut states: Vec<(String, String, Option<Vec<String>>)> = (0..)
        .map(BlockStateId)
        .take_while(|state| photon_world::block::state(*state).is_some())
        .filter_map(|state| {
            let model = SpecialModel::of(state)?;
            let name = photon_world::block::rules::serialize_state(state);
            let matrix = model.item_pose(&Pose::default()).map(|pose| {
                pose.pose
                    .m
                    .iter()
                    .flatten()
                    .map(|v| format!("{:08x}", v.to_bits()))
                    .collect::<Vec<_>>()
            });
            Some((name, model.describe(state), matrix))
        })
        .collect();
    states.sort();
    let states: Vec<Value> = states
        .into_iter()
        .map(|(state, model, matrix)| {
            let mut row = serde_json::json!({"state": state, "model": model});
            if let Some(matrix) = matrix {
                row["matrix"] = serde_json::json!(matrix);
            }
            row
        })
        .collect();
    serde_json::json!({ "states": states })
}
