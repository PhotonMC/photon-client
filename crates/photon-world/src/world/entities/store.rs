use bevy_ecs::entity::Entity;
use bevy_ecs::world::{Mut, World};

use crate::*;

/// The client's entities by id, and which of them lie in each section
/// (`ClientLevel.entityStorage`). Each entity is an entity of an ECS world, with its state as
/// components; `ids` is the index by network id, in id order, which is the order everything that
/// is observable (ticks, removals, sections, drawing) walks them in.
#[derive(Debug, Default)]
pub struct Entities {
    ecs: World,
    ids: BTreeMap<EntityId, Entity>,
    /// The ids of the entities in each section, as `EntitySectionStorage` indexes them.
    pub(crate) sections: BTreeMap<(i32, i32, i32), BTreeSet<EntityId>>,
    /// The entities the frame being drawn leaves out because they are wholly outside the view.
    /// Only the renderer sets it; nothing the world does reads it.
    pub(crate) out_of_view: BTreeSet<EntityId>,
}

impl Entities {
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn contains_key(&self, entity_id: &EntityId) -> bool {
        self.ids.contains_key(entity_id)
    }

    pub fn get(&self, entity_id: &EntityId) -> Option<&ClientEntity> {
        let entity = *self.ids.get(entity_id)?;
        self.ecs.get::<ClientEntity>(entity)
    }

    /// Every entity with its id, in id order.
    pub fn iter(&self) -> impl Iterator<Item = (&EntityId, &ClientEntity)> {
        self.ids
            .iter()
            .filter_map(|(id, &entity)| Some((id, self.ecs.get::<ClientEntity>(entity)?)))
    }

    pub fn values(&self) -> impl Iterator<Item = &ClientEntity> {
        self.iter().map(|(_, entity)| entity)
    }

    pub fn keys(&self) -> impl Iterator<Item = &EntityId> {
        self.ids.keys()
    }

    /// The entity of an id, to change.
    pub fn at_mut(&mut self, entity_id: EntityId) -> Option<&mut ClientEntity> {
        let entity = *self.ids.get(&entity_id)?;
        self.ecs
            .get_mut::<ClientEntity>(entity)
            .map(Mut::into_inner)
    }

    /// Change every entity, in id order.
    pub(crate) fn for_each_mut(&mut self, mut change: impl FnMut(&mut ClientEntity)) {
        for &entity in self.ids.values() {
            if let Some(entity) = self.ecs.get_mut::<ClientEntity>(entity) {
                change(entity.into_inner());
            }
        }
    }

    /// Add an entity, replacing the one of its id; the one replaced.
    pub(crate) fn insert(
        &mut self,
        entity_id: EntityId,
        entity: ClientEntity,
    ) -> Option<ClientEntity> {
        if let Some(&existing) = self.ids.get(&entity_id) {
            let slot = self.ecs.get_mut::<ClientEntity>(existing)?;
            return Some(std::mem::replace(slot.into_inner(), entity));
        }
        let spawned = self.ecs.spawn(entity).id();
        self.ids.insert(entity_id, spawned);
        None
    }

    /// Take an entity away, with its state.
    pub(crate) fn remove(&mut self, entity_id: EntityId) -> Option<ClientEntity> {
        let entity = self.ids.remove(&entity_id)?;
        let removed = self.ecs.entity_mut(entity).take::<ClientEntity>();
        self.ecs.despawn(entity);
        removed
    }

    pub(crate) fn clear(&mut self) {
        self.ids.clear();
        self.ecs.clear_entities();
    }

    /// Every entity, in id order.
    pub fn all(&self) -> impl Iterator<Item = &ClientEntity> {
        self.values()
    }

    /// The entities the frame draws: every entity but those left out as out of view, and the
    /// invisible mobs (`LivingEntityRenderer` skips the body of one the player is not allowed to
    /// see; what such a mob wears is skipped with it here, and a spectator's ghostly view of it
    /// is not drawn). Armor stands and players decide for themselves.
    pub fn drawn(&self) -> impl Iterator<Item = &ClientEntity> {
        // Both are in id order, so one pass over each finds the ids left out.
        let mut left_out = self.out_of_view.iter().copied().peekable();
        self.iter().filter_map(move |(&id, entity)| {
            while left_out.next_if(|&out| out < id).is_some() {}
            (left_out.next_if_eq(&id).is_none() && !entity.is_invisible_mob()).then_some(entity)
        })
    }

    /// Set which entities the next frames leave out as out of view.
    pub fn set_out_of_view(&mut self, ids: BTreeSet<EntityId>) {
        self.out_of_view = ids;
    }
}
