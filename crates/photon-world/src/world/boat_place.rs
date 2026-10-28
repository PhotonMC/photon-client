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
    ///
    /// `player` is the local player's box, which is not among the entities.
    pub fn boat_fits(&self, position: Vec3, area: Aabb, player: Aabb) -> bool {
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
            && !player.intersects(area.inflate_all(1.0e-7))
            && !self.pushable_in(area)
            && self.no_border_collision(Some(source), area)
    }

    /// The living entities `collision_candidate` leaves out that a boat cannot pass: `isPushable`
    /// is alive, not a spectator and not climbing, but for bats and armor stands (never), horses,
    /// llamas and camels carrying a rider (never) and parrots (always).
    fn pushable_in(&self, area: Aabb) -> bool {
        let query = area.inflate_all(1.0e-7);
        self.entities.values().any(|entity| {
            entity.is_living_entity()
                && entity
                    .collision_candidate(None, None, None, false)
                    .is_none()
                && entity.bounding_box().intersects(query)
                && Self::is_pushable_mob(entity)
        })
    }

    fn is_pushable_mob(entity: &crate::ClientEntity) -> bool {
        let name = photon_data::entity_type::name(entity.type_id);
        match name {
            Some("minecraft:bat" | "minecraft:armor_stand") => false,
            Some("minecraft:parrot") => true,
            Some("minecraft:llama" | "minecraft:trader_llama") if entity.is_vehicle() => false,
            _ if entity.is_horse() && entity.is_vehicle() => false,
            _ => !entity.is_dead_or_dying() && !entity.is_spectator() && !entity.on_climbable,
        }
    }
}
