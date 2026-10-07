use super::*;

/// `flier_item_pose`: where a vex's or an allay's hands carry what they hold.
pub(super) fn flier_item_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::held_item::{allay_hand_pose, vex_hand_pose};
    use voxel_render::entity::mob_model::{
        AllayRenderState, MobModel, QuadrupedState, VexRenderState,
    };
    use voxel_render::entity::player_model::Arm;
    use voxel_render::joml::Pose;
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
        let vex = row["kind"].as_str().ok_or("kind")? == "vex";
        let layer = voxel_data::entity_model::layer(if vex {
            "minecraft:vex#main"
        } else {
            "minecraft:allay#main"
        })
        .ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        let (holds_right, holds_left) = (flag("holds_right")?, flag("holds_left")?);
        // The row says what the state's right and left hands hold (which hand is the main one
        // only decides that at extraction, not in the layer).
        if vex {
            model.setup_vex(&VexRenderState {
                head_y_rot: 0.0,
                x_rot: 0.0,
                age_in_ticks: bits("age_in_ticks")?,
                is_charging: flag("charging")?,
                right_hand_item: holds_right,
                left_hand_item: holds_left,
            });
        } else {
            model.setup_allay(&AllayRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: 0.0,
                    x_rot: 0.0,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                is_dancing: false,
                is_spinning: false,
                spinning_progress: 0.0,
                holding_animation_progress: bits("holding_progress")?,
            });
        }
        let child = |part: &voxel_render::entity::player_model::Part, name: &str| {
            part.children()
                .iter()
                .find(|child| child.name() == name)
                .cloned()
        };
        let inner = child(model.root(), "root").ok_or("root")?;
        let body = child(&inner, "body").ok_or("body")?;
        let base = Pose::default();
        let mut poses = Vec::new();
        for (side, name, holds) in [
            (Arm::Right, "right_arm", holds_right),
            (Arm::Left, "left_arm", holds_left),
        ] {
            if !holds {
                continue;
            }
            let arm = child(&body, name).ok_or("arm")?;
            let pose = if vex {
                vex_hand_pose(&base, (&inner, &body, &arm), side)
            } else {
                let right = child(&body, "right_arm").ok_or("arm")?;
                allay_hand_pose(&base, (&inner, &body, &right), side)
            };
            // The stack's own display transform is none: the model is moved to its corner.
            let mut pose = pose;
            pose.translate(-0.5, -0.5, -0.5);
            poses.push(matrix_bits(&pose));
        }
        out.push(json!({"label": row["label"], "poses": poses}));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn boat_state_of(
    row: &Value,
) -> Result<voxel_render::objects::boat::BoatRenderState, String> {
    let bits = |key: &str| -> Result<f32, String> {
        u32::from_str_radix(
            row[key].as_str().ok_or(format!("{key} must be a string"))?,
            16,
        )
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
    };
    Ok(voxel_render::objects::boat::BoatRenderState {
        y_rot: bits("y_rot")?,
        hurt_dir: i32_field(row, "hurt_dir")?,
        hurt_time: bits("hurt_time")?,
        damage_time: bits("damage_time")?,
        bubble_angle: bits("bubble_angle")?,
        is_under_water: row["under_water"].as_bool().ok_or("under_water")?,
        rowing_time_left: bits("rowing_left")?,
        rowing_time_right: bits("rowing_right")?,
    })
}

/// `boat_model`: a boat's, chest boat's or raft's model, or the water patch, posed and drawn.
pub(super) fn boat_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = voxel_data::entity_model::layer(id).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        if !id.contains("water_patch") {
            model.setup_boat(&boat_state_of(row)?);
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `boat_pose`: where a boat's model (and its water patch, above water) is drawn from an
/// identity pose.
pub(super) fn boat_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::objects::boat::boat_pose;
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
        let state = boat_state_of(row)?;
        let mut base = Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let pose = boat_pose(&base, &state);
        // The model, then the water patch (drawn from the same pose) unless under water.
        let count = if state.is_under_water { 1 } else { 2 };
        let poses: Vec<Vec<String>> = (0..count).map(|_| matrix_bits(&pose)).collect();
        out.push(json!({"label": row["label"], "poses": poses}));
    }
    Ok(json!({"rows": out}))
}

/// `thrown_item_pose`: the pose a thrown item (or a firework rocket) is drawn from, seen by a camera.
pub(super) fn thrown_item_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::name_tag::camera_orientation;
    use voxel_render::joml::Pose;
    use voxel_render::objects::projectile::{firework_pose, thrown_item_pose};
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
        let mut base = Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let orientation = camera_orientation(bits("camera_y_rot")?, bits("camera_x_rot")?);
        let mut pose = if row["firework"].as_bool() == Some(true) {
            firework_pose(
                &base,
                &orientation,
                row["shot_at_angle"].as_bool() == Some(true),
            )
        } else {
            thrown_item_pose(&base, bits("scale")?, &orientation)
        };
        // The layer the reference's state makes here has no display transform of its own
        // (`ItemTransform.NO_TRANSFORM`), which only moves the model's corner to the middle.
        pose.translate(-0.5, -0.5, -0.5);
        out.push(json!({"label": row["label"], "poses": [matrix_bits(&pose)]}));
    }
    Ok(json!({"rows": out}))
}

/// `painting_geometry`: the vertices a painting of this size and facing is drawn with.
pub(super) fn painting_geometry_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::objects::painting::{SpriteUv, painting_pose, painting_vertices};
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
        let sprite = |name: &str| -> Result<SpriteUv, String> {
            Ok(SpriteUv {
                u0: bits(&format!("{name}_u0"))?,
                u1: bits(&format!("{name}_u1"))?,
                v0: bits(&format!("{name}_v0"))?,
                v1: bits(&format!("{name}_v1"))?,
            })
        };
        let mut base = Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let pose = painting_pose(&base, i32_field(row, "direction_2d")?);
        let lights: Vec<i32> = row["lights"]
            .as_array()
            .ok_or("lights")?
            .iter()
            .map(|light| light.as_i64().map(|light| light as i32).ok_or("light"))
            .collect::<Result<_, _>>()?;
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let vertices: Vec<String> = painting_vertices(
            &pose,
            (i32_field(row, "width")?, i32_field(row, "height")?),
            &lights,
            (sprite("front")?, sprite("back")?),
        )
        .iter()
        .map(|vertex| {
            format!(
                "{}{}{}{}{}ffffffff{:08x}{}{}{}",
                hex(vertex.position[0]),
                hex(vertex.position[1]),
                hex(vertex.position[2]),
                hex(vertex.uv[0]),
                hex(vertex.uv[1]),
                vertex.light,
                hex(vertex.normal[0]),
                hex(vertex.normal[1]),
                hex(vertex.normal[2]),
            )
        })
        .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `lightning_bolt`: the vertices a bolt of this seed is drawn with.
pub(super) fn lightning_bolt_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::objects::lightning::lightning_bolt;
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
        let seed: i64 = row["seed"]
            .as_str()
            .ok_or("seed")?
            .parse()
            .map_err(|_| "seed")?;
        let mut base = Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let vertices: Vec<String> = lightning_bolt(&base, seed)
            .iter()
            .map(|vertex| {
                format!(
                    "{}{}{}{}{}{:08x}{:08x}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    hex(0.0),
                    hex(0.0),
                    vertex.colour,
                    0,
                    "0".repeat(24),
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `registry` (the sound family): `sounds.json` registered and dumped, and picked from.
pub(super) fn sound_registry_result(request: &Value) -> Result<Value, String> {
    use voxel_core::random::RandomSource;
    use voxel_game::sounds::{Entry, SoundRegistry};
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let (text, files): (String, std::collections::HashSet<String>) =
            if row.get("real").is_some() {
                let assets = std::path::Path::new(
                    &std::env::var("VOXEL_ASSETS").map_err(|error| error.to_string())?,
                )
                .parent()
                .and_then(std::path::Path::parent)
                .ok_or("assets")?
                .to_path_buf();
                let index = voxel_core::json::read_object(&assets.join("indexes/32.json"))
                    .ok_or("index")?;
                let objects = index.object("objects").ok_or("objects")?;
                let hash = objects
                    .object("minecraft/sounds.json")
                    .and_then(|entry| entry.text("hash"))
                    .ok_or("sounds.json")?;
                let text =
                    std::fs::read_to_string(assets.join("objects").join(&hash[..2]).join(&hash))
                        .map_err(|error| error.to_string())?;
                let files = objects
                    .entries()
                    .filter_map(|(key, _)| String::from_utf16(key).ok())
                    .filter(|key| key.starts_with("minecraft/sounds/") && key.ends_with(".ogg"))
                    .map(|key| format!("minecraft:{}", &key["minecraft/".len()..]))
                    .collect();
                (text, files)
            } else {
                let files = row["files"]
                    .as_array()
                    .ok_or("files")?
                    .iter()
                    .filter_map(|file| file.as_str().map(|file| format!("minecraft:{file}")))
                    .collect();
                (row["json"].as_str().ok_or("json")?.to_owned(), files)
            };
        let mut registry = SoundRegistry::default();
        let valid = registry
            .register_file("minecraft", &text, &|path| files.contains(path))
            .is_ok();
        let mut names: Vec<&str> = registry.names().collect();
        names.sort_unstable();
        let wanted: Option<Vec<&str>> = row["events"]
            .as_array()
            .map(|events| events.iter().filter_map(Value::as_str).collect());
        let events: Vec<Value> = names
            .iter()
            .filter(|name| wanted.as_ref().is_none_or(|wanted| wanted.contains(name)))
            .map(|name| {
                let event = registry.event(name).expect("named");
                let entries: Vec<Value> = event
                    .entries
                    .iter()
                    .map(|entry| match entry {
                        Entry::File(sound) => json!({
                            "kind": "file", "name": sound.location.as_string(),
                            "weight": sound.weight, "volume": bits(sound.volume.sample()),
                            "pitch": bits(sound.pitch.sample()), "stream": sound.stream,
                            "preload": sound.preload,
                            "attenuation_distance": sound.attenuation_distance,
                        }),
                        Entry::Event(_) => {
                            json!({"kind": "event", "weight": registry.entry_weight(entry)})
                        }
                    })
                    .collect();
                let mut described = json!({"name": name, "weight": registry.weight(event),
                    "entries": entries});
                // The reference's writer leaves a null out.
                if let Some(subtitle) = &event.subtitle {
                    described["subtitle"] = json!(subtitle);
                }
                described
            })
            .collect();
        let mut picks = Vec::new();
        for pick in row["picks"].as_array().ok_or("picks")? {
            let name = pick["event"].as_str().ok_or("event")?;
            let seed: i64 = pick["seed"]
                .as_str()
                .ok_or("seed")?
                .parse()
                .map_err(|_| "seed")?;
            let event = registry.event(name).ok_or("no such event")?;
            let mut random = RandomSource::create(seed);
            let sequence: Vec<Value> = (0..pick["count"].as_i64().ok_or("count")?)
                .map(|_| {
                    let sound = registry.pick(event, &mut random);
                    json!({"location": sound.location.as_string(),
                        "volume": bits(sound.volume.sample()), "pitch": bits(sound.pitch.sample()),
                        "stream": sound.stream, "preload": sound.preload,
                        "attenuation_distance": sound.attenuation_distance})
                })
                .collect();
            picks.push(json!({"event": name, "seed": pick["seed"], "sequence": sequence}));
        }
        out.push(json!({"label": row["label"], "valid": valid, "events": events, "picks": picks}));
    }
    Ok(json!({"rows": out}))
}
