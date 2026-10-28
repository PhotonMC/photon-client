use super::super::*;

#[test]
fn collide_entity_movement_follows_java_axis_order() {
    let moving = Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let x_wall = Shapes::box_shape(2.0, 0.0, 1.5, 3.0, 1.0, 2.5).unwrap();
    let z_wall = Shapes::box_shape(1.5, 0.0, 2.0, 2.5, 1.0, 3.0).unwrap();
    let shapes = [x_wall, z_wall];

    let x_first = collide_entity_movement(Vec3::new(2.0, 0.0, 1.0), moving, &shapes);
    assert_eq!((x_first.x, x_first.y, x_first.z), (2.0, 0.0, 0.5));

    let z_first = collide_entity_movement(Vec3::new(1.0, 0.0, 2.0), moving, &shapes);
    assert_eq!((z_first.x, z_first.y, z_first.z), (0.5, 0.0, 2.0));

    let y_first_shapes = [
        Shapes::box_shape(-1.0, 0.0, -1.0, 2.0, 1.0, 2.0).unwrap(),
        Shapes::box_shape(1.0, 1.0, -1.0, 2.0, 4.0, 2.0).unwrap(),
    ];
    let y_first = collide_entity_movement(
        Vec3::new(2.0, -2.0, 0.0),
        Aabb::new(0.25, 2.0, 0.25, 0.75, 3.0, 0.75),
        &y_first_shapes,
    );
    assert_eq!((y_first.x, y_first.y, y_first.z), (0.25, -1.0, 0.0));

    let no_collision = collide_entity_movement(Vec3::new(1.25, -2.5, 3.75), moving, &[]);
    assert_eq!(
        (no_collision.x, no_collision.y, no_collision.z),
        (1.25, -2.5, 3.75)
    );
}

#[test]
fn collision_state_follows_java_movement_flags() {
    let moving = Aabb::new(0.25, 2.0, 0.25, 0.75, 3.0, 0.75);
    let floor = Shapes::box_shape(-1.0, 0.0, -1.0, 2.0, 1.0, 2.0).unwrap();
    let falling = collide_entity_movement_state(Vec3::new(0.0, -2.0, 0.0), moving, &[floor]);
    assert_eq!(
        (falling.movement.x, falling.movement.y, falling.movement.z),
        (0.0, -1.0, 0.0)
    );
    assert!(!falling.x_collision);
    assert!(falling.y_collision);
    assert!(!falling.z_collision);
    assert!(!falling.horizontal_collision);
    assert!(falling.vertical_collision);
    assert!(falling.vertical_collision_below);
    assert!(falling.on_ground);

    let ceiling = Shapes::box_shape(-1.0, 4.0, -1.0, 2.0, 5.0, 2.0).unwrap();
    let rising = collide_entity_movement_state(Vec3::new(0.0, 2.0, 0.0), moving, &[ceiling]);
    assert_eq!(
        (rising.movement.x, rising.movement.y, rising.movement.z),
        (0.0, 1.0, 0.0)
    );
    assert!(!rising.horizontal_collision);
    assert!(rising.vertical_collision);
    assert!(!rising.vertical_collision_below);
    assert!(!rising.on_ground);
}

#[test]
fn zero_y_remote_movement_retains_vertical_state() {
    let previous = ClientEntityMovementState {
        movement: Vec3::ZERO,
        x_collision: false,
        y_collision: false,
        z_collision: false,
        horizontal_collision: false,
        vertical_collision: true,
        vertical_collision_below: true,
        on_ground: true,
    };
    let retained =
        apply_entity_movement_state(Vec3::new(2.0, 0.0, 0.0), Vec3::ZERO, previous, false);
    assert!(retained.horizontal_collision);
    assert!(retained.vertical_collision);
    assert!(retained.vertical_collision_below);
    assert!(retained.on_ground);

    let authoritative =
        apply_entity_movement_state(Vec3::new(2.0, 0.0, 0.0), Vec3::ZERO, previous, true);
    assert!(!authoritative.vertical_collision);
    assert!(!authoritative.vertical_collision_below);
    assert!(!authoritative.on_ground);

    let falling = apply_entity_movement_state(
        Vec3::new(0.0, -2.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        previous,
        false,
    );
    assert!(falling.vertical_collision);
    assert!(falling.vertical_collision_below);
    assert!(falling.on_ground);
}

#[test]
fn candidate_step_up_heights_follow_java_filter_and_sort_order() {
    let colliders = [
        Shapes::box_shape(0.0, 2.5, 0.0, 1.0, 2.75, 1.0).unwrap(),
        Shapes::box_shape(0.0, 2.0, 0.0, 1.0, 2.5, 1.0).unwrap(),
        Shapes::box_shape(0.0, 2.25, 0.0, 1.0, 2.5, 1.0).unwrap(),
    ];
    let candidates = collect_candidate_step_up_heights(
        Aabb::new(0.0, 2.0, 0.0, 1.0, 3.0, 1.0),
        &colliders,
        0.75,
        -1.0,
    );
    assert_eq!(
        candidates
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0.0_f32, 0.25, 0.5, 0.75]
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    );

    let filtered = [
        Shapes::box_shape(0.0, 9.0, 0.0, 1.0, 10.25, 1.0).unwrap(),
        Shapes::box_shape(0.0, 10.125, 0.0, 1.0, 10.5, 1.0).unwrap(),
        Shapes::box_shape(0.0, 10.375, 0.0, 1.0, 11.0, 1.0).unwrap(),
    ];
    let candidates = collect_candidate_step_up_heights(
        Aabb::new(0.0, 10.0, 0.0, 1.0, 11.0, 1.0),
        &filtered,
        0.5,
        0.25,
    );
    assert_eq!(
        candidates
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0.125_f32, 0.375, 0.5]
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    );
}

#[test]
fn step_up_selection_uses_first_strictly_longer_candidate() {
    let low_obstacle = [Shapes::box_shape(1.0, 0.0, 0.0, 2.0, 0.5, 1.0).unwrap()];
    let selected = select_step_up_movement(
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::ZERO,
        (
            Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        &low_obstacle,
        (0.6, 0.0),
    );
    assert_eq!((selected.x, selected.y, selected.z), (2.0, 0.5, 0.0));

    let tie = select_step_up_movement(
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        (
            Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        &low_obstacle,
        (0.6, 0.0),
    );
    assert_eq!((tie.x, tie.y, tie.z), (2.0, 0.0, 0.0));

    let offset = [Shapes::box_shape(1.0, 0.5, 0.0, 2.0, 1.0, 1.0).unwrap()];
    let corrected = select_step_up_movement(
        Vec3::new(2.0, -1.0, 0.0),
        Vec3::new(0.0, -0.5, 0.0),
        (
            Aabb::new(0.0, 1.0, 0.0, 1.0, 2.0, 1.0),
            Aabb::new(0.0, 0.5, 0.0, 1.0, 1.5, 1.0),
        ),
        &offset,
        (0.6, -0.5),
    );
    assert_eq!((corrected.x, corrected.y, corrected.z), (2.0, 0.0, 0.0));
}
