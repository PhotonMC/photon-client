//! Doors, trap doors, fence gates and buttons used with an empty hand: the state they take and
//! the sound the server leaves the user out of (`Level.playSound(player, ..)`), which the
//! client's own use plays.

use super::*;

/// What `useWithoutItem` does to an openable block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenUse {
    /// `InteractionResult.PASS`: the block cannot be opened by hand.
    Pass,
    /// `InteractionResult.CONSUME`: a pressed button does nothing.
    Consume,
    /// `InteractionResult.SUCCESS`: the block is set, then the sound plays.
    Done(Opened),
}

/// The change an openable block makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opened {
    pub write: Write,
    pub sound: &'static str,
    /// Whether the pitch is `nextFloat() * 0.1 + 0.9` (doors, trap doors, fence gates) or 1.
    pub random_pitch: bool,
}

/// The sound set of a block (`BlockSetType` and `WoodType`): door, trap door, button and fence
/// gate events. Open comes before close.
struct Family {
    door: [&'static str; 2],
    trapdoor: [&'static str; 2],
    button_on: &'static str,
    gate: [&'static str; 2],
}

macro_rules! wood {
    ($name:literal) => {
        Family {
            door: [
                concat!("minecraft:block.", $name, "_door.open"),
                concat!("minecraft:block.", $name, "_door.close"),
            ],
            trapdoor: [
                concat!("minecraft:block.", $name, "_trapdoor.open"),
                concat!("minecraft:block.", $name, "_trapdoor.close"),
            ],
            button_on: concat!("minecraft:block.", $name, "_button.click_on"),
            gate: [
                concat!("minecraft:block.", $name, "_fence_gate.open"),
                concat!("minecraft:block.", $name, "_fence_gate.close"),
            ],
        }
    };
}

const WOODEN: Family = Family {
    door: [
        "minecraft:block.wooden_door.open",
        "minecraft:block.wooden_door.close",
    ],
    trapdoor: [
        "minecraft:block.wooden_trapdoor.open",
        "minecraft:block.wooden_trapdoor.close",
    ],
    button_on: "minecraft:block.wooden_button.click_on",
    gate: [
        "minecraft:block.fence_gate.open",
        "minecraft:block.fence_gate.close",
    ],
};
const BAMBOO: Family = wood!("bamboo_wood");
const CHERRY: Family = wood!("cherry_wood");
const NETHER: Family = wood!("nether_wood");
const IRON: Family = Family {
    door: [
        "minecraft:block.iron_door.open",
        "minecraft:block.iron_door.close",
    ],
    trapdoor: [
        "minecraft:block.iron_trapdoor.open",
        "minecraft:block.iron_trapdoor.close",
    ],
    button_on: "minecraft:block.stone_button.click_on",
    gate: ["", ""],
};
const COPPER: Family = Family {
    door: [
        "minecraft:block.copper_door.open",
        "minecraft:block.copper_door.close",
    ],
    trapdoor: [
        "minecraft:block.copper_trapdoor.open",
        "minecraft:block.copper_trapdoor.close",
    ],
    button_on: "minecraft:block.stone_button.click_on",
    gate: ["", ""],
};
const STONE: Family = Family {
    door: IRON.door,
    trapdoor: IRON.trapdoor,
    ..IRON
};

/// The family of a block by its name: the iron and copper doors and the stone buttons have their
/// own, the bamboo, cherry and nether woods theirs, every other wood the plain one.
fn family(name: &str) -> (&'static Family, bool) {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    if name.starts_with("iron_") {
        // Neither the iron door nor the iron trap door opens by hand.
        (&IRON, false)
    } else if name.contains("copper") {
        (&COPPER, true)
    } else if name == "stone_button" || name == "polished_blackstone_button" {
        (&STONE, true)
    } else if name.starts_with("bamboo_") {
        (&BAMBOO, true)
    } else if name.starts_with("cherry_") {
        (&CHERRY, true)
    } else if name.starts_with("crimson_") || name.starts_with("warped_") {
        (&NETHER, true)
    } else {
        (&WOODEN, true)
    }
}

/// A set block that plays an event, its pitch random or 1.
fn done(write: Write, sound: &'static str, random_pitch: bool) -> Rule<OpenUse> {
    Rule::Known(OpenUse::Done(Opened {
        write,
        sound,
        random_pitch,
    }))
}

/// The open door, trap door or fence gate in the state: `state.cycle(OPEN)`.
fn toggled(state: BlockStateId) -> Option<BlockStateId> {
    let open = property(state, "open") == Some("true");
    with_property(state, "open", flag(!open))
}

/// The event of closing or opening, by the state it starts from.
fn sound_of(state: BlockStateId, events: [&'static str; 2]) -> &'static str {
    events[usize::from(property(state, "open") == Some("true"))]
}

/// `FenceGateBlock.useWithoutItem`: a closed gate facing the player turns to face away.
fn gate_use(
    state: BlockStateId,
    pos: (i32, i32, i32),
    facing: Direction,
    family: &Family,
) -> Rule<OpenUse> {
    let mut next = state;
    if property(state, "open") != Some("true")
        && property(state, "facing") == Some(facing.opposite().name())
        && let Some(turned) = with_property(state, "facing", facing.name())
    {
        next = turned;
    }
    toggled(next).map_or(Rule::Unported, |open| {
        let write = Write {
            pos,
            state: open,
            flags: 10,
        };
        done(write, sound_of(state, family.gate), true)
    })
}

/// `ButtonBlock.useWithoutItem`: a pressed button does nothing, else it is pressed (flags 3).
fn button_use(state: BlockStateId, pos: (i32, i32, i32), family: &Family) -> Rule<OpenUse> {
    if property(state, "powered") == Some("true") {
        return Rule::Known(OpenUse::Consume);
    }
    with_property(state, "powered", "true").map_or(Rule::Unported, |pressed| {
        let write = Write {
            pos,
            state: pressed,
            flags: 3,
        };
        done(write, family.button_on, false)
    })
}

/// `DoorBlock` (flags 10, the other half follows by its shape update) and `TrapDoorBlock`
/// (flags 2) `.useWithoutItem`.
fn door_use(
    state: BlockStateId,
    pos: (i32, i32, i32),
    (door, events): (bool, [&'static str; 2]),
) -> Rule<OpenUse> {
    toggled(state).map_or(Rule::Unported, |open| {
        let flags = if door { 10 } else { 2 };
        done(
            Write {
                pos,
                state: open,
                flags,
            },
            sound_of(state, events),
            true,
        )
    })
}

/// `DoorBlock`, `TrapDoorBlock`, `FenceGateBlock` and `ButtonBlock.useWithoutItem` at `pos`,
/// for a player facing `facing`. `Unported` for any other class.
pub fn use_openable(state: BlockStateId, pos: (i32, i32, i32), facing: Direction) -> Rule<OpenUse> {
    let (Some(class), Some(block)) = (state_owner(state, Method::UseWithoutItem), block(state))
    else {
        return Rule::Unported;
    };
    let (family, by_hand) = family(block.name);
    match class {
        BlockClass::DoorBlock | BlockClass::TrapDoorBlock if !by_hand => Rule::Known(OpenUse::Pass),
        BlockClass::DoorBlock => door_use(state, pos, (true, family.door)),
        BlockClass::TrapDoorBlock => door_use(state, pos, (false, family.trapdoor)),
        BlockClass::FenceGateBlock => gate_use(state, pos, facing, family),
        BlockClass::ButtonBlock => button_use(state, pos, family),
        _ => Rule::Unported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every event of the table is a sound the registry has.
    #[test]
    fn every_family_sound_is_registered() {
        for family in [&WOODEN, &BAMBOO, &CHERRY, &NETHER, &IRON, &COPPER, &STONE] {
            let gate_exists = !family.gate[0].is_empty();
            let events = family
                .door
                .into_iter()
                .chain(family.trapdoor)
                .chain([family.button_on])
                .chain(if gate_exists {
                    family.gate.to_vec()
                } else {
                    Vec::new()
                });
            for event in events {
                assert!(photon_data::sound_event::id(event).is_some(), "{event}");
            }
        }
    }
}
