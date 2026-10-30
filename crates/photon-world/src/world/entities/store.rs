use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryState;
use bevy_ecs::world::World;

use crate::*;

/// The client's entities by id, and which of them lie in each section
/// (`ClientLevel.entityStorage`). Each entity is an entity of an ECS world, with its state as
/// components; `ids` is the index by network id, in id order, which is the order everything that
/// is observable (ticks, removals, sections, drawing) walks them in.
#[derive(Debug)]
pub struct Entities {
    ecs: World,
    /// The queries that read and change an entity's components, made once: asking the world
    /// for the components of one entity builds the query again every time, which is
    /// the cost of a view for each entity of a frame. Every entity has the same components, so
    /// they are in one archetype; the states are brought up to date wherever one is added.
    read: QueryState<ReadView>,
    write: QueryState<WriteView>,
    ids: BTreeMap<EntityId, Entity>,
    /// The entities of `ids` that are not in `out_of_view`, in id order: what every renderer of
    /// a frame walks, kept beside the map so each of them does not walk all the entities again.
    visible: Vec<(EntityId, Entity)>,
    /// The ids of the entities in each section, as `EntitySectionStorage` indexes them.
    pub(crate) sections: BTreeMap<(i32, i32, i32), BTreeSet<EntityId>>,
    /// The entities the frame being drawn leaves out because they are wholly outside the view.
    /// Only the renderer sets it; nothing the world does reads it.
    pub(crate) out_of_view: BTreeSet<EntityId>,
    /// The invisible entities the local player is allowed to see as ghosts (same team, which
    /// sees its friendly invisibles). Set by the session; the world does not read it.
    pub(crate) friendly_ghosts: BTreeSet<EntityId>,
}

impl Default for Entities {
    fn default() -> Self {
        let mut ecs = World::new();
        let read = QueryState::new(&mut ecs);
        let write = QueryState::new(&mut ecs);
        Self {
            ecs,
            read,
            write,
            ids: BTreeMap::new(),
            visible: Vec::new(),
            sections: BTreeMap::new(),
            out_of_view: BTreeSet::new(),
            friendly_ghosts: BTreeSet::new(),
        }
    }
}

impl Entities {
    /// Entities with the level's resources already in their world.
    pub(crate) fn with_resources(level: Level, weather: Weather, chunks: Chunks) -> Self {
        let mut entities = Self::default();
        entities.ecs.insert_resource(level);
        entities.ecs.insert_resource(weather);
        entities.ecs.insert_resource(chunks);
        entities
    }

    /// The chunks, borrowed alone so the rest of the world stays free to change beside them.
    pub(crate) fn chunks_mut(&mut self) -> &mut Chunks {
        self.resource_mut()
    }

    pub(crate) fn resource<R: bevy_ecs::prelude::Resource>(&self) -> &R {
        self.ecs.resource::<R>()
    }

    pub(crate) fn resource_mut<
        R: bevy_ecs::prelude::Resource<Mutability = bevy_ecs::component::Mutable>,
    >(
        &mut self,
    ) -> &mut R {
        self.ecs.resource_mut::<R>().into_inner()
    }

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
        let item = self.read.get_manual(&self.ecs, entity).ok()?;
        Some(EntityView::from_query(item))
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
        Self::view_mut(&mut self.ecs, &mut self.write, entity)
    }

    fn view_mut<'w>(
        ecs: &'w mut World,
        state: &'w mut QueryState<WriteView>,
        entity: Entity,
    ) -> Option<EntityViewMut<'w>> {
        let item = state.get_mut(ecs, entity).ok()?;
        Some(EntityViewMut::from_query(item))
    }

    /// Bring the cached queries up to date with the archetypes the world has now.
    fn refresh_queries(&mut self) {
        self.read.update_archetypes(&self.ecs);
        self.write.update_archetypes(&self.ecs);
    }

    /// Change every entity, in id order.
    pub(crate) fn for_each_mut(&mut self, mut change: impl FnMut(EntityViewMut<'_>)) {
        for &entity in self.ids.values() {
            if let Some(view) = Self::view_mut(&mut self.ecs, &mut self.write, entity) {
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
            let view = Self::view_mut(&mut self.ecs, &mut self.write, existing)?;
            return Some(view.replace_parts(parts));
        }
        let spawned = self.ecs.spawn(parts).id();
        self.ids.insert(entity_id, spawned);
        if !self.out_of_view.contains(&entity_id) {
            let at = self.visible.partition_point(|&(id, _)| id < entity_id);
            self.visible.insert(at, (entity_id, spawned));
        }
        self.refresh_queries();
        None
    }

    /// Take an entity away, with its state.
    pub(crate) fn remove(&mut self, entity_id: EntityId) -> Option<EntityParts> {
        let entity = self.ids.remove(&entity_id)?;
        if let Ok(at) = self.visible.binary_search_by_key(&entity_id, |&(id, _)| id) {
            self.visible.remove(at);
        }
        let mut taken = self.ecs.entity_mut(entity);
        let parts = taken.take::<EntityParts>();
        self.ecs.despawn(entity);
        self.refresh_queries();
        parts
    }

    pub(crate) fn clear(&mut self) {
        // The world's resources are entities too: only the client's entities go.
        self.visible.clear();
        for (_, entity) in std::mem::take(&mut self.ids) {
            self.ecs.despawn(entity);
        }
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
        self.visible
            .iter()
            .filter_map(|&(_, entity)| self.view(entity))
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
        self.visible = self
            .ids
            .iter()
            .filter(|(id, _)| !ids.contains(id))
            .map(|(&id, &entity)| (id, entity))
            .collect();
        self.out_of_view = ids;
    }
}
