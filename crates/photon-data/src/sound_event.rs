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

/// The full name of the vanilla event `short` (`entity.armadillo.peek`), resolved while
/// compiling: a name the registry lacks stops the build.
pub const fn vanilla(short: &str) -> &'static str {
    let mut index = 0;
    while index < SOUND_EVENTS.len() {
        if crate::name::is_vanilla(SOUND_EVENTS[index].name, short) {
            return SOUND_EVENTS[index].name;
        }
        index += 1;
    }
    panic!("the sound event registry has no such name");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_name_resolves_to_the_full_one() {
        const PEEK: &str = vanilla("entity.armadillo.peek");
        assert_eq!(PEEK, "minecraft:entity.armadillo.peek");
        assert!(id(PEEK).is_some());
    }
}
