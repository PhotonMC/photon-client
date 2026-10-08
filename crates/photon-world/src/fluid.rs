//! What an entity's box finds in the fluids around it and what a current does
//! to it (`EntityFluidInteraction`), which the local player and the other
//! entities the client moves share.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::math::{Vec3, floor_f64};
use photon_data::block::definition::Fluid;

use crate::java_math;
use crate::local_player::{Level, Unsupported};

/// `Entity`'s water and lava current scales.
pub(crate) const WATER_FLOW_SCALE: f64 = 0.014;
pub(crate) const LAVA_FLOW_SCALE: f64 = 0.0023333333333333335;

/// One fluid's share of an entity's fluid interaction
/// (`EntityFluidInteraction.Tracker`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct FluidTracker {
    pub(crate) height: f64,
    pub(crate) eyes_inside: bool,
    pub(crate) accumulated_current: Vec3,
    pub(crate) current_count: i32,
}

impl Default for FluidTracker {
    fn default() -> Self {
        Self {
            height: 0.0,
            eyes_inside: false,
            accumulated_current: Vec3::ZERO,
            current_count: 0,
        }
    }
}

/// What a scan found: the water's and the lava's trackers.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Fluids {
    pub(crate) water: FluidTracker,
    pub(crate) lava: FluidTracker,
}

/// `EntityFluidInteraction.update` for the box shrunk by a thousandth
/// (`Entity.getFluidInteractionBox`): every fluid block it overlaps, the
/// height the fluid reaches over the entity's feet, whether the eyes are in
/// it, and, unless the entity ignores currents, the flow it feels.
pub(crate) fn scan(
    level: &Level<'_>,
    (bounding_box, position, eye_height): (Aabb, Vec3, f32),
    ignore_current: bool,
) -> Result<Fluids, Unsupported> {
    scan_in(
        level,
        Some(bounding_box.deflate(0.001)),
        (bounding_box.min_y, position, eye_height),
        ignore_current,
    )
}

/// [`scan`] of a given interaction box (`Entity.getFluidInteractionBox`, which a vehicle may
/// change or take away for its passenger), with where the entity's feet are.
pub(crate) fn scan_in(
    level: &Level<'_>,
    area: Option<Aabb>,
    (entity_y, position, eye_height): (f64, Vec3, f32),
    ignore_current: bool,
) -> Result<Fluids, Unsupported> {
    let mut fluids = Fluids::default();
    let Some(area) = area else {
        return Ok(fluids);
    };
    let x0 = floor_f64(area.min_x);
    let y0 = floor_f64(area.min_y);
    let z0 = floor_f64(area.min_z);
    let x1 = photon_core::math::ceil_f64(area.max_x) - 1;
    let y1 = photon_core::math::ceil_f64(area.max_y) - 1;
    let z1 = photon_core::math::ceil_f64(area.max_z) - 1;
    if !has_fluid_and_loaded(level, (x0 - 1, y0, z0 - 1), (x1 + 1, y1, z1 + 1)) {
        return Ok(fluids);
    }
    let eye_x = floor_f64(position.x);
    let eye_y = position.y + f64::from(eye_height);
    let eye_z = floor_f64(position.z);
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let pos = BlockPos::new(x, y, z);
                let info = level.info(pos);
                if info.fluid == Fluid::Empty {
                    continue;
                }
                let bottom = f64::from(y);
                let top = bottom + f64::from(level.fluid_height(pos, &info));
                if top < area.min_y {
                    continue;
                }
                let tracker = if info.is_water() {
                    &mut fluids.water
                } else {
                    &mut fluids.lava
                };
                if x == eye_x && z == eye_z && eye_y >= bottom && eye_y <= top {
                    tracker.eyes_inside = true;
                }
                tracker.height = java_math::max(top - entity_y, tracker.height);
                if !ignore_current {
                    let mut flow = level.fluid_flow(pos, &info)?;
                    let tracker = if info.is_water() {
                        &mut fluids.water
                    } else {
                        &mut fluids.lava
                    };
                    if tracker.height < 0.4 {
                        flow = flow.scale(tracker.height);
                    }
                    tracker.accumulated_current = tracker.accumulated_current.add_vec3(flow);
                    tracker.current_count += 1;
                }
            }
        }
    }
    Ok(fluids)
}

/// `EntityFluidInteraction.hasFluidAndLoaded`: whether the chunks of the
/// block range are all loaded and some section of them holds a fluid.
fn has_fluid_and_loaded(
    level: &Level<'_>,
    (x0, y0, z0): (i32, i32, i32),
    (x1, y1, z1): (i32, i32, i32),
) -> bool {
    let mut has_fluid = false;
    for chunk_z in z0.div_euclid(16)..=z1.div_euclid(16) {
        for chunk_x in x0.div_euclid(16)..=x1.div_euclid(16) {
            let Some(chunk) = level.world.chunk_for_query(chunk_x, chunk_z) else {
                return false;
            };
            for section_y in y0.div_euclid(16)..=y1.div_euclid(16) {
                if let Some(section) = chunk.sections.get(&section_y) {
                    has_fluid |= section.fluid_count > 0;
                }
            }
        }
    }
    has_fluid
}

/// Whose movement a current pushes: a player feels the mean of the flows
/// it stands in, any other entity their direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pushed {
    Player,
    Other,
}

/// `Tracker.applyCurrentTo`: what the fluid's current adds to an entity's
/// movement.
pub(crate) fn apply_current(
    tracker: FluidTracker,
    delta: Vec3,
    scale: f64,
    pushed: Pushed,
) -> Vec3 {
    if tracker.current_count == 0
        || tracker.accumulated_current.length_squared() < f64::from(1.0e-5_f32)
    {
        return delta;
    }
    let mut impulse = match pushed {
        Pushed::Player => tracker
            .accumulated_current
            .scale(1.0 / f64::from(tracker.current_count)),
        Pushed::Other => tracker.accumulated_current.normalized(),
    }
    .scale(scale);
    if delta.x.abs() < 0.003 && delta.z.abs() < 0.003 && impulse.length() < 0.0045000000000000005 {
        impulse = impulse.normalized().scale(0.0045000000000000005);
    }
    delta.add_vec3(impulse)
}
