use super::*;

/// `villager_held_layers`: where the layers that hold something on a villager,
/// a trader and a witch put it, from an identity entity pose.
pub(super) fn villager_held_layers_result(request: &Value) -> Result<Value, String> {
    use voxel_render::held_item::{crossed_arms_item_pose, head_item_pose, witch_potion_pose};
    use voxel_render::joml::Pose;
    use voxel_render::mob_model::{MobModel, QuadrupedState, VillagerState, WitchState};
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
        let look = QuadrupedState {
            head_y_rot: bits("head_y_rot")?,
            x_rot: bits("x_rot")?,
            ..Default::default()
        };
        let kind = row["kind"].as_str().ok_or("kind")?;
        let baby = row["baby"].as_bool().ok_or("baby")?;
        let entity = Pose::default();
        let mut model = MobModel::bake(
            voxel_data::entity_model::layer(match (kind.starts_with("witch"), baby) {
                (true, _) => "minecraft:witch#main",
                (false, true) => "minecraft:villager_baby#main",
                (false, false) => "minecraft:villager#main",
            })
            .ok_or("no such layer")?,
        );
        let mut pose = if kind.starts_with("witch") {
            model.setup_witch(&WitchState {
                quadruped: look,
                age_in_ticks: bits("age_in_ticks")?,
                entity_id: row["entity_id"].as_i64().ok_or("entity_id")? as i32,
                holding_item: true,
            });
            let part = |from: &voxel_render::player_model::Part, name: &str| {
                from.children()
                    .iter()
                    .find(|part| part.name() == name)
                    .cloned()
                    .ok_or(format!("no {name}"))
            };
            if kind == "witch_potion" {
                let head = part(model.root(), "head")?;
                let nose = part(&head, "nose")?;
                witch_potion_pose(&entity, model.root(), &head, &nose)
            } else {
                crossed_arms_item_pose(&entity, model.root(), &part(model.root(), "arms")?)
            }
        } else {
            model.setup_villager(&VillagerState {
                quadruped: look,
                age_in_ticks: bits("age_in_ticks")?,
                unhappy: row["unhappy"].as_bool().ok_or("unhappy")?,
            });
            let part = |name: &str| {
                model
                    .root()
                    .children()
                    .iter()
                    .find(|part| part.name() == name)
                    .cloned()
                    .ok_or(format!("no {name}"))
            };
            if kind == "villager_head" {
                // The villager renderer's own head transforms, or the default.
                let transforms = if row["villager_transforms"].as_bool().ok_or("transforms")? {
                    (-0.117_187_5, 1.0)
                } else {
                    (0.0, 1.0)
                };
                head_item_pose(&entity, model.root(), &part("head")?, transforms)
            } else {
                crossed_arms_item_pose(&entity, model.root(), &part("arms")?)
            }
        };
        // The stack's own display transform is none: the model is moved to its corner.
        pose.translate(-0.5, -0.5, -0.5);
        let matrix: Vec<String> = (0..4)
            .flat_map(|column| (0..4).map(move |row| (column, row)))
            .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
            .collect();
        out.push(json!({"label": row["label"], "poses": [matrix]}));
    }
    Ok(json!({"rows": out}))
}

/// `goat_model`: the goat's model posed and drawn.
pub(super) fn goat_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{GoatState, MobModel};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let ramming = u32::from_str_radix(row["ramming"].as_str().ok_or("ramming")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.setup_goat(&GoatState {
            quadruped: quadruped_state(row)?,
            has_left_horn: row["left_horn"].as_bool().ok_or("left_horn")?,
            has_right_horn: row["right_horn"].as_bool().ok_or("right_horn")?,
            ramming_x_head_rot: ramming,
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `wolf_model`: a wolf's model posed and drawn.
pub(super) fn wolf_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, WolfState};
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
        model.setup_wolf(&WolfState {
            quadruped: quadruped_state(row)?,
            angry: row["angry"].as_bool().ok_or("angry")?,
            sitting: row["sitting"].as_bool().ok_or("sitting")?,
            baby: id.contains("_baby"),
            tail_angle: bits("tail_angle")?,
            head_roll_angle: bits("head_roll_angle")?,
            shake_anim: bits("shake_anim")?,
            age_scale: bits("age_scale")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `strider_textures`: the textures a strider wears.
pub(super) fn strider_textures_result() -> Value {
    let rows: Vec<Value> = [
        ("STRIDER_LOCATION", "entity/strider/strider"),
        ("STRIDER_BABY_LOCATION", "entity/strider/strider_baby"),
        ("COLD_LOCATION", "entity/strider/strider_cold"),
        ("COLD_BABY_LOCATION", "entity/strider/strider_cold_baby"),
    ]
    .iter()
    .map(|(label, path)| {
        json!({"label": label, "texture": format!("minecraft:textures/{path}.png")})
    })
    .collect();
    json!({"rows": rows})
}

/// `strider_model`: a strider's model (grown, baby or saddle) posed and drawn.
pub(super) fn strider_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, StriderState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let age = u32::from_str_radix(row["age_in_ticks"].as_str().ok_or("age_in_ticks")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())?;
        let mut model = MobModel::bake(layer);
        model.setup_strider(&StriderState {
            quadruped: quadruped_state(row)?,
            age_in_ticks: age,
            ridden: row["ridden"].as_bool().ok_or("ridden")?,
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `ravager_model`: a ravager's model posed and drawn.
pub(super) fn ravager_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, RavagerState};
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
        model.setup_ravager(&RavagerState {
            quadruped: quadruped_state(row)?,
            stunned_ticks_remaining: bits("stunned")?,
            attack_ticks_remaining: bits("attack")?,
            roar_animation: bits("roar")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `panda_textures`: the textures a panda wears by gene and age.
pub(super) fn panda_textures_result() -> Value {
    let mut rows = Vec::new();
    for (table, baby) in [("TEXTURES", false), ("BABY_TEXTURES", true)] {
        for gene in 0..7_u8 {
            rows.push(json!({
                "label": format!("{gene}-{table}"),
                "texture": format!(
                    "minecraft:textures/{}.png",
                    voxel_data::entity_type::panda_texture(gene, baby)
                ),
            }));
        }
    }
    json!({"rows": rows})
}

/// `panda_model`: a panda's model posed and drawn.
pub(super) fn panda_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::mob_model::{MobModel, PandaState};
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
        model.setup_panda(&PandaState {
            quadruped: quadruped_state(row)?,
            age_in_ticks: bits("age_in_ticks")?,
            unhappy: flag("unhappy")?,
            sneezing: flag("sneezing")?,
            sneeze_time: row["sneeze_time"].as_i64().ok_or("sneeze_time")? as i32,
            eating: flag("eating")?,
            scared: flag("scared")?,
            sit_amount: bits("sit_amount")?,
            lie_on_back_amount: bits("lie_on_back_amount")?,
            roll_amount: bits("roll_amount")?,
            baby: id.contains("_baby"),
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// A pose's matrix as the oracle prints it: the floats' bits, column by column.
pub(super) fn matrix_bits(pose: &voxel_render::joml::Pose) -> Vec<String> {
    (0..4)
        .flat_map(|column| (0..4).map(move |row| (column, row)))
        .map(|(column, row)| format!("{:08x}", pose.pose.m[column][row].to_bits()))
        .collect()
}

/// `setup_rotations`: what a panda's, a cat's or a fox's renderer leaves the pose at (from an
/// identity entity pose), and what a panda's layer carries its item with.
pub(super) fn setup_rotations_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::mob_model::{
        MobPlacement, PandaRotation, panda_item_pose, placement_rotations,
    };
    let matrix_of = matrix_bits;
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
        let (baby, x_rot, age) = (flag("baby")?, bits("x_rot")?, bits("age_in_ticks")?);
        let mut placed = MobPlacement {
            body_rot: bits("body_rot")?,
            ..MobPlacement::default()
        };
        let mut poses = Vec::new();
        let mut full_pose = None;
        let mut block_overlay = None;
        match row["kind"].as_str().ok_or("kind")? {
            "panda" => {
                let (sit, scared, eating) = (bits("sit_amount")?, flag("scared")?, flag("eating")?);
                placed.panda = Some(PandaRotation {
                    roll_time: bits("roll_time")?,
                    sit_amount: sit,
                    lie_on_back_amount: bits("lie_on_back_amount")?,
                    x_rot,
                    age_in_ticks: age,
                    scared,
                    baby,
                });
                if flag("sitting")? && !scared {
                    let mut pose = panda_item_pose(&Pose::default(), age, eating);
                    // The stack's own display transform is none: the model is moved to its corner.
                    pose.translate(-0.5, -0.5, -0.5);
                    poses.push(matrix_of(&pose));
                }
            }
            "squid" => {
                placed.squid = Some(voxel_render::mob_model::SquidTurns {
                    x_body_rot: bits("x_body_rot")?,
                    z_body_rot: bits("z_body_rot")?,
                    baby,
                });
            }
            kind @ ("cod" | "salmon" | "tropical_fish" | "pufferfish") => {
                use voxel_render::mob_model::{
                    cod_turns, pufferfish_turns, salmon_turns, tropical_fish_turns,
                };
                let in_water = flag("in_water")?;
                placed.fish = Some(match kind {
                    "cod" => cod_turns(age, in_water),
                    "salmon" => salmon_turns(age, in_water),
                    "tropical_fish" => tropical_fish_turns(age, in_water),
                    _ => pufferfish_turns(age),
                });
                if row.get("death_time").is_some() {
                    placed.death_time = bits("death_time")?;
                }
                if row.get("feet_x").is_some() {
                    placed.feet = [bits("feet_x")?, bits("feet_y")?, bits("feet_z")?];
                    placed.scale = bits("scale")?;
                }
            }
            "shulker" => {
                // `ShulkerRenderer.setupRotations` turns the body half way round first.
                placed.body_rot += 180.0;
                placed.attach_face = Some(i32_field(row, "attach_face")?);
                if row.get("death_time").is_some() {
                    placed.death_time = bits("death_time")?;
                }
                if row.get("feet_x").is_some() {
                    placed.feet = [bits("feet_x")?, bits("feet_y")?, bits("feet_z")?];
                    placed.scale = bits("scale")?;
                }
            }
            "sulfur_cube" => {
                placed.cube_mob = Some(voxel_render::mob_model::CubeMobScale {
                    size: i32_field(row, "size")?,
                    squish: bits("squish")?,
                    downscale: true,
                    sulfur: Some(voxel_render::mob_model::SulfurCubeScale {
                        fuse_remaining: bits("fuse_remaining")?,
                        baby,
                        holds_block: row.get("contained_block").is_some(),
                    }),
                });
                if row.get("feet_x").is_some() {
                    placed.feet = [bits("feet_x")?, bits("feet_y")?, bits("feet_z")?];
                    placed.scale = bits("scale")?;
                }
                let pose = voxel_render::mob_model::placement_pose(&placed);
                full_pose = Some(matrix_of(&pose));
                if row.get("contained_block").is_some() {
                    poses.push(matrix_of(
                        &voxel_render::entity_block::sulfur_cube_block_pose(&pose, baby),
                    ));
                    let (red, white) = voxel_render::mob_model::sulfur_cube_overlay(
                        bits("fuse_remaining")?,
                        row.get("red_overlay").is_some(),
                    );
                    block_overlay = Some(voxel_render::mob_model::overlay_coords(red, white));
                }
            }
            kind @ ("slime" | "magma_cube") => {
                placed.cube_mob = Some(voxel_render::mob_model::CubeMobScale {
                    size: i32_field(row, "size")?,
                    squish: bits("squish")?,
                    downscale: kind == "slime",
                    sulfur: None,
                });
                if row.get("death_time").is_some() {
                    placed.death_time = bits("death_time")?;
                }
                if row.get("feet_x").is_some() {
                    placed.feet = [bits("feet_x")?, bits("feet_y")?, bits("feet_z")?];
                    placed.scale = bits("scale")?;
                }
                full_pose = Some(matrix_of(&voxel_render::mob_model::placement_pose(&placed)));
            }
            "phantom" => {
                let scale =
                    1.0 + 0.15 * row["size"].as_i64().ok_or("size must be a number")? as f32;
                placed.hook_scale = Some([scale; 3]);
                placed.hook_translate = Some([0.0, 1.3125, 0.1875]);
                placed.pitch = Some(x_rot);
                if row.get("death_time").is_some() {
                    placed.death_time = bits("death_time")?;
                }
                full_pose = Some(matrix_of(&voxel_render::mob_model::placement_pose(&placed)));
            }
            "cat" => placed.cat_lying(bits("lie_down_amount")?, flag("on_sleeping_player")?),
            "fox" => {
                if flag("pouncing")? || flag("faceplanted")? {
                    placed.fox_tipping(x_rot);
                }
            }
            _ => return Err("unknown kind".into()),
        }
        let mut described = json!({
            "label": row["label"],
            "pose": matrix_of(&placement_rotations(&placed)),
            "poses": poses,
        });
        if let Some(full_pose) = full_pose {
            described["full_pose"] = json!(full_pose);
        }
        if let Some(overlay) = block_overlay {
            described["block_overlay"] = json!([overlay]);
        }
        out.push(described);
    }
    Ok(json!({"rows": out}))
}
