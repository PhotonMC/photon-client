use super::*;

/// `villager_model`: a villager's or a wandering trader's model posed and
/// drawn.
pub(super) fn villager_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, QuadrupedState, VillagerState};
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
        let mut model = MobModel::bake(layer);
        model.setup_villager(&VillagerState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            unhappy: row["unhappy"].as_bool().ok_or("unhappy")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `illager_model`: an illager's model posed for an arm pose and drawn.
pub(super) fn illager_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{IllagerArmPose, IllagerState, MobModel, QuadrupedState};
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
        let mut model = MobModel::bake(layer);
        model.setup_illager(&IllagerState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            riding: row["riding"].as_bool().ok_or("riding")?,
            arm_pose: IllagerArmPose::named(row["arm_pose"].as_str().ok_or("arm_pose")?)
                .ok_or("no such arm pose")?,
            main_arm_left: row["main_arm"].as_str() == Some("left"),
            attack_time: bits("attack_time")?,
            has_item: row["has_item"].as_bool().ok_or("has_item")?,
            max_crossbow_charge: row["max_crossbow_charge"].as_i64().ok_or("charge")? as i32,
            ticks_using_item: bits("ticks_using_item")?,
            hat_visible: row["hat"].as_bool().ok_or("hat")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// The look and stride fields the golems' rows name.
pub(super) fn quadruped_state(
    row: &Value,
) -> Result<voxel_render::mob_model::QuadrupedState, String> {
    let bits = |key: &str| -> Result<f32, String> {
        u32::from_str_radix(
            row[key].as_str().ok_or(format!("{key} must be a string"))?,
            16,
        )
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
    };
    Ok(voxel_render::mob_model::QuadrupedState {
        head_y_rot: bits("head_y_rot")?,
        x_rot: bits("x_rot")?,
        walk_position: bits("walk_position")?,
        walk_speed: bits("walk_speed")?,
    })
}

/// `iron_golem_model`: the iron golem's model posed and drawn.
pub(super) fn iron_golem_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{IronGolemState, MobModel};
    let layer = voxel_data::entity_model::layer("minecraft:iron_golem#main")
        .ok_or("no iron golem layer")?;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let attack = u32::from_str_radix(row["attack_ticks"].as_str().ok_or("attack_ticks")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.setup_iron_golem(&IronGolemState {
            quadruped: quadruped_state(row)?,
            attack_ticks_remaining: attack,
            offer_flower_tick: row["offer_flower_tick"].as_i64().ok_or("offer")? as i32,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `snow_golem_model`: the snow golem's model posed and drawn.
pub(super) fn snow_golem_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::MobModel;
    let layer = voxel_data::entity_model::layer("minecraft:snow_golem#main")
        .ok_or("no snow golem layer")?;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut model = MobModel::bake(layer);
        model.setup_snow_golem(&quadruped_state(row)?);
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `enderman_model`: the enderman's model posed and drawn.
pub(super) fn enderman_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{EndermanState, MobModel};
    let layer =
        voxel_data::entity_model::layer("minecraft:enderman#main").ok_or("no enderman layer")?;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut model = MobModel::bake(layer);
        model.setup_enderman(&EndermanState {
            humanoid: humanoid_state(row)?,
            carrying: row["carrying"].as_bool().ok_or("carrying")?,
            creepy: row["creepy"].as_bool().ok_or("creepy")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `iron_golem_render`: the sway the iron golem's renderer adds to the pose
/// while it walks, as the pose's matrix, and the crackiness its health shows.
pub(super) fn iron_golem_render_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::joml::Quaternion;
    use voxel_render::mob_model::{Crackiness, golem_crackiness, iron_golem_roll};
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
        let mut pose = Pose::default();
        pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
            180.0 - bits("body_rot")?,
        )));
        if let Some(roll) = iron_golem_roll(bits("walk_position")?, bits("walk_speed")?) {
            pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
                roll,
            )));
        }
        let matrix: Vec<String> = (0..4)
            .flat_map(|column| (0..4).map(move |row| (column, row)))
            .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
            .collect();
        let crackiness = match golem_crackiness(bits("fraction")?) {
            Crackiness::None => "NONE",
            Crackiness::Low => "LOW",
            Crackiness::Medium => "MEDIUM",
            Crackiness::High => "HIGH",
        };
        out.push(json!({"label": row["label"], "pose": matrix, "crackiness": crackiness}));
    }
    Ok(json!({"rows": out}))
}

/// `entity_block_layers`: the pose each layer that draws a block on an entity
/// puts it at, from an identity entity pose.
pub(super) fn entity_block_layers_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity_block::{
        carried_block_pose, iron_golem_flower_pose, mooshroom_mushroom_poses, snow_golem_head_pose,
    };
    use voxel_render::joml::Pose;
    use voxel_render::mob_model::{IronGolemState, MobModel};
    let bake = |name: &str| -> Result<MobModel, String> {
        Ok(MobModel::bake(
            voxel_data::entity_model::layer(name).ok_or("no such layer")?,
        ))
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let look = quadruped_state(row)?;
        let entity = Pose::default();
        let part = |model: &MobModel, name: &str| {
            model
                .root()
                .children()
                .iter()
                .find(|part| part.name() == name)
                .cloned()
                .ok_or(format!("no {name}"))
        };
        let poses = match row["layer"].as_str().ok_or("layer")? {
            "carried" => vec![carried_block_pose(&entity)],
            "mushrooms" => {
                let mut model = bake("minecraft:mooshroom#main")?;
                model.setup_quadruped(&look);
                mooshroom_mushroom_poses(&entity, &part(&model, "head")?).to_vec()
            }
            "snow_head" => {
                let mut model = bake("minecraft:snow_golem#main")?;
                model.setup_snow_golem(&look);
                vec![snow_golem_head_pose(&entity, &part(&model, "head")?)]
            }
            _ => {
                let attack =
                    u32::from_str_radix(row["attack_ticks"].as_str().ok_or("attack_ticks")?, 16)
                        .map(f32::from_bits)
                        .map_err(|error| error.to_string())?;
                let mut model = bake("minecraft:iron_golem#main")?;
                model.setup_iron_golem(&IronGolemState {
                    quadruped: look,
                    attack_ticks_remaining: attack,
                    offer_flower_tick: row["offer_flower_tick"].as_i64().ok_or("offer")? as i32,
                });
                vec![iron_golem_flower_pose(&entity, &part(&model, "right_arm")?)]
            }
        };
        let matrices: Vec<Vec<String>> = poses
            .iter()
            .map(|pose| {
                (0..4)
                    .flat_map(|column| (0..4).map(move |row| (column, row)))
                    .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
                    .collect()
            })
            .collect();
        out.push(json!({
            "label": row["label"],
            "drawn": true,
            "pose": matrices.last(),
            "poses": matrices,
        }));
    }
    Ok(json!({"rows": out}))
}

/// `segmented_model`: the silverfish's or the endermite's model posed and drawn.
pub(super) fn segmented_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let age = u32::from_str_radix(row["age_in_ticks"].as_str().ok_or("age")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.setup_segmented(age, id.starts_with("minecraft:silverfish"));
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `bee_model`: a bee's model posed and drawn.
pub(super) fn bee_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{BeeState, MobModel};
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
        model.setup_bee(&BeeState {
            age_in_ticks: bits("age_in_ticks")?,
            roll_amount: bits("roll_amount")?,
            has_stinger: row["has_stinger"].as_bool().ok_or("has_stinger")?,
            on_ground: row["on_ground"].as_bool().ok_or("on_ground")?,
            angry: row["angry"].as_bool().ok_or("angry")?,
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `rabbit_model`: a rabbit's model posed with its animation states, drawn.
pub(super) fn rabbit_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, QuadrupedState, RabbitState};
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
        model.setup_rabbit(&RabbitState {
            quadruped: QuadrupedState {
                head_y_rot: bits("head_y_rot")?,
                x_rot: bits("x_rot")?,
                ..Default::default()
            },
            age_in_ticks: bits("age_in_ticks")?,
            hop: started("hop_start"),
            idle_head_tilt: started("idle_start"),
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `squid_model`: a squid's model posed and drawn.
pub(super) fn squid_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let angle = u32::from_str_radix(row["tentacle_angle"].as_str().ok_or("angle")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.setup_squid(angle);
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `sea_animal_model`: a dolphin's or a turtle's model posed and drawn.
pub(super) fn sea_animal_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{DolphinState, MobModel, QuadrupedState, TurtleState};
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
        if id.starts_with("minecraft:dolphin") {
            model.setup_dolphin(&DolphinState {
                y_rot: bits("y_rot")?,
                x_rot: bits("x_rot")?,
                age_in_ticks: bits("age_in_ticks")?,
                is_moving: flag("is_moving")?,
            });
        } else {
            model.setup_turtle(&TurtleState {
                quadruped: QuadrupedState {
                    head_y_rot: bits("head_y_rot")?,
                    x_rot: bits("x_rot")?,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                is_on_land: flag("on_land")?,
                is_laying_egg: flag("laying_egg")?,
                has_egg: flag("has_egg")?,
            });
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}
