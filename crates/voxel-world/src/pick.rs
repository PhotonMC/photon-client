//! Which entities the crosshair can land on: `Entity.isPickable`,
//! `getPickRadius`, `canBePickedFromInside`, and
//! `ProjectileUtil.getEntityHitResult` over the world's entities.
//!
//! The per-type answers are a table captured from the reference
//! (`tools/generate/entity_picks.py`). Not carried: an armor stand's marker
//! flag (a marker would still be picked), a sulfur cube's body item, the
//! dragon's parts (not entities here) and the scale attribute.

use voxel_core::aabb::Aabb;
use voxel_core::math::Vec3;

use crate::{ClientEntity, ClientEntityDimensions, ClientWorld};

impl ClientEntity {
    /// `Entity.isPickable` of the entity as it stands: the type's answer,
    /// and a player who only watches is not aimed at.
    pub fn is_pickable(&self) -> bool {
        if self.is_player() {
            // `Player.isPickable`.
            return !self.is_spectator();
        }
        voxel_data::entity_type::pick(self.type_id).is_some_and(|pick| pick.pickable)
    }

    /// The size the type starts with (a baby's, for the types that have
    /// babies, a player's by pose). Not carried: the scale attribute and the
    /// poses of the mobs that change size with them, bar a fish's.
    pub(crate) fn dimensions(&self) -> ClientEntityDimensions {
        match voxel_data::entity_type::pick(self.type_id) {
            Some(pick) if !self.is_player() => {
                let (width, height, eye_height) = match pick.baby {
                    Some(baby) if self.baby => baby,
                    _ => pick.size,
                };
                let scale = self.fish_dimension_scale();
                ClientEntityDimensions::from_values(width, height, eye_height, false)
                    .scale(scale, scale)
            }
            _ => self.pose.player_dimensions(),
        }
    }

    /// `Entity.getBoundingBox`: the box of those dimensions around where the
    /// entity is.
    pub fn bounding_box(&self) -> Aabb {
        let (x, y, z) = self.position();
        self.dimensions().make_bounding_box(Vec3::new(x, y, z))
    }

    /// `Entity.getPickRadius`: what the aim inflates the box by.
    pub fn pick_radius(&self) -> f32 {
        voxel_data::entity_type::pick(self.type_id).map_or(0.0, |pick| pick.pick_radius)
    }

    /// `Entity.canBePickedFromInside`.
    pub fn can_be_picked_from_inside(&self) -> bool {
        voxel_data::entity_type::pick(self.type_id).is_none_or(|pick| pick.from_inside)
    }
}

/// An entity the aim lands on, and where.
#[derive(Debug, Clone, Copy)]
pub struct EntityHit {
    pub entity_id: i32,
    pub location: Vec3,
}

impl ClientWorld {
    /// `ProjectileUtil.getEntityHitResult(except, from, to, box, CAN_BE_PICKED,
    /// maxValue)`: the nearest pickable entity the segment crosses, as the
    /// local player (`except`) sees it.
    pub fn entity_hit_result(
        &self,
        except: i32,
        from: Vec3,
        to: Vec3,
        search: Aabb,
        max_value: f64,
    ) -> Option<EntityHit> {
        let mut nearest = max_value;
        let mut hovered: Option<EntityHit> = None;
        let except_root = self.root_vehicle_id(except);
        for entity in self.entities.values() {
            if entity.id == except
                || !entity.is_pickable()
                || !entity.bounding_box().intersects(search)
            {
                continue;
            }
            let bb = entity
                .bounding_box()
                .inflate_all(f64::from(entity.pick_radius()));
            let clip_point = bb.clip(from, to);
            if bb.contains_vec3(from) {
                if nearest >= 0.0 && entity.can_be_picked_from_inside() {
                    hovered = Some(EntityHit {
                        entity_id: entity.id,
                        location: clip_point.unwrap_or(from),
                    });
                    nearest = 0.0;
                }
            } else if let Some(location) = clip_point {
                let distance = from.distance_to_sqr(location);
                if distance < nearest || nearest == 0.0 {
                    if self.root_vehicle_id(entity.id) == except_root {
                        if nearest == 0.0 {
                            hovered = Some(EntityHit {
                                entity_id: entity.id,
                                location,
                            });
                        }
                    } else {
                        hovered = Some(EntityHit {
                            entity_id: entity.id,
                            location,
                        });
                        nearest = distance;
                    }
                }
            }
        }
        hovered
    }
}
