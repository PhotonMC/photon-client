use super::*;
use photon_core::ids::EntityId;

impl ClientEntity {
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub(crate) fn collision_candidate(
        &self,
        source_entity_id: Option<EntityId>,
        source_type_id: Option<EntityTypeId>,
        source_position: Option<Vec3>,
        same_vehicle: bool,
    ) -> Option<ClientEntityCollisionCandidate> {
        let (width, height) = if is_vanilla_boat_entity_type(self.type_id) {
            (
                (VANILLA_BOAT_HALF_WIDTH * 2.0) as f32,
                VANILLA_BOAT_HEIGHT as f32,
            )
        } else if is_vanilla_shulker_entity_type(self.type_id) {
            (1.0, 1.0)
        } else if is_vanilla_minecart_entity_type(self.type_id) {
            (0.98, 0.7)
        } else if is_vanilla_player_entity_type(self.type_id) {
            (0.6, 1.8)
        } else if is_vanilla_happy_ghast_entity_type(self.type_id) {
            (4.0, 4.0)
        } else {
            return None;
        };
        let (x, y, z) = self.position();
        let dimensions = if is_vanilla_happy_ghast_entity_type(self.type_id) && self.baby {
            ClientEntityDimensions::scalable(width, height).scale(0.2375, 0.2375)
        } else {
            ClientEntityDimensions::scalable(width, height)
        };
        let bounding_box = if is_vanilla_shulker_entity_type(self.type_id) {
            self.shulker_bounding_box()
        } else {
            dimensions.make_bounding_box(Vec3::new(x, y, z))
        };
        let can_be_collided_with = if is_vanilla_happy_ghast_entity_type(self.type_id) {
            !self.baby
                && self.alive
                && ((source_type_id == Some(VANILLA_PLAYER_ENTITY_TYPE_ID)
                    && source_position.is_some_and(|position| position.y >= bounding_box.max_y))
                    || (source_type_id == Some(VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID)
                        && self.happy_ghast_is_vehicle)
                    || self.happy_ghast_still_timeout)
        } else {
            is_vanilla_boat_entity_type(self.type_id)
                || is_vanilla_shulker_entity_type(self.type_id)
        };
        let is_pushable = if is_vanilla_player_entity_type(self.type_id) {
            self.alive
                && !self.spectator
                && (self.flying || !self.on_climbable)
                && self
                    .health
                    .is_none_or(|health| f32::from_bits(health) > 0.0)
        } else {
            is_vanilla_pushable_entity_type(self.type_id)
                || (is_vanilla_happy_ghast_entity_type(self.type_id) && self.alive)
        };
        Some(ClientEntityCollisionCandidate {
            bounding_box,
            is_source: source_entity_id == Some(self.id),
            spectator: self.spectator,
            can_be_collided_with,
            is_pushable,
            same_vehicle,
        })
    }

    pub fn vehicle_id(&self) -> Option<EntityId> {
        self.vehicle_id
    }

    pub fn passenger_ids(&self) -> &[EntityId] {
        &self.passenger_ids
    }

    /// Whether the type is a `LivingEntity`.
    pub fn is_living_entity(&self) -> bool {
        self.living_kind().is_some()
    }

    /// `Entity.blocksBuilding`: living entities (an armor stand unless it is a marker), falling
    /// blocks, end crystals, primed tnt, boats and minecarts keep a block from being placed
    /// into them.
    pub fn blocks_building(&self) -> bool {
        if self.type_id == armor_stand_type_id() {
            return !self.armor_stand().is_marker();
        }
        self.is_living_entity()
            || self.type_id == crate::constants::VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID
            || self.is_end_crystal()
            || self.is_primed_tnt()
            || self.is_boat()
            || self.is_minecart()
    }

    /// Whether the type is a `Mob`: a living entity that is not a player, an armor stand or a
    /// mannequin.
    pub fn is_mob(&self) -> bool {
        self.living_kind() == Some(LivingKind::Mob)
            && photon_data::entity_type::name(self.type_id) != Some("minecraft:mannequin")
    }

    /// Whether this is an invisible mob: not a player and not an armor stand, which are drawn
    /// by their own rules.
    pub fn is_invisible_mob(&self) -> bool {
        self.is_invisible() && self.living_kind() == Some(LivingKind::Mob)
    }

    /// Which kind of living entity this is, as far as its drawn pose goes.
    pub(crate) fn living_kind(&self) -> Option<LivingKind> {
        if self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            Some(LivingKind::Player)
        } else if !vanilla_passenger_is_living(self.type_id) {
            None
        } else if self.type_id == armor_stand_type_id() {
            Some(LivingKind::Other)
        } else {
            Some(LivingKind::Mob)
        }
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn is_ticking(&self) -> bool {
        self.ticking
    }

    pub fn is_always_ticking(&self) -> bool {
        self.always_ticking
    }

    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
    }

    /// The block the entity was added in (`FallingBlockEntity.getStartPos`); for an item frame or
    /// a painting, the block it is attached to (`BlockAttachedEntity.getPos`).
    pub fn start_block(&self) -> (i32, i32, i32) {
        self.start_block
    }

    /// The id of the entity this one is leashed to, if it is.
    pub fn leash_holder_id(&self) -> Option<EntityId> {
        (self.leash_holder_id != EntityId(0)).then_some(self.leash_holder_id)
    }

    /// Whether the entity is a player.
    pub fn is_player(&self) -> bool {
        is_vanilla_player_entity_type(self.type_id)
    }

    /// An `Avatar`: a player or a mannequin, which share the main hand and the model layers.
    pub fn is_avatar(&self) -> bool {
        self.is_player()
            || photon_data::entity_type::name(self.type_id) == Some("minecraft:mannequin")
    }
}

pub(crate) fn is_vanilla_player_entity_type(entity_type_id: EntityTypeId) -> bool {
    entity_type_id == VANILLA_PLAYER_ENTITY_TYPE_ID
}

pub(crate) fn is_vanilla_pushable_entity_type(entity_type_id: EntityTypeId) -> bool {
    is_vanilla_boat_entity_type(entity_type_id)
        || is_vanilla_shulker_entity_type(entity_type_id)
        || is_vanilla_minecart_entity_type(entity_type_id)
        || is_vanilla_player_entity_type(entity_type_id)
}

pub(crate) fn source_accepts_pushable_entities(entity_type_id: EntityTypeId) -> bool {
    is_vanilla_boat_entity_type(entity_type_id) || is_vanilla_minecart_entity_type(entity_type_id)
}
