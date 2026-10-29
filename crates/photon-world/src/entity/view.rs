//! An entity as the world hands it out: its `ClientEntity` and the components split out of it.
//! A view dereferences to the `ClientEntity`, so what has not moved into a component is read the
//! way it always was; a method that needs a split-out component lives on the view.
//!
//! The components are listed once, in `entity_components!` below; it makes the two views, the
//! bundle an entity is added and taken as, and the queries that read the views from the world.

use super::*;
use bevy_ecs::query::QueryData;

macro_rules! entity_components {
    ($($field:ident: $ty:ty),* $(,)?) => {
        /// One entity, to read.
        #[derive(Clone, Copy)]
        pub struct EntityView<'a> {
            $(pub(crate) $field: &'a $ty,)*
        }

        /// One entity, to change.
        pub struct EntityViewMut<'a> {
            $(pub(crate) $field: &'a mut $ty,)*
        }

        /// An entity taken apart: its `ClientEntity` and the components split out of it. What is
        /// built when an entity is added and what is left when one is taken away.
        #[derive(Debug, Clone, PartialEq, Eq, bevy_ecs::bundle::Bundle)]
        pub struct EntityParts {
            $(pub(crate) $field: $ty,)*
        }

        /// What the world reads to make an [`EntityView`].
        #[derive(QueryData)]
        pub(crate) struct ReadView {
            $($field: &'static $ty,)*
        }

        /// What the world reads to make an [`EntityViewMut`].
        #[derive(QueryData)]
        #[query_data(mutable)]
        pub(crate) struct WriteView {
            $($field: &'static mut $ty,)*
        }

        impl<'a> EntityView<'a> {
            pub(crate) fn from_query(item: ReadViewItem<'a, 'static>) -> Self {
                Self { $($field: item.$field,)* }
            }
        }

        impl<'a> EntityViewMut<'a> {
            pub(crate) fn from_query(item: WriteViewItem<'a, 'static>) -> Self {
                Self { $($field: item.$field.into_inner(),)* }
            }

            /// The same entity, to read.
            pub fn as_view(&self) -> EntityView<'_> {
                EntityView { $($field: &*self.$field,)* }
            }

            /// Puts the parts in place of what the entity has, and gives back what was there.
            pub(crate) fn replace_parts(self, parts: EntityParts) -> EntityParts {
                EntityParts { $($field: std::mem::replace(self.$field, parts.$field),)* }
            }
        }

        impl EntityParts {
            pub fn view(&self) -> EntityView<'_> {
                EntityView { $($field: &self.$field,)* }
            }

            pub fn view_mut(&mut self) -> EntityViewMut<'_> {
                EntityViewMut { $($field: &mut self.$field,)* }
            }
        }
    };
}

entity_components! {
    entity: ClientEntity,
    rotation: Rotation,
    pos: Position,
    vel: Motion,
    lerp: Interpolation,
    kind: crate::entity_kind::EntityKind,
    worn: Equipment,
    living: EntityState,
    item: DroppedItem,
    ride: Riding,
    links: Links,
    vitals: Vitals,
    status: EntityStatus,
    anim: BodyAnimation,
    net: NetworkId,
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
