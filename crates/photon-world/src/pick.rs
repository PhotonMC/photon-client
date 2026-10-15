//! Which entities the crosshair can land on: `Entity.isPickable`,
//! `getPickRadius`, `canBePickedFromInside`, and
//! `ProjectileUtil.getEntityHitResult` over the world's entities.
//!
//! The per-type answers are a table captured from the reference
//! (`tools/generate/entity_picks.py`). Not carried: an armor stand's marker
//! flag (a marker would still be picked), a sulfur cube's body item and the
//! scale attribute. An ender dragon is not pickable itself; its parts are
//! (`EnderDragonPart.isPickable`), each with the id after the dragon's.

use photon_core::aabb::Aabb;
use photon_core::direction::{Axis, Direction};
use photon_core::ids::EntityId;
use photon_core::math::{Vec3, floor_f64};

use crate::{ClientEntity, ClientEntityDimensions, ClientWorld};

impl ClientEntity {
    /// `Entity.isPickable` of the entity as it stands: the type's answer,
    /// and a player who only watches is not aimed at.
    pub fn is_pickable(&self) -> bool {
        if self.is_player() {
            // `Player.isPickable`.
            return !self.is_spectator();
        }
        photon_data::entity_type::pick(self.type_id).is_some_and(|pick| pick.pickable)
    }

    /// `Entity.getEyeHeight`, by the size the type starts with.
    pub fn eye_height(&self) -> f32 {
        self.dimensions().eye_height()
    }

    /// The size the type starts with (a baby's, for the types that have
    /// babies, a player's by pose). Not carried: the scale attribute and the
    /// poses of the mobs that change size with them, bar a fish's.
    /// `Entity.getBbHeight`.
    pub fn bb_height(&self) -> f32 {
        self.dimensions().height()
    }

    pub(crate) fn dimensions(&self) -> ClientEntityDimensions {
        match photon_data::entity_type::pick(self.type_id) {
            Some(pick) if !self.is_player() => {
                let (width, height, eye_height) = match pick.baby {
                    Some(baby) if self.baby => baby,
                    _ => pick.size,
                };
                let scale = if self.is_cube_mob() {
                    // `AbstractCubeMob.getDefaultDimensions`.
                    self.cube_size() as f32
                } else {
                    self.fish_dimension_scale()
                };
                ClientEntityDimensions::from_values(width, height, eye_height, false)
                    .scale(scale, scale)
            }
            _ => self.pose.player_dimensions(),
        }
    }

    /// The size a leash is tied by (`getBbWidth`, `getBbHeight`, `getEyeHeight`): the size the
    /// type starts with, grown by the scale attribute.
    pub fn leash_size(&self) -> photon_data::leash::Size {
        let scale = if self.is_player() { 1.0 } else { self.scale() };
        let dimensions = self.dimensions().scale(scale, scale);
        photon_data::leash::Size {
            width: dimensions.width(),
            height: dimensions.height(),
            eye_height: dimensions.eye_height(),
        }
    }

    /// `Entity.getPreciseBodyRotation`: a living entity's body, else the way it faces.
    pub fn precise_body_rotation(&self, partial_tick: f32) -> f32 {
        let (old, current) = if self.is_living_entity() {
            (self.previous_body_y_rotation(), self.body_y_rotation())
        } else {
            (self.old_y_rot.get(), self.y_rot.get())
        };
        photon_core::math::mth_lerp_f32(partial_tick, old, current)
    }

    /// What a camel's leash offset follows: whether it sits, and the game time since its pose
    /// last changed.
    pub fn camel_leash_pose(
        &self,
        game_time: i64,
        partial_tick: f32,
    ) -> photon_data::leash::CamelPose {
        self.horse().map_or_else(Default::default, |horse| {
            let (sitting, pose_time) = horse.camel.leash_pose(game_time);
            photon_data::leash::CamelPose {
                sitting,
                pose_time,
                partial: partial_tick,
            }
        })
    }

    /// `Entity.getBoundingBox`: the box of those dimensions around where the
    /// entity is.
    pub fn bounding_box(&self) -> Aabb {
        if self.is_shulker() {
            return self.shulker_bounding_box();
        }
        if let Some(hanging) = self.hanging_bounding_box() {
            return hanging;
        }
        let (x, y, z) = self.position();
        if matches!(
            photon_data::entity_type::name(self.type_id),
            Some("minecraft:wind_charge" | "minecraft:breeze_wind_charge")
        ) {
            // `AbstractWindCharge.makeBoundingBox`: the box hangs a little below the position.
            let dimensions = self.dimensions();
            let half = f64::from(dimensions.width() / 2.0);
            let low = y - f64::from(0.15_f32);
            return Aabb::new(
                x - half,
                low,
                z - half,
                x + half,
                low + f64::from(dimensions.height()),
                z + half,
            );
        }
        self.dimensions().make_bounding_box(Vec3::new(x, y, z))
    }

    /// `ItemFrame.createBoundingBox`/`Painting.calculateBoundingBox`: an item frame's or a
    /// painting's box, centred `0.46875` off the wall it is attached to (`ClientboundAddEntityPacket.
    /// getData()`'s 3D direction value, `this.data`) and sized by that direction's axis; a framed
    /// map widens an item frame to 1x1, a painting's registry variant gives its own width/height
    /// (shifted along the wall and up for an even size, so it still centres on the block grid).
    /// Nothing for another type.
    fn hanging_bounding_box(&self) -> Option<Aabb> {
        let is_item_frame = self.is_item_frame();
        let is_painting =
            photon_data::entity_type::name(self.type_id) == Some("minecraft:painting");
        if !is_item_frame && !is_painting {
            return None;
        }
        let direction = Direction::from_3d_data_value(self.data);
        let (x, y, z) = self.position();
        let block = (floor_f64(x), floor_f64(y), floor_f64(z));
        // `Vec3.atCenterOf(block).relative(direction, -0.46875)`.
        const WALL_SHIFT: f64 = 0.46875;
        let (step_x, step_y, step_z) = direction.step();
        let wall_center = (
            f64::from(block.0) + 0.5 + f64::from(step_x) * -WALL_SHIFT,
            f64::from(block.1) + 0.5 + f64::from(step_y) * -WALL_SHIFT,
            f64::from(block.2) + 0.5 + f64::from(step_z) * -WALL_SHIFT,
        );
        let (center, width, height) = if is_item_frame {
            let has_framed_map = self.item_stack().is_some_and(|stack| {
                let mut bytes = stack.patch_bytes();
                crate::item_components::read_patch(&mut bytes).is_ok_and(|patch| {
                    patch
                        .added
                        .iter()
                        .any(|(id, _)| *id == photon_data::item::component_types::MAP_ID)
                })
            });
            let size = if has_framed_map { 1.0 } else { 0.75 };
            (wall_center, size, size)
        } else {
            let variant = self
                .living
                .variant
                .and_then(|id| photon_data::entity_type::painting_variants().nth(id as usize))?;
            let offset_for = |size: i32| if size % 2 == 0 { 0.5 } else { 0.0 };
            let left = direction.counter_clockwise_y().unwrap_or(direction);
            let (left_x, _, left_z) = left.step();
            let horizontal = offset_for(variant.width);
            let vertical = offset_for(variant.height);
            let center = (
                wall_center.0 + f64::from(left_x) * horizontal,
                wall_center.1 + vertical,
                wall_center.2 + f64::from(left_z) * horizontal,
            );
            (center, f64::from(variant.width), f64::from(variant.height))
        };
        let axis = direction.axis();
        let x_size = if axis == Axis::X { 0.0625 } else { width };
        let y_size = if axis == Axis::Y { 0.0625 } else { height };
        let z_size = if axis == Axis::Z { 0.0625 } else { width };
        Some(Aabb::new(
            center.0 - x_size / 2.0,
            center.1 - y_size / 2.0,
            center.2 - z_size / 2.0,
            center.0 + x_size / 2.0,
            center.1 + y_size / 2.0,
            center.2 + z_size / 2.0,
        ))
    }

    /// `Entity.getPickRadius`: what the aim inflates the box by.
    pub fn pick_radius(&self) -> f32 {
        photon_data::entity_type::pick(self.type_id).map_or(0.0, |pick| pick.pick_radius)
    }

    /// `Entity.canBePickedFromInside`.
    pub fn can_be_picked_from_inside(&self) -> bool {
        photon_data::entity_type::pick(self.type_id).is_none_or(|pick| pick.from_inside)
    }
}

/// An entity the aim lands on (a dragon's part, by its own id), where, and
/// where it stands and what box it has.
#[derive(Debug, Clone, Copy)]
pub struct EntityHit {
    pub entity_id: EntityId,
    pub location: Vec3,
    pub position: (f64, f64, f64),
    pub bounds: Aabb,
}

/// An entity a segment can be cast at: its id (a dragon's part has its own), where it stands, its
/// box, how far the aim inflates that box, whether it is picked from inside, and whether a
/// projectile can hit it (`Entity.canBeHitByProjectile`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PickableBox {
    pub(crate) id: EntityId,
    pub(crate) position: (f64, f64, f64),
    pub(crate) bounds: Aabb,
    pub(crate) pick_radius: f32,
    pub(crate) from_inside: bool,
    pub(crate) can_be_hit: bool,
}

impl ClientWorld {
    /// The pickable entities but `except`, then the dragons' parts (`Level.getEntities` lists
    /// those last).
    pub(crate) fn pickable_boxes(
        &self,
        except: EntityId,
    ) -> impl Iterator<Item = PickableBox> + '_ {
        let parts = self
            .entities
            .values()
            .filter(move |entity| entity.is_ender_dragon() && entity.id != except)
            .flat_map(|dragon| {
                dragon
                    .ender_dragon()
                    .into_iter()
                    .flat_map(|state| state.parts().iter().enumerate())
                    .map(|(index, part)| {
                        let size = ClientEntityDimensions::from_values(
                            part.width,
                            part.height,
                            part.height * 0.85,
                            false,
                        );
                        let (x, y, z) = part.position;
                        PickableBox {
                            id: EntityId(dragon.id.0 + 1 + index as i32),
                            position: part.position,
                            bounds: size.make_bounding_box(Vec3::new(x, y, z)),
                            pick_radius: 0.0,
                            from_inside: true,
                            can_be_hit: true,
                        }
                    })
            });
        let others = self
            .entities
            .values()
            .filter(move |entity| entity.id != except && entity.is_pickable())
            .map(|entity| PickableBox {
                id: entity.id,
                position: entity.position(),
                bounds: entity.bounding_box(),
                pick_radius: entity.pick_radius(),
                from_inside: entity.can_be_picked_from_inside(),
                can_be_hit: !entity.is_dead_or_dying(),
            });
        others.chain(parts)
    }

    /// `ProjectileUtil.getEntityHitResult(except, from, to, box, CAN_BE_PICKED,
    /// maxValue)`: the nearest pickable entity the segment crosses, as the
    /// local player (`except`) sees it.
    pub fn entity_hit_result(
        &self,
        except: EntityId,
        from: Vec3,
        to: Vec3,
        search: Aabb,
        max_value: f64,
    ) -> Option<EntityHit> {
        let mut nearest = max_value;
        let mut hovered: Option<EntityHit> = None;
        let except_root = self.root_vehicle_id(except);
        for PickableBox {
            id,
            position,
            bounds,
            pick_radius,
            from_inside,
            ..
        } in self.pickable_boxes(except)
        {
            if !bounds.intersects(search) {
                continue;
            }
            let bb = bounds.inflate_all(f64::from(pick_radius));
            let clip_point = bb.clip(from, to);
            let hit = |location| EntityHit {
                entity_id: id,
                location,
                position,
                bounds,
            };
            if bb.contains_vec3(from) {
                if nearest >= 0.0 && from_inside {
                    hovered = Some(hit(clip_point.unwrap_or(from)));
                    nearest = 0.0;
                }
            } else if let Some(location) = clip_point {
                let distance = from.distance_to_sqr(location);
                if distance < nearest || nearest == 0.0 {
                    // A part is its own root vehicle; so is an entity that rides nothing.
                    if self.root_vehicle_id(id) == except_root {
                        if nearest == 0.0 {
                            hovered = Some(hit(location));
                        }
                    } else {
                        hovered = Some(hit(location));
                        nearest = distance;
                    }
                }
            }
        }
        hovered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DimensionBounds, EntityInput};
    use photon_core::binary::Uuid;

    fn world_with_dragon() -> ClientWorld {
        let mut world = ClientWorld::new(DimensionBounds::default());
        let dragon = photon_data::entity_type::id("minecraft:ender_dragon").expect("a dragon");
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(100),
            uuid: Uuid::new(0, 100),
            type_id: dragon,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        // No chunk is loaded here to tick it in: it ticks all the same.
        world
            .entities
            .at_mut(EntityId(100))
            .expect("the dragon")
            .ticking = true;
        world.tick_entities();
        world
    }

    /// The dragon itself is not pickable, its parts are: an aim at its head lands on the
    /// head's id (the dragon's, plus its place, plus one), and an aim at where it has no
    /// part lands on nothing.
    #[test]
    fn the_aim_lands_on_a_dragons_part_and_not_on_the_dragon() {
        let world = world_with_dragon();
        let head = world
            .entities()
            .at(EntityId(100))
            .and_then(|dragon| dragon.ender_dragon())
            .map(|state| state.parts()[0].position)
            .expect("a head");
        // From in front of the dragon (the head is the part farthest ahead), towards it.
        let from = Vec3::new(head.0, head.1 + 0.5, head.2 - 10.0);
        let to = Vec3::new(head.0, head.1 + 0.5, head.2 + 10.0);
        let search = Aabb::new(
            from.x - 1.0,
            from.y - 1.0,
            from.z - 1.0,
            to.x + 1.0,
            to.y + 1.0,
            to.z + 1.0,
        );
        let hit = world
            .entity_hit_result(EntityId(1), from, to, search, 400.0)
            .expect("a hit");
        assert_eq!(hit.entity_id, EntityId(101));
        // A box the size of the head, round where it stands.
        assert!((hit.bounds.max_x - hit.bounds.min_x - 1.0).abs() < 1e-6);
        let away = Vec3::new(head.0 + 60.0, head.1, head.2 - 10.0);
        let away_to = Vec3::new(head.0 + 60.0, head.1, head.2 + 10.0);
        let far = Aabb::new(
            away.x - 1.0,
            away.y - 1.0,
            away.z - 1.0,
            away_to.x + 1.0,
            away_to.y + 1.0,
            away_to.z + 1.0,
        );
        assert!(
            world
                .entity_hit_result(EntityId(1), away, away_to, far, 400.0)
                .is_none()
        );
    }
}
