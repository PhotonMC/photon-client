//! What `HangingEntityItem.useOn` asks of the level before an item frame is hung: the frame's
//! thin box meets no block, boat or world border, the block behind it holds it, and no other
//! hanging entity faces the same way in the same place.

use photon_core::aabb::Aabb;
use photon_core::direction::Direction;
use photon_core::math::Vec3;

use crate::ClientWorld;
use crate::collision::ClientWorldCollisionSource;

/// The depth of a hanging entity's box along its facing axis, and its other two sizes.
const DEPTH: f64 = 0.0625;

/// `HangingEntity.recalculateBoundingBox` for a hanging entity `span` wide and high in the block
/// `pos` facing `direction`: the box sits against the wall behind it.
fn pop_box(pos: (i32, i32, i32), direction: Direction, span: f64) -> Aabb {
    let (dx, dy, dz) = direction.step();
    let centre = Vec3::new(
        f64::from(pos.0) + 0.5 - f64::from(dx) * 0.46875,
        f64::from(pos.1) + 0.5 - f64::from(dy) * 0.46875,
        f64::from(pos.2) + 0.5 - f64::from(dz) * 0.46875,
    );
    let size = |step: i32| if step == 0 { span } else { DEPTH } / 2.0;
    let (hx, hy, hz) = (size(dx), size(dy), size(dz));
    Aabb::new(
        centre.x - hx,
        centre.y - hy,
        centre.z - hz,
        centre.x + hx,
        centre.y + hy,
        centre.z + hz,
    )
}

/// The entity types that hang on a wall.
const HANGING: [photon_core::ids::EntityTypeId; 3] = [
    photon_data::entity_type::ids::ITEM_FRAME,
    photon_data::entity_type::ids::GLOW_ITEM_FRAME,
    photon_data::entity_type::ids::PAINTING,
];

impl ClientWorld {
    /// `ItemFrame.survives` for a new frame in `pos` facing `direction`.
    pub fn item_frame_survives(&self, pos: (i32, i32, i32), direction: Direction) -> bool {
        self.hanging_survives(pos, direction, (0.75, false))
    }

    /// `Painting.survives` for a one by one painting, the smallest variant: when it does not
    /// survive, no variant does.
    pub fn small_painting_survives(&self, pos: (i32, i32, i32), direction: Direction) -> bool {
        self.hanging_survives(pos, direction, (1.0, true))
    }

    fn hanging_survives(
        &self,
        pos: (i32, i32, i32),
        direction: Direction,
        (span, painting): (f64, bool),
    ) -> bool {
        let area = pop_box(pos, direction, span);
        let centre = Vec3::new(
            (area.min_x + area.max_x) / 2.0,
            (area.min_y + area.max_y) / 2.0,
            (area.min_z + area.max_z) / 2.0,
        );
        if !self.no_block_collision_in_registry(area)
            || !self
                .no_border_collision(Some(ClientWorldCollisionSource { position: centre }), area)
        {
            return false;
        }
        let (dx, dy, dz) = direction.step();
        let behind = self.block_state_or_air(pos.0 - dx, pos.1 - dy, pos.2 - dz);
        let holds =
            crate::block::state(behind).is_some_and(|info| {
                info.solid
                    || !direction.axis().is_vertical()
                        && info.block().class.lineage().any(|class| {
                            class == photon_data::block::classes::BlockClass::DiodeBlock
                        })
            });
        holds && !self.hanging_blocks(area, direction, painting)
    }

    /// `HangingEntity.canCoexist`: whether a hanging entity inside `area` faces `direction` (a
    /// frame's rule) or, for a painting, is a painting at all.
    fn hanging_blocks(&self, area: Aabb, direction: Direction, painting: bool) -> bool {
        self.entities.values().any(|entity| {
            HANGING.contains(&entity.type_id)
                && (entity.data == direction.data3d()
                    || painting && entity.type_id == photon_data::entity_type::ids::PAINTING)
                && entity.bounding_box().intersects(area)
        })
    }
}
