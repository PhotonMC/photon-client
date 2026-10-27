//! `Level.getEntities(entity, box)` as a spinning player's touch reads it.

use std::collections::BTreeSet;

use photon_core::aabb::Aabb;

use crate::ClientWorld;
use crate::entity::entity_section_coord;

/// What a box sweeps into among the level's entities (spectators left out).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Touch {
    Nothing,
    /// Entities, none of them living.
    Lifeless,
    Living,
}

impl ClientWorld {
    /// The entities whose boxes meet `area` (the local player is not among them).
    pub(crate) fn touch(&self, area: Aabb) -> Touch {
        let range = |low: f64, high: f64, below: f64, above: f64| {
            entity_section_coord(low - below)..=entity_section_coord(high + above)
        };
        let mut ids = BTreeSet::new();
        for x in range(area.min_x, area.max_x, 2.0, 2.0) {
            for y in range(area.min_y, area.max_y, 4.0, 0.0) {
                for z in range(area.min_z, area.max_z, 2.0, 2.0) {
                    if let Some(found) = self.entities.sections.get(&(x, y, z)) {
                        ids.extend(found.iter().copied());
                    }
                }
            }
        }
        let mut met = ids
            .into_iter()
            .filter_map(|id| self.entities.at(id))
            .filter(|entity| !entity.is_spectator() && entity.bounding_box().intersects(area))
            .peekable();
        if met.peek().is_none() {
            Touch::Nothing
        } else if met.any(|entity| entity.is_living_entity()) {
            Touch::Living
        } else {
            Touch::Lifeless
        }
    }
}
