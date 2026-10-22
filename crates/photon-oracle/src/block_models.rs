//! The block models the client builds in (`BuiltInBlockModels`): the operation by name.

use super::*;

/// The built-in block model operation; none for an operation of another kind.
pub(super) fn block_models_result(operation: &str) -> Option<Result<Value, String>> {
    (operation == "builtin_block_models").then(|| Ok(builtin_block_models_result()))
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
