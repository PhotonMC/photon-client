//! The poses of the block entities the entity family compares: the operations by name.

use super::*;

/// One of the block entity pose operations; none for an operation of another kind.
pub(super) fn block_pose_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    Some(match operation {
        "chest_model" => chest_model_result(request),
        "chest_pose" => chest_pose_result(request),
        "bell_pose" => bell_pose_result(request),
        "enchant_table_pose" => enchant_table_pose_result(request),
        "banner_pose" => banner_pose_result(request),
        "skull_block_pose" => skull_block_pose_result(request),
        "shulker_box_pose" => shulker_box_pose_result(request),
        _ => return None,
    })
}

/// `enchant_table_pose`: what an enchanting table's numbers come to at a partial tick, and the pose
/// and model its book is drawn with.
pub(super) fn enchant_table_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::block_entity::enchant_table::{LAYER, TEXTURE, book_pose, table_pose};
    use photon_render::entity::mob_model::MobModel;
    use photon_render::joml::Pose;
    use photon_world::{BookState, EnchantTableView};
    let layer = photon_data::entity_model::layer(LAYER).ok_or("no layer")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            let text = row[key].as_str().ok_or_else(|| key.to_owned())?;
            u32::from_str_radix(text, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())
        };
        let book = BookState {
            time: i32_field(row, "time")?,
            flip: float("flip")?,
            o_flip: float("o_flip")?,
            open: float("open")?,
            o_open: float("o_open")?,
            rot: float("rot")?,
            o_rot: float("o_rot")?,
            ..BookState::default()
        };
        let view = EnchantTableView::of(&book, float("partial")?);
        let pose = book_pose(view.time, view.flip, view.open);
        let mut model = MobModel::bake(layer);
        model.setup_book(pose.openness, pose.pages);
        let placed = table_pose(&Pose::default(), view.time, view.y_rot);
        let bits = |values: &[f32]| -> Vec<String> {
            values
                .iter()
                .map(|value| format!("{:08x}", value.to_bits()))
                .collect()
        };
        rows.push(json!({
            "label": row["label"],
            "view": bits(&[view.flip, view.open, view.time, view.y_rot]),
            "matrix": super::strider::matrix_bits(&placed),
            "book": bits(&[pose.openness, pose.pages.0, pose.pages.1]),
            "texture": TEXTURE,
            "quads": posed_quads(&model),
        }));
    }
    Ok(json!({"rows": rows}))
}
