//! `ClientWorld` collision layer split out of the former single-file module.

use super::*;

use photon_core::math::{MTH_DEG_TO_RAD, Vec3, clamp_f64, equal_f64, mth_cos_f64, mth_sin_f64};
use photon_core::{
    aabb::Aabb,
    direction::{Axis, Direction},
    shape::{BooleanOp, Shapes, VoxelShape},
};

/// Resolve one entity movement against already collected world/entity shapes.
///
/// This is the Java `Entity.collideWithShapes` axis-resolution primitive. The
/// caller owns collider collection and step-up policy; keeping those decisions
/// outside this helper preserves the reference call order.
pub fn collide_entity_movement(movement: Vec3, bounding_box: Aabb, shapes: &[VoxelShape]) -> Vec3 {
    if shapes.is_empty() {
        return movement;
    }

    let mut resolved_movement = Vec3::ZERO;
    for axis in Direction::axis_step_order(movement.x, movement.z) {
        let axis_movement = movement.get(axis);
        if axis_movement != 0.0 {
            let collision = Shapes::collide(
                axis,
                bounding_box.move_by_vec3(resolved_movement),
                shapes,
                axis_movement,
            );
            resolved_movement = resolved_movement.with(axis, collision);
        }
    }
    resolved_movement
}

/// The collision flags produced from one resolved entity movement.
#[derive(Debug, Clone, Copy)]
pub struct ClientEntityMovementState {
    pub movement: Vec3,
    pub x_collision: bool,
    pub y_collision: bool,
    pub z_collision: bool,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    pub on_ground: bool,
}

/// Resolve a movement delta and derive Java's base collision flags.
///
/// The x/z flags use `Mth.equal`'s strict `1.0E-5` tolerance, while the y
/// flags use the exact comparison from `Entity.move`. This covers the base
/// movement result before a caller applies step-up policy or a subclass's
/// minor-horizontal-collision override.
pub fn collide_entity_movement_state(
    movement: Vec3,
    bounding_box: Aabb,
    shapes: &[VoxelShape],
) -> ClientEntityMovementState {
    let resolved_movement = collide_entity_movement(movement, bounding_box, shapes);
    let x_collision = !equal_f64(movement.x, resolved_movement.x);
    let y_collision = movement.y != resolved_movement.y;
    let z_collision = !equal_f64(movement.z, resolved_movement.z);
    let horizontal_collision = x_collision || z_collision;
    let vertical_collision_below = y_collision && movement.y < 0.0;

    ClientEntityMovementState {
        movement: resolved_movement,
        x_collision,
        y_collision,
        z_collision,
        horizontal_collision,
        vertical_collision: y_collision,
        vertical_collision_below,
        on_ground: vertical_collision_below,
    }
}

/// Apply Java `Entity.move` collision fields while retaining vertical state
/// for a non-authoritative zero-Y movement.
pub fn apply_entity_movement_state(
    delta: Vec3,
    resolved_movement: Vec3,
    previous: ClientEntityMovementState,
    is_local_instance_authoritative: bool,
) -> ClientEntityMovementState {
    let x_collision = !equal_f64(delta.x, resolved_movement.x);
    let y_collision = delta.y != resolved_movement.y;
    let z_collision = !equal_f64(delta.z, resolved_movement.z);
    let horizontal_collision = x_collision || z_collision;
    let mut vertical_collision = previous.vertical_collision;
    let mut vertical_collision_below = previous.vertical_collision_below;
    let mut on_ground = previous.on_ground;
    if delta.y.abs() > 0.0 || is_local_instance_authoritative {
        vertical_collision = y_collision;
        vertical_collision_below = vertical_collision && delta.y < 0.0;
        on_ground = vertical_collision_below;
    }

    ClientEntityMovementState {
        movement: resolved_movement,
        x_collision,
        y_collision,
        z_collision,
        horizontal_collision,
        vertical_collision,
        vertical_collision_below,
        on_ground,
    }
}

/// Return Java LocalPlayer's minor-horizontal-collision classification.
///
/// LocalPlayer compares the yaw-rotated input vector with the resolved
/// horizontal movement and accepts only angles below the f32 8-degree
/// threshold. The input and movement length gates intentionally use the
/// reference's strict `1.0E-5F` comparison.
pub fn is_minor_horizontal_collision(y_rot: f32, xxa: f32, zza: f32, movement: Vec3) -> bool {
    let y_rot_in_radians = y_rot * MTH_DEG_TO_RAD;
    let y_rot_sin = f64::from(mth_sin_f64(f64::from(y_rot_in_radians)));
    let y_rot_cos = f64::from(mth_cos_f64(f64::from(y_rot_in_radians)));
    let global_xa = f64::from(xxa) * y_rot_cos - f64::from(zza) * y_rot_sin;
    let global_za = f64::from(zza) * y_rot_cos + f64::from(xxa) * y_rot_sin;
    let input_length_squared = global_xa * global_xa + global_za * global_za;
    let movement_length_squared = movement.x * movement.x + movement.z * movement.z;
    let epsilon = f64::from(f32::from_bits(0x3727_c5ac));
    if input_length_squared >= epsilon && movement_length_squared >= epsilon {
        let dot_product = global_xa * movement.x + global_za * movement.z;
        let angle = (dot_product / (input_length_squared * movement_length_squared).sqrt()).acos();
        angle < f64::from(f32::from_bits(0x3e0e_fa35))
    } else {
        false
    }
}

/// Candidate entity data needed by Java's `EntityGetter` collision query.
#[derive(Debug, Clone, Copy)]
pub struct ClientEntityCollisionCandidate {
    pub bounding_box: Aabb,
    pub is_source: bool,
    pub spectator: bool,
    pub can_be_collided_with: bool,
    pub is_pushable: bool,
    pub same_vehicle: bool,
}

/// Position data needed by Java's world-border collision query.
#[derive(Debug, Clone, Copy)]
pub struct ClientWorldCollisionSource {
    pub position: Vec3,
}

/// Collect entity bounding-box shapes using Java's collision-query policy.
pub fn collect_entity_collision_shapes(
    source_present: bool,
    test_area: Aabb,
    candidates: &[ClientEntityCollisionCandidate],
) -> Vec<VoxelShape> {
    collect_entity_collision_shapes_for_source(None, source_present, test_area, candidates)
}

/// Collect entity shapes with the source type's Java collision dispatch.
pub fn collect_entity_collision_shapes_for_source(
    source_type_id: Option<i32>,
    source_present: bool,
    test_area: Aabb,
    candidates: &[ClientEntityCollisionCandidate],
) -> Vec<VoxelShape> {
    if test_area.size() < 1.0e-7 {
        return Vec::new();
    }

    let query_area = test_area.inflate_all(1.0e-7);
    candidates
        .iter()
        .filter(|candidate| {
            (!source_present || !candidate.is_source)
                && candidate.bounding_box.intersects(query_area)
                && !candidate.spectator
                && (candidate.can_be_collided_with
                    || (source_type_id.is_some_and(source_accepts_pushable_entities)
                        && candidate.is_pushable))
                && (!source_present || !candidate.same_vehicle)
        })
        .map(|candidate| {
            Shapes::create_from_aabb(candidate.bounding_box)
                .expect("entity collision candidates require normalized bounds")
        })
        .collect()
}

/// Build Java's static world-border collision shape.
pub fn world_border_collision_shape(
    center_x: f64,
    center_z: f64,
    size: f64,
    absolute_max_size: i32,
) -> VoxelShape {
    let absolute_max_size = f64::from(absolute_max_size);
    let min_x = clamp_f64(center_x - size / 2.0, -absolute_max_size, absolute_max_size).floor();
    let min_z = clamp_f64(center_z - size / 2.0, -absolute_max_size, absolute_max_size).floor();
    let max_x = clamp_f64(center_x + size / 2.0, -absolute_max_size, absolute_max_size).ceil();
    let max_z = clamp_f64(center_z + size / 2.0, -absolute_max_size, absolute_max_size).ceil();
    let inner = Shapes::box_shape(min_x, f64::NEG_INFINITY, min_z, max_x, f64::INFINITY, max_z)
        .expect("world border bounds must remain ordered");
    Shapes::join(&Shapes::infinity(), &inner, BooleanOp::OnlyFirst)
}

pub(crate) fn java_float_compare(first: &f32, second: &f32) -> std::cmp::Ordering {
    match first.partial_cmp(second) {
        Some(std::cmp::Ordering::Less) => std::cmp::Ordering::Less,
        Some(std::cmp::Ordering::Greater) => std::cmp::Ordering::Greater,
        _ => {
            let first_bits = if first.is_nan() {
                0x7fc0_0000_i32
            } else {
                first.to_bits() as i32
            };
            let second_bits = if second.is_nan() {
                0x7fc0_0000_i32
            } else {
                second.to_bits() as i32
            };
            first_bits.cmp(&second_bits)
        }
    }
}

/// Collect the sorted step-up height candidates used by Java entity movement.
///
/// The caller supplies the world/entity collider shapes already in world
/// coordinates. Candidate heights are relative to the moving box's minimum Y;
/// raw f32 bits are used for set membership just like Java's FloatArraySet.
pub fn collect_candidate_step_up_heights(
    bounding_box: Aabb,
    colliders: &[VoxelShape],
    max_step_height: f32,
    step_height_to_skip: f32,
) -> Vec<f32> {
    let mut candidates = Vec::with_capacity(4);
    for collider in colliders {
        for &coord in collider.coords(Axis::Y) {
            let relative_coord = (coord - bounding_box.min_y) as f32;
            if relative_coord.partial_cmp(&0.0) != Some(std::cmp::Ordering::Less)
                && relative_coord != step_height_to_skip
            {
                if relative_coord > max_step_height {
                    break;
                }
                if !candidates
                    .iter()
                    .any(|candidate: &f32| candidate.to_bits() == relative_coord.to_bits())
                {
                    candidates.push(relative_coord);
                }
            }
        }
    }
    candidates.sort_unstable_by(java_float_compare);
    candidates
}

/// Select Java's first step-up movement whose horizontal distance exceeds the
/// ordinary collision result.
///
/// Candidate order and collision resolution are delegated to the same helpers
/// used by the base movement path. The original and grounded boxes remain
/// separate because Java subtracts their minimum-Y difference only after a
/// candidate wins.
pub fn select_step_up_movement(
    movement: Vec3,
    movement_step: Vec3,
    bounding_box: Aabb,
    grounded_bounding_box: Aabb,
    colliders: &[VoxelShape],
    max_step_height: f32,
    step_height_to_skip: f32,
) -> Vec3 {
    let candidates = collect_candidate_step_up_heights(
        grounded_bounding_box,
        colliders,
        max_step_height,
        step_height_to_skip,
    );
    for candidate in candidates {
        let step_from_ground = collide_entity_movement(
            Vec3::new(movement.x, f64::from(candidate), movement.z),
            grounded_bounding_box,
            colliders,
        );
        if step_from_ground.horizontal_distance_squared()
            > movement_step.horizontal_distance_squared()
        {
            let distance_to_ground = bounding_box.min_y - grounded_bounding_box.min_y;
            return step_from_ground.subtract_components(0.0, distance_to_ground, 0.0);
        }
    }
    movement_step
}
