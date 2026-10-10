//! The last of `LevelEventHandler.levelEvent` that needs no block entity: the dragon's breath,
//! the end gateway and the dragon's growl, the sparks, wax and scrapes on a block, the sculk's
//! charge and shriek, an egg cracking and a cobweb placed, and the global events heard from afar.

use photon_core::direction::{Axis, Direction};
use photon_core::math::{mth_cos_f64, mth_sin_f64};
use photon_core::random::RandomSource;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::local_sound;
use super::engine::{ParticleData, ParticleEngine, Spawn};
use super::level_events::LevelEventContext;

/// `Direction.values()`.
const ALL_FACES: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `SculkShriekerBlock.TOP_Y`.
const SHRIEKER_TOP_Y: f64 = 0.5;

/// `UniformInt.sample`: both ends included.
fn sample(random: &mut RandomSource, min: i32, max: i32) -> i32 {
    random.next_int_bound(max - min + 1).unwrap_or(0) + min
}

/// `(random.nextFloat() - random.nextFloat()) * 0.2F + 1.0F`: the pitch a trial spawner's sounds
/// spread over.
fn spread_pitch(random: &mut RandomSource) -> f32 {
    (random.next_float() - random.next_float()) * 0.2 + 1.0
}

/// `Mth.nextDouble`.
fn next_double(random: &mut RandomSource, min: f64, max: f64) -> f64 {
    if min >= max {
        min
    } else {
        random.next_double() * (max - min) + min
    }
}

/// `ParticleUtils.getRandomSpeedRanges`.
fn random_speed(random: &mut RandomSource) -> (f64, f64, f64) {
    (
        next_double(random, -0.5, 0.5),
        next_double(random, -0.5, 0.5),
        next_double(random, -0.5, 0.5),
    )
}

impl ParticleEngine {
    /// The events of this file; false for another.
    pub(super) fn last_level_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) -> bool {
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        match event {
            2006 => self.dragon_breath(pos, data, context, sounds),
            3000 => {
                // `addAlwaysVisibleParticle`: past the limiter, and shown whatever the option.
                self.level_add_flags(context, "explosion_emitter", centre, (true, true));
                let random = &mut *context.level_random;
                let pitch = (1.0 + (random.next_float() - random.next_float()) * 0.2) * 0.7;
                let event = (
                    "minecraft:block.end_gateway.spawn",
                    LocalSoundSource::Blocks,
                );
                sounds.push(local_sound(event, (10.0, pitch), centre, context));
            }
            3001 => {
                let pitch = 0.8 + context.level_random.next_float() * 0.3;
                let event = (
                    "minecraft:entity.ender_dragon.growl",
                    LocalSoundSource::Hostile,
                );
                sounds.push(local_sound(event, (64.0, pitch), centre, context));
            }
            3002 => {
                if (0..3).contains(&data) {
                    let axis = [Axis::X, Axis::Y, Axis::Z][data as usize];
                    self.sparks_along_axis(axis, pos, context);
                } else {
                    self.particles_on_block_faces(pos, "electric_spark", (3, 5), context);
                }
            }
            3003 => {
                self.particles_on_block_faces(pos, "wax_on", (3, 5), context);
                let event = ("minecraft:item.honeycomb.wax_on", LocalSoundSource::Blocks);
                sounds.push(local_sound(event, (1.0, 1.0), centre, context));
            }
            3004 => self.particles_on_block_faces(pos, "wax_off", (3, 5), context),
            3005 => self.particles_on_block_faces(pos, "scrape", (3, 5), context),
            3006 => self.sculk_charge(pos, data, context, sounds),
            3007 => self.shriek(pos, context, sounds),
            3009 => self.particles_on_block_faces(pos, "egg_crack", (3, 6), context),
            3011..=3014 | 3017 | 3019..=3021 => {
                self.trial_spawner_event((event, pos, data), context, sounds);
            }
            3018 => {
                for _ in 0..10 {
                    let random = &mut *context.level_random;
                    let push = (
                        random.next_gaussian() * 0.02,
                        random.next_gaussian() * 0.02,
                        random.next_gaussian() * 0.02,
                    );
                    let at = (
                        f64::from(pos.0) + random.next_double(),
                        f64::from(pos.1) + random.next_double(),
                        f64::from(pos.2) + random.next_double(),
                    );
                    self.level_add(context, "poof", at, push);
                }
                let random = &mut *context.level_random;
                let pitch = (random.next_float() - random.next_float()) * 0.2 + 1.0;
                let event = ("minecraft:block.cobweb.place", LocalSoundSource::Blocks);
                sounds.push(local_sound(event, (1.0, pitch), centre, context));
            }
            _ => return false,
        }
        true
    }

    /// The trial spawner's events (`TrialSpawner.add*Particles` and its sounds).
    fn trial_spawner_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        match event {
            3011 | 3012 | 3021 => {
                if event != 3011 {
                    let (name, source) = if event == 3012 {
                        (
                            "minecraft:block.trial_spawner.spawn_mob",
                            LocalSoundSource::Blocks,
                        )
                    } else {
                        (
                            "minecraft:block.trial_spawner.spawn_item",
                            LocalSoundSource::Blocks,
                        )
                    };
                    let pitch = spread_pitch(context.level_random);
                    sounds.push(local_sound((name, source), (1.0, pitch), centre, context));
                }
                // `FlameParticle.decode`: a datum out of range is the normal flame.
                let flame = if data == 1 {
                    "soul_fire_flame"
                } else {
                    "flame"
                };
                self.trial_spawn_particles(pos, flame, context);
            }
            3013 | 3019 => {
                let pitch = spread_pitch(context.level_random);
                let event_sound = (
                    "minecraft:block.trial_spawner.detect_player",
                    LocalSoundSource::Blocks,
                );
                sounds.push(local_sound(event_sound, (1.0, pitch), centre, context));
                let kind = if event == 3013 {
                    "trial_spawner_detection"
                } else {
                    "trial_spawner_detection_ominous"
                };
                self.trial_detect_particles(pos, data, kind, context);
            }
            3014 => {
                let pitch = spread_pitch(context.level_random);
                let event_sound = (
                    "minecraft:block.trial_spawner.eject_item",
                    LocalSoundSource::Blocks,
                );
                sounds.push(local_sound(event_sound, (1.0, pitch), centre, context));
                self.trial_eject_particles(pos, context);
            }
            3017 => self.trial_eject_particles(pos, context),
            3020 => {
                let volume = if data == 0 { 0.3 } else { 1.0 };
                let pitch = spread_pitch(context.level_random);
                let event_sound = (
                    "minecraft:block.trial_spawner.ominous_activate",
                    LocalSoundSource::Blocks,
                );
                sounds.push(local_sound(event_sound, (volume, pitch), centre, context));
                self.trial_detect_particles(pos, 0, "trial_spawner_detection_ominous", context);
                self.trial_become_ominous_particles(pos, context);
            }
            _ => {}
        }
    }

    /// `Level.addParticle` of a plain type with the limiter's and the option's say (`override`,
    /// `always shown`).
    fn level_add_flags<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        name: &str,
        at: (f64, f64, f64),
        (override_limiter, always_show): (bool, bool),
    ) {
        let Some(type_id) = photon_data::particle_type::id(name) else {
            return;
        };
        let spawn = Spawn {
            type_id,
            data: ParticleData::Plain,
            override_limiter,
            always_show,
            at,
            push: (0.0, 0.0, 0.0),
        };
        self.add_particle(
            context.world,
            &spawn,
            context.view,
            context.level_random,
            &mut context.particle_random,
        );
    }

    /// Event 2006: two hundred specks of the dragon's breath flung out of a block, and with a
    /// fireball's data its explosion.
    fn dragon_breath<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        for _ in 0..200 {
            let random = &mut *context.level_random;
            let dist = random.next_float() * 4.0;
            let angle = random.next_float() * (std::f64::consts::PI * 2.0) as f32;
            let vx = f64::from(mth_cos_f64(f64::from(angle)) * dist);
            let vy = 0.01 + random.next_double() * 0.5;
            let vz = f64::from(mth_sin_f64(f64::from(angle)) * dist);
            let at = (x + vx * 0.1, y + 0.3, z + vz * 0.1);
            self.level_add_data_pushed(
                context,
                "dragon_breath",
                ParticleData::Power(dist),
                at,
                (vx, vy, vz),
            );
        }
        if data == 1 {
            let pitch = context.level_random.next_float() * 0.1 + 0.9;
            let event = (
                "minecraft:entity.dragon_fireball.explode",
                LocalSoundSource::Hostile,
            );
            let place = (x + 0.5, y + 0.5, z + 0.5);
            sounds.push(local_sound(event, (1.0, pitch), place, context));
        }
    }

    /// `ParticleUtils.spawnParticlesOnBlockFaces`.
    fn particles_on_block_faces<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        name: &str,
        (min, max): (i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for face in ALL_FACES {
            let count = sample(context.level_random, min, max);
            for _ in 0..count {
                let speed = random_speed(context.level_random);
                self.particle_on_face(
                    pos,
                    face,
                    (name, ParticleData::Plain),
                    (speed, 0.55),
                    context,
                );
            }
        }
    }

    /// `ParticleUtils.spawnParticleOnFace`: a particle out of a face of the block, anywhere across
    /// it, moving across it as the speed says.
    fn particle_on_face<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        face: Direction,
        (name, data): (&str, ParticleData),
        (speed, step_factor): ((f64, f64, f64), f64),
        context: &mut LevelEventContext<'_, F>,
    ) {
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let (step_x, step_y, step_z) = (face.get_step_x(), face.get_step_y(), face.get_step_z());
        let mut place = |centre: f64, step: i32| {
            centre
                + if step == 0 {
                    next_double(context.level_random, -0.5, 0.5)
                } else {
                    f64::from(step) * step_factor
                }
        };
        let x = place(centre.0, step_x);
        let y = place(centre.1, step_y);
        let z = place(centre.2, step_z);
        let push = (
            if step_x == 0 { speed.0 } else { 0.0 },
            if step_y == 0 { speed.1 } else { 0.0 },
            if step_z == 0 { speed.2 } else { 0.0 },
        );
        self.level_add_data_pushed(context, name, data, (x, y, z), push);
    }

    /// `ParticleUtils.spawnParticlesAlongAxis` of electric sparks, for a lightning rod's strike.
    fn sparks_along_axis<F: FnMut() -> RandomSource>(
        &mut self,
        axis: Axis,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        let radius = 0.125;
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let count = sample(context.level_random, 10, 19);
        for _ in 0..count {
            let random = &mut *context.level_random;
            let mut place = |centre: f64, along: bool| {
                centre + next_double(random, -1.0, 1.0) * if along { 0.5 } else { radius }
            };
            let x = place(centre.0, axis == Axis::X);
            let y = place(centre.1, axis == Axis::Y);
            let z = place(centre.2, axis == Axis::Z);
            let mut base = |along: bool| {
                if along {
                    next_double(random, -1.0, 1.0)
                } else {
                    0.0
                }
            };
            let push = (
                base(axis == Axis::X),
                base(axis == Axis::Y),
                base(axis == Axis::Z),
            );
            self.level_add(context, "electric_spark", (x, y, z), push);
        }
    }

    /// Event 3006: sculk charging. With a count above zero the charge creeps over the faces of a
    /// sculk vein (or a whole block); without, it pops.
    fn sculk_charge<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let count = data >> 6;
        let charge = ("minecraft:block.sculk.charge", LocalSoundSource::Blocks);
        if count > 0 {
            let random = &mut *context.level_random;
            if random.next_float() < 0.3 + count as f32 * 0.1 {
                let volume = 0.15 + 0.02 * count as f32 * count as f32 * random.next_float();
                let pitch = 0.4 + 0.3 * count as f32 * random.next_float();
                sounds.push(local_sound(charge, (volume, pitch), centre, context));
            }
            let particle_data = (data & 63) as u8;
            let faces: Vec<(Direction, f32, f64)> = if particle_data == 0 {
                ALL_FACES
                    .into_iter()
                    .map(|face| {
                        let rotation = if face == Direction::Down {
                            std::f32::consts::PI
                        } else {
                            0.0
                        };
                        let factor = if matches!(face, Direction::Down | Direction::Up) {
                            0.65
                        } else {
                            0.57
                        };
                        (face, rotation, factor)
                    })
                    .collect()
            } else {
                ALL_FACES
                    .into_iter()
                    .filter(|face| particle_data & (1 << *face as u8) > 0)
                    .map(|face| {
                        let rotation = if face == Direction::Up {
                            std::f32::consts::PI
                        } else {
                            0.0
                        };
                        (face, rotation, 0.35)
                    })
                    .collect()
            };
            for (face, rotation, factor) in faces {
                // `UniformInt.of(0, count)`, drawn for each face before its particles.
                let repeats = sample(context.level_random, 0, count);
                for _ in 0..repeats {
                    let random = &mut *context.level_random;
                    let speed = (
                        next_double(random, f64::from(-0.005_f32), f64::from(0.005_f32)),
                        next_double(random, f64::from(-0.005_f32), f64::from(0.005_f32)),
                        next_double(random, f64::from(-0.005_f32), f64::from(0.005_f32)),
                    );
                    self.particle_on_face(
                        pos,
                        face,
                        ("sculk_charge", ParticleData::Roll(rotation)),
                        (speed, factor),
                        context,
                    );
                }
            }
        } else {
            sounds.push(local_sound(charge, (1.0, 1.0), centre, context));
            let solid = crate::block::state_or_air(
                context
                    .world
                    .block_state_at(pos.0, pos.1, pos.2)
                    .map_or(0, i64::from),
            )
            .collision_full_block;
            let (count, spread) = if solid {
                (40, 0.45_f32)
            } else {
                (20, 0.25_f32)
            };
            for _ in 0..count {
                let random = &mut *context.level_random;
                let vx = 2.0 * random.next_float() - 1.0;
                let vy = 2.0 * random.next_float() - 1.0;
                let vz = 2.0 * random.next_float() - 1.0;
                let at = (
                    f64::from(pos.0) + 0.5 + f64::from(vx * spread),
                    f64::from(pos.1) + 0.5 + f64::from(vy * spread),
                    f64::from(pos.2) + 0.5 + f64::from(vz * spread),
                );
                let push = (
                    f64::from(vx * 0.07),
                    f64::from(vy * 0.07),
                    f64::from(vz * 0.07),
                );
                self.level_add(context, "sculk_charge_pop", at, push);
            }
        }
    }

    /// Event 3007: ten shrieks one after another, and the shrieker's sound unless it is under
    /// water.
    fn shriek<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let at = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + SHRIEKER_TOP_Y,
            f64::from(pos.2) + 0.5,
        );
        for index in 0..10 {
            self.level_add_data(
                context,
                "shriek",
                ParticleData::Shriek { delay: index * 5 },
                at,
            );
        }
        let state = context
            .world
            .block_state_at(pos.0, pos.1, pos.2)
            .unwrap_or(0);
        let waterlogged = crate::block::rules::property(state, "waterlogged") == Some("true");
        if !waterlogged {
            let pitch = 0.6 + context.level_random.next_float() * 0.4;
            let event = (
                "minecraft:block.sculk_shrieker.shriek",
                LocalSoundSource::Blocks,
            );
            sounds.push(local_sound(event, (2.0, pitch), at, context));
        }
    }

    /// `TrialSpawner.addSpawnParticles`: smoke and the flame, twenty times about the block.
    fn trial_spawn_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        flame: &str,
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..20 {
            let at = self.trial_about(pos, context.level_random);
            self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
            self.level_add(context, flame, at, (0.0, 0.0, 0.0));
        }
    }

    /// `TrialSpawner.addBecomeOminousParticles`.
    fn trial_become_ominous_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..20 {
            let at = self.trial_about(pos, context.level_random);
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            self.level_add(context, "trial_omen", at, push);
            self.level_add(context, "soul_fire_flame", at, push);
        }
    }

    /// A place within a block's width of its middle, as the trial spawner's puffs are thrown.
    fn trial_about(&self, pos: (i32, i32, i32), random: &mut RandomSource) -> (f64, f64, f64) {
        let mut about = |base: i32| f64::from(base) + 0.5 + (random.next_double() - 0.5) * 2.0;
        let x = about(pos.0);
        let y = about(pos.1);
        let z = about(pos.2);
        (x, y, z)
    }

    /// `TrialSpawner.addDetectPlayerParticles`: more of them the more players are about, up to ten.
    fn trial_detect_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        name: &str,
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..30 + data.min(10) * 5 {
            let random = &mut *context.level_random;
            let spread_x = f64::from(2.0 * random.next_float() - 1.0) * 0.65;
            let spread_z = f64::from(2.0 * random.next_float() - 1.0) * 0.65;
            let at = (
                f64::from(pos.0) + 0.5 + spread_x,
                f64::from(pos.1) + 0.1 + f64::from(random.next_float()) * 0.8,
                f64::from(pos.2) + 0.5 + spread_z,
            );
            self.level_add(context, name, at, (0.0, 0.0, 0.0));
        }
    }

    /// `TrialSpawner.addEjectItemParticles`: flames and smoke out of the middle of the block.
    fn trial_eject_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..20 {
            let random = &mut *context.level_random;
            let mut within = |base: i32| f64::from(base) + 0.4 + random.next_double() * 0.2;
            let x = within(pos.0);
            let y = within(pos.1);
            let z = within(pos.2);
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            self.level_add(
                context,
                "small_flame",
                (x, y, z),
                (push.0, push.1, push.2 * 0.25),
            );
            self.level_add(context, "smoke", (x, y, z), push);
        }
    }

    /// `LevelEventHandler.globalLevelEvent`: a wither's spawn, a dragon's death and an end portal
    /// opening are heard two blocks from the camera, in the direction they happened.
    pub(super) fn global_level_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos): (i32, (i32, i32, i32)),
        context: &mut LevelEventContext<'_, F>,
    ) -> Vec<LocalSound> {
        let (name, volume) = match event {
            1023 => ("minecraft:entity.wither.spawn", 1.0),
            1038 => ("minecraft:block.end_portal.spawn", 1.0),
            1028 => ("minecraft:entity.ender_dragon.death", 5.0),
            _ => return Vec::new(),
        };
        let camera = context.view.camera;
        let centre = photon_core::math::Vec3::new(
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let direction = centre.subtract_vec3(camera).normalized();
        let at = camera.add_vec3(direction.scale(2.0));
        vec![local_sound(
            (name, LocalSoundSource::Hostile),
            (volume, 1.0),
            (at.x, at.y, at.z),
            context,
        )]
    }
}
