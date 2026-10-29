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
}

/// One entity, to change.
pub struct EntityViewMut<'a> {
    pub(crate) entity: &'a mut ClientEntity,
    pub(crate) rotation: &'a mut Rotation,
    pub(crate) pos: &'a mut Position,
    pub(crate) vel: &'a mut Motion,
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

    /// The same entity, to read.
    pub fn as_view(&self) -> EntityView<'_> {
        EntityView {
            entity: self.entity,
            rotation: self.rotation,
            pos: self.pos,
            vel: self.vel,
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
}

impl EntityParts {
    pub fn view(&self) -> EntityView<'_> {
        EntityView {
            entity: &self.entity,
            rotation: &self.rotation,
            pos: &self.pos,
            vel: &self.vel,
        }
    }

    pub fn view_mut(&mut self) -> EntityViewMut<'_> {
        EntityViewMut {
            entity: &mut self.entity,
            rotation: &mut self.rotation,
            pos: &mut self.pos,
            vel: &mut self.vel,
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
