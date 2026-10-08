use super::*;

/// `primed_tnt_pose`: the pose and overlay a block of primed tnt is drawn with.
pub(super) fn primed_tnt_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::joml::Pose;
    use photon_render::objects::minecart::{primed_tnt_pose, tnt_is_lit};
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
        let fuse = bits("fuse")?;
        let pose = primed_tnt_pose(&base, fuse);
        let overlay = if tnt_is_lit(fuse) {
            (10 << 16) | 15
        } else {
            10 << 16
        };
        out.push(
            json!({"label": row["label"], "poses": [matrix_bits(&pose)], "overlays": [overlay]}),
        );
    }
    Ok(json!({"rows": out}))
}

/// `fishing_hook`: a bobber's quad and its line's vertices.
pub(super) fn fishing_hook_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::name_tag::camera_orientation;
    use photon_render::joml::Pose;
    use photon_render::objects::fishing::string_vertices;
    use photon_render::objects::projectile::fishing_hook_quad;
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
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let light = i32_field(row, "light")?;
        let quad = fishing_hook_quad(&base, &orientation);
        let quad_vertices: Vec<String> = quad
            .corners
            .iter()
            .map(|(position, uv)| {
                format!(
                    "{}{}{}{}{}{:08x}{light:08x}{}{}{}",
                    hex(position[0]),
                    hex(position[1]),
                    hex(position[2]),
                    hex(uv[0]),
                    hex(uv[1]),
                    0xFFFF_FFFF_u32,
                    hex(quad.normal[0]),
                    hex(quad.normal[1]),
                    hex(quad.normal[2]),
                )
            })
            .collect();
        let offset = [bits("offset_x")?, bits("offset_y")?, bits("offset_z")?];
        let string: Vec<String> = string_vertices(&base, offset)
            .iter()
            .map(|vertex| {
                format!(
                    "{}{}{}{}{}{:08x}{:08x}{}{}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    hex(0.0),
                    hex(0.0),
                    0xFF00_0000_u32,
                    0,
                    hex(vertex.normal[0]),
                    hex(vertex.normal[1]),
                    hex(vertex.normal[2]),
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "quad": quad_vertices, "string": string}));
    }
    Ok(json!({"rows": out}))
}

/// `avatar_pose`: where a player is put, by the body's turn, the dying, swimming and gliding.
pub(super) fn avatar_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::player_renderer::{FallFlying, Placement, placement_pose};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let flag = |key: &str| row[key].as_bool().unwrap_or(false);
        let float = |key: &str| f32_field(row, key);
        let gliding = flag("fall_flying");
        let placement = Placement {
            feet: [float("feet_x")?, float("feet_y")?, float("feet_z")?],
            body_rot: float("body_rot")?,
            x_rot: float("x_rot")?,
            swim_amount: float("swim_amount")?,
            in_water: flag("in_water"),
            visually_swimming: flag("visually_swimming"),
            crouching: flag("crouching"),
            light: 0,
            skin: [0.0, 0.0, 1.0, 1.0],
            death_time: float("death_time")?,
            red_overlay: false,
            fall_flying: gliding.then(|| {
                let ticks = float("fall_flying_ticks").unwrap_or(0.0);
                FallFlying {
                    scale: (ticks * ticks / 100.0).clamp(0.0, 1.0),
                    y_rot: flag("apply_flying_y_rot").then(|| float("flying_y_rot").unwrap_or(0.0)),
                }
            }),
            auto_spin: flag("spin_attack").then(|| float("age_in_ticks").unwrap_or(0.0)),
        };
        let pose = placement_pose(&placement);
        out.push(json!({"label": row["label"], "pose": matrix_bits(&pose)}));
    }
    Ok(json!({"rows": out}))
}

/// `elytra_model`: the elytra models posed by the wings' spread, drawn.
pub(super) fn elytra_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::MobModel;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let layer =
            photon_data::entity_model::layer(string(row, "layer")?).ok_or("no such layer")?;
        let mut model = MobModel::bake(layer);
        model.setup_elytra(
            [
                f32_field(row, "rot_x")?,
                f32_field(row, "rot_y")?,
                f32_field(row, "rot_z")?,
            ],
            row["crouching"].as_bool().ok_or("crouching")?,
        );
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `elytra_state`: the wings' spread through a scenario, tick by tick.
pub(super) fn elytra_state_result(request: &Value) -> Result<Value, String> {
    use photon_world::elytra::{ElytraInput, ElytraState};
    let double = |tick: &Value, key: &str| -> Result<f64, String> {
        u64::from_str_radix(string(tick, key)?, 16)
            .map(f64::from_bits)
            .map_err(|error| error.to_string())
    };
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut state = ElytraState::default();
        let mut ticks = Vec::new();
        for tick in row["ticks"].as_array().ok_or("ticks")? {
            state.tick(ElytraInput {
                fall_flying: tick["fall_flying"].as_bool().ok_or("fall_flying")?,
                motion: (double(tick, "x")?, double(tick, "y")?, double(tick, "z")?),
                crouching: tick["crouching"].as_bool().ok_or("crouching")?,
            });
            let mut described = serde_json::Map::new();
            for partial in [0.0_f32, 0.5, 1.0] {
                described.insert(
                    format!("at_{partial:?}"),
                    json!(state.rotations(partial).map(hex)),
                );
            }
            ticks.push(Value::Object(described));
        }
        out.push(json!({"label": row["label"], "ticks": ticks}));
    }
    Ok(json!({"rows": out}))
}

/// `leash_strip`: a leash's strip as `LeashFeatureRenderer.prepare` builds it.
pub(super) fn leash_strip_result(request: &Value) -> Result<Value, String> {
    use photon_render::joml::Pose;
    use photon_render::objects::leash::{LeashState, leash_vertices};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let vector = |key: &str| -> Result<[f64; 3], String> {
            let words = row[key]
                .as_array()
                .ok_or(format!("{key} must be an array"))?;
            let mut out = [0.0; 3];
            for (slot, word) in out.iter_mut().zip(words) {
                *slot = u64::from_str_radix(word.as_str().ok_or("word")?, 16)
                    .map(f64::from_bits)
                    .map_err(|error| error.to_string())?;
            }
            Ok(out)
        };
        let mut base = Pose::default();
        base.translate(float("feet_x")?, float("feet_y")?, float("feet_z")?);
        let state = LeashState {
            offset: vector("offset")?,
            start: vector("start")?,
            end: vector("end")?,
            start_block_light: i32_field(row, "start_block")?,
            end_block_light: i32_field(row, "end_block")?,
            start_sky_light: i32_field(row, "start_sky")?,
            end_sky_light: i32_field(row, "end_sky")?,
            slack: row["slack"].as_bool().ok_or("slack")?,
        };
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let strip: Vec<String> = leash_vertices(&base, &state)
            .iter()
            .map(|vertex| {
                let [r, g, b] = vertex.colour.map(u32::from);
                format!(
                    "{}{}{}{}{}{:08x}{:08x}{}{}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    hex(0.0),
                    hex(0.0),
                    0xFF00_0000 | r << 16 | g << 8 | b,
                    vertex.light,
                    hex(0.0),
                    hex(0.0),
                    hex(0.0),
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "strip": strip}));
    }
    Ok(json!({"rows": out}))
}

/// `small_projectile_pose`: the poses a shulker bullet's, llama spit's or evoker fangs' models are drawn from.
pub(super) fn small_projectile_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::joml::Pose;
    use photon_render::objects::projectile::{
        evoker_fangs_pose, llama_spit_pose, shulker_bullet_poses,
    };
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
        let poses: Vec<Pose> = match row["kind"].as_str().ok_or("kind")? {
            "shulker_bullet" => shulker_bullet_poses(&base, bits("age_in_ticks")?).to_vec(),
            "leash_knot" => vec![photon_render::objects::projectile::leash_knot_pose(&base)],
            "llama_spit" => vec![llama_spit_pose(&base, bits("y_rot")?, bits("x_rot")?)],
            // Fangs are not drawn until they bite.
            _ if bits("bite_progress")? == 0.0 => Vec::new(),
            _ => vec![evoker_fangs_pose(&base, bits("y_rot")?)],
        };
        let described: Vec<Value> = poses.iter().map(|pose| json!(matrix_bits(pose))).collect();
        out.push(json!({"label": row["label"], "poses": described}));
    }
    Ok(json!({"rows": out}))
}

/// `dragon_fireball_quad`: the quad a dragon's fireball is drawn as, seen by a camera.
pub(super) fn dragon_fireball_quad_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::name_tag::camera_orientation;
    use photon_render::joml::Pose;
    use photon_render::objects::projectile::dragon_fireball_quad;
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
        let quad = dragon_fireball_quad(&base, &orientation);
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let light = i32_field(row, "light")?;
        let vertices: Vec<String> = quad
            .corners
            .iter()
            .map(|(position, uv)| {
                format!(
                    "{}{}{}{}{}{:08x}{light:08x}{}{}{}",
                    hex(position[0]),
                    hex(position[1]),
                    hex(position[2]),
                    hex(uv[0]),
                    hex(uv[1]),
                    0xFFFF_FFFF_u32,
                    hex(quad.normal[0]),
                    hex(quad.normal[1]),
                    hex(quad.normal[2]),
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `arrow_pose`: the pose an arrow's model is drawn from.
pub(super) fn arrow_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::joml::Pose;
    use photon_render::objects::projectile::arrow_pose;
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
        let pose = if row.get("trident").is_some() {
            photon_render::objects::projectile::trident_pose(&base, bits("y_rot")?, bits("x_rot")?)
        } else {
            arrow_pose(&base, bits("y_rot")?, bits("x_rot")?)
        };
        out.push(json!({"label": row["label"], "poses": [matrix_bits(&pose)]}));
    }
    Ok(json!({"rows": out}))
}

/// `minecart_pose`: the poses a minecart's model and its carried block are drawn from.
pub(super) fn minecart_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::joml::Pose;
    use photon_render::objects::minecart::{
        MinecartRenderState, RailPlace, minecart_poses, tnt_is_lit, tnt_swell_amount,
    };
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
        let double = |value: &Value| -> Result<f64, String> {
            u64::from_str_radix(value.as_str().ok_or("a double must be a string")?, 16)
                .map(f64::from_bits)
                .map_err(|error| error.to_string())
        };
        let vec3 = |value: &Value| -> Result<(f64, f64, f64), String> {
            let parts = value.as_array().ok_or("a vector must be an array")?;
            Ok((double(&parts[0])?, double(&parts[1])?, double(&parts[2])?))
        };
        let rail = match row.get("rail") {
            Some(rail) => Some(RailPlace {
                pos_on_rail: vec3(&rail["pos"])?,
                front: vec3(&rail["front"])?,
                back: vec3(&rail["back"])?,
            }),
            None => None,
        };
        let state = MinecartRenderState {
            position: (double(&row["x"])?, double(&row["y"])?, double(&row["z"])?),
            x_rot: bits("x_rot")?,
            y_rot: bits("y_rot")?,
            offset_seed: row["offset_seed"]
                .as_str()
                .ok_or("offset_seed")?
                .parse()
                .map_err(|error: std::num::ParseIntError| error.to_string())?,
            hurt_dir: i32_field(row, "hurt_dir")?,
            hurt_time: bits("hurt_time")?,
            damage_time: bits("damage_time")?,
            display_offset: i32_field(row, "display_offset")?,
            block_swell: if row.get("tnt_fuse").is_some() {
                let fuse = bits("tnt_fuse")?;
                (fuse > -1.0 && fuse < 10.0).then(|| tnt_swell_amount(fuse))
            } else {
                None
            },
            new_render: row.get("new_render").is_some(),
            rail,
        };
        let mut base = Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let poses = minecart_poses(&base, &state);
        // The block first, if it carries one, then the model.
        let mut described = Vec::new();
        let mut overlays = Vec::new();
        if row["block"].as_bool() == Some(true) {
            described.push(matrix_bits(&poses.block));
            // `OverlayTexture.pack(u, v)`: no overlay is `(0, 10)`, a white flash `(15, 10)`.
            let white = row.get("tnt_fuse").is_some() && tnt_is_lit(bits("tnt_fuse")?);
            overlays.push(if white { (10 << 16) | 15 } else { 10 << 16 });
        }
        described.push(matrix_bits(&poses.model));
        out.push(json!({"label": row["label"], "poses": described, "overlays": overlays}));
    }
    Ok(json!({"rows": out}))
}
