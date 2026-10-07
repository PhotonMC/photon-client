use super::*;

pub(super) fn collide_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
    let distance = f64_field(request, "distance")?;
    Ok(json!({
        "distance": format!("{:016x}", shape.collide(axis, moving, distance).to_bits()),
    }))
}

pub(super) fn collide_many_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
    let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    let shapes = entries
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let distance = f64_field(request, "distance")?;
    Ok(json!({
        "distance": format!(
            "{:016x}",
            Shapes::collide(axis, moving, &shapes, distance).to_bits()
        ),
    }))
}

pub(super) fn collide_movement_case(request: &Value) -> Result<Value, String> {
    let movement = f64_vec3(
        request
            .get("movement")
            .ok_or("movement must be an object")?,
    )?;
    let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
    let shapes = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "movement": vec3_snapshot(collide_entity_movement(movement, moving, &shapes)),
    }))
}

pub(super) fn entity_movement_state_case(request: &Value) -> Result<Value, String> {
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
                .ok_or("movement state input must be an object")?;
            let movement = f64_vec3(input.get("movement").ok_or("movement must be an object")?)?;
            let moving = aabb_box(input.get("moving").ok_or("moving must be an object")?)?;
            let shapes = input
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let state = collide_entity_movement_state(movement, moving, &shapes);
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "movement": vec3_snapshot(state.movement),
                "x_collision": state.x_collision,
                "y_collision": state.y_collision,
                "z_collision": state.z_collision,
                "horizontal_collision": state.horizontal_collision,
                "vertical_collision": state.vertical_collision,
                "vertical_collision_below": state.vertical_collision_below,
                "on_ground": state.on_ground,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn entity_movement_state_retention_case(request: &Value) -> Result<Value, String> {
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
                .ok_or("movement state input must be an object")?;
            let delta = f64_vec3(input.get("delta").ok_or("delta must be an object")?)?;
            let moving = aabb_box(input.get("moving").ok_or("moving must be an object")?)?;
            let shapes = input
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let resolved = collide_entity_movement(delta, moving, &shapes);
            let previous_input = input
                .get("previous")
                .and_then(Value::as_object)
                .ok_or("previous must be an object")?;
            let previous = voxel_world::ClientEntityMovementState {
                movement: Vec3::ZERO,
                x_collision: false,
                y_collision: false,
                z_collision: false,
                horizontal_collision: false,
                vertical_collision: previous_input
                    .get("vertical_collision")
                    .and_then(Value::as_bool)
                    .ok_or("previous.vertical_collision must be a boolean")?,
                vertical_collision_below: previous_input
                    .get("vertical_collision_below")
                    .and_then(Value::as_bool)
                    .ok_or("previous.vertical_collision_below must be a boolean")?,
                on_ground: previous_input
                    .get("on_ground")
                    .and_then(Value::as_bool)
                    .ok_or("previous.on_ground must be a boolean")?,
            };
            let state = apply_entity_movement_state(
                delta,
                resolved,
                previous,
                input
                    .get("authoritative")
                    .and_then(Value::as_bool)
                    .ok_or("authoritative must be a boolean")?,
            );
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "movement": vec3_snapshot(state.movement),
                "x_collision": state.x_collision,
                "y_collision": state.y_collision,
                "z_collision": state.z_collision,
                "horizontal_collision": state.horizontal_collision,
                "vertical_collision": state.vertical_collision,
                "vertical_collision_below": state.vertical_collision_below,
                "on_ground": state.on_ground,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn entity_collision_shapes_case(request: &Value) -> Result<Value, String> {
    let source_present = request
        .get("source_present")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let test_area = aabb_box(
        request
            .get("test_area")
            .ok_or("test_area must be an object")?,
    )?;
    let candidates = entity_collision_candidates(request)?;
    let shapes = collect_entity_collision_shapes(source_present, test_area, &candidates);
    Ok(json!({
        "shapes": shapes.iter().map(shape_snapshot).collect::<Vec<_>>(),
    }))
}

pub(super) fn collect_step_up_heights_case(request: &Value) -> Result<Value, String> {
    let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
    let shapes = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let candidates = collect_candidate_step_up_heights(
        moving,
        &shapes,
        f32_field(request, "max_step_height")?,
        f32_field(request, "step_height_to_skip")?,
    );
    Ok(json!({
        "candidates": candidates
            .into_iter()
            .map(|value| format!("{:08x}", value.to_bits()))
        .collect::<Vec<_>>(),
    }))
}

pub(super) fn select_step_up_movement_case(request: &Value) -> Result<Value, String> {
    let bounding_box = aabb_box(
        request
            .get("bounding_box")
            .ok_or("bounding_box must be an object")?,
    )?;
    let grounded_box = aabb_box(
        request
            .get("grounded_box")
            .ok_or("grounded_box must be an object")?,
    )?;
    let movement = f64_vec3(
        request
            .get("movement")
            .ok_or("movement must be an object")?,
    )?;
    let movement_step = f64_vec3(
        request
            .get("movement_step")
            .ok_or("movement_step must be an object")?,
    )?;
    let shapes = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let selected = select_step_up_movement(
        movement,
        movement_step,
        bounding_box,
        grounded_box,
        &shapes,
        f32_field(request, "max_step_height")?,
        f32_field(request, "step_height_to_skip")?,
    );
    Ok(json!({"movement": vec3_snapshot(selected)}))
}

pub(super) fn collect_block_collisions_case(request: &Value) -> Result<Value, String> {
    let bounding_box = aabb_box(request.get("box").ok_or("box must be an object")?)?;
    let (world, shapes_by_state) = block_collision_fixture(request)?;
    let collisions = world.collect_block_collision_shapes(bounding_box, &shapes_by_state);
    Ok(json!({
        "collisions": collisions
            .iter()
            .map(|collision| json!({
                "position": blockpos_snapshot(collision.position),
                "shape": shape_snapshot(&collision.shape),
            }))
            .collect::<Vec<_>>(),
    }))
}
