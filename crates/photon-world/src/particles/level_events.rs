//! `LevelEventHandler.levelEvent`: the sounds and particles a level event plays, drawn from the
//! level's random in the order the reference draws them.

use photon_core::binary::BlockPos;
use photon_core::ids::BlockStateId;
use photon_core::math::fdlibm_pow;
use photon_core::random::RandomSource;
use photon_data::particle_type;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::super::{ClientWorld, LevelEvent};
use super::engine::{AddView, ParticleData, ParticleEngine, Spawn};
use super::level_events_dripstone::DripstoneAttributes;
use super::level_events_vault::PlayerSpot;
use LocalSoundSource::{Blocks, Hostile, Neutral};
use Pitch::{Fixed, Slight, Spread, Thrown};

/// How a level event's sound is pitched.
#[derive(Debug, Clone, Copy)]
enum Pitch {
    Fixed(f32),
    /// `base + (random - random) * scale`.
    Spread(f32, f32),
    /// `random * 0.1 + 0.9`.
    Slight,
    /// `0.4 / (random * 0.4 + 0.8)`: a wind charge thrown.
    Thrown,
}

/// The events that only play a sound at the block's middle (besides what else they do): the
/// sound, its kind, volume and pitch.
#[rustfmt::skip]
const SOUND_EVENTS: &[(i32, &str, LocalSoundSource, f32, Pitch)] = &[
    (1000, "minecraft:block.dispenser.dispense", Blocks, 1.0, Fixed(1.0)),
    (1001, "minecraft:block.dispenser.fail", Blocks, 1.0, Fixed(1.2)),
    (1002, "minecraft:block.dispenser.launch", Blocks, 1.0, Fixed(1.2)),
    (1004, "minecraft:entity.firework_rocket.shoot", Neutral, 1.0, Fixed(1.2)),
    (1015, "minecraft:entity.ghast.warn", Hostile, 10.0, MOB),
    (1016, "minecraft:entity.ghast.shoot", Hostile, 10.0, MOB),
    (1017, "minecraft:entity.ender_dragon.shoot", Hostile, 10.0, MOB),
    (1018, "minecraft:entity.blaze.shoot", Hostile, 2.0, MOB),
    (1019, "minecraft:entity.zombie.attack_wooden_door", Hostile, 2.0, MOB),
    (1020, "minecraft:entity.zombie.attack_iron_door", Hostile, 2.0, MOB),
    (1021, "minecraft:entity.zombie.break_wooden_door", Hostile, 2.0, MOB),
    (1022, "minecraft:entity.wither.break_block", Hostile, 2.0, MOB),
    (1024, "minecraft:entity.wither.shoot", Hostile, 2.0, MOB),
    (1025, "minecraft:entity.bat.takeoff", Neutral, 0.05, MOB),
    (1026, "minecraft:entity.zombie.infect", Hostile, 2.0, MOB),
    (1027, "minecraft:entity.zombie_villager.converted", Hostile, 2.0, MOB),
    (1029, "minecraft:block.anvil.destroy", Blocks, 1.0, Slight),
    (1030, "minecraft:block.anvil.use", Blocks, 1.0, Slight),
    (1031, "minecraft:block.anvil.land", Blocks, 0.3, Slight),
    (1033, "minecraft:block.chorus_flower.grow", Blocks, 1.0, Fixed(1.0)),
    (1034, "minecraft:block.chorus_flower.death", Blocks, 1.0, Fixed(1.0)),
    (1035, "minecraft:block.brewing_stand.brew", Blocks, 1.0, Fixed(1.0)),
    (1039, "minecraft:entity.phantom.bite", Hostile, 0.3, Slight),
    (1040, "minecraft:entity.zombie.converted_to_drowned", Hostile, 2.0, MOB),
    (1041, "minecraft:entity.husk.converted_to_zombie", Hostile, 2.0, MOB),
    (1042, "minecraft:block.grindstone.use", Blocks, 1.0, Slight),
    (1043, "minecraft:item.book.page_turn", Blocks, 1.0, Slight),
    (1044, "minecraft:block.smithing_table.use", Blocks, 1.0, Slight),
    (1045, "minecraft:block.pointed_dripstone.land", Blocks, 2.0, Slight),
    (1046, "minecraft:block.pointed_dripstone.drip_lava_into_cauldron", Blocks, 2.0, Slight),
    (1047, "minecraft:block.pointed_dripstone.drip_water_into_cauldron", Blocks, 2.0, Slight),
    (1048, "minecraft:entity.skeleton.converted_to_stray", Hostile, 2.0, MOB),
    (1049, "minecraft:block.crafter.craft", Blocks, 1.0, Fixed(1.0)),
    (1050, "minecraft:block.crafter.fail", Blocks, 1.0, Fixed(1.0)),
    (1051, "minecraft:entity.wind_charge.throw", Blocks, 0.5, Thrown),
    (1052, "minecraft:block.sulfur_spike.land", Blocks, 2.0, Slight),
    (1501, "minecraft:block.lava.extinguish", Blocks, 0.5, Spread(2.6, 0.8)),
    (1502, "minecraft:block.redstone_torch.burnout", Blocks, 0.5, Spread(2.6, 0.8)),
    (1503, "minecraft:block.end_portal_frame.fill", Blocks, 1.0, Fixed(1.0)),
];

/// A mob's sound: `(random - random) * 0.2 + 1`.
const MOB: Pitch = Pitch::Spread(1.0, 0.2);

/// 1032: the portal travel sound, heard from the listener (`forLocalAmbience`), its pitch the
/// level's next float.
fn ambience_event<F: FnMut() -> RandomSource>(
    event: i32,
    context: &mut LevelEventContext<'_, F>,
) -> Option<LocalSound> {
    (event == 1032).then(|| {
        let pitch = context.level_random.next_float() * 0.4 + 0.8;
        LocalSound::ambience("minecraft:block.portal.travel", pitch, 0.25)
    })
}

/// The sound an event plays alone: the table's, or a fire put out (1009) by its data.
fn sound_only(event: i32, data: i32) -> Option<(&'static str, LocalSoundSource, f32, Pitch)> {
    match (event, data) {
        (1009, 0) => Some((
            "minecraft:block.fire.extinguish",
            Blocks,
            0.5,
            Spread(2.6, 0.8),
        )),
        (1009, 1) => Some((
            "minecraft:entity.generic.extinguish_fire",
            Blocks,
            0.7,
            Spread(1.6, 0.4),
        )),
        (1009, _) => None,
        _ => SOUND_EVENTS
            .iter()
            .find(|row| row.0 == event)
            .map(|&(_, name, source, volume, pitch)| (name, source, volume, pitch)),
    }
}

/// `Direction.from3DDataValue`'s unit step: down, up, north, south, west, east.
fn step_of(data: i32) -> (i32, i32, i32) {
    const STEPS: [(i32, i32, i32); 6] = [
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
        (-1, 0, 0),
        (1, 0, 0),
    ];
    STEPS[(data % 6).unsigned_abs() as usize]
}

/// The level, the option and the random sources a level event's particles are added with.
pub struct LevelEventContext<'a, F: FnMut() -> RandomSource> {
    pub world: &'a ClientWorld,
    pub view: &'a AddView,
    pub level_random: &'a mut RandomSource,
    pub particle_random: F,
    pub dripstone: DripstoneAttributes,
    /// The local player, whom the level knows as one of its players.
    pub local_player: Option<PlayerSpot>,
    /// What the environment attributes say (the clock's flags, the biomes' ambient particles).
    pub ambience: &'a dyn super::Ambience,
}

impl<'a, F: FnMut() -> RandomSource> LevelEventContext<'a, F> {
    /// A context in the overworld, whose dripstone has the defaults.
    pub fn new(
        world: &'a ClientWorld,
        view: &'a AddView,
        level_random: &'a mut RandomSource,
        particle_random: F,
    ) -> Self {
        Self {
            world,
            view,
            level_random,
            particle_random,
            dripstone: DripstoneAttributes::default(),
            local_player: None,
            ambience: &super::NoAmbience,
        }
    }
}

/// The pitch a sound-only event draws.
fn spread_pitch(pitch: Pitch, random: &mut RandomSource) -> f32 {
    match pitch {
        Pitch::Fixed(pitch) => pitch,
        Pitch::Spread(base, scale) => base + (random.next_float() - random.next_float()) * scale,
        Pitch::Slight => random.next_float() * 0.1 + 0.9,
        Pitch::Thrown => 0.4 / (random.next_float() * 0.4 + 0.8),
    }
}

impl ParticleEngine {
    /// The events a block entity's ticker asks for: what they add and the sounds they make.
    fn block_entity_level_event<F: FnMut() -> RandomSource>(
        &mut self,
        level_event: LevelEvent,
        context: &mut LevelEventContext<'_, F>,
    ) -> Vec<LocalSound> {
        match level_event {
            LevelEvent::NoxiousGasCloud { at } => {
                self.level_add(context, "noxious_gas_cloud", at, (0.0, 0.0, 0.0));
                Vec::new()
            }
            LevelEvent::ConduitSpecks {
                pos,
                tick_count,
                frame,
                target,
            } => {
                self.conduit_specks(context, (pos, tick_count), (&frame, target));
                Vec::new()
            }
            LevelEvent::ParrotImitation { id } => {
                self.parrot_imitation(id, context);
                Vec::new()
            }
            LevelEvent::VaultTick { pos } => {
                let mut sounds = Vec::new();
                self.vault_tick(pos, context, &mut sounds);
                sounds
            }
            LevelEvent::TrialSpawnerTick { pos } => {
                let mut sounds = Vec::new();
                self.trial_spawner_tick(pos, context, &mut sounds);
                sounds
            }
            LevelEvent::SpawnerTick { pos } => {
                self.spawner_tick(pos, context);
                Vec::new()
            }
            LevelEvent::Geyser { at, water_blocks } => {
                self.add_geyser(context, at, water_blocks);
                Vec::new()
            }
            LevelEvent::GeyserSound { at, sound } => {
                vec![super::animate::local_sound(
                    (sound, LocalSoundSource::Blocks),
                    (1.0, 1.0),
                    at,
                    context,
                )]
            }
            _ => Vec::new(),
        }
    }

    /// An event other than a plain level event is played at once: the sounds it makes.
    fn other_level_event<F: FnMut() -> RandomSource>(
        &mut self,
        level_event: LevelEvent,
        context: &mut LevelEventContext<'_, F>,
    ) -> Vec<LocalSound> {
        match level_event {
            LevelEvent::Level { .. } => Vec::new(),
            LevelEvent::Note { pos, sound, note } => {
                vec![self.note_block(context, pos, (sound, note))]
            }
            LevelEvent::Explosion(explosion) => self.explode(*explosion, context),
            LevelEvent::Global { event, pos } => self.global_level_event((event, pos), context),
            LevelEvent::EntityBurst(burst) => {
                self.entity_burst(burst, context);
                Vec::new()
            }
            LevelEvent::Fireworks(launch) => {
                self.create_fireworks(*launch, context);
                Vec::new()
            }
            LevelEvent::NoxiousGasCloud { .. }
            | LevelEvent::ConduitSpecks { .. }
            | LevelEvent::VaultTick { .. }
            | LevelEvent::SpawnerTick { .. }
            | LevelEvent::TrialSpawnerTick { .. }
            | LevelEvent::ParrotImitation { .. }
            | LevelEvent::Geyser { .. }
            | LevelEvent::GeyserSound { .. } => self.block_entity_level_event(level_event, context),
            LevelEvent::PistonSound => {
                // `PistonBaseBlock.triggerEvent`: the pitch is asked of the level's random source
                // whether or not the client plays the sound (the server does that for the player).
                context.level_random.next_float();
                Vec::new()
            }
        }
    }

    /// `ConduitBlockEntity.animationTick` with no target: a speck now and then from each frame
    /// block, flying to the conduit's eye.
    fn conduit_specks<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        ((x, y, z), tick_count): ((i32, i32, i32), i32),
        (frame, target): (&[(i32, i32, i32)], Option<photon_core::binary::Uuid>),
    ) {
        let height = photon_core::math::mth_sin_f64(f64::from((tick_count + 35) as f32 * 0.1_f32))
            / 2.0_f32
            + 0.5_f32;
        let height = f64::from(height);
        let height = (height * height + height) * f64::from(0.3_f32);
        let end = (
            f64::from(x) + 0.5,
            f64::from(y) + 1.5 + height,
            f64::from(z) + 0.5,
        );
        for (fx, fy, fz) in frame {
            if context.level_random.next_int_bound(50) != Ok(0) {
                continue;
            }
            let dx = -0.5_f32 + context.level_random.next_float() + (fx - x) as f32;
            let dy = -2.0_f32 + context.level_random.next_float() + (fy - y) as f32;
            let dz = -0.5_f32 + context.level_random.next_float() + (fz - z) as f32;
            let push = (f64::from(dx), f64::from(dy), f64::from(dz));
            self.level_add(context, "nautilus", end, push);
        }
        // `ConduitBlockEntity.animationTick`'s target: a speck from the mob's eyes to the conduit,
        // spread by its size; the target is a living entity the level knows by its UUID.
        let found = target.and_then(|uuid| {
            context
                .world
                .entities()
                .values()
                .find(|entity| entity.is_living_entity() && entity.uuid == uuid)
                .map(|entity| {
                    let (x, y, z) = entity.position();
                    let size = entity.dimensions();
                    (
                        (x, y + f64::from(size.eye_height()), z),
                        (size.width(), size.height()),
                    )
                })
        });
        if let Some((at, (width, height))) = found {
            let random = &mut *context.level_random;
            let x = (-0.5_f32 + random.next_float()) * (3.0_f32 + width);
            let y = -1.0_f32 + random.next_float() * height;
            let z = (-0.5_f32 + random.next_float()) * (3.0_f32 + width);
            let push = (f64::from(x), f64::from(y), f64::from(z));
            self.level_add(context, "nautilus", at, push);
        }
    }

    /// `LevelEventHandler.levelEvent` for the events carried: the sounds it plays, in order. Its
    /// particles go through the level as it asks for them.
    pub fn level_event<F: FnMut() -> RandomSource>(
        &mut self,
        level_event: LevelEvent,
        context: &mut LevelEventContext<'_, F>,
    ) -> Vec<LocalSound> {
        let LevelEvent::Level { event, pos, data } = level_event else {
            return self.other_level_event(level_event, context);
        };
        let mut sounds: Vec<LocalSound> = ambience_event(event, context).into_iter().collect();
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        if let Some((name, source, volume, pitch)) = sound_only(event, data) {
            let random = &mut *context.level_random;
            let pitch = spread_pitch(pitch, random);
            sounds.push(local_sound(name, source, (volume, pitch), pos, random));
        }
        if self.more_level_event((event, pos, data), context, &mut sounds) {
            return sounds;
        }
        if self.last_level_event((event, pos, data), context, &mut sounds) {
            return sounds;
        }
        match event {
            1501 => {
                for _ in 0..8 {
                    let random = &mut *context.level_random;
                    let at = (x + random.next_double(), y + 1.2, z + random.next_double());
                    self.level_add(context, "large_smoke", at, (0.0, 0.0, 0.0));
                }
            }
            1502 => {
                for _ in 0..5 {
                    let random = &mut *context.level_random;
                    let x = x + random.next_double() * 0.6 + 0.2;
                    let y = y + random.next_double() * 0.6 + 0.2;
                    let z = z + random.next_double() * 0.6 + 0.2;
                    self.level_add(context, "smoke", (x, y, z), (0.0, 0.0, 0.0));
                }
            }
            1503 => {
                for _ in 0..16 {
                    let random = &mut *context.level_random;
                    let x = x + (5.0 + random.next_double() * 6.0) / 16.0;
                    let z = z + (5.0 + random.next_double() * 6.0) / 16.0;
                    self.level_add(context, "smoke", (x, y + 0.8125, z), (0.0, 0.0, 0.0));
                }
            }
            2000 => self.shoot_particles(context, (pos, data), "smoke"),
            2010 => self.shoot_particles(context, (pos, data), "white_smoke"),
            2001 => {
                if let Some(sound) = level_event.break_sound() {
                    // `playSound`'s seed.
                    context.level_random.next_long();
                    sounds.push(sound);
                }
                let at = BlockPos::new(pos.0, pos.1, pos.2);
                self.add_destroy_block_effect(
                    context.world,
                    (at, BlockStateId(data)),
                    &mut context.particle_random,
                );
            }
            2008 => self.level_add(
                context,
                "explosion",
                (x + 0.5, y + 0.5, z + 0.5),
                (0.0, 0.0, 0.0),
            ),
            2009 => {
                for _ in 0..8 {
                    let random = &mut *context.level_random;
                    let at = (x + random.next_double(), y + 1.2, z + random.next_double());
                    self.level_add(context, "cloud", at, (0.0, 0.0, 0.0));
                }
            }
            _ => {}
        }
        sounds
    }

    /// `NoteBlock.triggerEvent`: a tuned block's note rises from its top (its colour from the note),
    /// then its sound plays three times as loud, pitched by the note, its seed drawn last.
    fn note_block<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        pos: (i32, i32, i32),
        (sound, note): (&'static str, Option<i32>),
    ) -> LocalSound {
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        let pitch = match note {
            Some(note) => {
                let at = (x + 0.5, y + 1.2, z + 0.5);
                self.level_add(context, "note", at, (f64::from(note) / 24.0, 0.0, 0.0));
                // `NoteBlock.getPitchFromNote`.
                fdlibm_pow(2.0, f64::from(note - 12) / 12.0) as f32
            }
            None => 1.0,
        };
        local_sound(
            sound,
            LocalSoundSource::Records,
            (3.0, pitch),
            pos,
            context.level_random,
        )
    }

    /// `LevelEventHandler.shootParticles`: ten puffs out of a face (a dispenser's), spread across it.
    fn shoot_particles<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        (pos, data): ((i32, i32, i32), i32),
        name: &str,
    ) {
        let (nx, ny, nz) = step_of(data);
        let (nx, ny, nz) = (f64::from(nx), f64::from(ny), f64::from(nz));
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        for _ in 0..10 {
            let random = &mut *context.level_random;
            let pow = random.next_double() * 0.2 + 0.01;
            let at = (
                x + nx * 0.6 + 0.5 + nx * 0.01 + (random.next_double() - 0.5) * nz * 0.5,
                y + ny * 0.6 + 0.5 + ny * 0.01 + (random.next_double() - 0.5) * ny * 0.5,
                z + nz * 0.6 + 0.5 + nz * 0.01 + (random.next_double() - 0.5) * nx * 0.5,
            );
            let push = (
                nx * pow + random.next_gaussian() * 0.01,
                ny * pow + random.next_gaussian() * 0.01,
                nz * pow + random.next_gaussian() * 0.01,
            );
            self.level_add(context, name, at, push);
        }
    }

    /// `spawnTamingParticles` and the love hearts: seven particles in the entity's box, each
    /// pushed by a small gaussian of the entity's random.
    fn entity_burst<F: FnMut() -> RandomSource>(
        &mut self,
        burst: crate::mobs::burst::EntityBurst,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let (x, y, z) = burst.at;
        let (width, height) = (f64::from(burst.size.0), f64::from(burst.size.1));
        for _ in 0..crate::mobs::burst::BURST_COUNT {
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            let at = (
                x + width * (2.0 * random.next_double() - 1.0),
                y + height * random.next_double() + 0.5,
                z + width * (2.0 * random.next_double() - 1.0),
            );
            self.level_add(context, burst.particle, at, push);
        }
    }

    /// `Level.addParticle` of a plain type.
    pub(super) fn level_add<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        name: &str,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
    ) {
        let Some(type_id) = particle_type::id(name) else {
            return;
        };
        let spawn = Spawn {
            type_id,
            data: ParticleData::Plain,
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

/// `Level.playLocalSound(BlockPos, ..)`: at the block's middle, its seed drawn from the level's
/// random after its pitch.
fn local_sound(
    event: &'static str,
    source: LocalSoundSource,
    (volume, pitch): (f32, f32),
    pos: (i32, i32, i32),
    random: &mut RandomSource,
) -> LocalSound {
    random.next_long();
    LocalSound {
        relative: false,
        delay_ticks: 0,
        event,
        source,
        volume,
        pitch,
        position: (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        ),
    }
}
