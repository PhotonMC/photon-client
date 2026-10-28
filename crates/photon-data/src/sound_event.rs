//! The sound event registry: each event's name, and its id (its place in the list, which is
//! what the sound packets carry).

use crate::tables::sound_events::SOUND_EVENTS;

/// The name of the event with this id (`minecraft:block.stone.break`).
pub fn name(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| SOUND_EVENTS.get(index))
        .map(|entry| entry.name)
}

/// Every event's name, in id order.
pub fn names() -> impl Iterator<Item = &'static str> {
    crate::named::names(&SOUND_EVENTS)
}

/// The id of the event with this name.
pub fn id(name: &str) -> Option<i32> {
    SOUND_EVENTS
        .iter()
        .position(|candidate| candidate.name == name)
        .map(|index| index as i32)
}
