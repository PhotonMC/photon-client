use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

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
    /// The invisible entities the local player is allowed to see as ghosts (same team, which
    /// sees its friendly invisibles). Set by the session; the world does not read it.
    pub(crate) friendly_ghosts: BTreeSet<EntityId>,
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

    pub fn get(&self, entity_id: &EntityId) -> Option<EntityView<'_>> {
        self.view(*self.ids.get(entity_id)?)
    }

    fn view(&self, entity: Entity) -> Option<EntityView<'_>> {
        let (entity, rotation, pos, vel) = self
            .ecs
            .get_entity(entity)
            .ok()?
            .get_components::<(&ClientEntity, &Rotation, &Position, &Motion)>()
            .ok()?;
        Some(EntityView {
            entity,
            rotation,
            pos,
            vel,
        })
    }

    /// Every entity with its id, in id order.
    pub fn iter(&self) -> impl Iterator<Item = (&EntityId, EntityView<'_>)> {
        self.ids
            .iter()
            .filter_map(|(id, &entity)| Some((id, self.view(entity)?)))
    }

    pub fn values(&self) -> impl Iterator<Item = EntityView<'_>> {
        self.iter().map(|(_, entity)| entity)
    }

    pub fn keys(&self) -> impl Iterator<Item = &EntityId> {
        self.ids.keys()
    }

    /// The entity of an id, to change.
    pub fn at_mut(&mut self, entity_id: EntityId) -> Option<EntityViewMut<'_>> {
        let entity = *self.ids.get(&entity_id)?;
        Self::view_mut(&mut self.ecs, entity)
    }

    fn view_mut(ecs: &mut World, entity: Entity) -> Option<EntityViewMut<'_>> {
        let (entity, rotation, pos, vel) = ecs
            .get_entity_mut(entity)
            .ok()?
            .into_components_mut::<(&mut ClientEntity, &mut Rotation, &mut Position, &mut Motion)>()
            .ok()?;
        Some(EntityViewMut {
            entity: entity.into_inner(),
            rotation: rotation.into_inner(),
            pos: pos.into_inner(),
            vel: vel.into_inner(),
        })
    }

    /// Change every entity, in id order.
    pub(crate) fn for_each_mut(&mut self, mut change: impl FnMut(EntityViewMut<'_>)) {
        for &entity in self.ids.values() {
            if let Some(view) = Self::view_mut(&mut self.ecs, entity) {
                change(view);
            }
        }
    }

    /// Add an entity, replacing the one of its id; the one replaced.
    pub(crate) fn insert(
        &mut self,
        entity_id: EntityId,
        parts: EntityParts,
    ) -> Option<EntityParts> {
        if let Some(&existing) = self.ids.get(&entity_id) {
            let view = Self::view_mut(&mut self.ecs, existing)?;
            return Some(EntityParts {
                entity: std::mem::replace(view.entity, parts.entity),
                rotation: std::mem::replace(view.rotation, parts.rotation),
                pos: std::mem::replace(view.pos, parts.pos),
                vel: std::mem::replace(view.vel, parts.vel),
            });
        }
        let spawned = self
            .ecs
            .spawn((parts.entity, parts.rotation, parts.pos, parts.vel))
            .id();
        self.ids.insert(entity_id, spawned);
        None
    }

    /// Take an entity away, with its state.
    pub(crate) fn remove(&mut self, entity_id: EntityId) -> Option<EntityParts> {
        let entity = self.ids.remove(&entity_id)?;
        let mut taken = self.ecs.entity_mut(entity);
        let parts = taken.take::<(ClientEntity, Rotation, Position, Motion)>();
        self.ecs.despawn(entity);
        parts.map(|(entity, rotation, pos, vel)| EntityParts {
            entity,
            rotation,
            pos,
            vel,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.ids.clear();
        self.ecs.clear_entities();
    }

    /// Every entity, in id order.
    pub fn all(&self) -> impl Iterator<Item = EntityView<'_>> {
        self.values()
    }

    /// The entities the frame draws: every entity but those left out as out of view, and the
    /// invisible mobs (`LivingEntityRenderer` skips the body of one the player is not allowed to
    /// see). The families that keep what an invisible mob wears, holds or glows with use
    /// [`Self::drawn_with_invisible`], which also gives a spectator the ghost of its body. Players decide
    /// for themselves, and an armor stand is not a mob here: its renderer hides only the body.
    pub fn drawn(&self) -> impl Iterator<Item = EntityView<'_>> {
        self.drawn_with_invisible()
            .filter(|entity| !entity.is_invisible_mob())
    }

    /// [`Self::drawn`] with the invisible mobs, for the renderers that keep what such a mob wears
    /// and holds and leave only its body out.
    pub fn drawn_with_invisible(&self) -> impl Iterator<Item = EntityView<'_>> {
        // Both are in id order, so one pass over each finds the ids left out.
        let mut left_out = self.out_of_view.iter().copied().peekable();
        self.iter().filter_map(move |(&id, entity)| {
            while left_out.next_if(|&out| out < id).is_some() {}
            left_out.next_if_eq(&id).is_none().then_some(entity)
        })
    }

    /// Set which invisible entities the local player sees as ghosts (`Entity.isInvisibleTo`).
    pub fn set_friendly_ghosts(&mut self, ids: BTreeSet<EntityId>) {
        self.friendly_ghosts = ids;
    }

    /// Whether the local player sees the invisible entity as a ghost because of a team.
    pub fn is_friendly_ghost(&self, entity_id: EntityId) -> bool {
        self.friendly_ghosts.contains(&entity_id)
    }

    /// Set which entities the next frames leave out as out of view.
    pub fn set_out_of_view(&mut self, ids: BTreeSet<EntityId>) {
        self.out_of_view = ids;
    }
}
