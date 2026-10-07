use super::*;

pub(super) fn rotate_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let rotation =
        ShapeRotation::from_name(string(request, "rotation")?).ok_or("unknown shape rotation")?;
    Ok(json!({
        "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
    }))
}

pub(super) fn rotate_center_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let rotation =
        ShapeRotation::from_name(string(request, "rotation")?).ok_or("unknown shape rotation")?;
    let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
    Ok(json!({
        "shape": shape_snapshot(&Shapes::rotate_with_center(&shape, rotation, center)),
    }))
}

pub(super) fn rotate_join_case(request: &Value) -> Result<Value, String> {
    let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
    let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
    let rotation =
        ShapeRotation::from_name(string(request, "rotation")?).ok_or("unknown shape rotation")?;
    let shape = Shapes::or(&first, &second);
    Ok(json!({
        "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
    }))
}

pub(super) fn rotate_horizontal_axis_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let shapes = Shapes::rotate_horizontal_axis(&shape);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(axis, shape)| json!({
                "axis": axis.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_horizontal_axis_center_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
    let shapes = Shapes::rotate_horizontal_axis_with_center(&shape, center);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(axis, shape)| json!({
                "axis": axis.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_all_axis_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let shapes = Shapes::rotate_all_axis(&shape);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(axis, shape)| json!({
                "axis": axis.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_all_axis_center_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
    let shapes = Shapes::rotate_all_axis_with_center(&shape, center);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(axis, shape)| json!({
                "axis": axis.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_horizontal_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let shapes = Shapes::rotate_horizontal(&shape);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(direction, shape)| json!({
                "direction": direction.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_horizontal_initial_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let initial = ShapeRotation::from_name(string(request, "initial")?)
        .ok_or("unknown initial shape rotation")?;
    let shapes = Shapes::rotate_horizontal_with_initial(&shape, initial);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(direction, shape)| json!({
                "direction": direction.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_all_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let shapes = Shapes::rotate_all(&shape);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(direction, shape)| json!({
                "direction": direction.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_all_initial_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let initial = ShapeRotation::from_name(string(request, "initial")?)
        .ok_or("unknown initial shape rotation")?;
    let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
    let shapes = Shapes::rotate_all_with_initial(&shape, initial, center);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(direction, shape)| json!({
                "direction": direction.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_all_center_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
    let shapes = Shapes::rotate_all_with_center(&shape, center);
    Ok(json!({
        "shapes": shapes
            .into_iter()
            .map(|(direction, shape)| json!({
                "direction": direction.name(),
                "shape": shape_snapshot(&shape),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_attach_face_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let faces = Shapes::rotate_attach_face(&shape);
    Ok(json!({
        "faces": faces
            .into_iter()
            .map(|(face, shapes)| json!({
                "face": face.name(),
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    }))
}

pub(super) fn rotate_attach_face_initial_case(request: &Value) -> Result<Value, String> {
    let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
    let initial = ShapeRotation::from_name(string(request, "initial")?)
        .ok_or("unknown initial shape rotation")?;
    let faces = Shapes::rotate_attach_face_with_initial(&shape, initial);
    Ok(json!({
        "faces": faces
            .into_iter()
            .map(|(face, shapes)| json!({
                "face": face.name(),
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    }))
}
