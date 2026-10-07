use super::*;

pub(super) fn shape_result(request: &Value) -> Result<Value, String> {
    match string(request, "kind")? {
        "player_abilities_packet" => {
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
                        .ok_or("player abilities input must be an object")?;
                    let bytes = decode_hex(
                        input
                            .get("hex")
                            .and_then(Value::as_str)
                            .ok_or("hex must be a string")?,
                    )?;
                    let packet = parse_player_abilities_packet(&bytes)
                        .map_err(|error| format!("player abilities packet: {error:?}"))?;
                    let abilities = ClientPlayerAbilities::new(
                        packet.invulnerable,
                        packet.flying,
                        packet.can_fly,
                        packet.can_instabuild,
                        packet.flying_speed,
                        packet.walking_speed,
                    );
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "invulnerable": abilities.invulnerable(),
                        "flying": abilities.flying(),
                        "may_fly": abilities.may_fly(),
                        "instabuild": abilities.instabuild(),
                        "flying_speed": format!("{:08x}", abilities.flying_speed().to_bits()),
                        "walking_speed": format!("{:08x}", abilities.walking_speed().to_bits()),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "player_game_mode_event" => {
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
                        .ok_or("player game mode input must be an object")?;
                    let bytes = decode_hex(
                        input
                            .get("hex")
                            .and_then(Value::as_str)
                            .ok_or("hex must be a string")?,
                    )?;
                    let packet = parse_game_event_packet(&bytes)
                        .map_err(|error| format!("game event packet: {error:?}"))?;
                    if packet.event_id != 3 {
                        return Err("expected change-game-mode event".into());
                    }
                    let flying_speed = input
                        .get("initial_flying_speed")
                        .map(|value| f32_value(value, "initial_flying_speed"))
                        .transpose()?
                        .unwrap_or(0.05);
                    let walking_speed = input
                        .get("initial_walking_speed")
                        .map(|value| f32_value(value, "initial_walking_speed"))
                        .transpose()?
                        .unwrap_or(0.1);
                    let abilities = ClientPlayerAbilities::new(
                        false,
                        input
                            .get("initial_flying")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        false,
                        false,
                        flying_speed,
                        walking_speed,
                    );
                    let mode = ClientPlayerGameMode::from_game_event_param(packet.param);
                    let abilities = mode.update_player_abilities(abilities);
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "param": format!("{:08x}", packet.param.to_bits()),
                        "mode_id": mode.id(),
                        "mode_name": mode.name(),
                        "is_spectator": mode.is_spectator(),
                        "is_creative": mode.is_creative(),
                        "is_survival": mode.is_survival(),
                        "may_build": abilities.may_build(),
                        "invulnerable": abilities.invulnerable(),
                        "flying": abilities.flying(),
                        "may_fly": abilities.may_fly(),
                        "instabuild": abilities.instabuild(),
                        "flying_speed": format!("{:08x}", abilities.flying_speed().to_bits()),
                        "walking_speed": format!("{:08x}", abilities.walking_speed().to_bits()),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "single" => {
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
        "join" => {
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
        "join_many" => {
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
        "single_encompassing" => {
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
        "optimize" => {
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
        "edges" => {
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
        "join_coords" => {
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
        "boxes" => {
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
        "create_aabb" => {
            let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
            let shape = Shapes::create_from_aabb(aabb)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            Ok(json!({"shape": shape_snapshot(&shape)}))
        }
        "create_coords" => {
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
        "infinity" => {
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
        "constants" => match string(request, "name")? {
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
        },
        "axis_limits" => {
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
        "axis_limits_at" => {
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
                    let axis =
                        Axis::from_name(string(query, "axis")?).ok_or("unknown shape axis")?;
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
        "equal" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            Ok(json!({"equal": Shapes::equal(&first, &second)}))
        }
        "object_equals" => {
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
        "object_equals_constants" => {
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
        "to_string" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            Ok(json!({"value": shape.to_string_java()}))
        }
        "rotate" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
            }))
        }
        "rotate_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate_with_center(&shape, rotation, center)),
            }))
        }
        "rotate_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
            }))
        }
        "rotate_horizontal_axis" => {
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
        "rotate_horizontal_axis_center" => {
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
        "rotate_all_axis" => {
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
        "rotate_all_axis_center" => {
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
        "rotate_horizontal" => {
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
        "rotate_horizontal_initial" => {
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
        "rotate_all" => {
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
        "rotate_all_initial" => {
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
        "rotate_all_center" => {
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
        "rotate_attach_face" => {
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
        "rotate_attach_face_initial" => {
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
        "collide" => {
            let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let distance = f64_field(request, "distance")?;
            Ok(json!({
                "distance": format!("{:016x}", shape.collide(axis, moving, distance).to_bits()),
            }))
        }
        "collide_many" => {
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
        "collide_movement" => {
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
        "entity_movement_state" => {
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
                    let movement =
                        f64_vec3(input.get("movement").ok_or("movement must be an object")?)?;
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
        "entity_movement_state_retention" => {
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
        "entity_collision_shapes" => {
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
        "client_world_collision" => {
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
        "client_world_indexed_collision" => {
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
                        .ok_or("indexed collision input must be an object")?;
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
                        let position =
                            f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
                        world.apply_entity(EntityInput {
                            entity_id,
                            uuid: Uuid::new(0, u64::from(entity_id as u32)),
                            type_id: i32_field(entity_value, "type_id")?,
                            x: position.x,
                            y: position.y,
                            z: position.z,
                            movement: (0.0, 0.0, 0.0),
                            rotation: (0.0, 0.0, 0.0),
                            data: 0,
                        });
                        world.apply_entity_living_collision_state(
                            entity_id,
                            entity.get("alive").and_then(Value::as_bool).unwrap_or(true),
                            entity
                                .get("spectator")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                            entity
                                .get("on_climbable")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                        );
                        world.apply_entity_player_flying_state(
                            entity_id,
                            entity
                                .get("flying")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                        );
                        if let Some(context) =
                            entity.get("climbable_context").and_then(Value::as_object)
                        {
                            let kind = context
                                .get("kind")
                                .and_then(Value::as_str)
                                .ok_or("climbable context kind must be a string")?;
                            if kind == "trapdoor_ladder" {
                                world.apply_entity_living_climbable_trapdoor_state(
                                    entity_id,
                                    context
                                        .get("trapdoor_open")
                                        .and_then(Value::as_bool)
                                        .unwrap_or(false),
                                    context
                                        .get("trapdoor_facing")
                                        .and_then(Value::as_str)
                                        .ok_or("trapdoor_facing must be a string")?,
                                    context
                                        .get("below_is_ladder")
                                        .and_then(Value::as_bool)
                                        .unwrap_or(true),
                                    context
                                        .get("ladder_facing")
                                        .and_then(Value::as_str)
                                        .ok_or("ladder_facing must be a string")?,
                                );
                            } else if kind == "climbable_tag" {
                                let state_id = context
                                    .get("state_id")
                                    .and_then(Value::as_i64)
                                    .ok_or("climbable state_id must be an integer")?
                                    .try_into()
                                    .map_err(|_| "climbable state_id must fit in i32")?;
                                world
                                    .apply_entity_living_climbable_block_state(entity_id, state_id);
                            } else if kind == "glide_through_tag" {
                                let state_id = context
                                    .get("state_id")
                                    .and_then(Value::as_i64)
                                    .ok_or("glide-through state_id must be an integer")?
                                    .try_into()
                                    .map_err(|_| "glide-through state_id must fit in i32")?;
                                let fall_flying = context
                                    .get("fall_flying")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false);
                                world.apply_entity_living_climbable_glide_through_state(
                                    entity_id,
                                    state_id,
                                    fall_flying,
                                );
                            } else if kind == "update_tags_packet" {
                                let state_id = context
                                    .get("state_id")
                                    .and_then(Value::as_i64)
                                    .ok_or("update-tags state_id must be an integer")?
                                    .try_into()
                                    .map_err(|_| "update-tags state_id must fit in i32")?;
                                let block_tags = context
                                    .get("block_tags")
                                    .and_then(Value::as_object)
                                    .ok_or("block_tags must be an object")?;
                                let tags = block_tags
                                    .iter()
                                    .map(|(tag_id, values)| {
                                        let values = values
                                            .as_array()
                                            .ok_or("block tag values must be an array")?
                                            .iter()
                                            .map(|value| {
                                                value
                                                    .as_i64()
                                                    .ok_or("block tag value must be an integer")?
                                                    .try_into()
                                                    .map_err(|_| {
                                                        "block tag value must fit in i32".to_owned()
                                                    })
                                            })
                                            .collect::<Result<Vec<i32>, String>>()?;
                                        let id = Identifier::parse(tag_id).map_err(|_| {
                                            "invalid block tag identifier".to_owned()
                                        })?;
                                        Ok(TagEntry::new(id, values))
                                    })
                                    .collect::<Result<Vec<_>, String>>()?;
                                let registry = Identifier::parse("minecraft:block")
                                    .map_err(|_| "invalid block registry identifier".to_owned())?;
                                let packet =
                                    ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
                                        registry, tags,
                                    )]);
                                world.apply_update_tags_packet(&packet);
                                let fall_flying = context
                                    .get("fall_flying")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false);
                                world.apply_entity_living_climbable_glide_through_state(
                                    entity_id,
                                    state_id,
                                    fall_flying,
                                );
                            } else {
                                return Err("unknown climbable context kind".to_owned());
                            }
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
                        "entity_count": world.loaded_entity_count(),
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
        "client_world_happy_ghast_collision" => {
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
                            world.apply_happy_ghast_collision_state(
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
                        "entity_count": world.loaded_entity_count(),
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
        "client_world_entity_section_index" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let tracks = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("entity section input must be an object")?;
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
                        let position =
                            f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
                        world.apply_entity(EntityInput {
                            entity_id: i32_field(entity_value, "entity_id")?,
                            uuid: Uuid::new(
                                0,
                                u64::from(i32_field(entity_value, "entity_id")? as u32),
                            ),
                            type_id: i32_field(entity_value, "type_id")?,
                            x: position.x,
                            y: position.y,
                            z: position.z,
                            movement: (0.0, 0.0, 0.0),
                            rotation: (0.0, 0.0, 0.0),
                            data: 0,
                        });
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
                    let mut states = Vec::new();
                    for event_value in input
                        .get("events")
                        .ok_or("events must be an array")?
                        .as_array()
                        .ok_or("events must be an array")?
                    {
                        let event = event_value
                            .as_object()
                            .ok_or("entity section event must be an object")?;
                        match event
                            .get("op")
                            .and_then(Value::as_str)
                            .ok_or("entity section event op must be a string")?
                        {
                            "move" => {
                                let position = f64_vec3(
                                    event.get("position").ok_or("position must be an object")?,
                                )?;
                                let entity_id = i32_field(event_value, "entity_id")?;
                                if !world.update_entity_position(
                                    entity_id,
                                    (position.x, position.y, position.z),
                                ) {
                                    return Err("cannot move missing entity".into());
                                }
                            }
                            "remove" => {
                                world.remove_entity(i32_field(event_value, "entity_id")?);
                            }
                            "add" => {
                                let entity_value =
                                    event.get("entity").ok_or("add entity must be an object")?;
                                let entity = entity_value
                                    .as_object()
                                    .ok_or("add entity must be an object")?;
                                let position = f64_vec3(
                                    entity.get("position").ok_or("position must be an object")?,
                                )?;
                                let entity_id = i32_field(entity_value, "entity_id")?;
                                world.apply_entity(EntityInput {
                                    entity_id,
                                    uuid: Uuid::new(0, u64::from(entity_id as u32)),
                                    type_id: i32_field(entity_value, "type_id")?,
                                    x: position.x,
                                    y: position.y,
                                    z: position.z,
                                    movement: (0.0, 0.0, 0.0),
                                    rotation: (0.0, 0.0, 0.0),
                                    data: 0,
                                });
                            }
                            "query" => {
                                let test_area = aabb_box(
                                    event
                                        .get("test_area")
                                        .ok_or("test_area must be an object")?,
                                )?;
                                states.push(json!({
                                    "label": event
                                        .get("label")
                                        .and_then(Value::as_str)
                                        .ok_or("label must be a string")?,
                                    "entity_count": world.loaded_entity_count(),
                                    "section_count": world.indexed_entity_section_count(),
                                    "no_entity_collision": world.no_entity_collision_for_entity(
                                        source_entity_id,
                                        test_area,
                                    ),
                                    "no_collision": world.no_collision_for_entity(
                                        source_entity_id,
                                        test_area,
                                        &shapes_by_state,
                                    ),
                                }));
                            }
                            _ => return Err("unknown entity section event".into()),
                        }
                    }
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "states": states,
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"tracks": tracks}))
        }
        "client_world_stair_neighbor_collision" => {
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
        "local_player_minor_horizontal_collision" => {
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
                    let movement = f64_vec3(
                        input
                            .get("movement")
                            .ok_or("movement must be an object")?,
                    )?;
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
        "client_world_registry_collision" => {
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
        "world_border_collision_shape" => {
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
                        .ok_or("world border input must be an object")?;
                    let center_x = f64_value(
                        input
                            .get("center_x")
                            .ok_or("center_x must be a bit string")?,
                        "center_x",
                    )?;
                    let center_z = f64_value(
                        input
                            .get("center_z")
                            .ok_or("center_z must be a bit string")?,
                        "center_z",
                    )?;
                    let size = f64_value(
                        input.get("size").ok_or("size must be a bit string")?,
                        "size",
                    )?;
                    let absolute_max_size = input
                        .get("absolute_max_size")
                        .and_then(Value::as_i64)
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or("absolute_max_size must be an i32")?;
                    Ok(shape_snapshot(&world_border_collision_shape(
                        center_x,
                        center_z,
                        size,
                        absolute_max_size,
                    )))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "world_border_collision_shape_moving" => {
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
                        .ok_or("world border input must be an object")?;
                    let center_x = f64_value(
                        input
                            .get("center_x")
                            .ok_or("center_x must be a bit string")?,
                        "center_x",
                    )?;
                    let center_z = f64_value(
                        input
                            .get("center_z")
                            .ok_or("center_z must be a bit string")?,
                        "center_z",
                    )?;
                    let from = f64_value(
                        input.get("from").ok_or("from must be a bit string")?,
                        "from",
                    )?;
                    let to = f64_value(
                        input.get("to").ok_or("to must be a bit string")?,
                        "to",
                    )?;
                    let absolute_max_size = input
                        .get("absolute_max_size")
                        .and_then(Value::as_i64)
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or("absolute_max_size must be an i32")?;
                    let duration = input
                        .get("duration")
                        .and_then(Value::as_i64)
                        .ok_or("duration must be an i64")?;
                    let tick_count = input
                        .get("tick_count")
                        .and_then(Value::as_i64)
                        .filter(|value| *value >= 0)
                        .ok_or("tick_count must be a non-negative i64")?;
                    let partial_ticks = input
                        .get("partial_ticks")
                        .ok_or("partial_ticks must be an array")?
                        .as_array()
                        .ok_or("partial_ticks must be an array")?
                        .iter()
                        .map(|value| f32_value(value, "partial_tick"))
                        .collect::<Result<Vec<_>, String>>()?;
                    let mut border = ClientWorldBorder::new(
                        center_x,
                        center_z,
                        absolute_max_size,
                        from,
                    );
                    border.lerp_size_between(from, to, duration);
                    let mut states = Vec::new();
                    for tick in 0..=tick_count {
                        let samples = partial_ticks
                            .iter()
                            .map(|partial_tick| {
                                json!({
                                    "partial_tick": format!("{:08x}", partial_tick.to_bits()),
                                    "min_x": format!("{:016x}", border.min_x(*partial_tick).to_bits()),
                                    "max_x": format!("{:016x}", border.max_x(*partial_tick).to_bits()),
                                    "min_z": format!("{:016x}", border.min_z(*partial_tick).to_bits()),
                                    "max_z": format!("{:016x}", border.max_z(*partial_tick).to_bits()),
                                })
                            })
                            .collect::<Vec<_>>();
                        let status = match border.status() {
                            ClientWorldBorderStatus::Stationary => "STATIONARY",
                            ClientWorldBorderStatus::Shrinking => "SHRINKING",
                            ClientWorldBorderStatus::Growing => "GROWING",
                        };
                        states.push(json!({
                            "size": format!("{:016x}", border.size().to_bits()),
                            "lerp_time": border.lerp_time(),
                            "lerp_target": format!("{:016x}", border.lerp_target().to_bits()),
                            "status": status,
                            "samples": samples,
                            "collision_shape": shape_snapshot(&border.collision_shape()),
                        }));
                        if tick < tick_count {
                            border.tick();
                        }
                    }
                    Ok(json!(states))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "client_world_border_state" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let tracks = inputs
                .iter()
                .map(|entry| {
                    let events = entry
                        .get("events")
                        .ok_or("events must be an array")?
                        .as_array()
                        .ok_or("events must be an array")?;
                    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
                    let mut states = Vec::new();
                    for event in events {
                        let operation = string(event, "op")?;
                        match operation {
                            "initialize" => world.apply_world_border_initialize(
                                ClientWorldBorderInitialization {
                                    center_x: f64_field(event, "center_x")?,
                                    center_z: f64_field(event, "center_z")?,
                                    old_size: f64_field(event, "old_size")?,
                                    new_size: f64_field(event, "new_size")?,
                                    lerp_time: i64_field(event, "lerp_time")?,
                                    absolute_max_size: i32_field(event, "absolute_max_size")?,
                                    warning_blocks: i32_field(event, "warning_blocks")?,
                                    warning_time: i32_field(event, "warning_time")?,
                                },
                            ),
                            "center" => world.apply_world_border_center(
                                f64_field(event, "center_x")?,
                                f64_field(event, "center_z")?,
                            ),
                            "lerp" => world.apply_world_border_lerp_size(
                                f64_field(event, "old_size")?,
                                f64_field(event, "new_size")?,
                                i64_field(event, "lerp_time")?,
                            ),
                            "size" => world.apply_world_border_size(f64_field(event, "size")?),
                            "warning_blocks" => {
                                world.apply_world_border_warning_blocks(i32_field(event, "value")?)
                            }
                            "warning_time" => {
                                world.apply_world_border_warning_time(i32_field(event, "value")?)
                            }
                            "tick" => world.tick(),
                            _ => return Err(format!("unknown world border event: {operation}")),
                        }
                        states.push(client_world_border_snapshot(
                            operation,
                            world.game_time(),
                            world.world_border(),
                        ));
                    }
                    Ok(json!(states))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"tracks": tracks}))
        }
        "collect_step_up_heights" => {
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
        "select_step_up_movement" => {
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
        "collect_block_collisions" => {
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
        "entity_fit" => {
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
            if dimension_input.get("width_scale").is_some()
                || dimension_input.get("height_scale").is_some()
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
        "living_entity_dimensions_scale" => {
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
                        default_dimensions = default_dimensions
                            .with_eye_height(f32_field(dimension_input, "eye_height")?);
                    }
                    let dimensions = living_entity_dimensions(
                        pose,
                        default_dimensions,
                        f32_field(value, "age_scale")?,
                        f32_field(value, "entity_scale")?,
                    );
                    let position =
                        f64_vec3(input.get("position").ok_or("position must be an object")?)?;
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
        "entity_dimensions_refresh" => {
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
                    let position =
                        f64_vec3(input.get("position").ok_or("position must be an object")?)?;
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
        "player_pose_dimensions" => {
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
        "player_pose_selection" => {
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
        "clip" => {
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
        "clip_join" => {
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
        "closest" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
            Ok(match shape.closest_point_to(point) {
                Some(point) => json!({"point": vec3_snapshot(point)}),
                None => json!({}),
            })
        }
        "closest_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let shape = Shapes::join(&first, &second, BooleanOp::Or);
            let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
            Ok(match shape.closest_point_to(point) {
                Some(point) => json!({"point": vec3_snapshot(point)}),
                None => json!({}),
            })
        }
        "face" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown face direction")?;
            Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
        }
        "face_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown face direction")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
        }
        "block_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown occlusion direction")?;
            Ok(json!({
                "occludes": Shapes::block_occludes(&shape, &occluder, direction),
            }))
        }
        "block_occludes_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown occlusion direction")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({
                "occludes": Shapes::block_occludes(&shape, &occluder, direction),
            }))
        }
        "merged_occludes" => {
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
        "face_occludes" => {
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
        _ => Err("unknown VoxelShape kind".into()),
    }
}
