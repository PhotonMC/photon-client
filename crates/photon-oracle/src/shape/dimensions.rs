use super::*;

pub(super) fn entity_fit_case(request: &Value) -> Result<Value, String> {
    let position = f64_vec3(
        request
            .get("position")
            .ok_or("position must be an object")?,
    )?;
    let dimension_input = request
        .get("dimensions")
        .ok_or("dimensions must be an object")?;
    let width = f32_field(dimension_input, "width")?;
    let height = f32_field(dimension_input, "height")?;
    let fixed = dimension_input
        .get("fixed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut dimensions = if fixed {
        ClientEntityDimensions::fixed(width, height)
    } else {
        ClientEntityDimensions::scalable(width, height)
    };
    if dimension_input.get("width_scale").is_some() || dimension_input.get("height_scale").is_some()
    {
        let width_scale = dimension_input
            .get("width_scale")
            .map(|_| f32_field(dimension_input, "width_scale"))
            .transpose()?
            .unwrap_or(1.0);
        let height_scale = dimension_input
            .get("height_scale")
            .map(|_| f32_field(dimension_input, "height_scale"))
            .transpose()?
            .unwrap_or(1.0);
        dimensions = dimensions.scale(width_scale, height_scale);
    }
    if dimension_input.get("eye_height").is_some() {
        dimensions = dimensions.with_eye_height(f32_field(dimension_input, "eye_height")?);
    }
    let bounding_box = dimensions.make_bounding_box(position);
    let query_box = bounding_box.deflate(Shapes::EPSILON);
    let (world, shapes_by_state) = block_collision_fixture(request)?;
    Ok(json!({
        "dimensions": entity_dimensions_snapshot(dimensions),
        "box": aabb_snapshot(bounding_box),
        "query": aabb_snapshot(query_box),
        "fits": world.entity_fits_in_blocks(position, dimensions, &shapes_by_state),
    }))
}

pub(super) fn living_entity_dimensions_scale_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|value| {
            let input = value
                .as_object()
                .ok_or("living dimensions input must be an object")?;
            let pose_id = input
                .get("pose_id")
                .and_then(Value::as_i64)
                .ok_or("pose_id must be an integer")?
                .try_into()
                .map_err(|_| "pose_id must fit in i32".to_owned())?;
            let pose = ClientEntityPose::from_id(pose_id);
            let dimension_input = input
                .get("default_dimensions")
                .ok_or("default_dimensions must be an object")?;
            let width = f32_field(dimension_input, "width")?;
            let height = f32_field(dimension_input, "height")?;
            let fixed = dimension_input
                .get("fixed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let mut default_dimensions = if fixed {
                ClientEntityDimensions::fixed(width, height)
            } else {
                ClientEntityDimensions::scalable(width, height)
            };
            if dimension_input.get("eye_height").is_some() {
                default_dimensions =
                    default_dimensions.with_eye_height(f32_field(dimension_input, "eye_height")?);
            }
            let dimensions = living_entity_dimensions(
                pose,
                default_dimensions,
                f32_field(value, "age_scale")?,
                f32_field(value, "entity_scale")?,
            );
            let position = f64_vec3(input.get("position").ok_or("position must be an object")?)?;
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "requested_pose_id": pose_id,
                "resolved_pose_id": pose.id(),
                "dimensions": entity_dimensions_snapshot(dimensions),
                "box": aabb_snapshot(dimensions.make_bounding_box(position)),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn entity_dimensions_refresh_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|value| {
            let input = value
                .as_object()
                .ok_or("dimensions refresh input must be an object")?;
            let dimensions = |name: &str| -> Result<ClientEntityDimensions, String> {
                let dimension_input = input
                    .get(name)
                    .ok_or_else(|| format!("{name} must be an object"))?;
                let width = f32_field(dimension_input, "width")?;
                let height = f32_field(dimension_input, "height")?;
                let fixed = dimension_input
                    .get("fixed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let dimensions = if fixed {
                    ClientEntityDimensions::fixed(width, height)
                } else {
                    ClientEntityDimensions::scalable(width, height)
                };
                Ok(if dimension_input.get("eye_height").is_some() {
                    dimensions.with_eye_height(f32_field(dimension_input, "eye_height")?)
                } else {
                    dimensions
                })
            };
            let old_dimensions = dimensions("old_dimensions")?;
            let new_dimensions = dimensions("new_dimensions")?;
            let position = f64_vec3(input.get("position").ok_or("position must be an object")?)?;
            let refreshed = refresh_entity_dimensions(
                old_dimensions,
                new_dimensions,
                position,
                input
                    .get("level_client_side")
                    .and_then(Value::as_bool)
                    .ok_or("level_client_side must be a boolean")?,
                input
                    .get("first_tick")
                    .and_then(Value::as_bool)
                    .ok_or("first_tick must be a boolean")?,
                input
                    .get("no_physics")
                    .and_then(Value::as_bool)
                    .ok_or("no_physics must be a boolean")?,
                input
                    .get("is_player")
                    .and_then(Value::as_bool)
                    .ok_or("is_player must be a boolean")?,
            );
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "old_dimensions": entity_dimensions_snapshot(old_dimensions),
                "new_dimensions": entity_dimensions_snapshot(refreshed.dimensions),
                "position_after_refresh": vec3_snapshot(position),
                "bounding_box": aabb_snapshot(refreshed.bounding_box),
                "should_fudge_position": refreshed.should_fudge_position,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn player_pose_dimensions_case(request: &Value) -> Result<Value, String> {
    let position = f64_vec3(
        request
            .get("position")
            .ok_or("position must be an object")?,
    )?;
    let pose_ids = request
        .get("pose_ids")
        .ok_or("pose_ids must be an array")?
        .as_array()
        .ok_or("pose_ids must be an array")?;
    let poses = pose_ids
        .iter()
        .map(|value| {
            let requested_id = value
                .as_i64()
                .ok_or_else(|| "pose ID must be an integer".to_owned())?
                .try_into()
                .map_err(|_| "pose ID must fit in i32".to_owned())?;
            let pose = ClientEntityPose::from_id(requested_id);
            let dimensions = pose.player_dimensions();
            Ok(json!({
                "requested_id": requested_id,
                "resolved_id": pose.id(),
                "dimensions": entity_dimensions_snapshot(dimensions),
                "box": aabb_snapshot(dimensions.make_bounding_box(position)),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"poses": poses}))
}

pub(super) fn player_pose_selection_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let selections = inputs
        .iter()
        .map(|value| {
            let input = value
                .as_object()
                .ok_or("pose selection input must be an object")?;
            let boolean = |name: &str| {
                input
                    .get(name)
                    .and_then(Value::as_bool)
                    .ok_or_else(|| format!("{name} must be a boolean"))
            };
            let pose_input = ClientPlayerPoseInput {
                sleeping: boolean("sleeping")?,
                swimming: boolean("swimming")?,
                fall_flying: boolean("fall_flying")?,
                auto_spin_attack: boolean("auto_spin_attack")?,
                shift_key_down: boolean("shift_key_down")?,
                flying: boolean("flying")?,
                spectator: boolean("spectator")?,
                passenger: boolean("passenger")?,
                swimming_fits: boolean("swimming_fits")?,
                desired_fits: boolean("desired_fits")?,
                crouching_fits: boolean("crouching_fits")?,
            };
            let mut selection = json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "desired_id": pose_input.desired_pose().id(),
            });
            if let Some(actual_pose) = pose_input.updated_pose() {
                selection["actual_id"] = json!(actual_pose.id());
            }
            Ok(selection)
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"selections": selections}))
}
