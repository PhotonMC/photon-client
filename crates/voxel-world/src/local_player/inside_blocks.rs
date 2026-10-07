use super::*;

impl LocalPlayer {
    pub(super) fn apply_effects_from_blocks(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        let mut movements = std::mem::take(&mut self.movements_this_tick);
        match movements.last() {
            None => movements.push(Movement {
                from: self.old_position,
                to: self.position,
                original: None,
            }),
            Some(last)
                if last.to.distance_to_sqr(self.position) > f64::from(9.999_999_4e-11_f32) =>
            {
                let from = last.to;
                movements.push(Movement {
                    from,
                    to: self.position,
                    original: None,
                });
            }
            Some(_) => {}
        }
        if self.no_physics {
            return Ok(());
        }
        if self.on_ground {
            let effect_pos = self.get_on_pos(level, 0.2);
            if level.info(effect_pos).name == "minecraft:slime_block" {
                let abs_delta_y = self.delta.y.abs();
                if abs_delta_y < 0.1 && !self.is_shift_key_down() {
                    let scale = 0.4 + abs_delta_y * 0.2;
                    self.delta = self.delta.multiply_components(scale, 1.0, scale);
                }
            }
        }
        self.check_inside_blocks(level, &movements)
    }

    /// `Entity.checkInsideBlocks` for the player: the shared walk, with the blocks that act on a
    /// player acting as they are visited.
    pub(super) fn check_inside_blocks(
        &mut self,
        level: &Level<'_>,
        movements: &[Movement],
    ) -> Result<(), Unsupported> {
        walk_inside_blocks(self, level, movements)
    }

    /// What a block the player is inside does to it (see [`living_inside_effect`]).
    pub(super) fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let block_pos = self.block_position();
        living_inside_effect(
            level,
            visit,
            LivingEffects {
                delta: &mut self.delta,
                fall_distance: &mut self.fall_distance,
                stuck: &mut self.stuck_speed_multiplier,
                position: self.position,
                on_ground: self.on_ground,
                width: self.dimensions.width(),
                block_pos,
            },
        );
    }

    pub(super) fn check_supporting_block(
        &mut self,
        level: &Level<'_>,
        on_ground: bool,
        movement: Option<Vec3>,
    ) -> Result<(), Unsupported> {
        level.check_supporting_block(
            (
                &mut self.main_supporting_block,
                &mut self.on_ground_no_blocks,
            ),
            (self.bounding_box, self.position, self.fall_distance),
            on_ground,
            movement,
        )
    }

    pub(super) fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.main_supporting_block, self.position, offset)
    }
}

/// What a block a living entity is inside does to it: honey slides it, a cobweb or a berry bush
/// holds it, powder snow holds it only when its own block is powder snow, and a bubble column
/// carries it. The other blocks have no effect here.
pub(crate) fn living_inside_effect(
    level: &Level<'_>,
    visit: &InsideVisit,
    body: LivingEffects<'_>,
) {
    let InsideVisit {
        pos,
        state,
        ref info,
        precise,
    } = *visit;
    let make_stuck = |body: &mut LivingEffects<'_>, speed_multiplier: Vec3| {
        *body.fall_distance = 0.0;
        *body.stuck = speed_multiplier;
    };
    let mut body = body;
    match info.name {
        "minecraft:honey_block" if is_sliding_down_honey(&body, pos) => {
            let old_delta_y = |delta_y: f64| delta_y / f64::from(0.98_f32) + 0.08;
            let new_delta_y = |delta_y: f64| (delta_y - 0.08) * f64::from(0.98_f32);
            let delta = *body.delta;
            *body.delta = if old_delta_y(delta.y) < -0.13 {
                let factor = -0.05 / old_delta_y(delta.y);
                Vec3::new(delta.x * factor, new_delta_y(-0.05), delta.z * factor)
            } else {
                Vec3::new(delta.x, new_delta_y(-0.05), delta.z)
            };
            *body.fall_distance = 0.0;
        }
        "minecraft:cobweb" => {
            make_stuck(&mut body, Vec3::new(0.25, f64::from(0.05_f32), 0.25));
        }
        "minecraft:sweet_berry_bush" => {
            make_stuck(
                &mut body,
                Vec3::new(f64::from(0.8_f32), 0.75, f64::from(0.8_f32)),
            );
        }
        "minecraft:powder_snow" => {
            if level.info(body.block_pos).name == "minecraft:powder_snow" {
                make_stuck(
                    &mut body,
                    Vec3::new(f64::from(0.9_f32), 1.5, f64::from(0.9_f32)),
                );
            }
        }
        "minecraft:bubble_column" if precise => {
            let drag_down = property(state, "drag") == Some("true");
            let above = pos.offset(0, 1, 0);
            let above_state = level.state_id(above);
            let nothing_above = above_state.is_none_or(|above_state| {
                level
                    .world
                    .block_state_registry
                    .collision_shape(above_state)
                    .is_empty()
                    && crate::block::state_or_air(i64::from(above_state))
                        .fluid
                        .is_empty()
            });
            let delta = *body.delta;
            let y = if nothing_above {
                if drag_down {
                    (delta.y - 0.03).max(-0.9)
                } else {
                    (delta.y + 0.1).min(1.8)
                }
            } else {
                *body.fall_distance = 0.0;
                if drag_down {
                    (delta.y - 0.03).max(-0.3)
                } else {
                    (delta.y + 0.06).min(0.7)
                }
            };
            *body.delta = Vec3::new(delta.x, y, delta.z);
        }
        _ => {}
    }
}

/// `Entity.checkInsideBlocks`: walk the blocks each movement passed through, in the reference's
/// per-axis split, and let the ones that act on a body inside them act.
pub(crate) fn walk_inside_blocks<B: InsideBody>(
    body: &mut B,
    level: &Level<'_>,
    movements: &[Movement],
) -> Result<(), Unsupported> {
    let mut visited = std::collections::HashSet::new();
    for movement in movements {
        let mut pos = movement.from;
        let delta = movement.to.subtract_vec3(movement.from);
        let mut max_iterations = 16;
        if let Some(original) = movement.original.filter(|_| delta.length_squared() > 0.0) {
            for axis in voxel_core::direction::Direction::axis_step_order(original.x, original.z) {
                let axis_move = delta.get(axis);
                if axis_move != 0.0 {
                    let to = pos.with(axis, pos.get(axis) + axis_move);
                    max_iterations -=
                        walk_inside_between(body, level, (pos, to), &mut visited, max_iterations);
                    pos = to;
                }
            }
        } else {
            max_iterations -=
                walk_inside_between(body, level, (movement.from, movement.to), &mut visited, 16);
        }
        if max_iterations <= 0 {
            walk_inside_between(body, level, (movement.to, movement.to), &mut visited, 1);
        }
    }
    Ok(())
}

pub(super) fn walk_inside_between<B: InsideBody>(
    body: &mut B,
    level: &Level<'_>,
    (from, to): (Vec3, Vec3),
    visited: &mut std::collections::HashSet<(i32, i32, i32)>,
    max_iterations: i32,
) -> i32 {
    let at_target = body.box_at(to).deflate(f64::from(1.0e-5_f32));
    let moved_far = from.distance_to_sqr(to) > 0.999_990_000_000_252_6_f64.powi(2);
    let mut last_iteration = 0;
    for (pos, iteration) in blocks_intersected_between(from, to, at_target) {
        if iteration >= max_iterations {
            break;
        }
        last_iteration = iteration;
        let state = level.state_id(pos);
        let info = level.info(pos);
        if info.is_air() {
            continue;
        }
        // `getEntityInsideCollisionShape`: the whole block, except powder snow, whose own
        // collision shape counts when it has one.
        let inside_block = match state {
            Some(state) if is_block(state, "minecraft:powder_snow") => {
                let shape = body.powder_snow_shape();
                shape.is_empty() || {
                    let boxes: Vec<Aabb> = shape
                        .to_aabbs()
                        .into_iter()
                        .map(|aabb| aabb.move_by_block_pos(pos))
                        .collect();
                    body.box_at(from)
                        .collided_along_vector(to.subtract_vec3(from), &boxes)
                }
            }
            _ => true,
        };
        // Only a block the body is not fully inside can miss, and the one such block, powder
        // snow, holds no fluid whose effects (fire) would still need the visit.
        if !inside_block || !visited.insert((pos.x, pos.y, pos.z)) {
            continue;
        }
        let precise = moved_far || at_target.intersects_block_pos(pos);
        body.inside_block_effect(
            level,
            &InsideVisit {
                pos,
                state,
                info,
                precise,
            },
        );
    }
    last_iteration + 1
}

/// `BlockGetter.forEachBlockIntersectedBetween`, as the blocks it visits and
/// the iteration each is visited in. The order within one iteration is not
/// kept: only honey acts here, and it acts at most once a tick.
#[expect(
    clippy::too_many_lines,
    reason = "kept whole to read against the reference method line by line"
)]
pub(super) fn blocks_intersected_between(
    from: Vec3,
    to: Vec3,
    at_target: Aabb,
) -> Vec<(BlockPos, i32)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    // `BlockPos.betweenClosed` (x fastest, then y, then z) without a
    // direction, `betweenCornersInDirection` with one: from the corner the
    // travel leaves, the axes in step order, the last one fastest.
    let mut add_box = |out: &mut Vec<(BlockPos, i32)>,
                       first: [i32; 3],
                       second: [i32; 3],
                       iteration: i32,
                       direction: Option<Vec3>| {
        let min = [0, 1, 2].map(|axis| first[axis].min(second[axis]));
        let max = [0, 1, 2].map(|axis| first[axis].max(second[axis]));
        let (order, forward) = match direction {
            None => ([2, 1, 0], [true; 3]),
            Some(direction) => {
                let order =
                    voxel_core::direction::Direction::axis_step_order(direction.x, direction.z)
                        .map(|axis| match axis {
                            voxel_core::direction::Axis::X => 0,
                            voxel_core::direction::Axis::Y => 1,
                            voxel_core::direction::Axis::Z => 2,
                        });
                (
                    order,
                    [direction.x >= 0.0, direction.y >= 0.0, direction.z >= 0.0],
                )
            }
        };
        let step = |axis: usize, index: i32| {
            if forward[axis] {
                min[axis] + index
            } else {
                max[axis] - index
            }
        };
        for a in 0..=max[order[0]] - min[order[0]] {
            for b in 0..=max[order[1]] - min[order[1]] {
                for c in 0..=max[order[2]] - min[order[2]] {
                    let mut pos = [0; 3];
                    pos[order[0]] = step(order[0], a);
                    pos[order[1]] = step(order[1], b);
                    pos[order[2]] = step(order[2], c);
                    if seen.insert((pos[0], pos[1], pos[2])) {
                        out.push((BlockPos::new(pos[0], pos[1], pos[2]), iteration));
                    }
                }
            }
        }
    };
    let corners = |aabb: Aabb| {
        (
            [
                floor_f64(aabb.min_x),
                floor_f64(aabb.min_y),
                floor_f64(aabb.min_z),
            ],
            [
                floor_f64(aabb.max_x),
                floor_f64(aabb.max_y),
                floor_f64(aabb.max_z),
            ],
        )
    };
    let travel = to.subtract_vec3(from);
    if travel.length_squared() < f64::from(1.0e-5_f32 * 1.0e-5_f32) {
        let (first, second) = corners(at_target);
        add_box(&mut out, first, second, 0, None);
        return out;
    }
    let (first, second) = corners(at_target.move_by(-travel.x, -travel.y, -travel.z));
    add_box(&mut out, first, second, 0, Some(travel));

    // addCollisionsAlongTravel.
    let size = [at_target.x_size(), at_target.y_size(), at_target.z_size()];
    let corner_dir = furthest_corner(travel);
    let centre = at_target.center();
    let to_corner = Vec3::new(
        centre.x + size[0] * 0.5 * f64::from(corner_dir[0]),
        centre.y + size[1] * 0.5 * f64::from(corner_dir[1]),
        centre.z + size[2] * 0.5 * f64::from(corner_dir[2]),
    );
    let from_corner = to_corner.subtract_vec3(travel);
    let mut block = [
        floor_f64(from_corner.x),
        floor_f64(from_corner.y),
        floor_f64(from_corner.z),
    ];
    let sign = |value: f64| {
        if value > 0.0 {
            1
        } else if value < 0.0 {
            -1
        } else {
            0
        }
    };
    let frac = |value: f64| value - voxel_core::math::lfloor_f64(value) as f64;
    let signs = [sign(travel.x), sign(travel.y), sign(travel.z)];
    let t_delta = [travel.x, travel.y, travel.z]
        .iter()
        .zip(signs)
        .map(|(value, sign)| {
            if sign == 0 {
                f64::MAX
            } else {
                f64::from(sign) / value
            }
        })
        .collect::<Vec<_>>();
    let starts = [from_corner.x, from_corner.y, from_corner.z];
    let mut t = [0.0_f64; 3];
    for axis in 0..3 {
        t[axis] = t_delta[axis]
            * if signs[axis] > 0 {
                1.0 - frac(starts[axis])
            } else {
                frac(starts[axis])
            };
    }
    let mut iterations = 0;
    while t[0] <= 1.0 || t[1] <= 1.0 || t[2] <= 1.0 {
        let axis = if t[0] < t[1] {
            if t[0] < t[2] { 0 } else { 2 }
        } else if t[1] < t[2] {
            1
        } else {
            2
        };
        block[axis] += signs[axis];
        t[axis] += t_delta[axis];
        let hit = Aabb::clip_coords(
            Vec3::new(
                f64::from(block[0]),
                f64::from(block[1]),
                f64::from(block[2]),
            ),
            Vec3::new(
                f64::from(block[0] + 1),
                f64::from(block[1] + 1),
                f64::from(block[2] + 1),
            ),
            from_corner,
            to_corner,
        );
        if let Some(hit) = hit {
            iterations += 1;
            let hit_coords = [hit.x, hit.y, hit.z];
            let mut opposite = [0; 3];
            for axis in 0..3 {
                let low = f64::from(block[axis] as f32 + 1.0e-5_f32);
                let high = f64::from(block[axis]) + 1.0 - f64::from(1.0e-5_f32);
                let clamped = java_math::clamp(hit_coords[axis], low, high);
                opposite[axis] = floor_f64(clamped - size[axis] * f64::from(corner_dir[axis]));
            }
            add_box(&mut out, block, opposite, iterations, Some(travel));
        }
    }
    let (first, second) = corners(at_target);
    add_box(&mut out, first, second, iterations + 1, Some(travel));
    out
}

/// `PowderSnowBlock.getCollisionShape` for a player without leather boots:
/// nothing until it has fallen more than 2.5 blocks, then a floor 0.9 high.
pub(crate) fn powder_snow_collision_shape(fall_distance: f64) -> VoxelShape {
    if fall_distance > 2.5 {
        Shapes::box_shape(0.0, 0.0, 0.0, 1.0, f64::from(0.9_f32), 1.0)
            .expect("the falling powder snow shape is a valid box")
    } else {
        Shapes::empty()
    }
}

impl InsideBody for LocalPlayer {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> VoxelShape {
        powder_snow_collision_shape(self.fall_distance)
    }

    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        LocalPlayer::inside_block_effect(self, level, visit);
    }
}
