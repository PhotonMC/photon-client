//! `BaseSpawner.clientTick`: the spinning mob of a spawner turns while a player is near, and puffs
//! of smoke and flame rise from it.

use crate::block::piston::Pos;
use crate::*;

/// The mob a spawner shows, made once for the entity type its tag names.
#[derive(Debug)]
pub(crate) struct SpawnerDisplayEntry {
    pos: Pos,
    type_name: String,
    entity: ClientEntity,
}

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

    /// `BaseSpawner.getOrCreateDisplayEntity`: the mob of the type the spawner names is made when it
    /// first shows one and again when the type changes; a spawner that names none shows none.
    fn sync_spawner_display(&mut self, pos: Pos, type_name: Option<&str>) {
        let current = self
            .spawner_displays
            .iter()
            .position(|shown| shown.pos == pos);
        let kept = current.is_some_and(|index| {
            Some(self.spawner_displays[index].type_name.as_str()) == type_name
        });
        if kept {
            return;
        }
        if let Some(index) = current {
            self.spawner_displays.remove(index);
        }
        let Some(type_name) = type_name else { return };
        let Some(type_id) = photon_data::entity_type::id(type_name) else {
            return;
        };
        let id = EntityId(
            self.spawner_displays
                .iter()
                .map(|shown| shown.entity.id().0)
                .min()
                .unwrap_or(-1)
                - 1,
        );
        let placed = self.apply_entity(EntityInput {
            entity_id: id,
            uuid: photon_core::binary::Uuid::new(9, id.0 as u64),
            type_id,
            x: f64::from(pos.0) + 0.5,
            y: f64::from(pos.1) + 0.5,
            z: f64::from(pos.2) + 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        });
        // Never in the level: the entity is taken out again, as `displayEntity` is not added.
        let position = self.entities.at(id).map(ClientEntity::position);
        if let Some(position) = position {
            self.entities_mut().remove_from_section(id, position);
        }
        let entity = self.entities.remove(id);
        if let (true, Some(entity)) = (placed, entity) {
            self.spawner_displays.push(SpawnerDisplayEntry {
                pos,
                type_name: type_name.to_owned(),
                entity,
            });
        }
    }

    /// The mobs to draw: the entities the server sent and the ones spawners show.
    pub fn drawn_mobs(&self) -> impl Iterator<Item = &ClientEntity> {
        self.entities.drawn_with_invisible().chain(
            self.spawner_displays
                .iter()
                .filter(|shown| {
                    self.block_entity_at(shown.pos.0, shown.pos.1, shown.pos.2)
                        .is_some()
                })
                .map(|shown| &shown.entity),
        )
    }

    /// The mob a spawner shows, by the id it was made with.
    pub(crate) fn spawner_display_entity(&self, entity_id: EntityId) -> Option<&ClientEntity> {
        self.spawner_displays
            .iter()
            .find(|shown| shown.entity.id() == entity_id)
            .map(|shown| &shown.entity)
    }

    /// `TrialSpawnerRenderer.extractSpawnerData`: where and how the mob of a spawner is drawn.
    pub(crate) fn spawner_display_pose(
        &self,
        entity_id: EntityId,
        partial_tick: f32,
    ) -> Option<SpawnerDisplayPose> {
        let shown = self
            .spawner_displays
            .iter()
            .find(|shown| shown.entity.id() == entity_id)?;
        let spin = &self
            .block_entity_at(shown.pos.0, shown.pos.1, shown.pos.2)?
            .events
            .spawner;
        let (previous, current) = (spin.previous(), spin.current());
        let turn = (previous + f64::from(partial_tick) * (current - previous)) as f32 * 10.0;
        let dimensions = shown.entity.dimensions();
        let longest = dimensions.width().max(dimensions.height());
        let mut scale = 0.53125_f32;
        if longest > 1.0 {
            scale /= longest;
        }
        Some(SpawnerDisplayPose {
            block: shown.pos,
            spin: turn,
            scale,
        })
    }

    /// `BaseSpawner.clientTick`: with a player within the required range and a mob to show, the
    /// delay runs down, the spin turns and smoke and flame are asked of the particle engine.
    pub(super) fn tick_spawner(&mut self, pos: Pos) {
        let Some(entity) = self.block_entity_at(pos.0, pos.1, pos.2) else {
            self.block_tickers.unregister(pos);
            self.sync_spawner_display(pos, None);
            return;
        };
        let state = entity.spawner_state().cloned().unwrap_or_default();
        self.sync_spawner_display(pos, state.entity_id.as_deref());
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
