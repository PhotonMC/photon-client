use super::*;

/// What the world keeps of a vehicle the player rides, after a tick.
pub(super) fn vehicle_report(world: &ClientWorld, vehicle_id: i32) -> Result<Value, String> {
    use voxel_world::Status;
    let vehicle = world
        .entities()
        .all()
        .find(|entity| entity.id() == vehicle_id)
        .ok_or("the vehicle is gone")?;
    let (x, y, z) = vehicle.position();
    let (mx, my, mz) = vehicle.motion();
    let double = |value: f64| format!("{:016x}", value.to_bits());
    let single = |value: f32| format!("{:08x}", value.to_bits());
    let mut report = json!({
        "x": double(x), "y": double(y), "z": double(z),
        "motion_x": double(mx), "motion_y": double(my), "motion_z": double(mz),
        "y_rot": single(vehicle.rotation().0),
        "x_rot": single(vehicle.rotation().1),
        "on_ground": vehicle.on_ground(),
    });
    if vehicle.is_boat() {
        let ride = vehicle.boat_ride();
        let (paddle_left, paddle_right) = vehicle.paddle_state();
        report["horizontal_collision"] = json!(ride.horizontal_collision);
        report["vertical_collision"] = json!(ride.vertical_collision);
        report["fall_distance"] = json!(double(ride.fall_distance));
        report["status"] = json!(match ride.status {
            Some(Status::InWater) => "IN_WATER",
            Some(Status::UnderWater) => "UNDER_WATER",
            Some(Status::UnderFlowingWater) => "UNDER_FLOWING_WATER",
            Some(Status::OnLand) => "ON_LAND",
            Some(Status::InAir) => "IN_AIR",
            None => "null",
        });
        report["delta_rotation"] = json!(single(vehicle.boat_delta_rotation()));
        report["paddle_left"] = json!(paddle_left && !vehicle.passenger_ids().is_empty());
        report["paddle_right"] = json!(paddle_right && !vehicle.passenger_ids().is_empty());
        report["water_level"] = json!(double(ride.water_level));
        report["land_friction"] = json!(single(ride.land_friction));
        report["last_yd"] = json!(double(ride.last_yd));
    } else {
        let ride = vehicle.mount_ride();
        report["horizontal_collision"] = json!(ride.horizontal_collision);
        report["vertical_collision"] = json!(ride.vertical_collision);
        report["fall_distance"] = json!(double(ride.fall_distance));
        if vehicle.is_horse() {
            if vehicle.is_camel() {
                let camel = vehicle.horse().map(|horse| &horse.camel);
                report["dash_cooldown"] = json!(camel.map_or(0, |camel| camel.dash_cooldown()));
                report["camel_sitting"] = json!(camel.is_some_and(|camel| camel.sitting()));
                report["dashing"] = json!(camel.is_some_and(|camel| camel.is_dashing()));
            }
            report["pending_jump"] = json!(single(ride.jump_pending_scale));
            report["y_head_rot"] = json!(single(vehicle.rotation().2));
        }
    }
    Ok(report)
}

/// `experience_orb`: the quad an orb is drawn as.
pub(super) fn experience_orb_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::name_tag::camera_orientation;
    use voxel_render::objects::xp_orb::corners;
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let orientation = camera_orientation(bits(row, "yaw")?, bits(row, "pitch")?);
        let quad = corners(
            i32_field(row, "icon")?,
            bits(row, "age")?,
            [bits(row, "px")?, bits(row, "py")?, bits(row, "pz")?],
            &orientation,
        );
        let light = i32_field(row, "light")?;
        let vertices: Vec<Value> = quad
            .iter()
            .map(|corner| {
                json!({
                    "x": hex(corner.position[0]),
                    "y": hex(corner.position[1]),
                    "z": hex(corner.position[2]),
                    "color": corner.colour.map(|c| c.to_string()).join(","),
                    "u": hex(corner.uv[0]),
                    "v": hex(corner.uv[1]),
                    // `OverlayTexture.NO_OVERLAY`, and the light as given.
                    "overlay": 10 << 16,
                    "light": light,
                    "normal": corner.normal.map(hex).join(""),
                })
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `living_pose`: the pose a mob is drawn from.
pub(super) fn living_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{FLIP_DEGREES, MobPlacement, placement_pose};
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let pose = placement_pose(&MobPlacement {
            feet: [0.0; 3],
            body_rot: bits(row, "body_rot")?,
            scale: bits(row, "scale")?,
            light: 0,
            death_time: bits(row, "death_time")?,
            flip_degrees: if row.get("flip_degrees").is_some() {
                bits(row, "flip_degrees")?
            } else {
                FLIP_DEGREES
            },
            ..MobPlacement::default()
        });
        let matrix: Vec<String> = (0..4)
            .flat_map(|column| (0..4).map(move |row| (column, row)))
            .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
            .collect();
        out.push(json!({"label": row["label"], "pose": matrix}));
    }
    Ok(json!({"rows": out}))
}

/// A dropped item's poses for a model of a given extent.
pub(super) fn item_entity_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::objects::item::{ModelExtent, item_entity_poses};
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        // The bounding box is built of doubles from the extents' points as
        // the layer's transform moves them (half a block back, for a model
        // with no transform): the lowest point and the size along z.
        let moved = |key: &str| bits(row, key).map(|value| f64::from(value + -0.5_f32));
        let extent = ModelExtent {
            min_y: moved("min_y")?,
            depth: moved("max_z")? - moved("min_z")?,
        };
        let poses = item_entity_poses(
            Default::default(),
            bits(row, "age")?,
            bits(row, "bob")?,
            row["count"].as_i64().ok_or("count")? as i32,
            row["seed"].as_i64().ok_or("seed")? as i32,
            extent,
        );
        let described: Vec<Vec<String>> = poses
            .into_iter()
            .map(|mut pose| {
                // `ItemTransform.NO_TRANSFORM.apply` of the layer.
                pose.translate(-0.5, -0.5, -0.5);
                // Column by column, as `Matrix4f.get` writes them.
                (0..4)
                    .flat_map(|column| (0..4).map(move |row| (column, row)))
                    .map(|(column, row)| hex(pose.pose.m[column][row]))
                    .collect()
            })
            .collect();
        out.push(json!({"label": row["label"], "poses": described}));
    }
    Ok(json!({"rows": out}))
}

/// A quadruped's model from the layer table, posed and drawn.
pub(super) fn quadruped_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, QuadrupedState};
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
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_quadruped(&QuadrupedState {
            head_y_rot: bits(row, "head_y_rot")?,
            x_rot: bits(row, "x_rot")?,
            walk_position: bits(row, "walk_position")?,
            walk_speed: bits(row, "walk_speed")?,
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

/// The sorted quads of a posed model, as the oracle lists them.
pub(super) fn posed_quads(model: &voxel_render::entity::mob_model::MobModel) -> Vec<String> {
    sorted_quads(model.render(&Default::default()))
}

/// A model's faces as the oracle lists them: each quad's corners as bits,
/// sorted.
pub(super) fn sorted_quads(
    faces: Vec<(String, [voxel_render::entity::player_model::ModelVertex; 4])>,
) -> Vec<String> {
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let mut quads: Vec<String> = faces
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
    quads
}

/// The humanoid fields every humanoid operation's row names.
pub(super) fn humanoid_state(
    row: &Value,
) -> Result<voxel_render::entity::player_model::PlayerRenderState, String> {
    let bits = |key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    Ok(voxel_render::entity::player_model::PlayerRenderState {
        head_y_rot: bits("head_y_rot")?,
        x_rot: bits("x_rot")?,
        walk_position: bits("walk_position")?,
        walk_speed: bits("walk_speed")?,
        age_in_ticks: bits("age_in_ticks")?,
        attack_time: bits("attack_time")?,
        swim_amount: bits("swim_amount")?,
        ..Default::default()
    })
}

/// `armor_stand_model`: an armor stand's model, or the model of the armor it
/// wears, posed and drawn.
pub(super) fn armor_stand_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{ArmorStandState, MobModel};
    let bits = |text: &Value| -> Result<f32, String> {
        u32::from_str_radix(text.as_str().ok_or("bits must be strings")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let mut state = ArmorStandState {
            show_arms: row["show_arms"].as_bool().ok_or("show_arms")?,
            show_base_plate: row["show_base_plate"].as_bool().ok_or("show_base_plate")?,
            y_rot: bits(&row["y_rot"])?,
            ..ArmorStandState::default()
        };
        for (index, key) in [
            "head_pose",
            "body_pose",
            "left_arm_pose",
            "right_arm_pose",
            "left_leg_pose",
            "right_leg_pose",
        ]
        .into_iter()
        .enumerate()
        {
            let angles = row[key].as_array().ok_or("a pose is three angles")?;
            for (axis, angle) in angles.iter().take(3).enumerate() {
                state.poses[index][axis] = bits(angle)?;
            }
        }
        let mut model = MobModel::bake(layer);
        model.setup_armor_stand(&state, id.ends_with("#main"));
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `equipment_layers`: every equipment asset of the game's resources, the
/// layers of each for the humanoid layer types.
pub(super) fn equipment_layers_result() -> Result<Value, String> {
    use voxel_render::entity::equipment::parse_equipment_info;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets/minecraft/equipment");
    let mut names: Vec<String> = std::fs::read_dir(&folder)
        .map_err(|error| error.to_string())?
        .filter_map(|entry| Some(entry.ok()?.file_name().to_str()?.to_owned()))
        .collect();
    names.sort();
    let mut rows = Vec::new();
    for name in names {
        let text =
            std::fs::read_to_string(folder.join(&name)).map_err(|error| error.to_string())?;
        let info = parse_equipment_info(&text).ok_or(format!("{name} did not parse"))?;
        let mut types = serde_json::Map::new();
        for layer_type in ["humanoid", "humanoid_leggings", "humanoid_baby"] {
            let layers: Vec<Value> = info
                .layers(layer_type)
                .iter()
                .map(|layer| {
                    json!({
                        "texture": format!("minecraft:textures/{}.png", layer.texture_path(layer_type)),
                        "undyed": layer.colour(0) as i32,
                        "dyed": layer.colour(0xFF33_6699) as i32,
                        "player_texture": layer.use_player_texture,
                    })
                })
                .collect();
            types.insert(layer_type.to_owned(), Value::Array(layers));
        }
        rows.push(json!({"label": name, "types": types}));
    }
    Ok(json!({"rows": rows}))
}

/// `spider_model` and `creeper_model`: a spider's or a creeper's model posed
/// and drawn.
pub(super) fn spider_model_result(request: &Value, creeper: bool) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, QuadrupedState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = voxel_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let state = QuadrupedState {
            head_y_rot: bits("head_y_rot")?,
            x_rot: bits("x_rot")?,
            walk_position: bits("walk_position")?,
            walk_speed: bits("walk_speed")?,
        };
        let mut model = MobModel::bake(layer);
        if creeper {
            model.setup_creeper(&state);
        } else {
            model.setup_spider(&state);
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `villager_registries`: the types and professions in wire order, each with
/// its hat.
pub(super) fn villager_registries_result() -> Value {
    use voxel_data::villager::{Hat, professions, types};
    let hat_name = |hat: Hat| match hat {
        Hat::None => "none",
        Hat::Partial => "partial",
        Hat::Full => "full",
    };
    let mut rows = Vec::new();
    for (kind, table) in [("type", types()), ("profession", professions())] {
        for (id, (name, hat)) in table.iter().enumerate() {
            rows.push(json!({
                "label": format!("{kind}:{name}"),
                "kind": kind,
                "id": id,
                "name": name,
                "hat": hat_name(*hat),
            }));
        }
    }
    json!({"rows": rows})
}

/// `villager_layers`: the passes `VillagerProfessionLayer` makes for each
/// villager: the model and the texture of each.
pub(super) fn villager_layers_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::villager::profession_passes;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let baby = row["baby"].as_bool().ok_or("baby")?;
        let passes: Vec<Value> = profession_passes(
            row["path"].as_str().ok_or("path")?,
            row["type"].as_str().ok_or("type")?,
            row["profession"].as_str().ok_or("profession")?,
            row["level"].as_i64().ok_or("level")? as i32,
            baby,
        )
        .into_iter()
        .map(|pass| {
            let model = match (pass.hatless, baby) {
                (false, false) => "main",
                (false, true) => "main_baby",
                (true, false) => "no_hat",
                (true, true) => "no_hat_baby",
            };
            json!({
                "model": model,
                "texture": format!("textures/{}.png", pass.texture),
                "color": -1,
            })
        })
        .collect();
        out.push(json!({"label": row["label"], "passes": passes}));
    }
    Ok(json!({"rows": out}))
}
