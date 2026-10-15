use super::*;
use photon_core::ids::EntityId;

pub(super) fn observe_player_interpolation(
    trace: &mut PlayConnectionTrace,
    entity_id: EntityId,
    before_position: (f64, f64, f64),
    before_rotation: (f32, f32),
) {
    if !trace.entity_interpolation_observations.is_empty() {
        return;
    }
    let Some(entity) = trace.world.entities().at(entity_id) else {
        return;
    };
    if entity.type_id != 156 {
        return;
    }
    let Some(target_position) = entity.interpolation_target_position() else {
        return;
    };
    let Some(target_rotation) = entity.interpolation_target_rotation() else {
        return;
    };
    if target_position == before_position && target_rotation == before_rotation {
        return;
    }

    let mut positions_after_ticks = Vec::with_capacity(3);
    let mut rotations_after_ticks = Vec::with_capacity(3);
    for _ in 0..3 {
        trace.world.tick();
        let Some(entity) = trace.world.entities().at(entity_id) else {
            return;
        };
        positions_after_ticks.push(entity.position());
        rotations_after_ticks.push((entity.rotation().0, entity.rotation().1));
    }
    let Some(entity) = trace.world.entities().at(entity_id) else {
        return;
    };
    trace
        .entity_interpolation_observations
        .push(EntityInterpolationObservation {
            entity_id,
            entity_type_id: entity.type_id,
            before_position,
            target_position,
            before_rotation,
            target_rotation,
            positions_after_ticks,
            rotations_after_ticks,
            tick_count: entity.tick_count(),
        });
}

pub(super) fn observe_player_motion_interpolation(
    trace: &mut PlayConnectionTrace,
    entity_id: EntityId,
    before_motion: (f64, f64, f64),
    target_motion: (f64, f64, f64),
) {
    if !trace.entity_motion_interpolation_observations.is_empty() {
        return;
    }
    let Some(entity) = trace.world.entities().at(entity_id) else {
        return;
    };
    if entity.type_id != 156
        || before_motion == target_motion
        || entity.motion_interpolation_target() != Some(target_motion)
    {
        return;
    }

    let applied_motion = entity.motion();
    let mut motions_after_ticks = Vec::with_capacity(3);
    for _ in 0..3 {
        trace.world.tick();
        let Some(entity) = trace.world.entities().at(entity_id) else {
            return;
        };
        motions_after_ticks.push(entity.motion());
    }
    let Some(entity) = trace.world.entities().at(entity_id) else {
        return;
    };
    trace
        .entity_motion_interpolation_observations
        .push(EntityMotionInterpolationObservation {
            entity_id,
            entity_type_id: entity.type_id,
            before_motion,
            target_motion,
            applied_motion,
            motions_after_ticks,
            tick_count: entity.tick_count(),
        });
}

pub(super) fn observe_player_position_snap(
    trace: &mut PlayConnectionTrace,
    packet_kind: &'static str,
    entity_id: EntityId,
    before_position: (f64, f64, f64),
    before_rotation: (f32, f32),
) {
    if !trace.entity_position_snap_observations.is_empty() {
        return;
    }
    let Some(entity) = trace.world.entities().at(entity_id) else {
        return;
    };
    if entity.type_id != 156 || entity.interpolation_steps() != 0 {
        return;
    }
    let target_position = entity.position();
    let delta_x = target_position.0 - before_position.0;
    let delta_y = target_position.1 - before_position.1;
    let delta_z = target_position.2 - before_position.2;
    if delta_x * delta_x + delta_y * delta_y + delta_z * delta_z <= 4096.0 {
        return;
    }
    let target_rotation = (entity.rotation().0, entity.rotation().1);
    trace
        .entity_position_snap_observations
        .push(EntityPositionSnapObservation {
            packet_kind,
            entity_id,
            entity_type_id: entity.type_id,
            before_position,
            target_position,
            before_rotation,
            target_rotation,
            applied_position: entity.position(),
            applied_rotation: entity.rotation(),
            previous_position: entity.previous_position(),
            previous_rotation: entity.previous_rotation(),
            interpolation_steps: entity.interpolation_steps(),
        });
}

pub(crate) fn dimension_bounds_for_spawn_info(
    trace: &PlayConnectionTrace,
    spawn_info: &SpawnInfoObservation,
) -> DimensionBounds {
    usize::try_from(spawn_info.dimension_type_id)
        .ok()
        .and_then(|id| trace.dimension_type_registry_bounds.get(id))
        .and_then(|bounds| *bounds)
        .unwrap_or_else(|| DimensionBounds::for_builtin_dimension(&spawn_info.dimension))
}

pub(crate) fn dimension_environment_for_spawn_info(
    trace: &PlayConnectionTrace,
    spawn_info: &SpawnInfoObservation,
) -> DimensionEnvironment {
    usize::try_from(spawn_info.dimension_type_id)
        .ok()
        .and_then(|id| trace.dimension_type_registry_environments.get(id))
        .and_then(|environment| environment.clone())
        .unwrap_or_else(|| DimensionEnvironment::for_builtin_dimension(&spawn_info.dimension))
}

pub(super) fn observe_world_entity_tick(
    trace: &mut PlayConnectionTrace,
    observation: &AddEntityObservation,
) -> Result<(), ConnectionError> {
    let entity = trace
        .world
        .entities()
        .at(observation.entity_id)
        .ok_or(ConnectionError::Codec("world entity tick entity"))?;
    let before_position = entity.position();
    let before_motion = entity.motion();
    let before_rotation = entity.rotation();
    trace.world.tick();
    let entity = trace
        .world
        .entities()
        .at(observation.entity_id)
        .ok_or(ConnectionError::Codec("world entity tick result"))?;
    trace
        .world_entity_tick_observations
        .push(WorldEntityTickObservation {
            entity_id: observation.entity_id,
            entity_type_id: observation.entity_type_id,
            before_position,
            after_position: entity.position(),
            before_motion,
            after_motion: entity.motion(),
            before_rotation,
            after_rotation: entity.rotation(),
            tick_count: entity.tick_count(),
        });
    Ok(())
}

pub(super) fn observe_experience_orb_tick(
    trace: &mut PlayConnectionTrace,
    observation: &AddEntityObservation,
) -> Result<(), ConnectionError> {
    let entity = trace
        .world
        .entities()
        .at(observation.entity_id)
        .ok_or(ConnectionError::Codec("experience orb tick entity"))?;
    let before_position = entity.position();
    let before_motion = entity.motion();
    trace.world.tick();
    let entity = trace
        .world
        .entities()
        .at(observation.entity_id)
        .ok_or(ConnectionError::Codec("experience orb first tick result"))?;
    let after_first_tick_position = entity.position();
    let after_first_tick_motion = entity.motion();
    trace.world.tick();
    let entity = trace
        .world
        .entities()
        .at(observation.entity_id)
        .ok_or(ConnectionError::Codec("experience orb second tick result"))?;
    trace
        .experience_orb_tick_observations
        .push(ExperienceOrbTickObservation {
            entity_id: observation.entity_id,
            entity_type_id: observation.entity_type_id,
            before_position,
            after_first_tick_position,
            after_second_tick_position: entity.position(),
            before_motion,
            after_first_tick_motion,
            after_second_tick_motion: entity.motion(),
            tick_count: entity.tick_count(),
        });
    Ok(())
}
