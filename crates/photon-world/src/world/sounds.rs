use crate::*;
use photon_core::ids::BlockStateId;

impl ClientWorld {
    /// A sound the world makes at a place (`Level.playLocalSound`), for the sound engine to play.
    pub fn push_sound(&mut self, sound: local_player::LocalSound) {
        self.sounds.0.push(sound);
    }

    /// The pitch of an igniter's sound on a creeper, `random.nextFloat() * 0.4F + 0.8F`.
    pub fn entity_sound_pitch(&mut self) -> f32 {
        self.entity_sound_random.source().next_float() * 0.4 + 0.8
    }

    /// Fixes the source `entity_sound_pitch` draws from, so a comparison can be made.
    pub fn fix_entity_sound_seed(&mut self, seed: i64) {
        self.entity_sound_random.fix_seed(seed);
    }

    /// The particles the living entities' effects threw since the last call, in order.
    pub fn take_effect_particles(&mut self) -> Vec<particles::LevelParticles> {
        std::mem::take(&mut self.effect_particles.0)
    }

    /// The sounds made since the last call, in order.
    pub fn take_sounds(&mut self) -> Vec<local_player::LocalSound> {
        std::mem::take(&mut self.sounds.0)
    }

    /// A sound the level plays at the local player's hearing (`ClientLevel.playSound`).
    pub fn play_local_sound(&mut self, sound: local_player::LocalSound) {
        self.sounds.0.push(sound);
    }

    /// The sound a block makes (`SoundType`) at its middle, played now.
    pub(crate) fn block_sound(
        &mut self,
        pos: (i32, i32, i32),
        state: BlockStateId,
        pick: fn(&photon_data::block_sound::SoundTypeRow) -> &'static str,
        scale: (fn(f32) -> f32, f32),
    ) {
        if let Some(sound) = block_sound_of(pos, state, pick, scale) {
            self.sounds.0.push(sound);
        }
    }

    /// `BlockItem.place`'s sound: the block placed, a little softer than loud and lower.
    pub fn play_place_sound(&mut self, pos: (i32, i32, i32), state: BlockStateId) {
        self.block_sound(
            pos,
            state,
            photon_data::block_sound::SoundTypeRow::place_sound,
            (|volume| (volume + 1.0) / 2.0, 0.8),
        );
    }

    /// `MultiPlayerGameMode.continueDestroyBlock`'s hit sound: a block struck while it is broken.
    pub fn play_hit_sound(&mut self, pos: (i32, i32, i32), state: BlockStateId) {
        self.block_sound(
            pos,
            state,
            photon_data::block_sound::SoundTypeRow::hit_sound,
            (|volume| (volume + 1.0) / 8.0, 0.5),
        );
    }

    /// `LevelEventHandler.levelEvent`, for the events that make a sound the client knows: `2001`,
    /// a block broken (its state's number is the data).
    /// The level events since the last call, in the order they came.
    pub fn take_level_events(&mut self) -> Vec<LevelEvent> {
        std::mem::take(&mut self.level_events.0)
    }

    /// The block and face the player swung at breaking, since the last call.
    pub fn take_breaking_effects(
        &mut self,
    ) -> Vec<((i32, i32, i32), photon_core::direction::Direction)> {
        std::mem::take(&mut self.breaking_effects)
    }

    /// `ClientLevel.addBreakingBlockEffect` asked for, as `Minecraft.continueAttack` asks.
    pub fn add_breaking_block_effect(
        &mut self,
        pos: (i32, i32, i32),
        face: photon_core::direction::Direction,
    ) {
        self.breaking_effects.push((pos, face));
    }

    /// `ClientLevel.levelEvent`: the event waits for the particle engine, which plays its sounds
    /// and makes its particles with the level's random.
    pub fn apply_level_event(&mut self, event: i32, pos: (i32, i32, i32), data: i32) {
        self.level_events
            .0
            .push(LevelEvent::Level { event, pos, data });
    }

    /// `ClientLevel.globalLevelEvent`, for the particle engine to play.
    pub fn apply_global_level_event(&mut self, event: i32, pos: (i32, i32, i32)) {
        self.level_events.0.push(LevelEvent::Global { event, pos });
    }

    /// A firework rocket's explosions, for the particle engine to burst.
    pub fn create_fireworks(&mut self, launch: crate::particles::FireworkLaunch) {
        self.level_events
            .0
            .push(LevelEvent::Fireworks(Box::new(launch)));
    }

    /// An explosion the server reports, for the particle engine to play.
    pub fn apply_explosion(&mut self, explosion: crate::particles::Explosion) {
        self.level_events
            .0
            .push(LevelEvent::Explosion(Box::new(explosion)));
    }

    /// `ClientLevel.blockEvent`: the block as it stands takes the event.
    pub fn apply_block_event(&mut self, pos: (i32, i32, i32), event: u8, data: u8) {
        if !self.apply_chest_block_event(pos, event, data) && !self.apply_note_block_event(pos) {
            if self.is_piston_base(pos) {
                self.apply_piston_event(pos, i32::from(event), i32::from(data));
            } else {
                self.apply_block_entity_event(pos, event, data);
            }
        }
    }

    /// Whether the block at a place is a piston (`PistonBaseBlock`).
    fn is_piston_base(&self, pos: (i32, i32, i32)) -> bool {
        self.block_state_at(pos.0, pos.1, pos.2)
            .is_some_and(|state| {
                block::state_or_air(state)
                    .block()
                    .class
                    .is(photon_data::block::classes::BlockClass::PistonBaseBlock)
            })
    }

    /// `BaseEntityBlock.triggerEvent` of the blocks whose entity keeps what the event says: a
    /// decorated pot starts to wobble, an end gateway's cooldown starts, a spawner's delay is
    /// reset and a potent sulfur block notes when it erupted.
    fn apply_block_entity_event(&mut self, pos: (i32, i32, i32), event: u8, data: u8) {
        let Some(state) = self.block_state_at(pos.0, pos.1, pos.2) else {
            return;
        };
        let class = block::state_or_air(state).block().class;
        let time = self.game_time() as i64;
        let Some((type_name, react)) = BLOCK_ENTITY_REACTIONS
            .iter()
            .find(|(own, _, _)| class.is(*own))
            .map(|(_, name, react)| (*name, *react))
        else {
            return;
        };
        if let Some(entity) = self.block_entity_or_new(pos, type_name) {
            react(entity, (i32::from(event), i32::from(data)), time);
        }
    }

    /// `ClientLevel.blockEvent`, for what the client does with one beside a block entity's: a note
    /// block plays its instrument as it is now (`NoteBlock.triggerEvent`). False for another block.
    pub fn apply_note_block_event(&mut self, pos: (i32, i32, i32)) -> bool {
        let Some(state) = self.block_state_at(pos.0, pos.1, pos.2) else {
            return false;
        };
        if block::state_or_air(state).block().name != "minecraft:note_block" {
            return false;
        }
        let Some(sound) = block::rules::property(state, "instrument").and_then(note_block_sound)
        else {
            // `custom_head`: the head above's own sound is not read; it plays nothing.
            return false;
        };
        let tunable = !sound.contains(".imitate.");
        let note = block::rules::property(state, "note")
            .and_then(|note| note.parse().ok())
            .filter(|_| tunable);
        self.level_events
            .0
            .push(LevelEvent::Note { pos, sound, note });
        true
    }
}

/// What a block event does to the entity of a block: the block's class, its entity's type and the
/// reaction (the event, its data and the game time).
type Reaction = fn(&mut ClientBlockEntity, (i32, i32), i64);

const BLOCK_ENTITY_REACTIONS: [(photon_data::block::classes::BlockClass, &str, Reaction); 4] = {
    use photon_data::block::classes::BlockClass;
    [
        (
            BlockClass::DecoratedPotBlock,
            "minecraft:decorated_pot",
            pot_wobbles,
        ),
        (
            BlockClass::EndGatewayBlock,
            "minecraft:end_gateway",
            gateway_cools_down,
        ),
        (
            BlockClass::SpawnerBlock,
            "minecraft:mob_spawner",
            spawner_resets,
        ),
        (
            BlockClass::PotentSulfurBlock,
            "minecraft:potent_sulfur",
            sulfur_erupts,
        ),
    ]
};

/// `DecoratedPotBlockEntity.triggerEvent`: event 1 with one of the two wobble styles.
fn pot_wobbles(entity: &mut ClientBlockEntity, (event, style): (i32, i32), time: i64) {
    if event == 1 && (0..=1).contains(&style) {
        entity.events.wobble = Some((time, style));
    }
}

/// `TheEndGatewayBlockEntity.triggerEvent`.
fn gateway_cools_down(entity: &mut ClientBlockEntity, (event, _): (i32, i32), _: i64) {
    if event == 1 {
        entity.events.teleport_cooldown = 40;
    }
}

/// `BaseSpawner.onEventTriggered`.
fn spawner_resets(entity: &mut ClientBlockEntity, (event, _): (i32, i32), _: i64) {
    if event == 1 {
        let spawner = entity.spawner_state.get_or_insert_with(Default::default);
        spawner.delay = spawner.min_spawn_delay;
    }
}

/// `PotentSulfurBlock.triggerEvent`: whatever the event.
fn sulfur_erupts(entity: &mut ClientBlockEntity, _: (i32, i32), time: i64) {
    entity.events.eruption_tick = time;
}

/// The sound a note block's instrument plays (`NoteBlockInstrument.getSoundEvent`); none for
/// `custom_head`, whose sound is its head's.
fn note_block_sound(instrument: &str) -> Option<&'static str> {
    #[rustfmt::skip]
    const SOUNDS: [&str; 26] = [
        "harp", "basedrum", "snare", "hat", "bass", "flute", "bell", "guitar", "chime", "xylophone",
        "iron_xylophone", "cow_bell", "didgeridoo", "bit", "banjo", "pling", "trumpet",
        "trumpet_exposed", "trumpet_oxidized", "trumpet_weathered", "imitate.zombie",
        "imitate.skeleton", "imitate.creeper", "imitate.ender_dragon", "imitate.wither_skeleton",
        "imitate.piglin",
    ];
    #[rustfmt::skip]
    const EVENTS: [&str; 26] = [
        "minecraft:block.note_block.harp", "minecraft:block.note_block.basedrum",
        "minecraft:block.note_block.snare", "minecraft:block.note_block.hat",
        "minecraft:block.note_block.bass", "minecraft:block.note_block.flute",
        "minecraft:block.note_block.bell", "minecraft:block.note_block.guitar",
        "minecraft:block.note_block.chime", "minecraft:block.note_block.xylophone",
        "minecraft:block.note_block.iron_xylophone", "minecraft:block.note_block.cow_bell",
        "minecraft:block.note_block.didgeridoo", "minecraft:block.note_block.bit",
        "minecraft:block.note_block.banjo", "minecraft:block.note_block.pling",
        "minecraft:block.note_block.trumpet", "minecraft:block.note_block.trumpet_exposed",
        "minecraft:block.note_block.trumpet_oxidized",
        "minecraft:block.note_block.trumpet_weathered",
        "minecraft:block.note_block.imitate.zombie", "minecraft:block.note_block.imitate.skeleton",
        "minecraft:block.note_block.imitate.creeper",
        "minecraft:block.note_block.imitate.ender_dragon",
        "minecraft:block.note_block.imitate.wither_skeleton",
        "minecraft:block.note_block.imitate.piglin",
    ];
    let name = match instrument {
        "dragon" => "imitate.ender_dragon",
        "zombie" => "imitate.zombie",
        "skeleton" => "imitate.skeleton",
        "creeper" => "imitate.creeper",
        "wither_skeleton" => "imitate.wither_skeleton",
        "piglin" => "imitate.piglin",
        other => other,
    };
    SOUNDS
        .iter()
        .position(|sound| *sound == name)
        .map(|index| EVENTS[index])
}

/// The level events waiting for the particle engine. Which are waiting is no part of what a world is
/// equal to.
#[derive(Debug, Clone, Default)]
pub struct LevelEventQueue(pub(crate) Vec<LevelEvent>);

impl PartialEq for LevelEventQueue {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for LevelEventQueue {}

/// What the level plays when the particle engine takes it, in the order it came.
#[derive(Debug, Clone, PartialEq)]
pub enum LevelEvent {
    /// One `ClientboundLevelEventPacket` (or a level event of the client's own).
    Level {
        event: i32,
        pos: (i32, i32, i32),
        data: i32,
    },
    /// A note block struck: its instrument's sound, and its note when it is tuned.
    Note {
        pos: (i32, i32, i32),
        sound: &'static str,
        note: Option<i32>,
    },
    /// An explosion the server reports (`ClientboundExplodePacket`).
    Explosion(Box<crate::particles::Explosion>),
    /// A level event every player hears (`LevelEventHandler.globalLevelEvent`).
    Global { event: i32, pos: (i32, i32, i32) },
    /// Hearts or smoke around an animal (`Animal` and `TamableAnimal.handleEntityEvent`).
    EntityBurst(crate::mobs::burst::EntityBurst),
    /// A firework rocket went off (`ClientLevel.createFireworks`).
    Fireworks(Box<crate::particles::FireworkLaunch>),
    /// The sound a piston asks the level for when it moves: the server plays it to the player, and
    /// the client only draws the pitch from its random source.
    PistonSound,
    /// A potent sulfur block's ticker adds a cloud of its gas (`addParticle`) at a place.
    NoxiousGasCloud { at: (f64, f64, f64) },
    /// A parrot's tick (`Parrot.aiStep`): the draw that decides whether it imitates a mob nearby.
    ParrotImitation { id: photon_core::ids::EntityId },
    /// A vault's tick (`VaultBlockEntity.Client.tick`), less its spin: the connection particles,
    /// the idle smoke and flame and the idle sound.
    VaultTick { pos: (i32, i32, i32) },
    /// A spawner's tick with a player near (`BaseSpawner.clientTick`): a puff of smoke and flame.
    SpawnerTick { pos: (i32, i32, i32) },
    /// A trial spawner's tick (`TrialSpawner.tickClient`), less its spin: the particles of its
    /// state and its ambient sound.
    TrialSpawnerTick { pos: (i32, i32, i32) },
    /// A conduit's tick (`ConduitBlockEntity.animationTick`): its frame sends nautilus specks
    /// up to it.
    ConduitSpecks {
        pos: (i32, i32, i32),
        tick_count: i32,
        frame: Vec<(i32, i32, i32)>,
        /// The mob it attacks, if the update tag names one.
        target: Option<photon_core::binary::Uuid>,
    },
    /// An erupting potent sulfur block sends its geyser up through the water above it.
    Geyser {
        at: (f64, f64, f64),
        water_blocks: i32,
    },
    /// A sound the geyser makes at a place (`Level.playLocalSound` of the blocks' kind).
    GeyserSound {
        at: (f64, f64, f64),
        sound: &'static str,
    },
}

impl LevelEvent {
    /// The sound of a block broken (2001): its break sound, louder and lower, at its middle;
    /// nothing for air or another event.
    pub fn break_sound(&self) -> Option<local_player::LocalSound> {
        let Self::Level {
            event: 2001,
            pos,
            data,
        } = *self
        else {
            return None;
        };
        let state = BlockStateId(data);
        if block::state_or_air(state).block().name == "minecraft:air" {
            return None;
        }
        block_sound_of(
            pos,
            state,
            photon_data::block_sound::SoundTypeRow::break_sound,
            (|volume| (volume + 1.0) / 2.0, 0.8),
        )
    }
}

/// The sound a block makes (`SoundType`) at its middle: its volume and pitch scaled as the event
/// asks, whichever of the five sounds `pick` takes. Nothing for a state with no sound type.
pub fn block_sound_of(
    pos: (i32, i32, i32),
    state: BlockStateId,
    pick: fn(&photon_data::block_sound::SoundTypeRow) -> &'static str,
    (volume, pitch): (fn(f32) -> f32, f32),
) -> Option<local_player::LocalSound> {
    let info = block::state_or_air(state);
    let sound_type = u32::try_from(state.0)
        .ok()
        .and_then(|id| photon_data::block_sound::of_state(info.block().name, id))?;
    Some(local_player::LocalSound {
        relative: false,
        follows_player: false,
        delay_ticks: 0,
        event: pick(sound_type),
        source: local_player::LocalSoundSource::Blocks,
        volume: volume(sound_type.volume),
        pitch: sound_type.pitch * pitch,
        position: (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        ),
    })
}
