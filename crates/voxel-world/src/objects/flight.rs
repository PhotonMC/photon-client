//! The client tick of the projectiles the server updates in coarse steps: the hurting projectiles
//! (fireballs, skulls, wind charges, the dragon's fireball), the shulker bullet and the llama's
//! spit. Their position, motion and turn are guessed between the server's packets as the
//! reference does: inertia, the clipped step against blocks and entities, the turn towards the
//! movement and the current of a fluid.

use voxel_core::aabb::Aabb;
use voxel_core::java_float::{JavaF32, JavaF64};
use voxel_core::math::{Vec3, mth_atan2, mth_lerp_f32};

use crate::ClientWorld;
use crate::constants::PROJECTILE_DEGREES_PER_RADIAN;
use crate::entity::ClientEntity;
use crate::fluid::{self, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use crate::local_player::Level;

/// What the client keeps of a flying projectile between ticks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlightState {
    /// `Projectile.getOwner`, as the add packet named it.
    pub(crate) owner: Option<i32>,
    /// `Projectile.leftOwner`: it has been clear of its owner's boxes once.
    pub(crate) left_owner: bool,
}

/// How a projectile flies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Flight {
    /// `AbstractHurtingProjectile.tick`.
    Hurting {
        acceleration: f64,
        inertia: f32,
        liquid_inertia: f32,
    },
    /// `ShulkerBullet.tick` as a client runs it: it only drifts on the motion it was told.
    ShulkerBullet,
    /// `LlamaSpit.tick`.
    LlamaSpit,
}

/// The flight of an entity, if it is one of these.
pub(crate) fn flight_of(entity: &ClientEntity) -> Option<Flight> {
    let hurting = |acceleration, inertia, liquid_inertia| Flight::Hurting {
        acceleration,
        inertia,
        liquid_inertia,
    };
    Some(match voxel_data::entity_type::name(entity.type_id)? {
        "minecraft:fireball" | "minecraft:small_fireball" | "minecraft:dragon_fireball" => {
            hurting(0.1, 0.95, 0.8)
        }
        // `WitherSkull.getInertia`.
        "minecraft:wither_skull" => hurting(
            0.1,
            if entity.living.wither_skull_dangerous {
                0.73
            } else {
                0.95
            },
            0.8,
        ),
        // `AbstractWindCharge`: no acceleration of its own, and the same inertia in a fluid.
        "minecraft:wind_charge" | "minecraft:breeze_wind_charge" => hurting(0.0, 1.0, 1.0),
        "minecraft:shulker_bullet" => Flight::ShulkerBullet,
        "minecraft:llama_spit" => Flight::LlamaSpit,
        _ => return None,
    })
}

/// `ProjectileUtil.rotateTowardsMovement`: the entity turns `speed` of the way to face its movement.
/// (The reference's `* 180.0F / (float) Math.PI` is folded by the compiler into one constant.)
fn rotate_towards_movement(entity: &mut ClientEntity, speed: f32) {
    let movement = entity.motion();
    if movement.0 * movement.0 + movement.1 * movement.1 + movement.2 * movement.2 == 0.0 {
        return;
    }
    let horizontal = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
    let y_rot = (mth_atan2(movement.2, movement.0) * PROJECTILE_DEGREES_PER_RADIAN) as f32 + 90.0;
    let x_rot = (mth_atan2(horizontal, movement.1) * PROJECTILE_DEGREES_PER_RADIAN) as f32 - 90.0;
    let mut x_rot_old = entity.old_x_rot.get();
    let mut y_rot_old = entity.old_y_rot.get();
    while x_rot - x_rot_old < -180.0 {
        x_rot_old -= 360.0;
    }
    while x_rot - x_rot_old >= 180.0 {
        x_rot_old += 360.0;
    }
    while y_rot - y_rot_old < -180.0 {
        y_rot_old -= 360.0;
    }
    while y_rot - y_rot_old >= 180.0 {
        y_rot_old += 360.0;
    }
    entity.old_x_rot = JavaF32::new(x_rot_old);
    entity.old_y_rot = JavaF32::new(y_rot_old);
    entity.x_rot = JavaF32::new(mth_lerp_f32(speed, x_rot_old, x_rot));
    entity.y_rot = JavaF32::new(mth_lerp_f32(speed, y_rot_old, y_rot));
}

/// `Projectile.updateRotation`, which turns a fifth of the way to the movement.
fn update_rotation(entity: &mut ClientEntity) {
    let movement = entity.motion();
    let horizontal = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
    let target_x = (mth_atan2(movement.1, horizontal) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
    let target_y = (mth_atan2(movement.0, movement.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
    entity.x_rot = JavaF32::new(lerp_rotation(entity.old_x_rot.get(), target_x));
    entity.y_rot = JavaF32::new(lerp_rotation(entity.old_y_rot.get(), target_y));
}

/// `Projectile.lerpRotation`.
fn lerp_rotation(mut old: f32, target: f32) -> f32 {
    while target - old < -180.0 {
        old -= 360.0;
    }
    while target - old >= 180.0 {
        old += 360.0;
    }
    mth_lerp_f32(0.2, old, target)
}

/// `ProjectileUtil.computeMargin`: how far a target's box is inflated, growing over the first
/// ticks so that a projectile does not hit what it was shot from.
fn margin(tick_count: i32) -> f32 {
    ((tick_count - 2) as f32 / 20.0).clamp(0.0, 0.3)
}

fn vec3(position: (f64, f64, f64)) -> Vec3 {
    Vec3::new(position.0, position.1, position.2)
}

impl ClientWorld {
    /// One tick of a flying projectile, after its old position and turn were kept and its tick
    /// counted.
    pub(crate) fn tick_flying_projectile(&mut self, entity_id: i32, flight: Flight) {
        match flight {
            Flight::Hurting {
                acceleration,
                inertia,
                liquid_inertia,
            } => self.tick_hurting_projectile(entity_id, acceleration, inertia, liquid_inertia),
            Flight::ShulkerBullet => self.tick_shulker_bullet(entity_id),
            Flight::LlamaSpit => self.tick_llama_spit(entity_id),
        }
    }

    /// `AbstractHurtingProjectile.tick`.
    fn tick_hurting_projectile(
        &mut self,
        entity_id: i32,
        acceleration: f64,
        inertia: f32,
        liquid_inertia: f32,
    ) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        // `applyInertia`: what the last tick found in the water decides it.
        let movement = vec3(entity.motion());
        let inertia = f64::from(if entity.living.in_water {
            liquid_inertia
        } else {
            inertia
        });
        let movement = movement
            .add_vec3(movement.normalized().scale(acceleration))
            .scale(inertia);
        let position = vec3(entity.position());
        let target = position.add_vec3(movement);
        let mut end = target;
        let block = self.clip_blocks(position, target);
        if let Some(location) = block {
            end = location;
        }
        let search = entity
            .bounding_box()
            .expand_towards_vec3(movement)
            .inflate_all(1.0);
        let hit = self.projectile_entity_hit(
            entity_id,
            position,
            end,
            search,
            f64::from(margin(entity.tick_count)),
        );
        let new_position = hit.or(block).unwrap_or(target);
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the projectile was read above");
        entity.motion.x = JavaF64::new(movement.x);
        entity.motion.y = JavaF64::new(movement.y);
        entity.motion.z = JavaF64::new(movement.z);
        rotate_towards_movement(entity, 0.2);
        entity.set_position((new_position.x, new_position.y, new_position.z));
        // `Projectile.tick`: the owner check, then `Entity.baseTick`'s fluids.
        self.check_left_owner(entity_id, movement);
        self.tick_projectile_fluids(entity_id);
    }

    /// `ShulkerBullet.tick` where the level is a client's: no target and no hit are known, so the
    /// bullet goes on by the motion it has and turns half the way towards it.
    fn tick_shulker_bullet(&mut self, entity_id: i32) {
        self.check_left_owner_of(entity_id);
        self.tick_projectile_fluids(entity_id);
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return;
        };
        let movement = vec3(entity.motion());
        let position = vec3(entity.position()).add_vec3(movement);
        entity.set_position((position.x, position.y, position.z));
        rotate_towards_movement(entity, 0.5);
    }

    /// `LlamaSpit.tick`.
    fn tick_llama_spit(&mut self, entity_id: i32) {
        self.check_left_owner_of(entity_id);
        self.tick_projectile_fluids(entity_id);
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let movement = vec3(entity.motion());
        let position = vec3(entity.position());
        let target = position.add_vec3(movement);
        let in_water = entity.living.in_water;
        let no_gravity = entity.no_gravity;
        let bounds = entity.bounding_box();
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the projectile was read above");
        update_rotation(entity);
        // It dies in blocks (no air in its box) and in water.
        if self.box_has_no_air(bounds) || in_water {
            self.remove_entity(entity_id);
            return;
        }
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the projectile was read above");
        // `getAirDrag` and `getDefaultGravity` of the spit.
        let drag = f64::from(0.99_f32);
        let gravity = if no_gravity { 0.0 } else { 0.06 };
        entity.motion.x = JavaF64::new(movement.x * drag);
        entity.motion.y = JavaF64::new(movement.y * drag - gravity);
        entity.motion.z = JavaF64::new(movement.z * drag);
        entity.set_position((target.x, target.y, target.z));
    }

    /// `Level.getBlockStates(box).noneMatch(isAir)`: every block the box touches holds something.
    fn box_has_no_air(&self, bounds: Aabb) -> bool {
        use voxel_core::math::floor_f64;
        for x in floor_f64(bounds.min_x)..=floor_f64(bounds.max_x) {
            for y in floor_f64(bounds.min_y)..=floor_f64(bounds.max_y) {
                for z in floor_f64(bounds.min_z)..=floor_f64(bounds.max_z) {
                    // A block that is not loaded reads as air.
                    if self
                        .block_state_at(x, y, z)
                        .is_none_or(crate::block::rules::state_is_air)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// `Projectile.checkLeftOwner` with the movement the entity has.
    fn check_left_owner_of(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let movement = vec3(entity.motion());
        self.check_left_owner(entity_id, movement);
    }

    /// `Projectile.checkLeftOwner`: once the projectile and its owner's boxes no longer meet, the
    /// owner can be hit.
    fn check_left_owner(&mut self, entity_id: i32, movement: Vec3) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        if entity.flight.left_owner {
            return;
        }
        // The owner's boxes: its root vehicle and everything on it that can be picked.
        let reach = entity
            .bounding_box()
            .expand_towards_vec3(movement)
            .inflate_all(1.0);
        let left = match entity.flight.owner {
            None => true,
            Some(owner) => !self
                .family_of(self.root_vehicle_id(owner))
                .into_iter()
                .filter_map(|id| self.entities.get(&id))
                .filter(|member| member.is_pickable())
                .any(|member| reach.intersects(member.bounding_box())),
        };
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.flight.left_owner = left;
        }
    }

    /// `Entity.getSelfAndPassengers`: the entity and every passenger below it.
    fn family_of(&self, entity_id: i32) -> Vec<i32> {
        let mut family = vec![entity_id];
        let mut index = 0;
        while let Some(&id) = family.get(index) {
            if let Some(passengers) = self.entity_passengers(id) {
                let new: Vec<i32> = passengers
                    .iter()
                    .copied()
                    .filter(|p| !family.contains(p))
                    .collect();
                family.extend(new);
            }
            index += 1;
        }
        family
    }

    /// `Entity.updateFluidInteraction` of a projectile: whether it is in water for the next tick,
    /// and what a current adds to its movement.
    fn tick_projectile_fluids(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let Some(pick) = voxel_data::entity_type::pick(entity.type_id) else {
            return;
        };
        let position = vec3(entity.position());
        let Ok(found) = fluid::scan(
            &Level::new(self),
            (entity.bounding_box(), position, pick.size.2),
            false,
        ) else {
            return;
        };
        let mut movement = vec3(entity.motion());
        if found.water.height > 0.0 {
            movement = fluid::apply_current(found.water, movement, WATER_FLOW_SCALE, Pushed::Other);
        }
        if found.lava.height > 0.0 {
            movement = fluid::apply_current(found.lava, movement, LAVA_FLOW_SCALE, Pushed::Other);
        }
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the projectile was read above");
        entity.living.in_water = found.water.height > 0.0;
        entity.motion.x = JavaF64::new(movement.x);
        entity.motion.y = JavaF64::new(movement.y);
        entity.motion.z = JavaF64::new(movement.z);
    }

    /// `ProjectileUtil.getEntityHitResult` with `Projectile.canHitEntity`: where the nearest
    /// entity the step crosses is hit, if the projectile can hit one.
    fn projectile_entity_hit(
        &self,
        source: i32,
        from: Vec3,
        to: Vec3,
        search: Aabb,
        margin: f64,
    ) -> Option<Vec3> {
        let flight = self.entities.get(&source)?.flight;
        let owner_root = flight.owner.map(|owner| self.root_vehicle_id(owner));
        let mut nearest = f64::MAX;
        let mut location = None;
        for candidate in self.pickable_boxes(source) {
            if !candidate.can_be_hit {
                continue;
            }
            if let Some(root) = owner_root
                && !flight.left_owner
                && self.root_vehicle_id(candidate.id) == root
            {
                continue;
            }
            if !candidate.bounds.intersects(search) {
                continue;
            }
            if let Some(point) = candidate.bounds.inflate_all(margin).clip(from, to) {
                let distance = from.distance_to_sqr(point);
                if distance < nearest {
                    nearest = distance;
                    location = Some(point);
                }
            }
        }
        location
    }
}
