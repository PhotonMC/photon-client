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
        let size = spawned_mob_size(row)?;
        let o_spin = f64::from_bits(
            u64::from_str_radix(string(row, "o_spin")?, 16).map_err(|e| e.to_string())?,
        );
        let spin = f64::from_bits(
            u64::from_str_radix(string(row, "spin")?, 16).map_err(|e| e.to_string())?,
        );
        let partial = f32::from_bits(
            u32::from_str_radix(string(row, "partial")?, 16).map_err(|e| e.to_string())?,
        );
        let shown = SpawnerDisplayPose::of((0, 0, 0), (o_spin, spin), partial, size);
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

/// The width and height of the mob a spawner shows, made as the real world makes it from the
/// spawner's update tag (the entity's `id`, `IsBaby` and `Size`).
fn spawned_mob_size(row: &Value) -> Result<(f32, f32), String> {
    use photon_world::{BlockEntityInput, DimensionBounds, SectionInput};
    let mut world = crate::case_ids::world(DimensionBounds::new(0, 16));
    let spawner = photon_data::block::registry::block("minecraft:spawner")
        .ok_or("no spawner block")?
        .default_state_id();
    let mut blocks = vec![0; 4_096];
    // The spawner stands at 1, 1, 1: the section's index runs y, z, x.
    blocks[256 + 16 + 1] = spawner.0;
    let biomes = vec![0; 64];
    let section = SectionInput {
        non_empty_block_count: 1,
        fluid_count: 0,
        block_states: &blocks,
        biomes: &biomes,
    };
    if !world.replace_chunk(0, 0, [section]) {
        return Err("the spawner chunk was rejected".into());
    }
    let bytes = spawner_tag(row)?;
    let type_id =
        photon_data::block_entity_type::id("minecraft:mob_spawner").ok_or("no spawner entity")?;
    let input = BlockEntityInput {
        x: 1,
        y: 1,
        z: 1,
        type_id,
        tag: Some(&bytes),
    };
    if !world.apply_block_entity_update(input) {
        return Err("the spawner was rejected".into());
    }
    // The renderer draws it, which makes the mob the tick then keeps.
    world.extract_spawner((1, 1, 1));
    world
        .tick_block_entities(None)
        .map_err(|_| "the spawner failed to tick".to_owned())?;
    world
        .spawner_display_size_at((1, 1, 1))
        .ok_or_else(|| "the spawner shows no mob".to_owned())
}

/// The update tag of a spawner whose `SpawnData` entity has the row's `id`, `IsBaby` and `Size`.
fn spawner_tag(row: &Value) -> Result<Vec<u8>, String> {
    use photon_core::nbt::{Field, FieldValue};
    let mut entity = vec![Field {
        key: "id".into(),
        value: FieldValue::Str(string(row, "type")?.to_owned()),
    }];
    if row["baby"].as_bool() == Some(true) {
        entity.push(Field {
            key: "IsBaby".into(),
            value: FieldValue::Byte(1),
        });
    }
    if let Some(size) = row["size"].as_i64() {
        entity.push(Field {
            key: "Size".into(),
            value: FieldValue::Int(size as i32),
        });
    }
    let data = Field {
        key: "entity".into(),
        value: FieldValue::Compound(entity),
    };
    let tag = [Field {
        key: "SpawnData".into(),
        value: FieldValue::Compound(vec![data]),
    }];
    let mut bytes = Vec::new();
    photon_core::nbt::write_nbt(Some(&tag), &mut bytes);
    Ok(bytes)
}
