use crate::*;

impl ClientWorld {
    /// Where the living entity of an id is and how tall its eyes are, if the world holds one.
    pub(crate) fn living_target(&self, id: EntityId) -> Option<mobs::guardian::GuardianTarget> {
        if let Some(player) = self.local_player.as_ref().filter(|p| p.id() == id) {
            let dimensions = player.player_dimensions();
            return Some(mobs::guardian::GuardianTarget {
                position: player.position(),
                height: dimensions.height(),
                eye_height: dimensions.eye_height(),
            });
        }
        let target = self.entities.at(id)?;
        target.living_kind()?;
        let dimensions = target.dimensions().scale(target.scale(), target.scale());
        Some(mobs::guardian::GuardianTarget {
            position: target.position(),
            height: dimensions.height(),
            eye_height: dimensions.eye_height(),
        })
    }

    pub(crate) fn species_surroundings(&self, entity_id: EntityId) -> (bool, bool, bool) {
        let sleeper_near_cat = self.entities.get(&entity_id).is_some_and(|entity| {
            entity.is_lying_cat() && self.sleeping_player_near(entity.entity)
        });
        (
            sleeper_near_cat,
            self.is_thundering(),
            self.parrot_jukebox_near(entity_id),
        )
    }

    /// The test at the start of `Parrot.aiStep`: its jukebox is within 3.46 of it, middle to
    /// position, and still a jukebox.
    fn parrot_jukebox_near(&self, entity_id: EntityId) -> bool {
        let Some(entity) = self.entities.get(&entity_id) else {
            return false;
        };
        let Some(jukebox) = entity.parrot().and_then(|parrot| parrot.jukebox()) else {
            return false;
        };
        let (x, y, z) = entity.position();
        let dx = f64::from(jukebox.0) + 0.5 - x;
        let dy = f64::from(jukebox.1) + 0.5 - y;
        let dz = f64::from(jukebox.2) + 0.5 - z;
        let state = self.block_state_or_air(jukebox.0, jukebox.1, jukebox.2);
        dx * dx + dy * dy + dz * dz < 3.46 * 3.46
            && crate::block::state_or_air(state).block().name == "minecraft:jukebox"
    }

    /// `LevelEventHandler.notifyNearbyEntities`: the living entities within three blocks of a
    /// jukebox hear that a record plays or stops (only a parrot does anything about it).
    pub fn set_record_playing_nearby(&mut self, jukebox: (i32, i32, i32), playing: bool) {
        let reach = photon_core::aabb::Aabb::new(
            f64::from(jukebox.0),
            f64::from(jukebox.1),
            f64::from(jukebox.2),
            f64::from(jukebox.0) + 1.0,
            f64::from(jukebox.1) + 1.0,
            f64::from(jukebox.2) + 1.0,
        )
        .inflate_all(3.0);
        let near: Vec<EntityId> = self
            .entities
            .values()
            .filter(|entity| entity.is_parrot() && entity.bounding_box().intersects(reach))
            .map(|entity| entity.id())
            .collect();
        for id in near {
            if let Some(mut entity) = self.entities.at_mut(id) {
                entity.set_record_playing(jukebox, playing);
            }
        }
    }

    /// `Guardian.getActiveAttackTarget` on the client: the living entity the
    /// beam's id names, the local player among them.
    pub(crate) fn guardian_target_of(
        &self,
        entity_id: EntityId,
    ) -> Option<mobs::guardian::GuardianTarget> {
        let id = EntityId(self.entities.at(entity_id)?.guardian_attack_target()?);
        self.living_target(id)
    }

    /// What a tick of the entity looks up in the world.
    pub(crate) fn targets_of(&self, entity_id: EntityId) -> entity::EntityTargets {
        entity::EntityTargets {
            guardian: self.guardian_target_of(entity_id),
            wither: self.wither_targets_of(entity_id),
            dragon: self.entities().dragon_crystals(entity_id),
        }
    }

    /// The two entities a wither's side heads look at, by the ids it holds.
    pub(crate) fn wither_targets_of(
        &self,
        entity_id: EntityId,
    ) -> [Option<mobs::guardian::GuardianTarget>; 2] {
        let ids = self
            .entities
            .get(&entity_id)
            .map_or([0; 2], |entity| entity.wither_targets());
        ids.map(|id| (id > 0).then(|| self.living_target(EntityId(id))).flatten())
    }
}

impl Entities {
    /// `EnderDragon.checkCrystals`' look at the world: the nearest end crystal whose box
    /// meets the dragon's box made 32 blocks wider (by the distance between the two
    /// entities' places), and whether the crystal the dragon holds is still there.
    pub(crate) fn dragon_crystals(
        &self,
        entity_id: EntityId,
    ) -> mobs::ender_dragon::DragonCrystals {
        let Some(dragon) = self.get(&entity_id).filter(|e| e.is_ender_dragon()) else {
            return mobs::ender_dragon::DragonCrystals::default();
        };
        let current = dragon.ender_dragon().and_then(|own| own.nearest_crystal());
        let current_exists =
            current.is_some_and(|id| self.get(&id).is_some_and(|entity| entity.is_end_crystal()));
        let (x, y, z) = dragon.position();
        let size = dragon.dimensions();
        let half = f64::from(size.width()) / 2.0 + 32.0;
        let (low, high) = (
            [x - half, y - 32.0, z - half],
            [x + half, y + f64::from(size.height()) + 32.0, z + half],
        );
        let mut nearest = None;
        let mut distance = f64::MAX;
        for crystal in self.values().filter(|e| e.is_end_crystal()) {
            let (cx, cy, cz) = crystal.position();
            let crystal_size = crystal.dimensions();
            let reach = f64::from(crystal_size.width()) / 2.0;
            let height = f64::from(crystal_size.height());
            let meets = low[0] < cx + reach
                && high[0] > cx - reach
                && low[1] < cy + height
                && high[1] > cy
                && low[2] < cz + reach
                && high[2] > cz - reach;
            if !meets {
                continue;
            }
            let squared = (cx - x).powi(2) + (cy - y).powi(2) + (cz - z).powi(2);
            if squared < distance {
                distance = squared;
                nearest = Some(crystal.id());
            }
        }
        mobs::ender_dragon::DragonCrystals {
            nearest,
            current_exists,
        }
    }
}
