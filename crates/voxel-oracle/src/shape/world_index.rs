use super::*;

pub(super) fn client_world_indexed_collision_case(request: &Value) -> Result<Value, String> {
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
                world.entities_mut().apply_living_collision_state(
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
                world.entities_mut().apply_player_flying_state(
                    entity_id,
                    entity
                        .get("flying")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                );
                if let Some(context) = entity.get("climbable_context").and_then(Value::as_object) {
                    let kind = context
                        .get("kind")
                        .and_then(Value::as_str)
                        .ok_or("climbable context kind must be a string")?;
                    if kind == "trapdoor_ladder" {
                        world.entities_mut().apply_climbable_trapdoor_state(
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
                        world.apply_entity_living_climbable_block_state(entity_id, state_id);
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
                                let id = Identifier::parse(tag_id)
                                    .map_err(|_| "invalid block tag identifier".to_owned())?;
                                Ok(TagEntry::new(id, values))
                            })
                            .collect::<Result<Vec<_>, String>>()?;
                        let registry = Identifier::parse("minecraft:block")
                            .map_err(|_| "invalid block registry identifier".to_owned())?;
                        let packet = ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
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

pub(super) fn client_world_entity_section_index_case(request: &Value) -> Result<Value, String> {
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
                    uuid: Uuid::new(0, u64::from(i32_field(entity_value, "entity_id")? as u32)),
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
                        let position =
                            f64_vec3(event.get("position").ok_or("position must be an object")?)?;
                        let entity_id = i32_field(event_value, "entity_id")?;
                        if !world
                            .update_entity_position(entity_id, (position.x, position.y, position.z))
                        {
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
                        let position =
                            f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
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
                            "entity_count": world.entities().count(),
                            "section_count": world.entities().section_count(),
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
