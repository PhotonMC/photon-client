use super::*;

pub(super) fn client_world_collision_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|entry| {
            let input = entry
                .as_object()
                .ok_or("client world collision input must be an object")?;
            let test_area = aabb_box(
                input
                    .get("test_area")
                    .ok_or("test_area must be an object")?,
            )?;
            let (world, shapes_by_state) = block_collision_fixture(entry)?;
            let source = if input
                .get("source_present")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                Some(ClientWorldCollisionSource {
                    position: f64_vec3(
                        input
                            .get("source_position")
                            .ok_or("source_position must be an object")?,
                    )?,
                })
            } else {
                None
            };
            let border = input.get("border").ok_or("border must be an object")?;
            let size = f64_field(border, "size")?;
            let mut world = world;
            world.apply_world_border_initialize(ClientWorldBorderInitialization {
                center_x: f64_field(border, "center_x")?,
                center_z: f64_field(border, "center_z")?,
                old_size: size,
                new_size: size,
                lerp_time: 0,
                absolute_max_size: i32_field(border, "absolute_max_size")?,
                warning_blocks: 5,
                warning_time: 15,
            });
            let candidates = entity_collision_candidates(entry)?;
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "no_block_collision": world.no_block_collision(
                    test_area,
                    &shapes_by_state,
                ),
                "no_entity_collision": world.no_entity_collision(
                    source.is_some(),
                    test_area,
                    &candidates,
                ),
                "no_border_collision": world.no_border_collision(source, test_area),
                "no_collision": world.no_collision(
                    source,
                    test_area,
                    &shapes_by_state,
                    &candidates,
                ),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn client_world_happy_ghast_collision_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|entry| {
            let input = entry
                .as_object()
                .ok_or("Happy Ghast collision input must be an object")?;
            let test_area = aabb_box(
                input
                    .get("test_area")
                    .ok_or("test_area must be an object")?,
            )?;
            let (mut world, shapes_by_state) = block_collision_fixture(entry)?;
            for entity_value in input
                .get("entities")
                .ok_or("entities must be an array")?
                .as_array()
                .ok_or("entities must be an array")?
            {
                let entity = entity_value
                    .as_object()
                    .ok_or("indexed entity must be an object")?;
                let entity_id = i32_field(entity_value, "entity_id")?;
                let type_id = i32_field(entity_value, "type_id")?;
                let position =
                    f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
                world.apply_entity(EntityInput {
                    entity_id,
                    uuid: Uuid::new(0, u64::from(entity_id as u32)),
                    type_id,
                    x: position.x,
                    y: position.y,
                    z: position.z,
                    movement: (0.0, 0.0, 0.0),
                    rotation: (0.0, 0.0, 0.0),
                    data: 0,
                });
                if type_id == 58 {
                    world.entities_mut().apply_happy_ghast_collision_state(
                        entity_id,
                        entity.get("baby").and_then(Value::as_bool).unwrap_or(false),
                        entity.get("alive").and_then(Value::as_bool).unwrap_or(true),
                        entity
                            .get("still_timeout")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        entity
                            .get("is_vehicle")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    );
                }
            }
            let source_entity_id = input
                .get("source_entity_id")
                .filter(|value| !value.is_null())
                .map(|value| {
                    value
                        .as_i64()
                        .ok_or_else(|| "source_entity_id must be an integer".to_owned())
                        .and_then(|value| {
                            value
                                .try_into()
                                .map_err(|_| "source_entity_id must fit in i32".to_owned())
                        })
                })
                .transpose()?;
            let border = input.get("border").ok_or("border must be an object")?;
            let size = f64_field(border, "size")?;
            world.apply_world_border_initialize(ClientWorldBorderInitialization {
                center_x: f64_field(border, "center_x")?,
                center_z: f64_field(border, "center_z")?,
                old_size: size,
                new_size: size,
                lerp_time: 0,
                absolute_max_size: i32_field(border, "absolute_max_size")?,
                warning_blocks: 5,
                warning_time: 15,
            });
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "entity_count": world.entities().count(),
                "no_entity_collision": world.no_entity_collision_for_entity(
                    source_entity_id,
                    test_area,
                ),
                "no_collision": world.no_collision_for_entity(
                    source_entity_id,
                    test_area,
                    &shapes_by_state,
                ),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn client_world_stair_neighbor_collision_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|entry| {
            let input = entry
                .as_object()
                .ok_or("stair neighbor collision input must be an object")?;
            let target = input
                .get("target")
                .and_then(Value::as_object)
                .ok_or("stair target must be an object")?;
            let x = i32_field(&Value::Object(target.clone()), "x")?;
            let y = i32_field(&Value::Object(target.clone()), "y")?;
            let z = i32_field(&Value::Object(target.clone()), "z")?;
            let mut world = block_collision_world_fixture(entry)?;
            let before_state_id = world
                .block_state_at(x, y, z)
                .ok_or("stair target must be loaded")?;
            if !world.apply_stair_neighbor_shape_update(x, y, z) {
                return Err("stair neighbor shape update was not applicable".to_owned());
            }
            let updated_state_id = world
                .block_state_at(x, y, z)
                .ok_or("updated stair target must remain loaded")?;
            let collision_shape = world
                .block_state_registry()
                .collision_shape(updated_state_id);
            let updated_shape = world
                .block_state_registry()
                .stair_shape_name(updated_state_id)
                .ok_or("updated state must be a supported stair")?;
            let test_area = aabb_box(
                input
                    .get("test_area")
                    .ok_or("test_area must be an object")?,
            )?;
            let collisions = world.collect_registered_block_collision_shapes(test_area);
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "before_state_id": before_state_id,
                "updated_state_id": updated_state_id,
                "updated_shape": updated_shape,
                "updated_collision_shape": shape_snapshot(&collision_shape),
                "collisions": collisions
                    .iter()
                    .map(|collision| shape_snapshot(&collision.shape))
                    .collect::<Vec<_>>(),
                "no_block_collision": collisions.is_empty(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_minor_horizontal_collision_case(
    request: &Value,
) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|entry| {
            let input = entry
                .as_object()
                .ok_or("minor collision input must be an object")?;
            let movement = f64_vec3(input.get("movement").ok_or("movement must be an object")?)?;
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "minor_horizontal_collision": is_minor_horizontal_collision(
                    f32_value(
                        input.get("y_rot").ok_or("y_rot must be an f32 bit string")?,
                        "y_rot",
                    )?,
                    f32_value(input.get("xxa").ok_or("xxa must be an f32 bit string")?, "xxa")?,
                    f32_value(input.get("zza").ok_or("zza must be an f32 bit string")?, "zza")?,
                    movement,
                ),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn client_world_registry_collision_case(request: &Value) -> Result<Value, String> {
    let input = request
        .as_object()
        .ok_or("registry collision input must be an object")?;
    let test_area = aabb_box(
        input
            .get("test_area")
            .ok_or("test_area must be an object")?,
    )?;
    let world = block_collision_world_fixture(request)?;
    let states = input
        .get("state_ids")
        .ok_or("state_ids must be an array")?
        .as_array()
        .ok_or("state_ids must be an array")?
        .iter()
        .map(|state_value| {
            let requested_state_id = state_value
                .as_i64()
                .ok_or("state ID must be an integer")?
                .try_into()
                .map_err(|_| "state ID must fit in i32".to_owned())?;
            let registry = world.block_state_registry();
            let resolved_state_id = registry.resolve_state_id(requested_state_id);
            let collision_shape = registry.collision_shape(requested_state_id);
            Ok(json!({
                "requested_state_id": requested_state_id,
                "resolved_state_id": resolved_state_id,
                "collision_shape": shape_snapshot(&collision_shape),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let collisions = world.collect_registered_block_collision_shapes(test_area);
    Ok(json!({
        "states": states,
        "collisions": collisions
            .iter()
            .map(|collision| shape_snapshot(&collision.shape))
            .collect::<Vec<_>>(),
        "no_block_collision": collisions.is_empty(),
    }))
}
