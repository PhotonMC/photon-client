use super::*;

/// `villager_model`: a villager's or a wandering trader's model posed and
/// drawn.
pub(super) fn villager_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, QuadrupedState, VillagerState};
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
    use voxel_render::entity::mob_model::{IllagerArmPose, IllagerState, MobModel, QuadrupedState};
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
) -> Result<voxel_render::entity::mob_model::QuadrupedState, String> {
    let bits = |key: &str| -> Result<f32, String> {
        u32::from_str_radix(
            row[key].as_str().ok_or(format!("{key} must be a string"))?,
            16,
        )
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
    };
    Ok(voxel_render::entity::mob_model::QuadrupedState {
        head_y_rot: bits("head_y_rot")?,
        x_rot: bits("x_rot")?,
        walk_position: bits("walk_position")?,
        walk_speed: bits("walk_speed")?,
    })
}

/// `iron_golem_model`: the iron golem's model posed and drawn.
pub(super) fn iron_golem_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{IronGolemState, MobModel};
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
    use voxel_render::entity::mob_model::MobModel;
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
    use voxel_render::entity::mob_model::{EndermanState, MobModel};
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
    use voxel_render::entity::mob_model::{Crackiness, golem_crackiness, iron_golem_roll};
    use voxel_render::joml::Pose;
    use voxel_render::joml::Quaternion;
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

pub(super) fn held_item_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::held_item::{hand_pose, head_item_pose};
    use voxel_render::entity::mob_model::{MobModel, PiglinState, SkeletonState, ZombieState};
    use voxel_render::entity::player_model::{Arm, ArmPose, Part, PlayerModel};
    use voxel_render::joml::Pose;
    let arm_pose = |text: &str| ArmPose::named(text).unwrap_or_default();
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let kind = row["kind"].as_str().ok_or("kind")?;
        let baby = row["baby"].as_bool().ok_or("baby")?;
        let flag = |key: &str| row[key].as_bool().unwrap_or(false);
        let humanoid = voxel_render::entity::player_model::PlayerRenderState {
            crouching: flag("crouching"),
            main_arm: if row["main_arm"].as_str() == Some("left") {
                Arm::Left
            } else {
                Arm::Right
            },
            right_arm_pose: arm_pose(row["right_pose"].as_str().ok_or("right_pose")?),
            left_arm_pose: arm_pose(row["left_pose"].as_str().ok_or("left_pose")?),
            is_using_item: flag("using_item"),
            use_item_main_hand: row["use_hand"].as_str() != Some("off"),
            is_passenger: flag("passenger"),
            is_fall_flying: flag("fall_flying"),
            speed_value: f32_field(row, "speed_value")?,
            ticks_using_item: f32_field(row, "ticks_using_item")?,
            max_crossbow_charge_duration: f32_field(row, "max_crossbow_charge")?,
            ..humanoid_state(row)?
        };
        // The spears the rows' stacks are, and what they change.
        let spear_of = |key: &str| row[key].as_str().and_then(voxel_data::spear::spear_of);
        let attack_arm = if row["attack_arm"].as_str() == Some("left") {
            Arm::Left
        } else {
            Arm::Right
        };
        let spears = voxel_render::entity::held_item::spear_view(
            &voxel_render::entity::held_item::SpearInput {
                spears: (spear_of("right_item"), spear_of("left_item")),
                main_arm: humanoid.main_arm,
                attack: (humanoid.attack_time, attack_arm),
                swing_animation: if row["swing"].as_str() == Some("stab") {
                    voxel_render::entity::player_model::SwingAnimation::Stab
                } else {
                    voxel_render::entity::player_model::SwingAnimation::Whack
                },
                arm_poses: (humanoid.right_arm_pose, humanoid.left_arm_pose),
                using: (
                    humanoid.is_using_item,
                    humanoid.use_item_main_hand,
                    humanoid.ticks_using_item,
                ),
                hit_feedback: if row["hit_feedback"].is_null() {
                    0.0
                } else {
                    f32_field(row, "hit_feedback")?
                },
            },
        );
        let main_item = if humanoid.main_arm == Arm::Left {
            "left_item"
        } else {
            "right_item"
        };
        let humanoid = voxel_render::entity::player_model::PlayerRenderState {
            attack_arm,
            is_baby: baby,
            main_hand_empty: row[main_item].is_null(),
            swing_animation: spears.swing_animation,
            right_spear: spears.right,
            left_spear: spears.left,
            ..humanoid
        };
        let aggressive = flag("aggressive");
        let layer = |id: &str| {
            voxel_data::entity_model::layer(id)
                .map(MobModel::bake)
                .ok_or("no such layer")
        };
        let slim = kind == "player_slim";
        let (quads, (poses, head_poses, skull_quads)) = if kind.starts_with("player") {
            let mut model = PlayerModel::new(slim);
            model.setup_anim(&humanoid);
            let poses = held_poses(
                &model.root,
                row,
                (if slim { 0.5 } else { 0.0 }, baby),
                &spears.motions,
            );
            (sorted_quads(model.render(&Default::default())), poses)
        } else {
            let model = match kind {
                "zombie" => {
                    let mut model = layer(if baby {
                        "minecraft:zombie_baby#main"
                    } else {
                        "minecraft:zombie#main"
                    })?;
                    model.setup_zombie(&ZombieState {
                        humanoid,
                        aggressive,
                        drowned: false,
                    });
                    model
                }
                "skeleton" => {
                    let mut model = layer("minecraft:skeleton#main")?;
                    model.setup_skeleton(&SkeletonState {
                        humanoid,
                        aggressive,
                        holding_bow: flag("holding_bow"),
                    });
                    model
                }
                "piglin" => {
                    let mut model = layer(if baby {
                        "minecraft:piglin_baby#main"
                    } else {
                        "minecraft:piglin#main"
                    })?;
                    model.setup_piglin(&PiglinState {
                        humanoid,
                        baby,
                        ..Default::default()
                    });
                    model
                }
                _ => return Err(format!("unknown humanoid {kind}")),
            };
            let poses = held_poses(
                model.root(),
                row,
                (if kind == "skeleton" { 1.0 } else { 0.0 }, baby),
                &spears.motions,
            );
            (posed_quads(&model), poses)
        };
        /// The poses of the held items, and of the head item when the row
        /// asks for one.
        #[allow(clippy::type_complexity)]
        fn held_poses(
            root: &Part,
            row: &Value,
            (hand_shift, baby): (f32, bool),
            motions: &[voxel_render::entity::held_item::HandMotion; 2],
        ) -> (Vec<Vec<String>>, Vec<Vec<String>>, Option<Vec<String>>) {
            let hex = |value: f32| format!("{:08x}", value.to_bits());
            let mut poses = Vec::new();
            let mut head_poses = Vec::new();
            let mut skull_quads = None;
            if let Some(scale) = row.get("head_scale") {
                let bits = |value: &Value| {
                    value
                        .as_str()
                        .and_then(|text| u32::from_str_radix(text, 16).ok())
                        .map(f32::from_bits)
                        .expect("bits")
                };
                let head = root
                    .children()
                    .iter()
                    .find(|part| part.name() == "head")
                    .expect("the model has a head");
                let mut pose = head_item_pose(
                    &Pose::default(),
                    root,
                    head,
                    (bits(&row["head_y_offset"]), bits(scale)),
                );
                if let Some(kind) = row["skull"].as_str().and_then(|name| match name {
                    "skeleton" => Some(voxel_render::block_entity::skull::SkullKind::Skeleton),
                    "wither_skeleton" => {
                        Some(voxel_render::block_entity::skull::SkullKind::WitherSkeleton)
                    }
                    "player" => Some(voxel_render::block_entity::skull::SkullKind::Player),
                    "zombie" => Some(voxel_render::block_entity::skull::SkullKind::Zombie),
                    "creeper" => Some(voxel_render::block_entity::skull::SkullKind::Creeper),
                    "piglin" => Some(voxel_render::block_entity::skull::SkullKind::Piglin),
                    "dragon" => Some(voxel_render::block_entity::skull::SkullKind::Dragon),
                    _ => None,
                }) {
                    let mut skull = voxel_render::entity::mob_model::MobModel::bake(
                        voxel_data::entity_model::layer(kind.layer()).expect("the skull's layer"),
                    );
                    skull.setup_skull(kind, bits(&row["skull_animation"]));
                    let skull_pose = voxel_render::block_entity::skull::skull_pose(
                        &Pose::default(),
                        root,
                        head,
                        (bits(&row["head_skull_y"]), bits(scale)),
                    );
                    skull_quads = Some(sorted_quads(skull.render(&skull_pose)));
                }
                pose.translate(-0.5, -0.5, -0.5);
                head_poses.push(
                    (0..4)
                        .flat_map(|column| (0..4).map(move |row| (column, row)))
                        .map(|(column, row)| hex(pose.pose.m[column][row]))
                        .collect(),
                );
            }
            for (key, side, name) in [
                ("holds_right", Arm::Right, "right_arm"),
                ("holds_left", Arm::Left, "left_arm"),
            ] {
                if !row[key].as_bool().unwrap_or(false) {
                    continue;
                }
                let arm = root
                    .children()
                    .iter()
                    .find(|part| part.name() == name)
                    .expect("the model has its arms");
                let motion = &motions[usize::from(side == Arm::Left)];
                let mut pose = hand_pose(
                    &Pose::default(),
                    root,
                    arm,
                    side,
                    (hand_shift, baby),
                    motion,
                );
                // A spyglass in use is held to the eye instead.
                if row["kind"]
                    .as_str()
                    .is_some_and(|kind| kind.starts_with("player"))
                    && row["eye_item"].as_bool().unwrap_or(false)
                    && voxel_render::entity::held_item::eye_held_arm(
                        (
                            row["using_item"].as_bool().unwrap_or(false),
                            row["use_hand"].as_str() != Some("off"),
                        ),
                        (
                            if row["main_arm"].as_str() == Some("left") {
                                Arm::Left
                            } else {
                                Arm::Right
                            },
                            row["attack_time"]
                                .as_str()
                                .and_then(|text| u32::from_str_radix(text, 16).ok())
                                .map_or(0.0, f32::from_bits),
                        ),
                        true,
                        side,
                    )
                {
                    let head = root
                        .children()
                        .iter()
                        .find(|part| part.name() == "head")
                        .expect("the model has a head");
                    pose = voxel_render::entity::held_item::held_to_eye_pose(
                        &Pose::default(),
                        root,
                        head,
                        side,
                    );
                }
                // `ItemTransform.NO_TRANSFORM.apply` of the layer.
                pose.translate(-0.5, -0.5, -0.5);
                // Column by column, as `Matrix4f.get` writes them.
                poses.push(
                    (0..4)
                        .flat_map(|column| (0..4).map(move |row| (column, row)))
                        .map(|(column, row)| hex(pose.pose.m[column][row]))
                        .collect(),
                );
            }
            (poses, head_poses, skull_quads)
        }
        let mut described = json!({"label": row["label"], "quads": quads, "poses": poses});
        if row.get("head_scale").is_some() {
            described["head_poses"] = json!(head_poses);
        }
        if let Some(skull) = skull_quads {
            described["skull_quads"] = json!(skull);
        }
        out.push(described);
    }
    Ok(json!({"rows": out}))
}

/// `skeleton_model`: a skeleton family model posed and drawn.
pub(super) fn skeleton_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, SkeletonState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer = voxel_data::entity_model::layer(row["layer"].as_str().ok_or("layer")?)
            .ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_skeleton(&SkeletonState {
            humanoid: humanoid_state(row)?,
            aggressive: row["aggressive"].as_bool().ok_or("aggressive")?,
            holding_bow: row["holding_bow"].as_bool().ok_or("holding_bow")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `piglin_model`: a piglin family model posed and drawn.
pub(super) fn piglin_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, PiglinState};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_piglin(&PiglinState {
            humanoid: voxel_render::entity::player_model::PlayerRenderState {
                main_arm: if row["main_arm"].as_str() == Some("left") {
                    voxel_render::entity::player_model::Arm::Left
                } else {
                    voxel_render::entity::player_model::Arm::Right
                },
                ticks_using_item: f32_field(row, "ticks_using_item")?,
                max_crossbow_charge_duration: f32_field(row, "max_crossbow_charge")?,
                ..humanoid_state(row)?
            },
            aggressive: row["aggressive"].as_bool().ok_or("aggressive")?,
            zombified: id.starts_with("minecraft:zombified_piglin"),
            baby: id.contains("_baby"),
            arm_pose: voxel_render::entity::mob_model::PiglinArmPose::named(
                row["arm_pose"].as_str().ok_or("arm_pose")?,
            )
            .ok_or("unknown piglin arm pose")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `zombie_model`: a zombie family model posed and drawn.
pub(super) fn zombie_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, ZombieState};
    use voxel_render::entity::player_model::PlayerRenderState;
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
        model.setup_zombie(&ZombieState {
            humanoid: PlayerRenderState {
                head_y_rot: bits(row, "head_y_rot")?,
                x_rot: bits(row, "x_rot")?,
                walk_position: bits(row, "walk_position")?,
                walk_speed: bits(row, "walk_speed")?,
                age_in_ticks: bits(row, "age_in_ticks")?,
                attack_time: bits(row, "attack_time")?,
                swim_amount: bits(row, "swim_amount")?,
                ..Default::default()
            },
            aggressive: row["aggressive"].as_bool().ok_or("aggressive")?,
            drowned: id.starts_with("minecraft:drowned"),
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
