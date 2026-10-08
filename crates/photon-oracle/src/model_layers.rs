use super::*;

/// `entity_block_layers`: the pose each layer that draws a block on an entity
/// puts it at, from an identity entity pose.
pub(super) fn entity_block_layers_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::block::{
        carried_block_pose, iron_golem_flower_pose, mooshroom_mushroom_poses, snow_golem_head_pose,
    };
    use photon_render::entity::mob_model::{IronGolemState, MobModel};
    use photon_render::joml::Pose;
    let bake = |name: &str| -> Result<MobModel, String> {
        Ok(MobModel::bake(
            photon_data::entity_model::layer(name).ok_or("no such layer")?,
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

/// `swirl_layers`: what the charged creeper's swirl layer submits.
pub(super) fn swirl_layers_result(request: &Value) -> Result<Value, String> {
    use photon_render::render_types::{
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
    use photon_render::render_types::{EYES, EYES_ORDER, NO_OVERLAY, eyes_texture};
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

/// The model layers of the data crate, written as the reference's layers are.
pub(super) fn model_layers_result() -> Value {
    use photon_data::entity_model::{ModelPart, layers};
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let all = |values: &[f32]| values.iter().map(|value| hex(*value)).collect::<Vec<_>>();
    fn part(node: &ModelPart, all: &dyn Fn(&[f32]) -> Vec<String>) -> Value {
        let pose = [
            node.pose.position.as_slice(),
            node.pose.rotation.as_slice(),
            node.pose.scale.as_slice(),
        ]
        .concat();
        json!({
            "name": node.name,
            "pose": all(&pose),
            "cubes": node.cubes.iter().map(|cube| json!({
                "origin": all(&cube.origin),
                "dimensions": all(&cube.dimensions),
                "grow": all(&cube.grow),
                "mirror": cube.mirror,
                "uv": all(&cube.uv),
                "uv_scale": all(&cube.uv_scale),
                "faces": cube.faces.iter().map(|face| face.name()).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "children": node.children.iter().map(|child| part(child, all)).collect::<Vec<_>>(),
        })
    }
    let rows: Vec<Value> = layers()
        .iter()
        .map(|layer| {
            json!({
                "label": layer.id,
                "texture_width": layer.texture_width,
                "texture_height": layer.texture_height,
                "root": part(&layer.root, &all),
            })
        })
        .collect();
    json!({"rows": rows})
}
