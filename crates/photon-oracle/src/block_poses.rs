//! The poses of the block entities the entity family compares: the operations by name.

use super::*;

/// One of the block entity pose operations; none for an operation of another kind.
pub(super) fn block_pose_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    Some(match operation {
        "chest_model" => chest_model_result(request),
        "chest_pose" => chest_pose_result(request),
        "bell_pose" => bell_pose_result(request),
        "enchant_table_pose" => enchant_table_pose_result(request),
        "lectern_pose" => lectern_pose_result(request),
        "decorated_pot_pose" => decorated_pot_pose_result(request),
        "banner_pose" => banner_pose_result(request),
        "skull_block_pose" => skull_block_pose_result(request),
        "shulker_box_pose" => shulker_box_pose_result(request),
        _ => return beam_pose_result(operation, request),
    })
}

/// The block entity pose operations of the beam renderers.
fn beam_pose_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    (operation == "beacon_beam").then(|| beacon_beam_result(request))
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

/// `lectern_pose`: whether a lectern shows a book, the turn its facing gives it, and the pose and
/// model the book is drawn with.
fn lectern_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::block_entity::enchant_table::{LAYER, TEXTURE};
    use photon_render::block_entity::lectern::{lectern_book, lectern_pose};
    use photon_render::entity::mob_model::MobModel;
    use photon_render::joml::Pose;
    let layer = photon_data::entity_model::layer(LAYER).ok_or("no layer")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let facing =
            photon_core::direction::Direction::from_name(string(row, "facing")?).ok_or("facing")?;
        let has_book = row["has_book"].as_bool().ok_or("has_book")?;
        let turn = facing.clockwise_y().map_err(|_| "facing")?.to_y_rot();
        let turn_bits = format!("{:08x}", turn.to_bits());
        if !has_book {
            // No book: nothing is drawn.
            rows.push(json!({"label": row["label"], "has_book": false, "turn": turn_bits}));
            continue;
        }
        let book = lectern_book();
        let mut model = MobModel::bake(layer);
        model.setup_book(book.openness, book.pages);
        let placed = lectern_pose(&Pose::default(), turn);
        let bits = |values: &[f32]| -> Vec<String> {
            values
                .iter()
                .map(|value| format!("{:08x}", value.to_bits()))
                .collect()
        };
        rows.push(json!({
            "label": row["label"],
            "has_book": has_book,
            "turn": turn_bits,
            "matrix": super::strider::matrix_bits(&placed),
            "book": bits(&[book.openness, book.pages.0, book.pages.1]),
            "texture": TEXTURE,
            "quads": posed_quads(&model),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// `decorated_pot_pose`: the pose a pot is drawn from, and each part in the order it is drawn with
/// its picture and its quads.
fn decorated_pot_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::block_entity::decorated_pot::{
        BASE_LAYER, BASE_PARTS, BASE_TEXTURE, SIDE_PARTS, SIDES_LAYER, pot_pose, side_texture,
    };
    use photon_render::entity::mob_model::MobModel;
    use photon_render::joml::Pose;
    let base_layer = photon_data::entity_model::layer(BASE_LAYER).ok_or("no layer")?;
    let sides_layer = photon_data::entity_model::layer(SIDES_LAYER).ok_or("no layer")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let facing =
            photon_core::direction::Direction::from_name(string(row, "facing")?).ok_or("facing")?;
        let wobble =
            u32::from_str_radix(string(row, "wobble")?, 16).map_err(|error| error.to_string())?;
        let pose = pot_pose(&Pose::default(), facing, f32::from_bits(wobble));
        let sherds: Vec<Option<&str>> = row["sherds"]
            .as_array()
            .ok_or("sherds")?
            .iter()
            .map(Value::as_str)
            .collect();
        let sherd_of = |part: &str| match part {
            "back" => sherds[0],
            "left" => sherds[1],
            "right" => sherds[2],
            _ => sherds[3],
        };
        let mut base = MobModel::bake(base_layer);
        let mut sides = MobModel::bake(sides_layer);
        let mut parts = Vec::new();
        for part in BASE_PARTS {
            base.show_only(&BASE_PARTS, part);
            parts.push(json!({
                "part": part, "light": row["light"], "texture": BASE_TEXTURE, "quads": posed_quads(&base),
            }));
        }
        for part in SIDE_PARTS {
            sides.show_only(&SIDE_PARTS, part);
            parts.push(json!({
                "part": part, "light": row["light"], "texture": side_texture(sherd_of(part)),
                "quads": posed_quads(&sides),
            }));
        }
        rows.push(json!({
            "label": row["label"],
            "matrix": super::strider::matrix_bits(&pose),
            "parts": parts,
        }));
    }
    Ok(json!({"rows": rows}))
}

/// `beacon_beam`: each layer of a beam, in the order it is submitted, with the quads it writes.
fn beacon_beam_result(request: &Value) -> Result<Value, String> {
    use photon_render::block_entity::beacon::beam_layers;
    use photon_render::joml::Pose;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            let text = row[key].as_str().ok_or_else(|| key.to_owned())?;
            u32::from_str_radix(text, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())
        };
        let sections: Vec<(u32, i32)> = row["sections"]
            .as_array()
            .ok_or("sections")?
            .iter()
            .map(|section| {
                Ok((
                    section["color"].as_i64().ok_or("color")? as u32,
                    i32_field(section, "height")?,
                ))
            })
            .collect::<Result<_, String>>()?;
        let layers = beam_layers(
            &Pose::default(),
            &sections,
            (float("animation_time")?, float("radius_scale")?),
            2048,
        );
        let layers: Vec<Value> = layers.iter().map(layer_json).collect();
        rows.push(json!({"label": row["label"], "layers": layers}));
    }
    Ok(json!({"rows": rows}))
}

/// One layer of a beam: whether it is translucent and the bits of every vertex of its quads.
fn layer_json(layer: &photon_render::block_entity::beacon::BeamLayer) -> Value {
    use photon_render::block_entity::beacon::FULL_LIGHT;
    let bits = |values: &[f32]| -> String {
        values
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect()
    };
    let quads: Vec<String> = layer
        .quads
        .iter()
        .map(|quad| {
            quad.iter()
                .map(|vertex| {
                    format!(
                        "{}{}{:08x}{:08x}{}",
                        bits(&vertex.position),
                        bits(&vertex.uv),
                        vertex.color,
                        FULL_LIGHT,
                        bits(&vertex.normal)
                    )
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect();
    json!({"translucent": layer.translucent, "quads": quads})
}
