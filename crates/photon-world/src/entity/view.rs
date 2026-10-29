//! An entity as the world hands it out: its `ClientEntity` and the components split out of it.
//! A view dereferences to the `ClientEntity`, so what has not moved into a component is read the
//! way it always was; a method that needs a split-out component lives on the view.

use super::*;

/// One entity, to read.
#[derive(Clone, Copy)]
pub struct EntityView<'a> {
    pub(crate) entity: &'a ClientEntity,
    pub(crate) rotation: &'a Rotation,
    pub(crate) pos: &'a Position,
    pub(crate) vel: &'a Motion,
    pub(crate) lerp: &'a Interpolation,
    pub(crate) kind: &'a crate::entity_kind::EntityKind,
    pub(crate) worn: &'a Equipment,
    pub(crate) living: &'a EntityState,
    pub(crate) item: &'a DroppedItem,
    pub(crate) ride: &'a Riding,
}

/// One entity, to change.
pub struct EntityViewMut<'a> {
    pub(crate) entity: &'a mut ClientEntity,
    pub(crate) rotation: &'a mut Rotation,
    pub(crate) pos: &'a mut Position,
    pub(crate) vel: &'a mut Motion,
    pub(crate) lerp: &'a mut Interpolation,
    pub(crate) kind: &'a mut crate::entity_kind::EntityKind,
    pub(crate) worn: &'a mut Equipment,
    pub(crate) living: &'a mut EntityState,
    pub(crate) item: &'a mut DroppedItem,
    pub(crate) ride: &'a mut Riding,
}

impl<'a> EntityView<'a> {
    /// The `ClientEntity` with the view's own lifetime, for a borrow that outlives the view.
    pub fn client(self) -> &'a ClientEntity {
        self.entity
    }
}

impl std::ops::Deref for EntityView<'_> {
    type Target = ClientEntity;

    fn deref(&self) -> &ClientEntity {
        self.entity
    }
}

impl std::ops::Deref for EntityViewMut<'_> {
    type Target = ClientEntity;

    fn deref(&self) -> &ClientEntity {
        self.entity
    }
}

impl std::ops::DerefMut for EntityViewMut<'_> {
    fn deref_mut(&mut self) -> &mut ClientEntity {
        self.entity
    }
}

impl<'a> EntityViewMut<'a> {
    /// The `ClientEntity` alone, for as long as the entity is borrowed.
    pub fn into_entity(self) -> &'a mut ClientEntity {
        self.entity
    }

    /// The kind's state alone, for as long as the entity is borrowed.
    pub(crate) fn into_kind(self) -> &'a mut crate::entity_kind::EntityKind {
        self.kind
    }

    /// The same entity, to read.
    pub fn as_view(&self) -> EntityView<'_> {
        EntityView {
            entity: self.entity,
            rotation: self.rotation,
            pos: self.pos,
            vel: self.vel,
            lerp: self.lerp,
            kind: self.kind,
            worn: self.worn,
            living: self.living,
            item: self.item,
            ride: self.ride,
        }
    }
}

/// An entity taken apart: its `ClientEntity` and the components split out of it. What is built
/// when an entity is added and what is left when one is taken away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityParts {
    pub(crate) entity: ClientEntity,
    pub(crate) rotation: Rotation,
    pub(crate) pos: Position,
    pub(crate) vel: Motion,
    pub(crate) lerp: Interpolation,
    pub(crate) kind: crate::entity_kind::EntityKind,
    pub(crate) worn: Equipment,
    pub(crate) living: EntityState,
    pub(crate) item: DroppedItem,
    pub(crate) ride: Riding,
}

impl EntityParts {
    pub fn view(&self) -> EntityView<'_> {
        EntityView {
            entity: &self.entity,
            rotation: &self.rotation,
            pos: &self.pos,
            vel: &self.vel,
            lerp: &self.lerp,
            kind: &self.kind,
            worn: &self.worn,
            living: &self.living,
            item: &self.item,
            ride: &self.ride,
        }
    }

    pub fn view_mut(&mut self) -> EntityViewMut<'_> {
        EntityViewMut {
            entity: &mut self.entity,
            rotation: &mut self.rotation,
            pos: &mut self.pos,
            vel: &mut self.vel,
            lerp: &mut self.lerp,
            kind: &mut self.kind,
            worn: &mut self.worn,
            living: &mut self.living,
            item: &mut self.item,
            ride: &mut self.ride,
        }
    }
}

impl std::ops::Deref for EntityParts {
    type Target = ClientEntity;

    fn deref(&self) -> &ClientEntity {
        &self.entity
    }
}

impl std::ops::DerefMut for EntityParts {
    fn deref_mut(&mut self) -> &mut ClientEntity {
        &mut self.entity
    }
}

/// What both views read the same way, whichever of them holds the entity.
macro_rules! shared_reads {
    ($($view:ident),*) => {$(
        impl $view<'_> {
            pub fn position(&self) -> (f64, f64, f64) {
                self.pos.xyz()
            }

            pub fn previous_position(&self) -> (f64, f64, f64) {
                self.pos.previous_xyz()
            }

            pub fn motion(&self) -> (f64, f64, f64) {
                self.vel.xyz()
            }
        }
    )*};
}

shared_reads!(EntityView, EntityViewMut);

impl EntityViewMut<'_> {
    /// What the read view says, for the callers that hold the mutable one.
    pub(crate) fn dimensions(&self) -> ClientEntityDimensions {
        self.as_view().dimensions()
    }

    pub(crate) fn interpolation_length(&self) -> i32 {
        self.as_view().interpolation_length()
    }

    pub(crate) fn has_interpolation_handler(&self) -> bool {
        self.as_view().has_interpolation_handler()
    }

    pub(crate) fn minecart_track(&self) -> Option<&crate::objects::minecart_track::TrackLerp> {
        self.kind
            .minecart_slot()
            .and_then(|slot| slot.track.as_ref())
    }
}
