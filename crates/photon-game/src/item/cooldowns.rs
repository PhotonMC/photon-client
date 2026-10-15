//! `ItemCooldowns`: the groups of items the player cannot use for a while,
//! as the server sets them (`ClientboundCooldownPacket`), counted down by the
//! player's ticks.

use std::collections::BTreeMap;

use photon_world::ClientItemStackState as Stack;

use crate::item_components::{Cursor, ids};

/// The player's item cooldowns.
#[derive(Debug, Clone, Default)]
pub struct ItemCooldowns {
    /// Each group's start and end, in the player's ticks.
    cooldowns: BTreeMap<String, (i32, i32)>,
    tick_count: i32,
}

impl ItemCooldowns {
    /// `addCooldown`: the group waits `time` ticks from now.
    pub fn add(&mut self, group: String, time: i32) {
        self.cooldowns
            .insert(group, (self.tick_count, self.tick_count + time));
    }

    /// `removeCooldown`.
    pub fn remove(&mut self, group: &str) {
        self.cooldowns.remove(group);
    }

    /// `tick`: a tick passes, and the cooldowns that ended go.
    pub fn tick(&mut self) {
        self.tick_count += 1;
        let now = self.tick_count;
        self.cooldowns.retain(|_, (_, end)| *end > now);
    }

    /// `getCooldownPercent` of a group: what is left of its cooldown, from
    /// one to none.
    pub fn percent(&self, group: &str, partial: f32) -> f32 {
        self.cooldowns.get(group).map_or(0.0, |&(start, end)| {
            let duration = (end - start) as f32;
            let remaining = end as f32 - (self.tick_count as f32 + partial);
            (remaining / duration).clamp(0.0, 1.0)
        })
    }

    /// `isOnCooldown`.
    pub fn is_on_cooldown(&self, stack: &Stack) -> bool {
        self.percent(&group(stack), 0.0) > 0.0
    }

    /// Every group's `getCooldownPercent` at this moment, for a frame.
    pub fn percents(&self, partial: f32) -> Vec<(String, f32)> {
        self.cooldowns
            .keys()
            .map(|group| (group.clone(), self.percent(group, partial)))
            .collect()
    }
}

/// `getCooldownGroup`: the stack's `use_cooldown` group, or its item's id.
pub fn group(stack: &Stack) -> String {
    let values = crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    let named = values.raw(ids::USE_COOLDOWN).and_then(|value| {
        // `UseCooldown.STREAM_CODEC`: the seconds, then an optional group.
        let mut cursor = Cursor { data: value };
        cursor.float().ok()?;
        if !cursor.boolean().ok()? {
            return None;
        }
        let length = cursor.count().ok()?;
        let name = cursor.data.get(..length)?;
        Some(namespaced(&String::from_utf8_lossy(name)))
    });
    named.unwrap_or_else(|| {
        usize::try_from(stack.item_registry_id().0)
            .ok()
            .and_then(|id| photon_data::item::registry::ITEMS.get(id))
            .map_or_else(String::new, |item| item.name.to_owned())
    })
}

/// An identifier as `Identifier.parse` reads it: the `minecraft` namespace
/// when it names none.
pub fn namespaced(name: &str) -> String {
    if name.contains(':') {
        name.to_owned()
    } else {
        format!("minecraft:{name}")
    }
}
