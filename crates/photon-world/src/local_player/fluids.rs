use super::*;
use photon_core::ids::BlockStateId;

impl LocalPlayer {
    pub(super) fn update_fluid_interaction(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        let found = crate::fluid::scan_in(
            level,
            self.fluid_interaction_box(),
            (
                self.bounding_box.min_y,
                self.position,
                self.dimensions.eye_height(),
            ),
            self.abilities.flying,
        );
        let fluids = found.inspect_err(|_| {
            self.water = FluidTracker::default();
            self.lava = FluidTracker::default();
        })?;
        self.water = fluids.water;
        self.lava = fluids.lava;
        let in_water = self.water.height > 0.0;
        let in_lava = self.lava.height > 0.0;
        if in_water {
            self.fall_distance = 0.0;
        }
        if in_water && !self.was_touching_water && !self.first_tick {
            self.water_splash_sound();
        }
        self.was_touching_water = in_water;
        if !self.abilities.flying {
            if in_water {
                self.apply_current(self.water, WATER_FLOW_SCALE);
            }
            if in_lava {
                self.apply_current(self.lava, LAVA_FLOW_SCALE);
            }
        }
        Ok(())
    }

    /// `Entity.getFluidInteractionBox`: the box a thousandth smaller, which the vehicle the player
    /// sits in may change (`AbstractBoat.modifyPassengerFluidInteractionBox`: only what is above
    /// the boat's top, and none of it while the boat's top is above the player's own).
    pub(super) fn fluid_interaction_box(&self) -> Option<Aabb> {
        let area = self.bounding_box.deflate(0.001);
        let Some(riding) = self.riding else {
            return Some(area);
        };
        if !riding.modifies_fluid_box || riding.vehicle_under_water {
            return Some(area);
        }
        if riding.vehicle_max_y >= area.max_y {
            return None;
        }
        let min_y = java_math::max(area.min_y, riding.vehicle_max_y);
        Some(Aabb::new(
            area.min_x, min_y, area.min_z, area.max_x, area.max_y, area.max_z,
        ))
    }

    pub(super) fn contains_any_liquid(&self, level: &Level<'_>, area: Aabb) -> bool {
        let x0 = floor_f64(area.min_x);
        let x1 = photon_core::math::ceil_f64(area.max_x);
        let y0 = floor_f64(area.min_y);
        let y1 = photon_core::math::ceil_f64(area.max_y);
        let z0 = floor_f64(area.min_z);
        let z1 = photon_core::math::ceil_f64(area.max_z);
        for x in x0..x1 {
            for y in y0..y1 {
                for z in z0..z1 {
                    if level.info(BlockPos::new(x, y, z)).fluid != Fluid::Empty {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// The fluid a block state holds: whether it is water (rather than lava),
/// its own height, and whether it is falling. Nothing for a dry state.
pub fn state_fluid(state_id: BlockStateId) -> Option<(bool, f32, bool)> {
    let fluid = crate::block::state_or_air(state_id).fluid;
    (!fluid.is_empty()).then(|| {
        (
            fluid.is_same(Fluid::Water),
            fluid.own_height(),
            fluid.falling,
        )
    })
}

/// Whether a block state holds any fluid, as a section's fluid count sees it.
pub fn state_has_fluid(state_id: BlockStateId) -> bool {
    !crate::block::state_or_air(state_id).fluid.is_empty()
}
