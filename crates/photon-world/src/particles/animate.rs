//! `ClientLevel.animateTick`: each tick blocks around the player are picked at random (by the
//! level's random) and show what they show (by their own random): a torch's flame and smoke, a
//! lit redstone torch's dust, lava's pops, water's specks, and the drips under a fluid.

use photon_core::random::RandomSource;
use photon_data::block::definition::Fluid;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::engine::{ParticleData, ParticleEngine, Spawn};
use super::level_events::LevelEventContext;

impl ParticleEngine {
    /// `ClientLevel.animateTick` around `center`: `count` pairs of picks, near and far (the
    /// reference makes 667 pairs a tick); the sounds they play, in order.
    pub fn animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (center, count): ((i32, i32, i32), usize),
        animate_random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) -> Vec<LocalSound> {
        let mut sounds = Vec::new();
        for _ in 0..count {
            for radius in [16, 32] {
                self.do_animate_tick(center, radius, animate_random, context, &mut sounds);
            }
        }
        sounds
    }

    /// `ClientLevel.doAnimateTick`: one block picked, then its block's and its fluid's
    /// `animateTick`, and a fluid's drip below it one time in ten. The biome's ambient particles
    /// (`AMBIENT_PARTICLES`) are not carried.
    fn do_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (xt, yt, zt): (i32, i32, i32),
        radius: i32,
        animate_random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let pick = |random: &mut RandomSource| {
            random.next_int_bound(radius).unwrap_or(0) - random.next_int_bound(radius).unwrap_or(0)
        };
        let x = xt + pick(context.level_random);
        let y = yt + pick(context.level_random);
        let z = zt + pick(context.level_random);
        let state = context.world.block_state_or_air(x, y, z);
        self.block_animate_tick(state, (x, y, z), animate_random, (context, sounds));
        let Some(info) = crate::block::state(state) else {
            return;
        };
        let fluid = info.fluid;
        if fluid.is_empty() {
            return;
        }
        let lava = matches!(fluid.fluid, Fluid::Lava | Fluid::FlowingLava);
        if lava {
            self.lava_animate_tick((x, y, z), animate_random, context, sounds);
        } else {
            self.water_animate_tick((x, y, z), fluid, animate_random, context, sounds);
        }
        if context.level_random.next_int_bound(10) == Ok(0) {
            let sturdy = info.is_face_sturdy(photon_core::direction::Direction::Down);
            let drip = if lava {
                "dripping_lava"
            } else {
                "dripping_water"
            };
            self.try_spawn_drip_particles((x, y - 1, z), drip, sturdy, context);
        }
    }

    /// `LavaFluid.animateTick`: under open air, a pop now and then, and its murmur.
    fn lava_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (x, y, z): (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let above = context.world.block_state_or_air(x, y + 1, z);
        if !crate::block::state_or_air(above).block().air {
            return;
        }
        if random.next_int_bound(100) == Ok(0) {
            let at = (
                f64::from(x) + random.next_double(),
                f64::from(y) + 1.0,
                f64::from(z) + random.next_double(),
            );
            self.level_add(context, "lava", at, (0.0, 0.0, 0.0));
            let volume = 0.2 + random.next_float() * 0.2;
            let pitch = 0.9 + random.next_float() * 0.15;
            sounds.push(ambient(
                "minecraft:block.lava.pop",
                (volume, pitch),
                at,
                context,
            ));
        }
        if random.next_int_bound(200) == Ok(0) {
            let volume = 0.2 + random.next_float() * 0.2;
            let pitch = 0.9 + random.next_float() * 0.15;
            let at = (f64::from(x), f64::from(y), f64::from(z));
            sounds.push(ambient(
                "minecraft:block.lava.ambient",
                (volume, pitch),
                at,
                context,
            ));
        }
    }

    /// `WaterFluid.animateTick`: a flow that is not falling murmurs; still or falling water has
    /// specks.
    fn water_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (x, y, z): (i32, i32, i32),
        fluid: crate::block::fluid::FluidState,
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        if !fluid.is_source() && !fluid.falling {
            if random.next_int_bound(64) == Ok(0) {
                let volume = random.next_float() * 0.25 + 0.75;
                let pitch = random.next_float() + 0.5;
                let at = (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5);
                sounds.push(ambient(
                    "minecraft:block.water.ambient",
                    (volume, pitch),
                    at,
                    context,
                ));
            }
        } else if random.next_int_bound(10) == Ok(0) {
            let at = (
                f64::from(x) + random.next_double(),
                f64::from(y) + random.next_double(),
                f64::from(z) + random.next_double(),
            );
            self.level_add(context, "underwater", at, (0.0, 0.0, 0.0));
        }
    }

    /// `ClientLevel.trySpawnDripParticles`: a drip from the underside of what holds the fluid up,
    /// or through a block that lets it, where its place is drawn by the level's random.
    fn try_spawn_drip_particles<F: FnMut() -> RandomSource>(
        &mut self,
        (x, y, z): (i32, i32, i32),
        drip: &str,
        top_solid: bool,
        context: &mut LevelEventContext<'_, F>,
    ) {
        use photon_core::direction::Axis;
        let world = context.world;
        let state = world.block_state_or_air(x, y, z);
        let Some(info) = crate::block::state(state) else {
            return;
        };
        if !info.fluid.is_empty() {
            return;
        }
        let shape = crate::block::registry::collision_shape_at(state, [x, y, z]);
        let (fx, fy, fz) = (f64::from(x), f64::from(y), f64::from(z));
        let top = shape.max(Axis::Y);
        let span = |shape: &photon_core::shape::VoxelShape| {
            (
                fx + shape.min(Axis::X),
                fx + shape.max(Axis::X),
                fz + shape.min(Axis::Z),
                fz + shape.max(Axis::Z),
            )
        };
        if top < 1.0 {
            if top_solid {
                self.spawn_fluid_particle(
                    (fx, fx + 1.0, fz, fz + 1.0),
                    fy + 1.0 - 0.05,
                    drip,
                    context,
                );
            }
            return;
        }
        let impermeable = world
            .block_state_registry
            .tags()
            .contains(usize::from(info.block), "minecraft:impermeable");
        if impermeable {
            return;
        }
        let bottom = shape.min(Axis::Y);
        if bottom > 0.0 {
            self.spawn_fluid_particle(span(&shape), fy + bottom - 0.05, drip, context);
            return;
        }
        let below = world.block_state_or_air(x, y - 1, z);
        let below_top =
            crate::block::registry::collision_shape_at(below, [x, y - 1, z]).max(Axis::Y);
        let below_dry = crate::block::state(below).is_none_or(|info| info.fluid.is_empty());
        if below_top < 1.0 && below_dry {
            self.spawn_fluid_particle(span(&shape), fy - 0.05, drip, context);
        }
    }

    /// `ClientLevel.spawnFluidParticle`: somewhere across the span, at the height.
    fn spawn_fluid_particle<F: FnMut() -> RandomSource>(
        &mut self,
        (x1, x2, z1, z2): (f64, f64, f64, f64),
        y: f64,
        drip: &str,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let lerp = |delta: f64, from: f64, to: f64| from + delta * (to - from);
        let x = lerp(context.level_random.next_double(), x1, x2);
        let z = lerp(context.level_random.next_double(), z1, z2);
        self.level_add(context, drip, (x, y, z), (0.0, 0.0, 0.0));
    }

    /// `Level.addParticle` of a type with options, still.
    pub(super) fn level_add_data<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        name: &str,
        data: ParticleData,
        at: (f64, f64, f64),
    ) {
        self.level_add_data_pushed(context, name, data, at, (0.0, 0.0, 0.0));
    }

    /// `Level.addParticle` of a type with options, pushed.
    pub(super) fn level_add_data_pushed<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        name: &str,
        data: ParticleData,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
    ) {
        let Some(type_id) = photon_data::particle_type::id(name) else {
            return;
        };
        let spawn = Spawn {
            type_id,
            data,
            override_limiter: false,
            always_show: false,
            at,
            push,
        };
        self.add_particle(
            context.world,
            &spawn,
            context.view,
            context.level_random,
            &mut context.particle_random,
        );
    }
}

/// `Level.playLocalSound` of an ambient sound at a place: its seed drawn from the level's random.
fn ambient<F: FnMut() -> RandomSource>(
    event: &'static str,
    (volume, pitch): (f32, f32),
    position: (f64, f64, f64),
    context: &mut LevelEventContext<'_, F>,
) -> LocalSound {
    local_sound(
        (event, LocalSoundSource::Ambient),
        (volume, pitch),
        position,
        context,
    )
}

/// `Level.playLocalSound` at a place: its seed drawn from the level's random.
pub(super) fn local_sound<F: FnMut() -> RandomSource>(
    (event, source): (&'static str, LocalSoundSource),
    (volume, pitch): (f32, f32),
    position: (f64, f64, f64),
    context: &mut LevelEventContext<'_, F>,
) -> LocalSound {
    context.level_random.next_long();
    LocalSound {
        relative: false,
        event,
        source,
        volume,
        pitch,
        position,
    }
}
