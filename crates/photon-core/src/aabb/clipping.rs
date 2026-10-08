use super::*;

impl Aabb {
    /// Return the first segment point inside the box, matching Java `clip`.
    pub fn clip(self, from: Vec3, to: Vec3) -> Option<Vec3> {
        Self::clip_coords(
            Vec3::new(self.min_x, self.min_y, self.min_z),
            Vec3::new(self.max_x, self.max_y, self.max_z),
            from,
            to,
        )
    }

    /// Clip a segment against raw coordinates, without normalizing the bounds.
    pub fn clip_coords(min: Vec3, max: Vec3, from: Vec3, to: Vec3) -> Option<Vec3> {
        let mut state = ClipState::new(from, to);
        clip_bounds(min, max, &mut state);

        state.direction.map(|_| {
            Vec3::new(
                state.from.x + state.scale * state.dx,
                state.from.y + state.scale * state.dy,
                state.from.z + state.scale * state.dz,
            )
        })
    }

    /// Clip a segment against translated AABBs and return the nearest block hit.
    pub fn clip_many(
        boxes: &[Self],
        from: Vec3,
        to: Vec3,
        pos: BlockPos,
    ) -> Option<BlockHitResult> {
        let mut state = ClipState::new(from, to);

        for aabb in boxes {
            let translated = aabb.move_by_block_pos(pos);
            clip_bounds(
                translated.min_position(),
                translated.max_position(),
                &mut state,
            );
        }

        state.direction.map(|direction| {
            BlockHitResult::new(
                Vec3::new(
                    state.from.x + state.scale * state.dx,
                    state.from.y + state.scale * state.dy,
                    state.from.z + state.scale * state.dz,
                ),
                direction,
                pos,
                false,
            )
        })
    }

    /// Test whether the center's translated segment collides with any box.
    pub fn collided_along_vector(self, vector: Vec3, boxes: &[Self]) -> bool {
        let from = self.center();
        let to = Vec3::new(from.x + vector.x, from.y + vector.y, from.z + vector.z);
        let inflation = Vec3::new(
            self.x_size() * 0.5 - 1.0e-7,
            self.y_size() * 0.5 - 1.0e-7,
            self.z_size() * 0.5 - 1.0e-7,
        );

        boxes.iter().any(|shape_part| {
            let inflated = shape_part.inflate(inflation.x, inflation.y, inflation.z);
            inflated.contains_vec3(to)
                || inflated.contains_vec3(from)
                || inflated.clip(from, to).is_some()
        })
    }
}

pub(super) struct ClipState {
    pub(super) from: Vec3,
    pub(super) dx: f64,
    pub(super) dy: f64,
    pub(super) dz: f64,
    pub(super) scale: f64,
    pub(super) direction: Option<Direction>,
}

#[expect(
    clippy::too_many_lines,
    reason = "kept whole to read against the reference method line by line"
)]
pub(super) fn clip_bounds(min: Vec3, max: Vec3, state: &mut ClipState) {
    if state.dx > 1.0e-7 {
        if clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dx,
                db: state.dy,
                dc: state.dz,
                point: min.x,
                min_b: min.y,
                max_b: max.y,
                min_c: min.z,
                max_c: max.z,
                from_a: state.from.x,
                from_b: state.from.y,
                from_c: state.from.z,
            },
        ) {
            state.direction = Some(Direction::West);
        }
    } else if state.dx < -1.0e-7
        && clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dx,
                db: state.dy,
                dc: state.dz,
                point: max.x,
                min_b: min.y,
                max_b: max.y,
                min_c: min.z,
                max_c: max.z,
                from_a: state.from.x,
                from_b: state.from.y,
                from_c: state.from.z,
            },
        )
    {
        state.direction = Some(Direction::East);
    }

    if state.dy > 1.0e-7 {
        if clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dy,
                db: state.dz,
                dc: state.dx,
                point: min.y,
                min_b: min.z,
                max_b: max.z,
                min_c: min.x,
                max_c: max.x,
                from_a: state.from.y,
                from_b: state.from.z,
                from_c: state.from.x,
            },
        ) {
            state.direction = Some(Direction::Down);
        }
    } else if state.dy < -1.0e-7
        && clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dy,
                db: state.dz,
                dc: state.dx,
                point: max.y,
                min_b: min.z,
                max_b: max.z,
                min_c: min.x,
                max_c: max.x,
                from_a: state.from.y,
                from_b: state.from.z,
                from_c: state.from.x,
            },
        )
    {
        state.direction = Some(Direction::Up);
    }

    if state.dz > 1.0e-7 {
        if clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dz,
                db: state.dx,
                dc: state.dy,
                point: min.z,
                min_b: min.x,
                max_b: max.x,
                min_c: min.y,
                max_c: max.y,
                from_a: state.from.z,
                from_b: state.from.x,
                from_c: state.from.y,
            },
        ) {
            state.direction = Some(Direction::North);
        }
    } else if state.dz < -1.0e-7
        && clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dz,
                db: state.dx,
                dc: state.dy,
                point: max.z,
                min_b: min.x,
                max_b: max.x,
                min_c: min.y,
                max_c: max.y,
                from_a: state.from.z,
                from_b: state.from.x,
                from_c: state.from.y,
            },
        )
    {
        state.direction = Some(Direction::South);
    }
}

pub(super) fn lerp_half(start: f64, end: f64) -> f64 {
    start + 0.5 * (end - start)
}

pub(super) struct ClipPoint {
    pub(super) da: f64,
    pub(super) db: f64,
    pub(super) dc: f64,
    pub(super) point: f64,
    pub(super) min_b: f64,
    pub(super) max_b: f64,
    pub(super) min_c: f64,
    pub(super) max_c: f64,
    pub(super) from_a: f64,
    pub(super) from_b: f64,
    pub(super) from_c: f64,
}

pub(super) fn clip_point(scale: &mut f64, candidate: ClipPoint) -> bool {
    let candidate_scale = (candidate.point - candidate.from_a) / candidate.da;
    let point_b = candidate.from_b + candidate_scale * candidate.db;
    let point_c = candidate.from_c + candidate_scale * candidate.dc;
    if 0.0 < candidate_scale
        && candidate_scale < *scale
        && candidate.min_b - 1.0e-7 < point_b
        && point_b < candidate.max_b + 1.0e-7
        && candidate.min_c - 1.0e-7 < point_c
        && point_c < candidate.max_c + 1.0e-7
    {
        *scale = candidate_scale;
        true
    } else {
        false
    }
}

pub(super) fn expand_axis(min: f64, max: f64, delta: f64) -> (f64, f64) {
    if delta < 0.0 {
        (min + delta, max)
    } else if delta > 0.0 {
        (min, max + delta)
    } else {
        (min, max)
    }
}

pub(super) fn contract_axis(min: f64, max: f64, delta: f64) -> (f64, f64) {
    if delta < 0.0 {
        (min - delta, max)
    } else if delta > 0.0 {
        (min, max - delta)
    } else {
        (min, max)
    }
}

impl ClipState {
    pub(super) fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            from,
            dx: to.x - from.x,
            dy: to.y - from.y,
            dz: to.z - from.z,
            scale: 1.0,
            direction: None,
        }
    }
}
