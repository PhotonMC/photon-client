use super::*;

/// `bell_pose`: the bell's model and sprite for its ticks and the direction it was struck from.
pub(super) fn bell_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::MobModel;
    let layer = voxel_data::entity_model::layer(voxel_render::block_entity::bell::LAYER)
        .ok_or("no layer")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let ticks = f32::from_bits(
            u32::from_str_radix(row["ticks"].as_str().ok_or("ticks")?, 16)
                .map_err(|error| error.to_string())?,
        );
        let shake = row["shake"]
            .as_str()
            .map(|name| voxel_core::direction::Direction::from_name(name).ok_or("shake"))
            .transpose()?;
        let mut model = MobModel::bake(layer);
        model.setup_bell(ticks, shake);
        rows.push(json!({
            "label": row["label"],
            "ticks": format!("{:08x}", ticks.to_bits()),
            "light": row["light"],
            "texture": voxel_render::block_entity::bell::TEXTURE,
            "quads": posed_quads(&model),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// `skull_block_pose`: where, with what texture and light a skull block is drawn, and the model
/// posed for the animation.
pub(super) fn skull_block_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::block_entity::skull::{
        SkullKind, block_pose, ground_transformation, wall_transformation,
    };
    use voxel_render::entity::mob_model::MobModel;
    use voxel_render::joml::Pose;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let animation = f32::from_bits(
            u32::from_str_radix(row["animation"].as_str().ok_or("animation")?, 16)
                .map_err(|error| error.to_string())?,
        );
        let kind = match row["kind"].as_str().ok_or("kind")? {
            "SKELETON" => SkullKind::Skeleton,
            "WITHER_SKELETON" => SkullKind::WitherSkeleton,
            "PLAYER" => SkullKind::Player,
            "ZOMBIE" => SkullKind::Zombie,
            "CREEPER" => SkullKind::Creeper,
            "PIGLIN" => SkullKind::Piglin,
            "DRAGON" => SkullKind::Dragon,
            _ => return Err("kind".to_owned()),
        };
        let transformation = match row["wall"].as_str() {
            Some(name) => wall_transformation(
                voxel_core::direction::Direction::from_name(name).ok_or("wall")?,
            ),
            None => ground_transformation(row["rotation"].as_i64().ok_or("rotation")? as i32),
        };
        let pose = block_pose(&Pose::default(), &transformation);
        let matrix: Vec<String> = pose
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        let layer = voxel_data::entity_model::layer(kind.layer()).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_skull(kind, animation);
        rows.push(json!({
            "label": row["label"],
            "matrix": matrix,
            "light": row["light"],
            "texture": kind.texture(),
            "quads": posed_quads(&model),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// `banner_pose`: every model a banner block submits, in the order it is drawn.
pub(super) fn banner_pose_result(request: &Value) -> Result<Value, String> {
    use voxel_render::block_entity::banner::{
        BASE_TEXTURE, MAX_PATTERNS, PATTERN_BASE_TEXTURE, ground_transformation, layers,
        pattern_texture, wall_transformation,
    };
    use voxel_render::block_entity::skull::block_pose;
    use voxel_render::entity::mob_model::MobModel;
    use voxel_render::joml::Pose;
    let dye = |name: &str| -> Result<u32, String> {
        let id = [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "light_gray",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ]
        .iter()
        .position(|candidate| *candidate == name)
        .ok_or("dye")?;
        voxel_data::entity_type::dye_texture_colour(id).ok_or_else(|| "dye".to_owned())
    };
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let phase = f32::from_bits(
            u32::from_str_radix(row["phase"].as_str().ok_or("phase")?, 16)
                .map_err(|error| error.to_string())?,
        );
        let wall = row["wall"].as_str();
        let transformation = match wall {
            Some(name) => wall_transformation(
                voxel_core::direction::Direction::from_name(name).ok_or("wall")?,
            ),
            None => ground_transformation(row["rotation"].as_i64().ok_or("rotation")? as i32),
        };
        let pose = block_pose(&Pose::default(), &transformation);
        let matrix: Vec<String> = pose
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        let (main, flag) = layers(wall.is_some());
        let mut pole = MobModel::bake(voxel_data::entity_model::layer(main).ok_or("layer")?);
        pole.setup_banner_flag(phase);
        let mut cloth = MobModel::bake(voxel_data::entity_model::layer(flag).ok_or("layer")?);
        cloth.setup_banner_flag(phase);
        let call = |model: &MobModel, texture: &str, tint: u32, order: usize| {
            json!({
                "matrix": matrix,
                "order": order,
                "light": row["light"],
                "tint": format!("{tint:x}"),
                "texture": texture,
                "quads": posed_quads(model),
            })
        };
        let mut calls = vec![
            call(&pole, BASE_TEXTURE, 0xFFFF_FFFF, 0),
            call(&cloth, BASE_TEXTURE, 0xFFFF_FFFF, 0),
            call(
                &cloth,
                PATTERN_BASE_TEXTURE,
                dye(row["color"].as_str().ok_or("color")?)?,
                0,
            ),
        ];
        for (index, layer) in row["patterns"]
            .as_array()
            .ok_or("patterns")?
            .iter()
            .take(MAX_PATTERNS)
            .enumerate()
        {
            calls.push(call(
                &cloth,
                &pattern_texture(layer["pattern"].as_str().ok_or("pattern")?),
                dye(layer["color"].as_str().ok_or("color")?)?,
                index + 1,
            ));
        }
        rows.push(json!({"label": row["label"], "calls": calls}));
    }
    Ok(json!({"rows": rows}))
}

/// `camera_hurt`: the matrix `GameRenderer.bobHurt` turns the view by.
pub(super) fn camera_hurt_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(row[key].as_str().ok_or(key.to_owned())?, 16)
                    .map_err(|error| error.to_string())?,
            ))
        };
        let tilt = f64::from_bits(
            u64::from_str_radix(row["tilt"].as_str().ok_or("tilt")?, 16)
                .map_err(|error| error.to_string())?,
        );
        let view = voxel_render::camera::HurtView {
            hurt_time: float("hurt_time")?,
            hurt_duration: row["hurt_duration"].as_i64().ok_or("hurt_duration")? as i32,
            hurt_dir: float("hurt_dir")?,
            dying: row["dying"].as_bool().ok_or("dying")?,
            death_time: float("death_time")?,
            tilt_strength: tilt,
        };
        let matrix: Vec<String> = voxel_render::camera::view_hurt(&view)
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        rows.push(json!({"label": row["label"], "matrix": matrix}));
    }
    Ok(json!({"rows": rows}))
}

/// `parrot_textures`: the texture each parrot variant wears.
pub(super) fn parrot_textures_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = i32_field(row, "id")?;
        out.push(json!({
            "label": row["label"],
            "texture": format!("minecraft:textures/{}.png", voxel_world::mobs::parrot::variant_texture(id)),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `guardian_beam`: the vertices of a guardian's beam from the pose its eye leaves.
pub(super) fn guardian_beam_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::objects::guardian_beam::{BEAM_LIGHT, guardian_beam};
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
        let mut vector = [0.0_f64; 3];
        for (slot, part) in vector
            .iter_mut()
            .zip(row["vector"].as_array().ok_or("vector")?)
        {
            *slot = f64::from_bits(
                u64::from_str_radix(part.as_str().ok_or("vector part")?, 16)
                    .map_err(|error| error.to_string())?,
            );
        }
        let mut pose = Pose::default();
        pose.translate(bits("x")?, bits("y")?, bits("z")?);
        let beam = guardian_beam(
            pose,
            vector,
            bits("time")?,
            bits("scale")?,
            bits("tex_v_off")?,
        );
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let color = (0xFF_u32 << 24)
            | ((beam.colour[0] as u32) << 16)
            | ((beam.colour[1] as u32) << 8)
            | beam.colour[2] as u32;
        let vertices: Vec<String> = beam
            .vertices
            .iter()
            .map(|vertex| {
                format!(
                    "{}{}{}{}{}{color:08x}{:08x}{}{}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    hex(vertex.uv[0]),
                    hex(vertex.uv[1]),
                    BEAM_LIGHT,
                    hex(beam.normal[0]),
                    hex(beam.normal[1]),
                    hex(beam.normal[2]),
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `crystal_beam`: the vertices of an end crystal's beam from the pose it is submitted at.
pub(super) fn crystal_beam_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::objects::crystal_beam::crystal_beam;
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
        pose.translate(bits("x")?, bits("y")?, bits("z")?);
        let beam = crystal_beam(pose, [bits("dx")?, bits("dy")?, bits("dz")?], bits("time")?);
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let light = i32_field(row, "light")?;
        let vertices: Vec<String> = beam
            .vertices
            .iter()
            .map(|vertex| {
                let colour: u32 = if vertex.white {
                    0xFFFF_FFFF
                } else {
                    0xFF00_0000
                };
                format!(
                    "{}{}{}{}{}{colour:08x}{light:08x}{}{}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    hex(vertex.uv[0]),
                    hex(vertex.uv[1]),
                    hex(beam.normal[0]),
                    hex(beam.normal[1]),
                    hex(beam.normal[2]),
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `dragon_rays`: the vertices of a dying ender dragon's rays from the pose its death leaves.
pub(super) fn dragon_rays_result(request: &Value) -> Result<Value, String> {
    use voxel_render::joml::Pose;
    use voxel_render::objects::dragon_rays::dragon_rays;
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
        pose.translate(bits("x")?, bits("y")?, bits("z")?);
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let vertices: Vec<String> = dragon_rays(pose, bits("death_time")?)
            .iter()
            .map(|vertex| {
                // The consumer takes no texture, light or normal of a ray: zeros.
                format!(
                    "{}{}{}{}{}{:08x}{:08x}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    "00000000",
                    "00000000",
                    vertex.colour,
                    0,
                    "000000000000000000000000",
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `guardian_model`: a guardian's model posed and drawn from the look, the spikes' and the tail's
/// animation and what its eye follows.
pub(super) fn guardian_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{GuardianLook, GuardianState, MobModel};
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
        let vector = |value: &Value| -> Result<[f64; 3], String> {
            let parts = value.as_array().ok_or("vector")?;
            let mut out = [0.0; 3];
            for (slot, part) in out.iter_mut().zip(parts) {
                *slot = f64::from_bits(
                    u64::from_str_radix(part.as_str().ok_or("vector part")?, 16)
                        .map_err(|error| error.to_string())?,
                );
            }
            Ok(out)
        };
        let look = match row.get("eye") {
            Some(eye) => Some(GuardianLook {
                eye_position: vector(&eye["eye_position"])?,
                look_direction: vector(&eye["look_direction"])?,
                look_at_position: vector(&eye["look_at_position"])?,
            }),
            None => None,
        };
        let mut model = MobModel::bake(layer);
        model.setup_guardian(&GuardianState {
            head_y_rot: bits("y_rot")?,
            x_rot: bits("x_rot")?,
            age_in_ticks: bits("age_in_ticks")?,
            spikes_animation: bits("spikes")?,
            tail_animation: bits("tail")?,
            look,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `nautilus_model`: a nautilus's model posed and drawn from the look, the stride, the age and
/// whether it wears body armor.
pub(super) fn nautilus_model_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::mob_model::{MobModel, NautilusState, QuadrupedState};
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
        model.setup_nautilus(&NautilusState {
            quadruped: QuadrupedState {
                head_y_rot: bits("y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            has_body_armor: row["armored"].as_bool().ok_or("armored")?,
        });
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}
