use super::*;

/// Resolve Java `LivingEntity.getDimensions` from default dimensions and scales.
pub fn living_entity_dimensions(
    pose: ClientEntityPose,
    default_dimensions: ClientEntityDimensions,
    age_scale: f32,
    entity_scale: f32,
) -> ClientEntityDimensions {
    if pose == ClientEntityPose::Sleeping {
        ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(0.2)
    } else {
        default_dimensions
            .scale(age_scale, age_scale)
            .scale(entity_scale, entity_scale)
    }
}

/// Refresh dimensions at the current position and evaluate Java's resize gate.
pub fn refresh_entity_dimensions(
    old_dimensions: ClientEntityDimensions,
    new_dimensions: ClientEntityDimensions,
    position: Vec3,
    level_is_client_side: bool,
    first_tick: bool,
    no_physics: bool,
    is_player: bool,
) -> ClientEntityDimensionsRefresh {
    let is_small = new_dimensions.width() <= 4.0 && new_dimensions.height() <= 4.0;
    let grew = new_dimensions.width() > old_dimensions.width()
        || new_dimensions.height() > old_dimensions.height();
    ClientEntityDimensionsRefresh {
        dimensions: new_dimensions,
        bounding_box: new_dimensions.make_bounding_box(position),
        should_fudge_position: !level_is_client_side
            && !first_tick
            && !no_physics
            && is_small
            && grew
            && !is_player,
    }
}

pub(crate) fn shared_flag_is_set(flags: u8, flag: u8) -> bool {
    flags & (1_u8 << flag) != 0
}

pub(crate) fn advance_head_rotation(current: &mut JavaF32, target: JavaF32, steps: &mut u8) {
    if *steps == 0 {
        return;
    }
    let step_count = f64::from(*steps);
    let from = f64::from(current.get());
    let to = f64::from(target.get());
    *current = JavaF32::new(mth_rot_lerp_f64(1.0 / step_count, from, to) as f32);
    *steps -= 1;
}

pub(crate) fn vanilla_boat_passenger_rotation_clamped_values(
    passenger_y_rot: f32,
    passenger_old_y_rot: f32,
    vehicle_y_rot: f32,
) -> (f32, f32) {
    let delta = wrap_degrees_f32(passenger_y_rot - vehicle_y_rot);
    let target_delta = clamp_f32(
        delta,
        -VANILLA_BOAT_PASSENGER_MAX_ROTATION,
        VANILLA_BOAT_PASSENGER_MAX_ROTATION,
    );
    (
        passenger_old_y_rot + target_delta - delta,
        passenger_y_rot + target_delta - delta,
    )
}

pub(crate) fn vanilla_passenger_can_turn_in_boat(passenger_type_id: i32) -> bool {
    passenger_type_id == VANILLA_BREEZE_ENTITY_TYPE_ID
}

pub(crate) fn vanilla_passenger_is_animal(passenger_type_id: i32) -> bool {
    VANILLA_ANIMAL_ENTITY_TYPE_IDS.contains(&passenger_type_id)
}

pub(super) fn armor_stand_type_id() -> i32 {
    voxel_data::entity_type::id("minecraft:armor_stand").unwrap_or(-1)
}

pub(crate) fn vanilla_passenger_is_living(passenger_type_id: i32) -> bool {
    VANILLA_LIVING_ENTITY_TYPE_IDS.contains(&passenger_type_id)
}

pub(crate) fn vanilla_passenger_vehicle_attachment_y(passenger_type_id: i32, is_baby: bool) -> f64 {
    match passenger_type_id {
        VANILLA_PLAYER_ENTITY_TYPE_ID => VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y,
        VANILLA_SPIDER_ENTITY_TYPE_ID => VANILLA_SPIDER_VEHICLE_ATTACHMENT_Y,
        VANILLA_ZOMBIE_ENTITY_TYPE_ID if is_baby => 0.1875,
        _ => 0.0,
    }
}

pub(crate) fn vanilla_boat_max_passengers(vehicle_type_id: i32) -> Option<usize> {
    VANILLA_BOAT_PASSENGER_GEOMETRY
        .iter()
        .any(|&(type_id, _, _)| type_id == vehicle_type_id)
        .then(|| {
            if VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS.contains(&vehicle_type_id) {
                1
            } else {
                2
            }
        })
}

pub(crate) fn is_vanilla_boat_entity_type(entity_type_id: i32) -> bool {
    VANILLA_BOAT_PASSENGER_GEOMETRY
        .iter()
        .any(|&(type_id, _, _)| type_id == entity_type_id)
}

/// `Entity.blockPosition`: the block a position is in.
pub(super) fn block_of(position: (f64, f64, f64)) -> [i32; 3] {
    [position.0, position.1, position.2].map(|value| value.floor() as i32)
}

pub(crate) fn is_vanilla_shulker_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_SHULKER_ENTITY_TYPE_ID
}

/// Which display a type is, if it is one (`Display`).
pub(crate) fn display_kind_of(entity_type_id: i32) -> Option<crate::display::DisplayKind> {
    use crate::display::DisplayKind;
    match voxel_data::entity_type::name(entity_type_id)? {
        "minecraft:block_display" => Some(DisplayKind::Block),
        "minecraft:item_display" => Some(DisplayKind::Item),
        "minecraft:text_display" => Some(DisplayKind::Text),
        _ => None,
    }
}

pub(crate) fn is_vanilla_minecart_entity_type(entity_type_id: i32) -> bool {
    VANILLA_MINECART_ENTITY_TYPE_IDS.contains(&entity_type_id)
}

pub(crate) fn is_vanilla_happy_ghast_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID
}

pub(crate) fn is_vanilla_player_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_PLAYER_ENTITY_TYPE_ID
}

pub(crate) fn is_vanilla_pushable_entity_type(entity_type_id: i32) -> bool {
    is_vanilla_boat_entity_type(entity_type_id)
        || is_vanilla_shulker_entity_type(entity_type_id)
        || is_vanilla_minecart_entity_type(entity_type_id)
        || is_vanilla_player_entity_type(entity_type_id)
}

pub(crate) fn source_accepts_pushable_entities(entity_type_id: i32) -> bool {
    is_vanilla_boat_entity_type(entity_type_id) || is_vanilla_minecart_entity_type(entity_type_id)
}

pub(crate) fn entity_chunk_key(position: (f64, f64, f64)) -> (i32, i32) {
    (
        voxel_core::math::floor_f64(position.0).div_euclid(16),
        voxel_core::math::floor_f64(position.2).div_euclid(16),
    )
}

pub(crate) fn entity_section_key(position: (f64, f64, f64)) -> (i32, i32, i32) {
    (
        entity_section_coord(position.0),
        entity_section_coord(position.1),
        entity_section_coord(position.2),
    )
}

pub(crate) fn entity_section_coord(position: f64) -> i32 {
    voxel_core::math::floor_f64(position).div_euclid(16)
}

pub(crate) fn decode_entity_delta(base: f64, delta: i16) -> u64 {
    if delta == 0 {
        base.to_bits()
    } else {
        let encoded_base = (base * 4096.0 + 0.5).floor() as i64;
        ((encoded_base + i64::from(delta)) as f64 / 4096.0).to_bits()
    }
}

pub(crate) fn resolve_entity_teleport_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
    delta_movement: (f64, f64, f64),
    relative_flags: i32,
) -> (f64, f64, f64) {
    let current_motion = if relative_flags & ENTITY_RELATIVE_ROTATE_DELTA_FLAG != 0 {
        rotate_entity_motion(current_motion, current_rotation, target_rotation)
    } else {
        current_motion
    };
    (
        if relative_flags & ENTITY_RELATIVE_DELTA_X_FLAG != 0 {
            current_motion.0 + delta_movement.0
        } else {
            delta_movement.0
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Y_FLAG != 0 {
            current_motion.1 + delta_movement.1
        } else {
            delta_movement.1
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Z_FLAG != 0 {
            current_motion.2 + delta_movement.2
        } else {
            delta_movement.2
        },
    )
}

pub(crate) fn rotate_entity_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
) -> (f64, f64, f64) {
    let diff_y_rot = current_rotation.0 - target_rotation.0;
    let diff_x_rot = current_rotation.1 - target_rotation.1;
    let x_radians = (f64::from(diff_x_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let y_radians = (f64::from(diff_y_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let x_cos = f64::from(mth_cos_f64(f64::from(x_radians)));
    let x_sin = f64::from(mth_sin_f64(f64::from(x_radians)));
    let x_rotated = (
        current_motion.0,
        current_motion.1 * x_cos + current_motion.2 * x_sin,
        current_motion.2 * x_cos - current_motion.1 * x_sin,
    );
    let y_cos = f64::from(mth_cos_f64(f64::from(y_radians)));
    let y_sin = f64::from(mth_sin_f64(f64::from(y_radians)));
    (
        x_rotated.0 * y_cos + x_rotated.2 * y_sin,
        x_rotated.1,
        x_rotated.2 * y_cos - x_rotated.0 * y_sin,
    )
}

/// Shared `position + motion - gravity` step; the operand order matches the
/// reference, so the three entities using it stay bit-identical to Java.
pub(super) fn gravity_step(
    position: (f64, f64, f64),
    motion: (f64, f64, f64),
    no_gravity: bool,
    gravity: f64,
) -> (f64, f64, f64) {
    (
        position.0 + motion.0,
        position.1 + motion.1 - if no_gravity { 0.0 } else { gravity },
        position.2 + motion.2,
    )
}

/// Map a pinned entity type id onto its ballistic tick, if it has one.
pub(crate) fn vanilla_projectile_kind(type_id: i32) -> Option<VanillaProjectileKind> {
    match type_id {
        VANILLA_SNOWBALL_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Snowball),
        VANILLA_ARROW_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Arrow),
        VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID => Some(VanillaProjectileKind::ExperienceOrb),
        VANILLA_PRIMED_TNT_ENTITY_TYPE_ID => Some(VanillaProjectileKind::PrimedTnt),
        VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID => Some(VanillaProjectileKind::FallingBlock),
        VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Firework),
        _ => None,
    }
}
