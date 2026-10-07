use super::*;

/// `shaking_rotations`: what a cow's, a hoglin's or a strider's renderer leaves the pose at, with
/// whether it shakes decided by the client world from the entity's data and its dimension.
pub(super) fn shaking_rotations_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobPlacement, placement_rotations};
    use voxel_world::{DimensionEnvironment, DimensionInput};
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
        let flag = |key: &str| row[key].as_bool().ok_or(format!("{key} must be a bool"));
        let dimension = string(row, "dimension")?;
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        world.replace_dimension_with_environment(
            DimensionInput {
                key: dimension,
                dimension_type_id: 0,
                seed: 0,
                is_debug: false,
                is_flat: false,
                sea_level: 63,
            },
            DimensionBounds::new(0, 16),
            DimensionEnvironment::for_builtin_dimension(dimension),
        );
        let blocks = vec![0; 4_096];
        let biomes = vec![0; 64];
        if !world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        ) {
            return Err("shaking fixture chunk was rejected".into());
        }
        let type_name = match string(row, "kind")? {
            "hoglin" => "minecraft:hoglin",
            "strider" => "minecraft:strider",
            "cow" => "minecraft:cow",
            _ => return Err("unknown kind".into()),
        };
        let entity_id = 7;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, 7),
            type_id: voxel_data::entity_type::id(type_name).ok_or("no such entity type")?,
            x: 1.25,
            y: 8.0,
            z: 1.25,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }) {
            return Err("shaking fixture entity was rejected".into());
        }
        for _ in 0..i64_field(row, "ticks")? {
            world.tick();
        }
        // The data the server sends: the frozen ticks (an int), the hoglin's
        // immunity, the strider's suffocation and the mob's no-AI flag.
        let mut data = vec![7, 1];
        voxel_core::wire::write_varint(i32_field(row, "ticks_frozen")?, &mut data);
        data.extend([15, 0, u8::from(flag("no_ai")?)]);
        data.extend([
            voxel_world::HOGLIN_IMMUNE_DATA_ID,
            8,
            u8::from(flag("immune")?),
        ]);
        data.extend([
            voxel_world::STRIDER_SUFFOCATING_DATA_ID,
            8,
            u8::from(flag("suffocating")?),
        ]);
        data.push(255);
        world.apply_entity_metadata(entity_id, &data);
        let snapshot = world
            .entity_snapshot(entity_id, bits("partial")?)
            .ok_or("shaking fixture entity disappeared")?;
        let placed = MobPlacement {
            body_rot: bits("body_rot")?,
            shaking: snapshot.shaking.then_some(snapshot.age_in_ticks),
            ..MobPlacement::default()
        };
        out.push(json!({
            "label": row["label"],
            "pose": matrix_bits(&placement_rotations(&placed)),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `fox_textures`: the textures a fox wears by variant, state and age.
pub(super) fn fox_textures_result() -> Value {
    let mut rows = Vec::new();
    for variant in [0, 1] {
        for sleeping in [false, true] {
            for baby in [false, true] {
                rows.push(json!({
                    "label": format!(
                        "{variant}-{}-{}",
                        if sleeping { "sleeping" } else { "idle" },
                        if baby { "baby" } else { "adult" }
                    ),
                    "texture": format!(
                        "minecraft:textures/{}.png",
                        voxel_data::entity_type::fox_texture(variant, sleeping, baby)
                    ),
                }));
            }
        }
    }
    json!({"rows": rows})
}

/// `fox_model`: a fox's model posed and drawn, and the matrix its held item is carried with.
pub(super) fn fox_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::held_item::fox_item_pose;
    use voxel_render::joml::Pose;
    use voxel_render::mob_model::{FoxState, MobModel};
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
        let flag = |key: &str| row[key].as_bool().ok_or(format!("{key} must be a bool"));
        let baby = id.contains("_baby");
        let quadruped = quadruped_state(row)?;
        let state = FoxState {
            quadruped,
            head_roll_angle: bits("head_roll_angle")?,
            crouch_amount: bits("crouch_amount")?,
            crouching: flag("crouching")?,
            sleeping: flag("sleeping")?,
            sitting: flag("sitting")?,
            faceplanted: flag("faceplanted")?,
            pouncing: flag("pouncing")?,
            age_in_ticks: bits("age_in_ticks")?,
            age_scale: bits("age_scale")?,
            baby,
        };
        let mut model = MobModel::bake(layer);
        model.setup_fox(&state);
        let head = model
            .root()
            .children()
            .iter()
            .find(|part| part.name() == "head")
            .ok_or("no head")?;
        let mut pose = fox_item_pose(
            &Pose::default(),
            head,
            (baby, state.sleeping),
            (state.head_roll_angle, quadruped.head_y_rot, quadruped.x_rot),
        );
        // The stack's own display transform is none: the model is moved to its corner.
        pose.translate(-0.5, -0.5, -0.5);
        let matrix: Vec<String> = (0..4)
            .flat_map(|column| (0..4).map(move |row| (column, row)))
            .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
            .collect();
        out.push(json!({"label": row["label"], "quads": posed_quads(&model), "poses": [matrix]}));
    }
    Ok(json!({"rows": out}))
}

/// `hoglin_model`: a hoglin's or a zoglin's model posed and drawn.
pub(super) fn hoglin_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{HoglinState, MobModel};
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
        model.setup_hoglin(&HoglinState {
            head_y_rot: bits("head_y_rot")?,
            walk_position: bits("walk_position")?,
            walk_speed: bits("walk_speed")?,
            attack_ticks: row["attack_ticks"].as_i64().ok_or("attack_ticks")? as i32,
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `polar_bear_model`: a polar bear's model posed and drawn.
pub(super) fn polar_bear_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, PolarBearState};
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
        model.setup_polar_bear(&PolarBearState {
            quadruped: quadruped_state(row)?,
            stand_scale: bits("stand_scale")?,
            age_scale: bits("age_scale")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `feline_model`: a cat's or an ocelot's model posed and drawn.
pub(super) fn feline_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{FelineState, MobModel};
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
        let flag = |key: &str| row[key].as_bool().ok_or(format!("{key} must be a bool"));
        let mut model = MobModel::bake(layer);
        model.setup_feline(&FelineState {
            quadruped: quadruped_state(row)?,
            crouching: flag("crouching")?,
            sprinting: flag("sprinting")?,
            sitting: flag("sitting")?,
            baby: id.contains("_baby"),
            lie_down_amount: bits("lie_down_amount")?,
            lie_down_amount_tail: bits("lie_down_amount_tail")?,
            relax_state_one_amount: bits("relax_state_one_amount")?,
            age_scale: bits("age_scale")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `llama_model`: a llama's model posed and drawn.
pub(super) fn llama_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = voxel_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_llama(
            &quadruped_state(row)?,
            row["has_chest"].as_bool().ok_or("has_chest")?,
        );
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `camel_model`: a camel's model posed with its animation states, drawn.
pub(super) fn camel_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{CamelState, MobModel};
    use voxel_world::animation_state::AnimationState;
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
        let started = |key: &str| {
            let mut state = AnimationState::default();
            if let Some(tick) = row[key].as_i64() {
                state.start(tick as i32);
            }
            state
        };
        let mut model = MobModel::bake(layer);
        model.setup_camel(&CamelState {
            quadruped: quadruped_state(row)?,
            age_in_ticks: bits("age_in_ticks")?,
            jump_cooldown: bits("jump_cooldown")?,
            sit: started("sit"),
            sit_pose: started("sit_pose"),
            sit_up: started("sit_up"),
            idle: started("idle"),
            dash: started("dash"),
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `equine_model`: a horse, donkey or mule model posed and drawn.
pub(super) fn equine_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{EquineKind, EquineState, MobModel};
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
        let baby = id.contains("_baby");
        let donkey = id.starts_with("minecraft:donkey") || id.starts_with("minecraft:mule");
        let kind = match (donkey, baby) {
            (false, false) => EquineKind::Horse,
            (false, true) => EquineKind::BabyHorse,
            (true, false) => EquineKind::Donkey,
            (true, true) => EquineKind::BabyDonkey,
        };
        let mut model = MobModel::bake(layer);
        model.setup_equine(&EquineState {
            kind,
            quadruped: quadruped_state(row)?,
            age_in_ticks: bits("age_in_ticks")?,
            eat_animation: bits("eat")?,
            stand_animation: bits("stand")?,
            feeding_animation: bits("feeding")?,
            animate_tail: row["animate_tail"].as_bool().ok_or("animate_tail")?,
            in_water: row["in_water"].as_bool().ok_or("in_water")?,
            age_scale: if baby { 0.5 } else { 1.0 },
            has_chest: row["has_chest"].as_bool().ok_or("has_chest")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `swirl_layers`: what the charged creeper's swirl layer submits.
pub(super) fn swirl_layers_result(request: &Value) -> Result<Value, String> {
    use voxel_render::render_types::{
        ENERGY_SWIRL, EYES_ORDER, NO_OVERLAY, POWER_COLOUR, POWER_TEXTURE, swirl_offset,
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        if !row["powered"].as_bool().ok_or("powered")? {
            out.push(json!({"label": row["label"], "submitted": false}));
            continue;
        }
        let age = u32::from_str_radix(row["age_in_ticks"].as_str().ok_or("age")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let [u, v] = swirl_offset(age);
        out.push(json!({
            "label": row["label"],
            "order": EYES_ORDER,
            "light": 15_728_880,
            "overlay": NO_OVERLAY,
            "colour": POWER_COLOUR,
            "texture": format!("textures/{POWER_TEXTURE}.png"),
            "pipeline": ENERGY_SWIRL.location,
            "flags": ENERGY_SWIRL.flags,
            "values": ENERGY_SWIRL.values,
            "blend": ENERGY_SWIRL.blend,
            "depth_test": ENERGY_SWIRL.depth_test,
            "depth_write": ENERGY_SWIRL.depth_write,
            "cull": ENERGY_SWIRL.cull,
            "u_offset": format!("{:08x}", u.to_bits()),
            "v_offset": format!("{:08x}", v.to_bits()),
            "submitted": true,
        }));
    }
    Ok(json!({"rows": out}))
}

/// `eyes_layers`: what the glowing-eyes layers submit: the draw order, the
/// texture, the pipeline's state, the light passed and the overlay.
pub(super) fn eyes_layers_result(request: &Value) -> Result<Value, String> {
    use voxel_render::render_types::{EYES, EYES_ORDER, NO_OVERLAY, eyes_texture};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let entity = match row["layer"].as_str().ok_or("layer")? {
            "spider" => "minecraft:spider",
            _ => "minecraft:enderman",
        };
        let texture = eyes_texture(entity).ok_or("no eyes")?;
        out.push(json!({
            "label": row["label"],
            "order": EYES_ORDER,
            // The layer hands its light on unchanged; the pipeline ignores it.
            "light": 15_728_880,
            "overlay": NO_OVERLAY,
            "texture": format!("textures/{texture}.png"),
            "pipeline": EYES.location,
            "flags": EYES.flags,
            "values": EYES.values,
            "blend": EYES.blend,
            "depth_test": EYES.depth_test,
            "depth_write": EYES.depth_write,
            "cull": EYES.cull,
        }));
    }
    Ok(json!({"rows": out}))
}
