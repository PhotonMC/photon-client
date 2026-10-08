use crate::*;

/// The client's entities by id, and which of them lie in each section
/// (`ClientLevel.entityStorage`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entities {
    pub(crate) by_id: BTreeMap<i32, ClientEntity>,
    /// The ids of the entities in each section, as `EntitySectionStorage` indexes them.
    pub(crate) sections: BTreeMap<(i32, i32, i32), BTreeSet<i32>>,
    /// The entities the frame being drawn leaves out because they are wholly outside the view.
    /// Only the renderer sets it; nothing the world does reads it.
    pub(crate) out_of_view: BTreeSet<i32>,
}

/// Reads of the map go through; changes go through the methods and `by_id`, which keep the
/// section index in step.
impl std::ops::Deref for Entities {
    type Target = BTreeMap<i32, ClientEntity>;

    fn deref(&self) -> &Self::Target {
        &self.by_id
    }
}

impl Entities {
    /// The entity of an id, to change.
    pub fn at_mut(&mut self, entity_id: i32) -> Option<&mut ClientEntity> {
        self.by_id.get_mut(&entity_id)
    }

    /// Every entity, in id order.
    pub fn all(&self) -> impl Iterator<Item = &ClientEntity> {
        self.by_id.values()
    }

    /// The entities the frame draws: every entity but those left out as out of view.
    pub fn drawn(&self) -> impl Iterator<Item = &ClientEntity> {
        // Both are in id order, so one pass over each finds the ids left out.
        let mut left_out = self.out_of_view.iter().copied().peekable();
        self.by_id.iter().filter_map(move |(&id, entity)| {
            while left_out.next_if(|&out| out < id).is_some() {}
            left_out.next_if_eq(&id).is_none().then_some(entity)
        })
    }

    /// Set which entities the next frames leave out as out of view.
    pub fn set_out_of_view(&mut self, ids: BTreeSet<i32>) {
        self.out_of_view = ids;
    }
}
