use super::*;

pub(super) fn client_world_entity_mob_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "mob_flags": entity.mob_flags(),
        "is_no_ai": entity.is_no_ai(),
        "is_left_handed": entity.is_left_handed(),
        "is_aggressive": entity.is_aggressive(),
    })
}

pub(super) fn client_world_entity_mob_flags_result(request: &Value) -> Result<Value, String> {
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(i32_field(request, "entity_type_id")?),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("mob flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_mob_flags_state(
        "initial",
        true,
        world
            .entities()
            .at(entity_id)
            .ok_or("mob flags fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
            .ok_or("mob flags fixture entity disappeared")?;
        states.push(client_world_entity_mob_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_arrow_count_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "arrow_count": entity.arrow_count(),
        "living_flags": entity.living_flags(),
    })
}

pub(super) fn client_world_entity_arrow_count_result(request: &Value) -> Result<Value, String> {
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(i32_field(request, "entity_type_id")?),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("arrow count fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_arrow_count_state(
        "initial",
        true,
        world
            .entities()
            .at(entity_id)
            .ok_or("arrow count fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
            .ok_or("arrow count fixture entity disappeared")?;
        states.push(client_world_entity_arrow_count_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_effect_ambience_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "effect_ambience": entity.is_effect_ambient(),
    })
}

pub(super) fn client_world_entity_effect_ambience_result(request: &Value) -> Result<Value, String> {
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(i32_field(request, "entity_type_id")?),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("effect ambience fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_effect_ambience_state(
        "initial",
        true,
        world
            .entities()
            .at(entity_id)
            .ok_or("effect ambience fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
            .ok_or("effect ambience fixture entity disappeared")?;
        states.push(client_world_entity_effect_ambience_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

/// `LivingEntity.tickEffects` on a client, ticked with a seeded source: the particles it adds,
/// in order, with the tick each came on, for an armour stand (half a block wide, 1.975 tall).
fn effect_particle_spawns(spawn: &Value) -> Value {
    let particles: Vec<photon_world::EffectParticle> = spawn["particles"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|particle| photon_world::EffectParticle {
            type_id: particle["type_id"].as_i64().unwrap_or(0) as i32,
            color: particle["color"].as_i64().map(|color| color as u32),
        })
        .collect();
    let flag = |key: &str| spawn[key].as_bool().unwrap_or(false);
    let body = (
        photon_core::math::Vec3::new(1.25, 80.0, -35.5),
        (0.5, 1.975),
    );
    let mut random = photon_core::random::RandomSource::create(spawn["seed"].as_i64().unwrap_or(0));
    let bits = |value: f64| format!("{:016x}", value.to_bits());
    let mut added = Vec::new();
    for tick in 0..spawn["ticks"].as_i64().unwrap_or(0) {
        let flags = (flag("ambient"), flag("invisible"));
        let Some(thrown) =
            photon_world::particles::effect_particle(&particles, flags, body, &mut random)
        else {
            continue;
        };
        let (x, y, z) = thrown.at;
        let push = |spread: f32| f64::from(thrown.max_speed * spread);
        added.push(json!({
            "tick": tick, "type_id": thrown.type_id, "x": bits(x), "y": bits(y), "z": bits(z),
            "xd": bits(push(thrown.spread.0)), "yd": bits(push(thrown.spread.1)),
            "zd": bits(push(thrown.spread.2)),
        }));
    }
    json!({"added": added})
}

pub(super) fn client_world_entity_effect_particles_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "effect_ambience": entity.is_effect_ambient(),
        // Each particle's type and, for the entity effect, its ARGB colour as Java's signed int.
        "effect_particles": entity.effect_particles().iter().map(|particle| {
            let mut described = json!({"type_id": particle.type_id});
            if let Some(color) = particle.color {
                described["color"] = json!(color as i32);
            }
            described
        }).collect::<Vec<_>>(),
    })
}

pub(super) fn client_world_entity_effect_particles_result(
    request: &Value,
) -> Result<Value, String> {
    if let Some(spawn) = request.get("spawn") {
        return Ok(effect_particle_spawns(spawn));
    }
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(i32_field(request, "entity_type_id")?),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("effect particles fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_effect_particles_state(
        "initial",
        true,
        world
            .entities()
            .at(entity_id)
            .ok_or("effect particles fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
            .ok_or("effect particles fixture entity disappeared")?;
        states.push(client_world_entity_effect_particles_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_sleeping_position_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    let mut state = json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "is_sleeping": entity.is_sleeping(),
    });
    if let Some(position) = entity.sleeping_pos() {
        state
            .as_object_mut()
            .expect("sleeping position state object")
            .insert("sleeping_pos".into(), blockpos_snapshot(position));
    }
    state
}

pub(super) fn client_world_entity_sleeping_position_result(
    request: &Value,
) -> Result<Value, String> {
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(i32_field(request, "entity_type_id")?),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("sleeping position fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_sleeping_position_state(
        "initial",
        true,
        world
            .entities()
            .at(entity_id)
            .ok_or("sleeping position fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
            .ok_or("sleeping position fixture entity disappeared")?;
        states.push(client_world_entity_sleeping_position_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_stinger_count_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "stinger_count": entity.stinger_count(),
        "living_flags": entity.living_flags(),
    })
}

pub(super) fn client_world_entity_stinger_count_result(request: &Value) -> Result<Value, String> {
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(i32_field(request, "entity_type_id")?),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("stinger count fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_stinger_count_state(
        "initial",
        true,
        world
            .entities()
            .at(entity_id)
            .ok_or("stinger count fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
            .ok_or("stinger count fixture entity disappeared")?;
        states.push(client_world_entity_stinger_count_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_chunk_cache_snapshot_json(snapshot: &ClientChunkCacheSnapshot) -> Value {
    json!({
        "storage_radius": snapshot.storage_radius,
        "view_range": snapshot.view_range,
        "view_center_x": snapshot.view_center_x,
        "view_center_z": snapshot.view_center_z,
        "loaded_chunks": snapshot.loaded_chunks,
        "added_empty_sections": snapshot.added_empty_sections,
        "removed_empty_sections": snapshot.removed_empty_sections,
        "added_loaded_chunks": snapshot.added_loaded_chunks,
        "removed_loaded_chunks": snapshot.removed_loaded_chunks,
    })
}
