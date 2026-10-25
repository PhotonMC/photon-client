//! `BlockBehaviour.useItemOn` as the client runs it: what the block answers to the stack in the
//! hand, and the state it takes. A `Level.playSound(null, ..)` made here would play nothing on the
//! client (it plays only a sound that names the local player), and the server sends its own to
//! everyone, so none are played here.

use super::*;

const SUCCESS: UseResult = UseResult::Success { swing: true };
const SERVER: UseResult = UseResult::Success { swing: false };

/// What the stack in the used hand is, as far as the rules ask.
#[derive(Debug, Clone, Copy, Default)]
pub struct Held {
    pub empty: bool,
    /// Flint and steel or a fire charge.
    pub fire_starter: bool,
    pub shears: bool,
    pub honeycomb: bool,
    pub axe: bool,
    /// `ItemTags.NOTE_BLOCK_TOP_INSTRUMENTS`.
    pub note_block_top: bool,
    pub glowstone: bool,
    /// `ItemTags.LECTERN_BOOKS`.
    pub lectern_book: bool,
    /// A stack the campfire cooks (`RecipePropertySet.CAMPFIRE_INPUT`).
    pub campfire_input: bool,
    /// A stack with the `jukebox_playable` component.
    pub record: bool,
    /// The name of a candle item (`ItemTags.CANDLES`).
    pub candle: Option<&'static str>,
    /// The block a block item places.
    pub block: Option<&'static str>,
    /// A composter takes the item (`ComposterBlock.COMPOSTABLES`).
    pub compostable: bool,
    pub bone_meal: bool,
    /// A dye, ink sac, glow ink sac or honeycomb (`SignApplicator`).
    pub sign_applicator: bool,
    /// A hanging sign item.
    pub hanging_sign: bool,
}

/// Who uses the block with the stack, and where.
#[derive(Debug, Clone, Copy)]
pub struct ItemUser {
    pub main_hand: bool,
    pub may_build: bool,
    pub face: Direction,
    pub click_y: f64,
    /// The block has a block entity in the client's level.
    pub block_entity: bool,
    /// The stack in the other hand is glowstone.
    pub off_hand_glowstone: bool,
    /// The sign's block entity says it is waxed.
    pub waxed: bool,
}

/// What `useItemOn` does on the client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemOn {
    pub result: UseResult,
    /// The block set on the way.
    pub write: Option<Write>,
    /// One item of the stack is used up.
    pub shrink: bool,
}

impl ItemOn {
    const fn answer(result: UseResult) -> Self {
        Self {
            result,
            write: None,
            shrink: false,
        }
    }

    fn set(pos: (i32, i32, i32), state: Option<BlockStateId>, flags: u32, shrink: bool) -> Self {
        Self {
            result: SUCCESS,
            write: state.map(|state| Write { pos, state, flags }),
            shrink,
        }
    }
}

struct Use {
    state: BlockStateId,
    pos: (i32, i32, i32),
    held: Held,
    user: ItemUser,
}

type Rule1 = fn(&Use) -> Option<ItemOn>;

fn empty_hand() -> Option<ItemOn> {
    Some(ItemOn::answer(UseResult::TryEmptyHand))
}

fn pass() -> Option<ItemOn> {
    Some(ItemOn::answer(UseResult::Pass))
}

fn is(use_: &Use, name: &str, value: &str) -> bool {
    property(use_.state, name) == Some(value)
}

fn number(state: BlockStateId, name: &str) -> i32 {
    property(state, name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// `NoteBlock`: an instrument on top is placed instead.
fn note_block(use_: &Use) -> Option<ItemOn> {
    if use_.held.note_block_top && use_.user.face == Direction::Up {
        return pass();
    }
    empty_hand()
}

/// `PumpkinBlock`: shears carve it, on the server.
fn pumpkin(use_: &Use) -> Option<ItemOn> {
    if use_.held.shears {
        return Some(ItemOn::answer(SUCCESS));
    }
    empty_hand()
}

/// `TntBlock`: fire starters prime it, on the server; a disabled game rule only matters there.
fn tnt(use_: &Use) -> Option<ItemOn> {
    if use_.held.fire_starter {
        return Some(ItemOn::answer(SUCCESS));
    }
    empty_hand()
}

/// `VaultBlock`: a stack is offered as a key to an active vault, on the server.
fn vault(use_: &Use) -> Option<ItemOn> {
    if !use_.held.empty && is(use_, "vault_state", "active") {
        return Some(ItemOn::answer(SERVER));
    }
    empty_hand()
}

/// `CopperGolemStatueBlock.updatePose`.
fn next_pose(use_: &Use) -> ItemOn {
    let next = match property(use_.state, "copper_golem_pose") {
        Some("standing") => "sitting",
        Some("sitting") => "running",
        Some("running") => "star",
        _ => "standing",
    };
    ItemOn::set(
        use_.pos,
        with_property(use_.state, "copper_golem_pose", next),
        3,
        false,
    )
}

fn copper_golem_statue(use_: &Use) -> Option<ItemOn> {
    if use_.held.axe {
        return pass();
    }
    Some(next_pose(use_))
}

/// `WeatheringCopperGolemStatueBlock`: an axe frees the golem, which is not ported.
fn weathering_copper_golem_statue(use_: &Use) -> Option<ItemOn> {
    if use_.held.axe {
        return None;
    }
    if !use_.user.block_entity || use_.held.honeycomb {
        return pass();
    }
    Some(next_pose(use_))
}

fn unlit(use_: &Use) -> ItemOn {
    ItemOn::set(
        use_.pos,
        with_property(use_.state, "lit", "false"),
        11,
        false,
    )
}

/// `CandleBlock`: an empty hand puts out a lit candle.
fn candle(use_: &Use) -> Option<ItemOn> {
    if use_.held.empty && use_.user.may_build && is(use_, "lit", "true") {
        return Some(unlit(use_));
    }
    empty_hand()
}

/// `CandleCakeBlock`: the candle on top is put out by an empty hand; fire starters pass.
fn candle_cake(use_: &Use) -> Option<ItemOn> {
    if use_.held.fire_starter {
        return pass();
    }
    if use_.user.click_y > 0.5 && use_.held.empty && is(use_, "lit", "true") {
        return Some(unlit(use_));
    }
    empty_hand()
}

/// `CampfireBlock`: food is placed on the server.
fn campfire(use_: &Use) -> Option<ItemOn> {
    if use_.user.block_entity && use_.held.campfire_input {
        return Some(ItemOn::answer(SERVER));
    }
    empty_hand()
}

/// `RespawnAnchorBlock`: glowstone charges it.
fn respawn_anchor(use_: &Use) -> Option<ItemOn> {
    let charge = number(use_.state, "charges");
    let chargeable = charge < 4;
    if use_.held.glowstone && chargeable {
        let charged = with_property(use_.state, "charges", &(charge + 1).to_string());
        return Some(ItemOn::set(use_.pos, charged, 3, true));
    }
    if use_.user.main_hand && use_.user.off_hand_glowstone && chargeable {
        return pass();
    }
    empty_hand()
}

/// `JukeboxBlock`: a record goes in on the server.
fn jukebox(use_: &Use) -> Option<ItemOn> {
    if !is(use_, "has_record", "true") && use_.held.record {
        return Some(ItemOn::answer(SUCCESS));
    }
    empty_hand()
}

/// `LecternBlock`: a book is placed on the server.
fn lectern(use_: &Use) -> Option<ItemOn> {
    if is(use_, "has_book", "true") {
        return empty_hand();
    }
    if use_.held.lectern_book {
        return Some(ItemOn::answer(SUCCESS));
    }
    if use_.held.empty && use_.user.main_hand {
        return pass();
    }
    empty_hand()
}

/// `DecoratedPotBlock`: the client succeeds at once when the pot has its block entity.
fn decorated_pot(use_: &Use) -> Option<ItemOn> {
    if use_.user.block_entity {
        return Some(ItemOn::answer(SUCCESS));
    }
    pass()
}

/// `CakeBlock`: a candle goes on an uneaten cake.
fn cake(use_: &Use) -> Option<ItemOn> {
    let candle = use_.held.candle.filter(|_| is(use_, "bites", "0"));
    let Some(candle) = candle else {
        return empty_hand();
    };
    let cake = registry::block_index(&format!("{candle}_cake")).map(default_state);
    Some(ItemOn::set(use_.pos, cake, 3, true))
}

/// The pot a plant stands in (`FlowerPotBlock.POTTED_BY_CONTENT`): `potted_` and the plant's
/// name, the azaleas being `potted_azalea_bush` and `potted_flowering_azalea_bush`.
fn potted(block: &str) -> Option<BlockStateId> {
    let path = block.strip_prefix("minecraft:")?;
    let name = match path {
        "azalea" | "flowering_azalea" => format!("minecraft:potted_{path}_bush"),
        _ => format!("minecraft:potted_{path}"),
    };
    let index = registry::block_index(&name)?;
    (BLOCKS[index].class == BlockClass::FlowerPotBlock).then(|| default_state(index))
}

/// `FlowerPotBlock`: a plant goes into an empty pot; a full pot consumes the use.
fn flower_pot(use_: &Use) -> Option<ItemOn> {
    let Some(plant) = use_.held.block.and_then(potted) else {
        return empty_hand();
    };
    if block(use_.state).is_none_or(|block| block.name != "minecraft:flower_pot") {
        return Some(ItemOn::answer(SERVER));
    }
    Some(ItemOn::set(use_.pos, Some(plant), 3, true))
}

/// `SweetBerryBushBlock`: bone meal on a young bush is left to the item.
fn sweet_berry_bush(use_: &Use) -> Option<ItemOn> {
    if use_.held.bone_meal && !is(use_, "age", "3") {
        return pass();
    }
    empty_hand()
}

/// `ComposterBlock`: a compostable item is taken, on the server.
fn composter(use_: &Use) -> Option<ItemOn> {
    if number(use_.state, "level") < 8 && use_.held.compostable {
        return Some(ItemOn::answer(SUCCESS));
    }
    empty_hand()
}

/// `SignBlock.useItemOn` on the client: a sign without its block entity passes; an applicator, or
/// a waxed sign, is a success with a swing, and anything else only consumes the use.
fn sign(use_: &Use) -> Option<ItemOn> {
    if !use_.user.block_entity {
        return pass();
    }
    let applicator = use_.held.sign_applicator && use_.user.may_build;
    Some(ItemOn::answer(if applicator || use_.user.waxed {
        SUCCESS
    } else {
        SERVER
    }))
}

/// The hanging signs chain another hanging sign under them, which is not ported.
fn hanging_sign(use_: &Use) -> Option<ItemOn> {
    if use_.held.hanging_sign {
        return None;
    }
    sign(use_)
}

const USES: &[(BlockClass, Rule1)] = &[
    (BlockClass::SignBlock, sign),
    (BlockClass::CeilingHangingSignBlock, hanging_sign),
    (BlockClass::WallHangingSignBlock, hanging_sign),
    (BlockClass::FlowerPotBlock, flower_pot),
    (BlockClass::SweetBerryBushBlock, sweet_berry_bush),
    (BlockClass::ComposterBlock, composter),
    (BlockClass::CakeBlock, cake),
    (BlockClass::NoteBlock, note_block),
    (BlockClass::PumpkinBlock, pumpkin),
    (BlockClass::TntBlock, tnt),
    (BlockClass::VaultBlock, vault),
    (BlockClass::CopperGolemStatueBlock, copper_golem_statue),
    (
        BlockClass::WeatheringCopperGolemStatueBlock,
        weathering_copper_golem_statue,
    ),
    (BlockClass::CandleBlock, candle),
    (BlockClass::CandleCakeBlock, candle_cake),
    (BlockClass::CampfireBlock, campfire),
    (BlockClass::RespawnAnchorBlock, respawn_anchor),
    (BlockClass::JukeboxBlock, jukebox),
    (BlockClass::LecternBlock, lectern),
    (BlockClass::DecoratedPotBlock, decorated_pot),
];

/// What a block's `useItemOn` does on the client; `None` when its class is not written.
pub fn use_item_with(
    state: BlockStateId,
    pos: (i32, i32, i32),
    held: Held,
    user: ItemUser,
) -> Option<ItemOn> {
    let class = state_owner(state, Method::UseItemOn)?;
    let rule = USES.iter().find(|(owner, _)| *owner == class)?.1;
    rule(&Use {
        state,
        pos,
        held,
        user,
    })
}

/// Whether a block's `useItemOn` is one of the classes written here, and so asks for the stack's
/// description.
pub fn knows_item_on(state: BlockStateId) -> bool {
    state_owner(state, Method::UseItemOn)
        .is_some_and(|class| USES.iter().any(|(owner, _)| *owner == class))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(name: &str, properties: &[(&str, &str)]) -> BlockStateId {
        let index = registry::block_index(name).expect(name);
        let chosen = properties
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        BLOCKS[index].state_id(&chosen).expect(name)
    }

    fn user(block_entity: bool) -> ItemUser {
        ItemUser {
            main_hand: true,
            may_build: true,
            face: Direction::North,
            click_y: 0.5,
            block_entity,
            off_hand_glowstone: false,
            waxed: false,
        }
    }

    fn stone() -> Held {
        Held::default()
    }

    /// A pot or a campfire answers from its block entity, which the oracle's scenes do not have
    /// (the reference's blocks always do): the rules are read from the Java source.
    #[test]
    fn block_entity_blocks_answer_from_their_block_entity() {
        let pot = state(
            "minecraft:decorated_pot",
            &[
                ("cracked", "false"),
                ("facing", "north"),
                ("waterlogged", "false"),
            ],
        );
        let at = (0, 64, 2);
        assert_eq!(
            use_item_with(pot, at, stone(), user(true)).map(|on| on.result),
            Some(SUCCESS)
        );
        assert_eq!(
            use_item_with(pot, at, stone(), user(false)).map(|on| on.result),
            Some(UseResult::Pass)
        );
        let fire = state(
            "minecraft:campfire",
            &[
                ("facing", "north"),
                ("lit", "true"),
                ("signal_fire", "false"),
                ("waterlogged", "false"),
            ],
        );
        let meat = Held {
            campfire_input: true,
            ..stone()
        };
        assert_eq!(
            use_item_with(fire, at, meat, user(true)).map(|on| on.result),
            Some(SERVER)
        );
        assert_eq!(
            use_item_with(fire, at, meat, user(false)).map(|on| on.result),
            Some(UseResult::TryEmptyHand)
        );
        assert_eq!(
            use_item_with(fire, at, stone(), user(true)).map(|on| on.result),
            Some(UseResult::TryEmptyHand)
        );
    }

    /// A waxed sign is a success with a swing for any stack; the hanging sign chain is not ported.
    #[test]
    fn a_waxed_sign_is_a_success_for_any_stack() {
        let sign = state(
            "minecraft:oak_sign",
            &[("rotation", "0"), ("waterlogged", "false")],
        );
        let at = (0, 64, 2);
        let waxed = ItemUser {
            waxed: true,
            ..user(true)
        };
        assert_eq!(
            use_item_with(sign, at, stone(), waxed).map(|on| on.result),
            Some(SUCCESS)
        );
        assert_eq!(
            use_item_with(sign, at, stone(), user(true)).map(|on| on.result),
            Some(SERVER)
        );
        let hanging = state(
            "minecraft:oak_hanging_sign",
            &[
                ("attached", "false"),
                ("rotation", "0"),
                ("waterlogged", "false"),
            ],
        );
        let sign_item = Held {
            hanging_sign: true,
            ..stone()
        };
        assert_eq!(use_item_with(hanging, at, sign_item, user(true)), None);
    }

    /// The weathering statue passes without its block entity, and turns with it; an axe is not
    /// ported.
    #[test]
    fn a_weathering_statue_turns_with_its_block_entity() {
        let statue = state(
            "minecraft:copper_golem_statue",
            &[
                ("copper_golem_pose", "star"),
                ("facing", "north"),
                ("waterlogged", "false"),
            ],
        );
        let at = (0, 64, 2);
        let turned = use_item_with(statue, at, stone(), user(true)).expect("known");
        assert_eq!(turned.result, SUCCESS);
        assert_eq!(
            turned
                .write
                .map(|write| property(write.state, "copper_golem_pose")),
            Some(Some("standing"))
        );
        assert_eq!(
            use_item_with(statue, at, stone(), user(false)).map(|on| on.result),
            Some(UseResult::Pass)
        );
        let honey = Held {
            honeycomb: true,
            ..stone()
        };
        assert_eq!(
            use_item_with(statue, at, honey, user(true)).map(|on| on.result),
            Some(UseResult::Pass)
        );
        let axe = Held {
            axe: true,
            ..stone()
        };
        assert_eq!(use_item_with(statue, at, axe, user(true)), None);
    }
}
