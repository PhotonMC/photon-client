//! `ClientLevel.tickWeatherEffects`: in the rain, drops fall on the columns around the camera, and
//! now and then the rain is heard.

use photon_core::binary::BlockPos;
use photon_core::direction::Axis;
use photon_core::random::RandomSource;
use photon_data::block::definition::Fluid;

use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;
use crate::local_player::{LocalSound, LocalSoundSource};

/// `Options.weatherRadius`' default.
const WEATHER_RADIUS: i32 = 10;

impl ParticleEngine {
    /// One tick of the weather around `camera`: the drops, and the rain's sound if one is heard.
    pub fn tick_weather<F: FnMut() -> RandomSource>(
        &mut self,
        camera: (f64, f64, f64),
        context: &mut LevelEventContext<'_, F>,
    ) -> Option<LocalSound> {
        let world = context.world;
        let rain_level = world.rain_level();
        if rain_level <= 0.0 {
            return None;
        }
        let materials = self.terrain_materials();
        let mut random = RandomSource::create((world.game_time() as i64).wrapping_mul(312_987_231));
        let [cx, cy, cz] = [camera.0, camera.1, camera.2].map(photon_core::math::floor_f64);
        let diameter = 2 * WEATHER_RADIUS + 1;
        let drops = (0.225_f32 * (diameter * diameter) as f32 * rain_level * rain_level) as i32;
        let mut landed = None;
        for _ in 0..drops {
            let x = cx + random.next_int_bound(diameter).unwrap_or(0) - WEATHER_RADIUS;
            let z = cz + random.next_int_bound(diameter).unwrap_or(0) - WEATHER_RADIUS;
            let top = world.motion_blocking_height(x, z);
            let rains = materials
                .as_ref()
                .is_some_and(|materials| materials.rains_at(world, BlockPos::new(x, top, z)));
            if top > world.dimension().min_y && top <= cy + 10 && top >= cy - 10 && rains {
                landed = Some((x, top - 1, z));
                self.drop_on((x, top - 1, z), &mut random, context);
            }
        }
        let (x, y, z) = landed?;
        let time = self.rain_sound_time();
        let heard = random.next_int_bound(3).unwrap_or(0) < *time;
        *time += 1;
        if !heard {
            return None;
        }
        *time = 0;
        let above = y > cy + 1 && world.motion_blocking_height(cx, cz) > cy;
        let (event, volume, pitch) = if above {
            ("minecraft:weather.rain.above", 0.1, 0.5)
        } else {
            ("minecraft:weather.rain", 0.2, 1.0)
        };
        let at = (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5);
        Some(LocalSound::at(
            event,
            LocalSoundSource::Weather,
            (volume, pitch),
            at,
        ))
    }

    /// One drop (or the smoke of a hot block) at a random place on the top of the block at `pos`.
    fn drop_on<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let world = context.world;
        let (block_x, block_z) = (random.next_double(), random.next_double());
        let state = world.block_state_or_air(pos.0, pos.1, pos.2);
        let info = crate::block::state_or_air(state);
        let shape = crate::block::registry::collision_shape_at(state, [pos.0, pos.1, pos.2]);
        let block_top = shape.max_at(Axis::Y, block_x, block_z);
        let above = crate::block::state_or_air(world.block_state_or_air(pos.0, pos.1 + 1, pos.2));
        let fluid = info.fluid;
        let fluid_top = if fluid.is_empty() {
            0.0
        } else if above.fluid.is_same(fluid.fluid) {
            1.0
        } else {
            f64::from(fluid.own_height())
        };
        let name = info.block().name;
        let lit = crate::block::rules::property(state, "lit") == Some("true");
        let hot = matches!(fluid.fluid, Fluid::Lava | Fluid::FlowingLava)
            || name == "minecraft:magma_block"
            || (matches!(name, "minecraft:campfire" | "minecraft:soul_campfire") && lit);
        let at = (
            f64::from(pos.0) + block_x,
            f64::from(pos.1) + block_top.max(fluid_top),
            f64::from(pos.2) + block_z,
        );
        self.level_add(
            context,
            if hot { "smoke" } else { "rain" },
            at,
            (0.0, 0.0, 0.0),
        );
    }
}
