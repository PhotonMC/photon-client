use super::*;

pub(super) fn clip_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let queries = request
        .get("queries")
        .ok_or("queries must be an array")?
        .as_array()
        .ok_or("queries must be an array")?;
    let mut events = Vec::with_capacity(queries.len());
    for query in queries {
        let from = f64_vec3(query.get("from").ok_or("from must be an object")?)?;
        let to = f64_vec3(query.get("to").ok_or("to must be an object")?)?;
        let pos = blockpos(query.get("pos").ok_or("pos must be an object")?)?;
        events.push(block_hit_result_snapshot(shape.clip(from, to, pos)));
    }
    Ok(json!({"events": events}))
}

pub(super) fn clip_join_case(request: &Value) -> Result<Value, String> {
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let shape = Shapes::join(&first, &second, BooleanOp::Or);
    let queries = request
        .get("queries")
        .ok_or("queries must be an array")?
        .as_array()
        .ok_or("queries must be an array")?;
    let mut events = Vec::with_capacity(queries.len());
    for query in queries {
        let from = f64_vec3(query.get("from").ok_or("from must be an object")?)?;
        let to = f64_vec3(query.get("to").ok_or("to must be an object")?)?;
        let pos = blockpos(query.get("pos").ok_or("pos must be an object")?)?;
        events.push(block_hit_result_snapshot(shape.clip(from, to, pos)));
    }
    Ok(json!({"events": events}))
}

pub(super) fn closest_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
    Ok(match shape.closest_point_to(point) {
        Some(point) => json!({"point": vec3_snapshot(point)}),
        None => json!({}),
    })
}

pub(super) fn closest_join_case(request: &Value) -> Result<Value, String> {
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let shape = Shapes::join(&first, &second, BooleanOp::Or);
    let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
    Ok(match shape.closest_point_to(point) {
        Some(point) => json!({"point": vec3_snapshot(point)}),
        None => json!({}),
    })
}

pub(super) fn face_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown face direction")?;
    Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
}

pub(super) fn face_join_case(request: &Value) -> Result<Value, String> {
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown face direction")?;
    let shape = Shapes::or(&first, &second);
    Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
}

pub(super) fn block_occludes_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let occluder = shape_box(
        request
            .get("occluder")
            .ok_or("occluder must be an object")?,
    )?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown occlusion direction")?;
    Ok(json!({
        "occludes": Shapes::block_occludes(&shape, &occluder, direction),
    }))
}

pub(super) fn block_occludes_join_case(request: &Value) -> Result<Value, String> {
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let occluder = shape_box(
        request
            .get("occluder")
            .ok_or("occluder must be an object")?,
    )?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown occlusion direction")?;
    let shape = Shapes::or(&first, &second);
    Ok(json!({
        "occludes": Shapes::block_occludes(&shape, &occluder, direction),
    }))
}

pub(super) fn merged_occludes_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let occluder = shape_box(
        request
            .get("occluder")
            .ok_or("occluder must be an object")?,
    )?;
    let direction = Direction::from_name(string(request, "direction")?)
        .ok_or("unknown merged occlusion direction")?;
    Ok(json!({
        "occludes": Shapes::merged_face_occludes(&shape, &occluder, direction),
    }))
}

pub(super) fn face_occludes_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let occluder = shape_box(
        request
            .get("occluder")
            .ok_or("occluder must be an object")?,
    )?;
    Ok(json!({
        "occludes": Shapes::face_shape_occludes(&shape, &occluder),
    }))
}
