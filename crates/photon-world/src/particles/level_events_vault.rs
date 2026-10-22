//! The vault's level events (`VaultBlockEntity.Client`): 3015 lights it and reaches for the
//! players that are connected to it, 3016 puts it out.

use photon_core::binary::Uuid;
use photon_core::math::{Vec3, mth_next_double, mth_next_int};
use photon_core::nbt::{Field, FieldValue};
use photon_core::random::RandomSource;
use photon_data::block::classes::BlockClass;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::{delayed_sound, local_sound};
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;
use crate::ClientWorld;

/// `VaultConfig.DEFAULT.deactivationRange()`: how far the connection particles reach by default.
const DEFAULT_CONNECTION_RANGE: f64 = 4.5;

/// A player as the vault sees them: who, where, and how tall.
#[derive(Clone, Copy, Debug)]
pub struct PlayerSpot {
    pub uuid: Uuid,
    pub position: Vec3,
    pub height: f32,
}

/// What a vault shares with the clients that see it (`VaultSharedData`), less its display item.
pub(crate) struct VaultShared {
    connected_players: Vec<Uuid>,
    connected_particles_range: f64,
    /// `VaultSharedData.getDisplayItem`: what it shows and how many, if it shows any.
    pub(crate) display_item: Option<(photon_core::ids::ItemId, i32)>,
    /// The `components` compound of the display item's tag.
    pub(crate) display_components: Vec<Field>,
}

fn field<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|own| own.key == key)
        .map(|own| &own.value)
}

/// `VaultSharedData.CODEC` read from the block entity's tag: the players in the order the server
/// wrote them, the range the connection particles reach.
pub(crate) fn shared_data(world: &ClientWorld, pos: (i32, i32, i32)) -> VaultShared {
    let mut shared = VaultShared {
        connected_players: Vec::new(),
        connected_particles_range: DEFAULT_CONNECTION_RANGE,
        display_item: None,
        display_components: Vec::new(),
    };
    let Some(fields) = world
        .block_entity_at(pos.0, pos.1, pos.2)
        .and_then(|entity| entity.tag.as_deref())
        .and_then(|mut tag| photon_core::nbt::read_nbt(&mut tag).ok().flatten())
    else {
        return shared;
    };
    let Some(FieldValue::Compound(data)) = field(&fields, "shared_data") else {
        return shared;
    };
    if let Some(FieldValue::List(players)) = field(data, "connected_players") {
        shared.connected_players = players.iter().filter_map(uuid_of).collect();
    }
    if let Some(FieldValue::Compound(held)) = field(data, "display_item") {
        shared.display_item = crate::block::chest::tag_stack(held);
        shared.display_components = crate::block::chest::stack_components(held);
    }
    if let Some(FieldValue::Double(range)) = field(data, "connected_particles_range") {
        shared.connected_particles_range = *range;
    }
    shared
}

/// `UUIDUtil.CODEC`: four ints, the high half first.
pub(crate) fn uuid_of(value: &FieldValue) -> Option<Uuid> {
    let FieldValue::IntArray(ints) = value else {
        return None;
    };
    let [a, b, c, d] = ints[..] else {
        return None;
    };
    let half = |high: i32, low: i32| (u64::from(high as u32) << 32) | u64::from(low as u32);
    Some(Uuid::new(half(a, b), half(c, d)))
}

/// `Level.getPlayerByUUID`: the local player, or a remote one the level knows.
fn player_by_uuid<F: FnMut() -> RandomSource>(
    context: &LevelEventContext<'_, F>,
    uuid: Uuid,
) -> Option<PlayerSpot> {
    if let Some(local) = context.local_player.filter(|local| local.uuid == uuid) {
        return Some(local);
    }
    context
        .world
        .entities()
        .values()
        .find(|entity| entity.is_player() && entity.uuid == uuid)
        .map(|entity| {
            let (x, y, z) = entity.position();
            PlayerSpot {
                uuid,
                position: Vec3 { x, y, z },
                height: entity.bb_height(),
            }
        })
}

impl ParticleEngine {
    /// The vault's events: 3015 lights it, 3016 puts it out.
    pub(super) fn vault_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        if event == 3015 {
            self.vault_activate(pos, data, context, sounds);
        } else {
            self.vault_deactivate(pos, data, context, sounds);
        }
    }

    /// `VaultBlockEntity.Client.tick`, as far as the level's random goes: every twenty ticks the
    /// connection particles, then each tick a half chance of smoke (and flame, with a display item
    /// to show) in the cage, and a rare ambient sound.
    pub(super) fn vault_tick<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let state = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        if context.world.game_time().is_multiple_of(20) {
            self.vault_connection_particles(pos, state, context);
        }
        let shows = shared_data(context.world, pos).display_item.is_some();
        let ominous = crate::block::rules::property(state, "ominous") == Some("true");
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        if context.level_random.next_float() <= 0.5 {
            let random = &mut *context.level_random;
            let at = (
                x + mth_next_double(random, 0.1, 0.9),
                y + mth_next_double(random, 0.25, 0.75),
                z + mth_next_double(random, 0.1, 0.9),
            );
            self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
            if shows {
                let flame = if ominous {
                    "soul_fire_flame"
                } else {
                    "small_flame"
                };
                self.level_add(context, flame, at, (0.0, 0.0, 0.0));
            }
        }
        if shows && context.level_random.next_float() <= 0.02 {
            let random = &mut *context.level_random;
            let volume = random.next_float() * 0.25 + 0.75;
            let pitch = random.next_float() + 0.5;
            let centre = (x + 0.5, y + 0.5, z + 0.5);
            let event = ("minecraft:block.vault.ambient", LocalSoundSource::Blocks);
            sounds.push(local_sound(event, (volume, pitch), centre, context));
        }
    }

    /// Event 3015: `VaultBlockEntity.Client.emitActivationParticles` then the vault's sound, where
    /// the block is a vault (the level makes its block entity as the block is placed).
    fn vault_activate<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let state = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        if !crate::block::state_or_air(state)
            .block()
            .class
            .is(BlockClass::VaultBlock)
        {
            return;
        }
        let flame = vault_flame(data);
        self.vault_connection_particles(pos, state, context);
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        for _ in 0..20 {
            let random = &mut *context.level_random;
            let at = (
                x + mth_next_double(random, 0.1, 0.9),
                y + mth_next_double(random, 0.25, 0.75),
                z + mth_next_double(random, 0.1, 0.9),
            );
            self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
            self.level_add(context, flame, at, (0.0, 0.0, 0.0));
        }
        let random = &mut *context.level_random;
        let pitch = (random.next_float() - random.next_float()) * 0.2 + 1.0;
        let centre = (x + 0.5, y + 0.5, z + 0.5);
        let event = ("minecraft:block.vault.activate", LocalSoundSource::Blocks);
        sounds.push(delayed_sound(event, (1.0, pitch), centre, context));
    }

    /// `emitConnectionParticlesForNearbyPlayers`: a few flying from the keyhole to each connected
    /// player within reach.
    fn vault_connection_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        state: photon_core::ids::BlockStateId,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let shared = shared_data(context.world, pos);
        if shared.connected_players.is_empty() {
            return;
        }
        // `keyholePos`: the middle of the block's foot, a block and three quarters up, half a
        // block towards the way it faces.
        let (step_x, step_z) = match crate::block::rules::property(state, "facing") {
            Some("south") => (0.0, 1.0),
            Some("west") => (-1.0, 0.0),
            Some("east") => (1.0, 0.0),
            _ => (0.0, -1.0),
        };
        let keyhole = (
            f64::from(pos.0) + 0.5 + step_x * 0.5,
            f64::from(pos.1) + 1.75,
            f64::from(pos.2) + 0.5 + step_z * 0.5,
        );
        let reach = shared.connected_particles_range * shared.connected_particles_range;
        for uuid in shared.connected_players {
            let Some(player) = player_by_uuid(context, uuid) else {
                continue;
            };
            // `player.blockPosition().distSqr(vaultPos)`.
            let block = [player.position.x, player.position.y, player.position.z]
                .map(photon_core::math::floor_f64);
            let distance = [block[0] - pos.0, block[1] - pos.1, block[2] - pos.2]
                .map(|d| f64::from(d) * f64::from(d))
                .iter()
                .sum::<f64>();
            if distance <= reach {
                self.vault_connection_to(keyhole, player, context);
            }
        }
    }

    /// `emitConnectionParticlesForPlayer`: two to five particles from the keyhole, each aimed at
    /// the middle of the player with a nudge.
    fn vault_connection_to<F: FnMut() -> RandomSource>(
        &mut self,
        keyhole: (f64, f64, f64),
        player: PlayerSpot,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let towards = (
            player.position.x - keyhole.0,
            player.position.y + f64::from(player.height / 2.0) - keyhole.1,
            player.position.z - keyhole.2,
        );
        let random = &mut *context.level_random;
        let count = mth_next_int(random, 2, 5).unwrap_or(2);
        for _ in 0..count {
            let random = &mut *context.level_random;
            // `Vec3.offsetRandom(random, 1.0F)`.
            let mut nudge = |base: f64| base + f64::from((random.next_float() - 0.5) * 1.0);
            let x = nudge(towards.0);
            let y = nudge(towards.1);
            let z = nudge(towards.2);
            self.level_add(context, "vault_connection", keyhole, (x, y, z));
        }
    }

    /// Event 3016, `VaultBlockEntity.Client.emitDeactivationParticles` then the vault's sound:
    /// twenty flames about the middle of its cage (a soul flame for an ominous vault).
    fn vault_deactivate<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let flame = vault_flame(data);
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        for _ in 0..20 {
            let random = &mut *context.level_random;
            let at = (
                x + mth_next_double(random, 0.4, 0.6),
                y + mth_next_double(random, 0.4, 0.6),
                z + mth_next_double(random, 0.4, 0.6),
            );
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            self.level_add(context, flame, at, push);
        }
        let random = &mut *context.level_random;
        let pitch = (random.next_float() - random.next_float()) * 0.2 + 1.0;
        let centre = (x + 0.5, y + 0.5, z + 0.5);
        let event = ("minecraft:block.vault.deactivate", LocalSoundSource::Blocks);
        sounds.push(delayed_sound(event, (1.0, pitch), centre, context));
    }
}

/// The flame of a vault event: a soul flame where the vault is ominous.
fn vault_flame(data: i32) -> &'static str {
    if data == 0 {
        "small_flame"
    } else {
        "soul_fire_flame"
    }
}
