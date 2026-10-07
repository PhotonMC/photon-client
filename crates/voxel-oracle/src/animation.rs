use super::*;

/// `dolphin_item_pose`: where a dolphin carries what it holds.
pub(super) fn dolphin_item_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let x_rot = u32::from_str_radix(row["x_rot"].as_str().ok_or("x_rot")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut pose = voxel_render::entity::mob_model::dolphin_item_pose(&Pose::default(), x_rot);
        // The stack's own display transform is none: the model is moved to its corner.
        pose.translate(-0.5, -0.5, -0.5);
        out.push(json!({"label": row["label"], "pose": matrix_bits(&pose)}));
    }
    Ok(json!({"rows": out}))
}

/// `fish_model`: a fish's model posed and drawn, by its layer.
pub(super) fn fish_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let age = u32::from_str_radix(row["age_in_ticks"].as_str().ok_or("age")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let in_water = row["in_water"].as_bool().ok_or("in_water")?;
        let mut model = MobModel::bake(layer);
        if id.starts_with("minecraft:cod#") {
            model.setup_tail_fish("tail_fin", age, in_water);
        } else if id.starts_with("minecraft:salmon") {
            model.setup_salmon(age, in_water);
        } else if id.starts_with("minecraft:tropical_fish") {
            model.setup_tail_fish("tail", age, in_water);
        } else if id.starts_with("minecraft:pufferfish_small") {
            model.setup_pufferfish(["right_fin", "left_fin"], age);
        } else {
            model.setup_pufferfish(["right_blue_fin", "left_blue_fin"], age);
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `fish_textures`: the textures the fish wear, and a tropical fish's by its
/// packed variant.
pub(super) fn fish_textures_result(request: &Value) -> Result<Value, String> {
    use voxel_world::mobs::fish::{
        COD_TEXTURE, PUFFERFISH_TEXTURE, SALMON_TEXTURE, TropicalPattern, tropical_base_dye,
        tropical_pattern_dye,
    };
    let path = |texture: &str| format!("minecraft:textures/{texture}.png");
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        if row.get("packed_id").is_none() {
            let texture = match string(row, "label")? {
                "cod" => COD_TEXTURE,
                "salmon" => SALMON_TEXTURE,
                _ => PUFFERFISH_TEXTURE,
            };
            out.push(json!({"label": row["label"], "texture": path(texture)}));
            continue;
        }
        let packed = i32_field(row, "packed_id")?;
        let pattern = TropicalPattern::from_packed(packed);
        out.push(json!({
            "label": row["label"],
            "packed_id": packed,
            "texture": path(pattern.base_texture()),
            "pattern_texture": path(pattern.pattern_texture()),
            "base_dye": tropical_base_dye(packed),
            "pattern_dye": tropical_pattern_dye(packed),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `shulker_model`: a shulker's model posed and drawn.
pub(super) fn shulker_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, ShulkerRenderState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let mut model = MobModel::bake(layer);
        model.setup_shulker(&ShulkerRenderState {
            peek_amount: bits("peek_amount")?,
            age_in_ticks: bits("age_in_ticks")?,
            x_rot: bits("x_rot")?,
            y_head_rot: bits("y_head_rot")?,
            y_body_rot: bits("y_body_rot")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `magma_cube_model`: a magma cube's model posed and drawn.
pub(super) fn magma_cube_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let squish = u32::from_str_radix(row["squish"].as_str().ok_or("squish")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.setup_magma_cube(squish);
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `glow_squid_light`: the block light a glow squid is drawn with.
pub(super) fn glow_squid_light_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let light = voxel_render::entity::mob_model::glow_squid_block_light(
            i32_field(row, "dark_ticks")?,
            i32_field(row, "block_light")?,
        );
        out.push(json!({"label": row["label"], "light": light}));
    }
    Ok(json!({"rows": out}))
}

/// `copper_golem_model`: a copper golem's model posed and drawn.
pub(super) fn copper_golem_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{CopperGolemModelState, MobModel, QuadrupedState};
    use voxel_world::animation_state::AnimationState;
    use voxel_world::mobs::copper_golem::CopperGolemPlaying;
    let layer = voxel_data::entity_model::layer("minecraft:copper_golem#main").ok_or("no layer")?;
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
        let started = |key: &str| {
            let mut state = AnimationState::default();
            if let Some(tick) = row[key].as_i64() {
                state.start(tick as i32);
            }
            state
        };
        let mut model = MobModel::bake(layer);
        model.setup_copper_golem(&CopperGolemModelState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            holds_item: row["holds_item"].as_bool().ok_or("holds_item")?,
            animations: CopperGolemPlaying {
                idle: started("idle"),
                get_item: started("get_item"),
                get_no_item: started("get_no_item"),
                drop_item: started("drop_item"),
                drop_no_item: started("drop_no_item"),
            },
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `copper_golem_layers`: where the layers a copper golem carries things with put them, on the
/// model as posed.
pub(super) fn copper_golem_layers_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::held_item::{
        copper_golem_antenna_pose, copper_golem_hand_pose, copper_golem_head_pose,
    };
    use voxel_render::entity::mob_model::{CopperGolemModelState, MobModel, QuadrupedState};
    use voxel_render::entity::player_model::Arm;
    use voxel_render::joml::Pose;
    let layer = voxel_data::entity_model::layer("minecraft:copper_golem#main").ok_or("no layer")?;
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
        model.setup_copper_golem(&CopperGolemModelState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            // The layers hold something in both hands: the walk with arms raised.
            holds_item: true,
            animations: Default::default(),
        });
        let root = model.root();
        fn part<'a>(
            name: &str,
            of: &'a voxel_render::entity::player_model::Part,
        ) -> Result<&'a voxel_render::entity::player_model::Part, String> {
            of.children()
                .iter()
                .find(|part| part.name() == name)
                .ok_or(format!("no {name}"))
        }
        let body = part("body", root)?;
        let (head, right, left) = (
            part("head", body)?,
            part("right_arm", body)?,
            part("left_arm", body)?,
        );
        let base = Pose::default();
        let idle = row["action"].as_str().ok_or("action")? == "IDLE";
        // A stack's own display transform is none: its model is moved to its corner.
        let corner = |mut pose: Pose| {
            pose.translate(-0.5, -0.5, -0.5);
            matrix_bits(&pose)
        };
        let poses: Vec<Vec<String>> = match row["layer"].as_str().ok_or("layer")? {
            "hands" => [(Arm::Right, right), (Arm::Left, left)]
                .into_iter()
                .map(|(side, arm)| {
                    corner(copper_golem_hand_pose(&base, (root, body, arm), side, idle))
                })
                .collect(),
            "head" => vec![corner(copper_golem_head_pose(&base, (root, body, head)))],
            _ => vec![matrix_bits(&copper_golem_antenna_pose(
                &base,
                (root, body, head),
            ))],
        };
        out.push(json!({"label": row["label"], "poses": poses}));
    }
    Ok(json!({"rows": out}))
}

/// `animation_definitions`: every keyframe animation definition, from the
/// generated table.
pub(super) fn animation_definitions_result() -> Value {
    use voxel_data::animation::{Interpolation, Target};
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let vector =
        |values: [f32; 3]| -> Vec<String> { values.iter().map(|value| hex(*value)).collect() };
    let rows: Vec<Value> = voxel_data::animation::definitions()
        .iter()
        .map(|definition| {
            let bones: Vec<Value> = definition
                .bones
                .iter()
                .map(|bone| {
                    let channels: Vec<Value> = bone
                        .channels
                        .iter()
                        .map(|channel| {
                            let frames: Vec<Value> = channel
                                .keyframes
                                .iter()
                                .map(|frame| {
                                    json!({
                                        "time": hex(frame.time),
                                        "pre": vector(frame.pre),
                                        "post": vector(frame.post),
                                        "interpolation": match frame.interpolation {
                                            Interpolation::Linear => "LINEAR",
                                            Interpolation::CatmullRom => "CATMULLROM",
                                        },
                                    })
                                })
                                .collect();
                            json!({
                                "target": match channel.target {
                                    Target::Position => "POSITION",
                                    Target::Rotation => "ROTATION",
                                    Target::Scale => "SCALE",
                                },
                                "keyframes": frames,
                            })
                        })
                        .collect();
                    json!({"name": bone.name, "channels": channels})
                })
                .collect();
            json!({
                "label": definition.name,
                "length": hex(definition.length),
                "looping": definition.looping,
                "bones": bones,
            })
        })
        .collect();
    json!({"rows": rows})
}

/// `keyframe_animation`: a rabbit model posed by a definition at a time, drawn.
pub(super) fn keyframe_animation_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = voxel_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let definition =
            voxel_data::animation::definition(row["definition"].as_str().ok_or("definition")?)
                .ok_or("no such definition")?;
        let scale = u32::from_str_radix(row["scale"].as_str().ok_or("scale")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.animate(definition, row["millis"].as_i64().ok_or("millis")?, scale);
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `animation_states`: what call sequences make of the time an animation has run.
pub(super) fn animation_states_result(request: &Value) -> Result<Value, String> {
    use voxel_world::animation_state::AnimationState;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut state = AnimationState::default();
        let mut steps = Vec::new();
        for call in row["steps"].as_array().ok_or("steps")? {
            let bits = |key: &str| -> Result<f32, String> {
                u32::from_str_radix(
                    call[key]
                        .as_str()
                        .ok_or(format!("{key} must be a string"))?,
                    16,
                )
                .map(f32::from_bits)
                .map_err(|error| error.to_string())
            };
            let tick = || i32_field(call, "tick");
            match call["call"].as_str().ok_or("call")? {
                "start" => state.start(tick()?),
                "start_if_stopped" => state.start_if_stopped(tick()?),
                "animate_when" => {
                    state.animate_when(call["condition"].as_bool().ok_or("condition")?, tick()?);
                }
                "stop" => state.stop(),
                "fast_forward" => state.fast_forward(i32_field(call, "ticks")?, bits("scale")?),
                other => return Err(format!("unknown call {other}")),
            }
            steps.push(json!({
                "started": state.is_started(),
                "millis": state.time_in_millis(bits("age")?),
            }));
        }
        out.push(json!({"label": row["label"], "steps": steps}));
    }
    Ok(json!({"rows": out}))
}

/// `entity_attributes`: the attributes each living entity type starts with and
/// each attribute's range, from the generated table.
pub(super) fn entity_attributes_result() -> Value {
    let hex = |value: f64| format!("{:016x}", value.to_bits());
    let mut rows = Vec::new();
    for (name, attributes) in voxel_data::entity_type::default_attributes() {
        let list: Vec<Value> = attributes
            .iter()
            .map(|(attribute, base)| json!({"name": attribute, "base": hex(*base)}))
            .collect();
        rows.push(json!({
            "label": name,
            "kind": "type",
            "id": voxel_data::entity_type::id(name),
            "attributes": list,
        }));
    }
    for (id, attribute) in voxel_data::entity_type::attribute_ranges()
        .iter()
        .enumerate()
    {
        let mut row = json!({
            "label": attribute.name,
            "kind": "range",
            "id": id,
            "default": hex(attribute.default),
        });
        if let Some((low, high)) = attribute.range {
            row["min"] = json!(hex(low));
            row["max"] = json!(hex(high));
        }
        rows.push(row);
    }
    json!({"rows": rows})
}
