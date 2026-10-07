use super::*;

impl Aabb {
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
