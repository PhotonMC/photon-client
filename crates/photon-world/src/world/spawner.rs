//! `BaseSpawner.clientTick`: the spinning mob of a spawner turns while a player is near, and puffs
//! of smoke and flame rise from it.

use crate::block::piston::Pos;
use crate::*;

impl ClientWorld {
    /// `EntityGetter.hasNearbyAlivePlayer`: a player that is not a spectator and still lives
    /// within `range` of the point (a negative range reaches everywhere).
    fn has_nearby_alive_player(&self, (x, y, z): (f64, f64, f64), range: f64) -> bool {
        let near = |(px, py, pz): (f64, f64, f64)| {
            range < 0.0 || (px - x).powi(2) + (py - y).powi(2) + (pz - z).powi(2) < range * range
        };
        let local = self.local_player.as_ref().is_some_and(|player| {
            !player.game_mode.is_spectator() && player.health() > 0.0 && near(player.position())
        });
        local
            || self.entities.iter().any(|(_, entity)| {
                entity.is_player() && !entity.is_dead_or_dying() && near(entity.position())
            })
    }

    /// `BaseSpawner.clientTick`: with a player within the required range and a mob to show, the
    /// delay runs down, the spin turns and smoke and flame are asked of the particle engine.
    pub(super) fn tick_spawner(&mut self, pos: Pos) {
        let Some(entity) = self.block_entity_at(pos.0, pos.1, pos.2) else {
            self.block_tickers.unregister(pos);
            return;
        };
        let state = entity.spawner_state().cloned().unwrap_or_default();
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let near = self.has_nearby_alive_player(centre, f64::from(state.required_player_range));
        let shows = near && state.entity_id.is_some();
        let Some(entity) = self.block_entity_mut_at(pos) else {
            return;
        };
        if !near {
            entity.events.spawner.rest();
        } else if shows {
            let spawner = entity.spawner_state.get_or_insert_with(Default::default);
            if spawner.delay > 0 {
                spawner.delay -= 1;
            }
            let delay = spawner.delay;
            entity.events.spawner.turn(delay);
            self.level_events.0.push(LevelEvent::SpawnerTick { pos });
        }
    }
}
