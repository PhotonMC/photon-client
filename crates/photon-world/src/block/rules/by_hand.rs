//! Blocks used with an empty hand (`BlockBehaviour.useWithoutItem`) as the client runs them: the
//! result the client sees, and for the doors, gates, buttons and the like the state they take
//! and the sound the server leaves the user out of (`Level.playSound(player, ..)`).
//!
//! Most blocks do their work on the server only (`!level.isClientSide()`) and answer the client
//! with a plain success; they are listed here so that the client no longer treats them as
//! unwritten.

use super::*;

/// What `useWithoutItem` does on the client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandUse {
    /// The result, with the level untouched.
    Result(UseResult),
    /// The block is set and the sound plays; the use succeeds with a swing.
    Set(Opened),
}

/// The change a block used by hand makes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opened {
    pub write: Write,
    pub sound: Option<HandSound>,
    /// The player eats: the food level takes `FoodData.eat(2, 0.1)`.
    pub eats: bool,
    /// The item the player takes into the inventory (`Player.addItem`).
    pub gives: Option<&'static str>,
}

/// The sound a use plays at the middle of the block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandSound {
    pub event: &'static str,
    pub volume: f32,
    /// The pitch, or `None` for `nextFloat() * 0.1 + 0.9` (doors, trap doors, fence gates).
    pub pitch: Option<f32>,
}

/// Who uses the block and where they hit it.
#[derive(Debug, Clone, Copy)]
pub struct User {
    pub facing: Direction,
    pub may_build: bool,
    /// The face clicked.
    pub face: Direction,
    /// The height of the click in the block, 0 to 1.
    pub click_y: f64,
    /// Where the click landed.
    pub location: [f64; 3],
    /// `Player.canEat(false)`.
    pub can_eat: bool,
    /// The main hand holds no stack.
    pub main_hand_empty: bool,
    /// The block has a block entity in the client's level.
    pub block_entity: bool,
}

/// A use in progress.
struct Hand {
    state: BlockStateId,
    pos: (i32, i32, i32),
    user: User,
}

const SUCCESS: UseResult = UseResult::Success { swing: true };
const SERVER: UseResult = UseResult::Success { swing: false };

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

/// A set block that plays an event at full volume, its pitch random or given.
fn done(write: Write, sound: &'static str, pitch: Option<f32>) -> Rule<HandUse> {
    Rule::Known(HandUse::Set(Opened {
        write,
        sound: Some(HandSound {
            event: sound,
            volume: 1.0,
            pitch,
        }),
        eats: false,
        gives: None,
    }))
}

fn result(result: UseResult) -> Rule<HandUse> {
    Rule::Known(HandUse::Result(result))
}

/// The family and whether the block opens by hand, by the block's name.
fn family_of(hand: &Hand) -> Option<(&'static Family, bool)> {
    block(hand.state).map(|block| family(block.name))
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

/// `DoorBlock` (flags 10, the other half follows by its shape update) and `TrapDoorBlock`
/// (flags 2) `.useWithoutItem`.
fn open_use(hand: &Hand, door: bool) -> Rule<HandUse> {
    let Some((family, by_hand)) = family_of(hand) else {
        return Rule::Unported;
    };
    if !by_hand {
        return result(UseResult::Pass);
    }
    let events = if door { family.door } else { family.trapdoor };
    toggled(hand.state).map_or(Rule::Unported, |open| {
        let flags = if door { 10 } else { 2 };
        let write = Write {
            pos: hand.pos,
            state: open,
            flags,
        };
        done(write, sound_of(hand.state, events), None)
    })
}

fn door_use(hand: &Hand) -> Rule<HandUse> {
    open_use(hand, true)
}

fn trapdoor_use(hand: &Hand) -> Rule<HandUse> {
    open_use(hand, false)
}

/// `FenceGateBlock.useWithoutItem`: a closed gate facing the player turns to face away.
fn gate_use(hand: &Hand) -> Rule<HandUse> {
    let (state, facing) = (hand.state, hand.user.facing);
    let Some((family, _)) = family_of(hand) else {
        return Rule::Unported;
    };
    let mut next = state;
    if property(state, "open") != Some("true")
        && property(state, "facing") == Some(facing.opposite().name())
        && let Some(turned) = with_property(state, "facing", facing.name())
    {
        next = turned;
    }
    toggled(next).map_or(Rule::Unported, |open| {
        let write = Write {
            pos: hand.pos,
            state: open,
            flags: 10,
        };
        done(write, sound_of(state, family.gate), None)
    })
}

/// `ButtonBlock.useWithoutItem`: a pressed button does nothing, else it is pressed (flags 3).
fn button_use(hand: &Hand) -> Rule<HandUse> {
    let Some((family, _)) = family_of(hand) else {
        return Rule::Unported;
    };
    if property(hand.state, "powered") == Some("true") {
        return result(UseResult::Success { swing: false });
    }
    with_property(hand.state, "powered", "true").map_or(Rule::Unported, |pressed| {
        let write = Write {
            pos: hand.pos,
            state: pressed,
            flags: 3,
        };
        done(write, family.button_on, Some(1.0))
    })
}

/// `ComparatorBlock.useWithoutItem`: a player who may build changes its mode (flags 2). The
/// output refresh that follows is the block entity's, which the client does not predict.
fn comparator_use(hand: &Hand) -> Rule<HandUse> {
    if !hand.user.may_build {
        return result(UseResult::Pass);
    }
    let to_subtract = property(hand.state, "mode") == Some("compare");
    let mode = if to_subtract { "subtract" } else { "compare" };
    with_property(hand.state, "mode", mode).map_or(Rule::Unported, |next| {
        Rule::Known(HandUse::Set(Opened {
            write: Write {
                pos: hand.pos,
                state: next,
                flags: 2,
            },
            sound: Some(HandSound {
                event: "minecraft:block.comparator.click",
                volume: 0.3,
                pitch: Some(if to_subtract { 0.55 } else { 0.5 }),
            }),
            eats: false,
            gives: None,
        }))
    })
}

/// `RepeaterBlock.useWithoutItem`: a player who may build raises the delay, wrapping (flags 3).
fn repeater_use(hand: &Hand) -> Rule<HandUse> {
    if !hand.user.may_build {
        return result(UseResult::Pass);
    }
    let delay: i32 = property(hand.state, "delay")
        .and_then(|delay| delay.parse().ok())
        .unwrap_or(1);
    with_property(hand.state, "delay", &(delay % 4 + 1).to_string()).map_or(
        Rule::Unported,
        |next| {
            Rule::Known(HandUse::Set(Opened {
                write: Write {
                    pos: hand.pos,
                    state: next,
                    flags: 3,
                },
                sound: None,
                eats: false,
                gives: None,
            }))
        },
    )
}

/// `BellBlock.useWithoutItem`: a hit on the right side counts, though only the server rings.
fn bell_use(hand: &Hand) -> Rule<HandUse> {
    let (state, user) = (hand.state, hand.user);
    let facing = property(state, "facing").and_then(Direction::from_name);
    let (Some(facing), Some(attachment)) = (facing, property(state, "attachment")) else {
        return Rule::Unported;
    };
    let proper = user.face.axis() != Axis::Y
        && user.click_y <= f64::from(0.8124_f32)
        && match attachment {
            "floor" => facing.axis() == user.face.axis(),
            "single_wall" | "double_wall" => facing.axis() != user.face.axis(),
            "ceiling" => true,
            _ => false,
        };
    result(if proper { SUCCESS } else { UseResult::Pass })
}

/// A flag property that decides between a success and a pass.
fn success_if(state: BlockStateId, name: &str) -> Rule<HandUse> {
    result(if property(state, name) == Some("true") {
        SUCCESS
    } else {
        UseResult::Pass
    })
}

/// The blocks whose use changes nothing on the client and ends in a success: they open their
/// screen or act on the server.
const SERVER_WORK: &[BlockClass] = &[
    BlockClass::DispenserBlock,
    BlockClass::NoteBlock,
    BlockClass::ChestBlock,
    BlockClass::CraftingTableBlock,
    BlockClass::AbstractFurnaceBlock,
    BlockClass::EnchantingTableBlock,
    BlockClass::BrewingStandBlock,
    BlockClass::DragonEggBlock,
    BlockClass::EnderChestBlock,
    BlockClass::AnvilBlock,
    BlockClass::HopperBlock,
    BlockClass::ShulkerBoxBlock,
    BlockClass::LoomBlock,
    BlockClass::BarrelBlock,
    BlockClass::CartographyTableBlock,
    BlockClass::GrindstoneBlock,
    BlockClass::SmithingTableBlock,
    BlockClass::StonecutterBlock,
    BlockClass::CrafterBlock,
    BlockClass::BeaconBlock,
    BlockClass::LeverBlock,
];

type UseFn = fn(&Hand) -> Rule<HandUse>;

/// The classes with their own client branch, by the class that declares `useWithoutItem`.
/// `CakeBlock.eat`: one more bite, or the cake gone with the last. `None` when the player cannot
/// eat.
fn bite(pos: (i32, i32, i32), cake: BlockStateId, user: &User) -> Option<Opened> {
    if !user.can_eat {
        return None;
    }
    let bites: u32 = property(cake, "bites")?.parse().ok()?;
    let state = if bites < 6 {
        with_property(cake, "bites", &(bites + 1).to_string())?
    } else {
        crate::constants::VANILLA_AIR_BLOCK_STATE_ID
    };
    Some(Opened {
        write: Write {
            pos,
            state,
            flags: 3,
        },
        sound: None,
        eats: true,
        gives: None,
    })
}

fn eat(opened: Option<Opened>) -> Rule<HandUse> {
    Rule::Known(opened.map_or(HandUse::Result(UseResult::Pass), HandUse::Set))
}

/// `CakeBlock.useWithoutItem`: the client eats; if it cannot, an empty hand consumes the use.
fn cake_use(hand: &Hand) -> Rule<HandUse> {
    match bite(hand.pos, hand.state, &hand.user) {
        Some(opened) => Rule::Known(HandUse::Set(opened)),
        None if hand.user.main_hand_empty => result(SERVER),
        None => result(UseResult::Pass),
    }
}

/// `CandleCakeBlock.useWithoutItem`: the cake under the candle is eaten, leaving a cake.
fn candle_cake_use(hand: &Hand) -> Rule<HandUse> {
    let cake = registry::block_index("minecraft:cake").map(default_state);
    eat(cake.and_then(|cake| bite(hand.pos, cake, &hand.user)))
}

/// A use that is a success, with a swing, only where the block holds something the client knows
/// of, and a pass otherwise.
fn success_if_held(held: bool) -> Rule<HandUse> {
    result(if held { SUCCESS } else { UseResult::Pass })
}

/// The item of the plant in a potted pot: its block's name without `potted_`, the azaleas
/// having `_bush` more.
fn potted_plant(pot: &str) -> Option<&'static str> {
    let plant = match pot.strip_prefix("minecraft:potted_")? {
        "azalea_bush" => "azalea",
        "flowering_azalea_bush" => "flowering_azalea",
        other => other,
    };
    let index = registry::block_index(&format!("minecraft:{plant}"))?;
    Some(BLOCKS[index].name)
}

/// `FlowerPotBlock.useWithoutItem`: a potted plant comes out (the plant joins the inventory on
/// the server's word); an empty pot consumes the use.
fn flower_pot_use(hand: &Hand) -> Rule<HandUse> {
    let emptied = block(hand.state)
        .filter(|block| block.name != "minecraft:flower_pot")
        .and_then(|_| registry::block_index("minecraft:flower_pot"))
        .map(default_state);
    match emptied {
        Some(state) => Rule::Known(HandUse::Set(Opened {
            write: Write {
                pos: hand.pos,
                state,
                flags: 3,
            },
            sound: None,
            eats: false,
            gives: block(hand.state).and_then(|block| potted_plant(block.name)),
        })),
        None => result(SERVER),
    }
}

/// `ComposterBlock.useWithoutItem`: a full composter is emptied (the bone meal is the server's).
fn composter_use(hand: &Hand) -> Rule<HandUse> {
    if property(hand.state, "level") != Some("8") {
        return result(UseResult::Pass);
    }
    let emptied = with_property(hand.state, "level", "0");
    Rule::Known(emptied.map_or(HandUse::Result(SUCCESS), |state| {
        HandUse::Set(Opened {
            write: Write {
                pos: hand.pos,
                state,
                flags: 3,
            },
            sound: None,
            eats: false,
            gives: None,
        })
    }))
}

/// `ChiseledBookShelfBlock.useWithoutItem`: a book in the slot hit comes out, on the server.
fn chiseled_bookshelf_use(hand: &Hand) -> Rule<HandUse> {
    if !hand.user.block_entity {
        return result(UseResult::Pass);
    }
    let slot = super::slots::hit_slot(
        hand.state,
        hand.pos,
        (hand.user.face, hand.user.location),
        (2, 3),
    );
    match slot {
        None => result(UseResult::Pass),
        Some(slot) if property(hand.state, &format!("slot_{slot}_occupied")) == Some("true") => {
            result(SUCCESS)
        }
        Some(_) => result(SERVER),
    }
}

const USES: &[(BlockClass, UseFn)] = &[
    (BlockClass::ChiseledBookShelfBlock, chiseled_bookshelf_use),
    // The server edits the sign or refuses; the client consumes the use where it has the sign.
    (BlockClass::SignBlock, |hand| {
        result(if hand.user.block_entity {
            SERVER
        } else {
            UseResult::Pass
        })
    }),
    (BlockClass::FlowerPotBlock, flower_pot_use),
    (BlockClass::ComposterBlock, composter_use),
    // The server picks the berries; two ripe stages and more count as a success.
    (BlockClass::SweetBerryBushBlock, |hand| {
        let age: i32 = property(hand.state, "age")
            .and_then(|age| age.parse().ok())
            .unwrap_or(0);
        result(if age > 1 { SUCCESS } else { UseResult::Pass })
    }),
    // Charged, the server sets the spawn or explodes; the client consumes the use.
    (BlockClass::RespawnAnchorBlock, |hand| {
        result(if property(hand.state, "charges") == Some("0") {
            UseResult::Pass
        } else {
            SERVER
        })
    }),
    (BlockClass::JukeboxBlock, |hand| {
        let record = property(hand.state, "has_record") == Some("true");
        success_if_held(record && hand.user.block_entity)
    }),
    (BlockClass::LecternBlock, |hand| {
        result(if property(hand.state, "has_book") == Some("true") {
            SUCCESS
        } else {
            SERVER
        })
    }),
    (BlockClass::DecoratedPotBlock, |hand| {
        success_if_held(hand.user.block_entity)
    }),
    (BlockClass::CakeBlock, cake_use),
    (BlockClass::CandleCakeBlock, candle_cake_use),
    (BlockClass::DoorBlock, door_use),
    (BlockClass::TrapDoorBlock, trapdoor_use),
    (BlockClass::FenceGateBlock, gate_use),
    (BlockClass::ButtonBlock, button_use),
    (BlockClass::ComparatorBlock, comparator_use),
    (BlockClass::RepeaterBlock, repeater_use),
    (BlockClass::BellBlock, bell_use),
    // The server removes a piston that lost its block entity; the client always passes.
    (BlockClass::MovingPistonBlock, |_| result(UseResult::Pass)),
    // The server answers the client's bed use itself (`SUCCESS_SERVER`: no swing here).
    (BlockClass::BedBlock, |_| result(SERVER)),
    // `LeadItem.bindPlayerMobs` is the server's; the client passes.
    (BlockClass::FenceBlock, |_| result(UseResult::Pass)),
    // The server changes the level; the client consumes the use.
    (BlockClass::LightBlock, |_| {
        result(UseResult::Success { swing: false })
    }),
    (BlockClass::CaveVinesBlock, |hand| {
        success_if(hand.state, "berries")
    }),
    (BlockClass::CaveVinesPlantBlock, |hand| {
        success_if(hand.state, "berries")
    }),
    // The server flips it; without the right to build the use passes.
    (BlockClass::DaylightDetectorBlock, |hand| {
        result(if hand.user.may_build {
            SUCCESS
        } else {
            UseResult::Pass
        })
    }),
];

/// `useWithoutItem` of the block at `pos`, as the client runs it. `Unported` for a class not
/// listed.
pub fn use_by_hand(state: BlockStateId, pos: (i32, i32, i32), user: User) -> Rule<HandUse> {
    let Some(class) = state_owner(state, Method::UseWithoutItem) else {
        return Rule::Unported;
    };
    let hand = Hand { state, pos, user };
    if let Some((_, rule)) = USES.iter().find(|(owner, _)| *owner == class) {
        return rule(&hand);
    }
    if SERVER_WORK.contains(&class) {
        return result(SUCCESS);
    }
    Rule::Unported
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
