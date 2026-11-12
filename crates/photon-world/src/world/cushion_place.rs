//! What `CushionItem.useOn` asks of the level before a cushion is set down: something must hold
//! it from below, the blocks around it must not all be suffocating, and the sliver it rests on
//! must not be shut in by block collisions.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::shape::{BooleanOp, Shapes};

use crate::ClientWorld;

/// The thickness of the sliver the cushion rests on (`1/64`).
const RESTING: f64 = 0.015625;

/// How far below the cushion the blocks that may hold it are looked for.
const REACH_BELOW: f64 = 0.125;

/// The block positions a box meets: every position from the one holding its minimum corner to
/// the one holding its maximum corner (`BlockPos.containing`, then `betweenClosed`).
fn positions(area: Aabb) -> impl Iterator<Item = BlockPos> {
    let floor = photon_core::math::floor_f64;
    let (x0, y0, z0) = (floor(area.min_x), floor(area.min_y), floor(area.min_z));
    let (x1, y1, z1) = (floor(area.max_x), floor(area.max_y), floor(area.max_z));
    (z0..=z1).flat_map(move |z| {
        (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| BlockPos::new(x, y, z)))
    })
}

impl ClientWorld {
    /// `Cushion.canBePlacedAt`: the cushion whose box is `area` survives there and is not buried.
    pub fn cushion_can_be_placed_at(&self, area: Aabb) -> bool {
        self.cushion_survives_at(area) && !self.cushion_is_buried(area)
    }

    /// `Cushion.wouldSurviveAt`.
    pub fn cushion_survives_at(&self, area: Aabb) -> bool {
        self.cushion_has_anchor_below(area) && !self.cushion_is_covered(area)
    }

    /// `Cushion.hasAnchorBelow`: a block whose outline reaches into the thin slab under the box.
    fn cushion_has_anchor_below(&self, area: Aabb) -> bool {
        let anchor = Aabb::new(
            area.min_x,
            area.min_y - RESTING,
            area.min_z,
            area.max_x.next_down(),
            area.min_y,
            area.max_z.next_down(),
        );
        positions(anchor.expand_towards(0.0, -REACH_BELOW, 0.0)).any(|pos| {
            let state = self.block_state_or_air(pos.x, pos.y, pos.z);
            crate::block::registry::outline_shape(state, [pos.x, pos.y, pos.z])
                .bounds()
                .is_some_and(|bounds| bounds.move_by_block_pos(pos).intersects(anchor))
        })
    }

    /// `Cushion.isCoveredBySuffocatingBlocks`: every block the box meets is suffocating.
    fn cushion_is_covered(&self, area: Aabb) -> bool {
        positions(area.deflate(Shapes::EPSILON)).all(|pos| {
            let state = self.block_state_or_air(pos.x, pos.y, pos.z);
            crate::block::state(state).is_some_and(|info| info.suffocating)
        })
    }

    /// `Cushion.isAnchorBuried`: block collisions leave nothing of the sliver the box rests on.
    /// Collisions are asked with no entity: powder snow gives none, scaffolding gives its stable
    /// shape (the context with no entity is above it and not descending), and a moving piston gives
    /// what its block entity fills now.
    pub(crate) fn cushion_is_buried(&self, area: Aabb) -> bool {
        let slice = Aabb::new(
            area.min_x,
            area.min_y,
            area.min_z,
            area.max_x,
            area.min_y + RESTING,
            area.max_z,
        )
        .deflate(Shapes::EPSILON);
        let Ok(mut exposed) = Shapes::create_from_aabb(slice) else {
            return false;
        };
        let colliders = self.collect_block_collisions_with(slice, |state, at| {
            Some(self.unattended_collision(state, at))
        });
        for collision in colliders {
            exposed = Shapes::join(&exposed, &collision.shape, BooleanOp::OnlyFirst);
            if exposed.is_empty() {
                return true;
            }
        }
        false
    }

    /// A block's collision shape when no entity asks (`CollisionContext.empty`).
    fn unattended_collision(
        &self,
        state: crate::BlockStateId,
        at: [i32; 3],
    ) -> photon_core::shape::VoxelShape {
        let name = crate::block::state(state).map(|info| info.block().name);
        match name {
            Some(name) if name.ends_with(":powder_snow") => Shapes::empty(),
            Some(name) if name.ends_with(":moving_piston") => {
                self.moving_piston_collision_shape((at[0], at[1], at[2]), None)
            }
            _ => self.block_state_registry.collision_shape_at(state, at),
        }
    }
}
