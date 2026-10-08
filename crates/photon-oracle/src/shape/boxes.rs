use super::*;

pub(super) fn single_case(request: &Value) -> Result<Value, String> {
    let mut value = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let mut events = vec![shape_snapshot(&value)];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        value = match string(step, "op")? {
            "move" => value.move_by(
                f64_field(step, "dx")?,
                f64_field(step, "dy")?,
                f64_field(step, "dz")?,
            ),
            "move_vec3" => value.move_by_vec3(f64_vec3(
                step.get("delta").ok_or("delta must be an object")?,
            )?),
            "move_block_pos" => value.move_by_block_pos(blockpos(
                step.get("delta").ok_or("delta must be an object")?,
            )?),
            _ => return Err("unknown VoxelShape operation".into()),
        };
        events.push(shape_snapshot(&value));
    }
    Ok(json!({"events": events}))
}

pub(super) fn join_case(request: &Value) -> Result<Value, String> {
    let operation = match string(request, "op")? {
        "or" => BooleanOp::Or,
        "and" => BooleanOp::And,
        "only_first" => BooleanOp::OnlyFirst,
        "only_second" => BooleanOp::OnlySecond,
        "false" => BooleanOp::False,
        "not_same" => BooleanOp::NotSame,
        "second" => BooleanOp::Second,
        "first" => BooleanOp::First,
        _ => return Err("unknown VoxelShape boolean operation".into()),
    };
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let joined = Shapes::join(&first, &second, operation);
    Ok(json!({
        "shape": shape_snapshot(&joined),
        "join_is_not_empty": Shapes::join_is_not_empty(&first, &second, operation),
    }))
}

pub(super) fn join_many_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("join_many needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = Shapes::or_many(&first, &tail);
    Ok(json!({"shape": shape_snapshot(&joined)}))
}

pub(super) fn single_encompassing_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("single_encompassing needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = Shapes::or_many(&first, &tail);
    Ok(json!({
        "shape": shape_snapshot(&joined.single_encompassing()),
    }))
}

pub(super) fn optimize_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("optimize needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = tail.iter().fold(first.clone(), |joined, shape| {
        Shapes::join_unoptimized(&joined, shape, BooleanOp::Or)
    });
    Ok(json!({
        "events": [
            shape_snapshot_with_coords(&joined),
            shape_snapshot_with_coords(&joined.optimize()),
        ],
    }))
}

pub(super) fn edges_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("edges needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = Shapes::or_many(&first, &tail);
    let mut edges = Vec::new();
    joined.for_all_edges(|x1, y1, z1, x2, y2, z2| {
        edges.push(vec![
            format!("{:016x}", x1.to_bits()),
            format!("{:016x}", y1.to_bits()),
            format!("{:016x}", z1.to_bits()),
            format!("{:016x}", x2.to_bits()),
            format!("{:016x}", y2.to_bits()),
            format!("{:016x}", z2.to_bits()),
        ]);
    });
    Ok(json!({"edges": edges}))
}

pub(super) fn join_coords_case(request: &Value) -> Result<Value, String> {
    let operation = match string(request, "op")? {
        "or" => BooleanOp::Or,
        "and" => BooleanOp::And,
        "only_first" => BooleanOp::OnlyFirst,
        "only_second" => BooleanOp::OnlySecond,
        "not_same" => BooleanOp::NotSame,
        "second" => BooleanOp::Second,
        "first" => BooleanOp::First,
        _ => return Err("unknown VoxelShape boolean operation".into()),
    };
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let unoptimized = Shapes::join_unoptimized(&first, &second, operation);
    let optimized = Shapes::join(&first, &second, operation);
    Ok(json!({
        "unoptimized": shape_snapshot_with_coords(&unoptimized),
        "optimized": shape_snapshot_with_coords(&optimized),
    }))
}

pub(super) fn boxes_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("boxes needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = Shapes::or_many(&first, &tail);
    let mut boxes = Vec::new();
    joined.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
        boxes.push(vec![
            format!("{:016x}", x1.to_bits()),
            format!("{:016x}", y1.to_bits()),
            format!("{:016x}", z1.to_bits()),
            format!("{:016x}", x2.to_bits()),
            format!("{:016x}", y2.to_bits()),
            format!("{:016x}", z2.to_bits()),
        ]);
    });
    Ok(json!({"boxes": boxes}))
}

pub(super) fn create_aabb_case(request: &Value) -> Result<Value, String> {
    let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
    let shape = Shapes::create_from_aabb(aabb)
        .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
    Ok(json!({"shape": shape_snapshot(&shape)}))
}

pub(super) fn create_coords_case(request: &Value) -> Result<Value, String> {
    let value = request.get("box").ok_or("box must be an object")?;
    let shape = Shapes::create_from_coords(
        f64_field(value, "min_x")?,
        f64_field(value, "min_y")?,
        f64_field(value, "min_z")?,
        f64_field(value, "max_x")?,
        f64_field(value, "max_y")?,
        f64_field(value, "max_z")?,
    );
    Ok(json!({"shape": shape_snapshot(&shape)}))
}

pub(super) fn infinity_case(request: &Value) -> Result<Value, String> {
    let infinity = Shapes::infinity();
    match string(request, "view")? {
        "snapshot" => Ok(json!({"shape": shape_snapshot(&infinity)})),
        "boxes" => {
            let mut boxes = Vec::new();
            infinity.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
                boxes.push(vec![
                    format!("{:016x}", x1.to_bits()),
                    format!("{:016x}", y1.to_bits()),
                    format!("{:016x}", z1.to_bits()),
                    format!("{:016x}", x2.to_bits()),
                    format!("{:016x}", y2.to_bits()),
                    format!("{:016x}", z2.to_bits()),
                ]);
            });
            Ok(json!({"boxes": boxes}))
        }
        "not_same_self" => Ok(json!({
            "shape": shape_snapshot(&Shapes::join(
                &infinity,
                &infinity,
                BooleanOp::NotSame,
            ))
        })),
        _ => Err("unknown infinity view".into()),
    }
}

pub(super) fn constants_case(request: &Value) -> Result<Value, String> {
    match string(request, "name")? {
        "epsilon" => Ok(json!({
            "epsilon": format!("{:016x}", Shapes::EPSILON.to_bits())
        })),
        "big_epsilon" => Ok(json!({
            "big_epsilon": format!("{:016x}", Shapes::BIG_EPSILON.to_bits())
        })),
        "both" => Ok(json!({
            "epsilon": format!("{:016x}", Shapes::EPSILON.to_bits()),
            "big_epsilon": format!("{:016x}", Shapes::BIG_EPSILON.to_bits()),
        })),
        _ => Err("unknown shape constant".into()),
    }
}

pub(super) fn axis_limits_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("axis_limits needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = Shapes::or_many(&first, &tail);
    Ok(shape_axis_limits_snapshot(&joined))
}

pub(super) fn axis_limits_at_case(request: &Value) -> Result<Value, String> {
    let entries = request
        .get("shapes")
        .ok_or("shapes must be an array")?
        .as_array()
        .ok_or("shapes must be an array")?;
    if entries.is_empty() {
        return Err("axis_limits_at needs at least one shape".into());
    }
    let first = shape_box(&entries[0])?;
    let tail = entries[1..]
        .iter()
        .map(shape_box)
        .collect::<Result<Vec<_>, _>>()?;
    let joined = Shapes::or_many(&first, &tail);
    let values = request["queries"]
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query| {
            let axis = Axis::from_name(string(query, "axis")?).ok_or("unknown shape axis")?;
            let b = f64_field(query, "b")?;
            let c = f64_field(query, "c")?;
            let value = if string(query, "op")? == "min" {
                joined.min_at(axis, b, c)
            } else {
                joined.max_at(axis, b, c)
            };
            Ok(format!("{:016x}", value.to_bits()))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"values": values}))
}

pub(super) fn equal_case(request: &Value) -> Result<Value, String> {
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({"equal": Shapes::equal(&first, &second)}))
}

pub(super) fn object_equals_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    let same_geometry = shape_box(request.get("same").ok_or("same must be an object")?)?;
    let different_geometry = shape_box(
        request
            .get("different")
            .ok_or("different must be an object")?,
    )?;
    Ok(json!({
        "self": shape.equals_java(&shape),
        "same_geometry": shape.equals_java(&same_geometry),
        "different_geometry": shape.equals_java(&different_geometry),
    }))
}

pub(super) fn object_equals_constants_case() -> Result<Value, String> {
    let empty = Shapes::empty();
    let block = Shapes::block();
    let infinity = Shapes::infinity();
    let empty_thin = Shapes::box_shape(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)
        .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
    let block_box = Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
        .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
    let infinity_box = Shapes::box_shape(
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::INFINITY,
        f64::INFINITY,
    )
    .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
    Ok(json!({
        "empty_self": empty.equals_java(&Shapes::empty()),
        "empty_thin": empty.equals_java(&empty_thin),
        "empty_move": empty.equals_java(&empty.move_by(1.0, 2.0, 3.0)),
        "block_self": block.equals_java(&Shapes::block()),
        "block_box": block.equals_java(&block_box),
        "block_move": block.equals_java(&block.move_by(0.0, 0.0, 0.0)),
        "infinity_self": infinity.equals_java(&Shapes::infinity()),
        "infinity_box": infinity.equals_java(&infinity_box),
    }))
}

pub(super) fn to_string_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
    Ok(json!({"value": shape.to_string_java()}))
}
