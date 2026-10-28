//! What `BoatItem.use` asks of the level before a boat is placed: no pickable entity holds the
//! eye, and nothing (block, entity or the world border) is in the new boat's box.

use photon_core::aabb::Aabb;
use photon_core::ids::EntityId;
use photon_core::math::Vec3;

use crate::ClientWorld;

impl ClientWorld {
    /// `BoatItem.use`'s guard: a pickable entity in `area` (but `player`) whose box, grown by its
    /// pick radius, holds the eye.
    pub fn pickable_holds_eye(&self, player: EntityId, area: Aabb, eye: Vec3) -> bool {
        self.pickable_boxes(player)
            .filter(|pickable| pickable.bounds.intersects(area))
            .any(|pickable| {
                pickable
                    .bounds
                    .inflate_all(f64::from(pickable.pick_radius))
                    .contains_vec3(eye)
            })
    }

    /// `Level.noCollision(boat, box)` for a boat about to stand at `position`: the blocks, the
    /// entities a boat cannot pass (pushable ones too, a boat being the source), and the border.
    pub fn boat_fits(&self, position: Vec3, area: Aabb) -> bool {
        let boat = photon_data::entity_type::id("minecraft:oak_boat");
        let candidates = self.indexed_entity_collision_candidates(None, boat, None, area);
        let entities = crate::collision::collect_entity_collision_shapes_for_source(
            boat,
            false,
            area,
            &candidates,
        );
        let source = crate::collision::ClientWorldCollisionSource { position };
        self.no_block_collision_in_registry(area)
            && entities.is_empty()
            && self.no_border_collision(Some(source), area)
    }
}
