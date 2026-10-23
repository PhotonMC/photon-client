//! The block rules using an item runs into: may the block be replaced, what
//! state a placed block takes, may it stand there, and how a block reshapes
//! when a neighbour changes.
//!
//! Each block runs the rule of the nearest class in its lineage that
//! declares the method, as the reference's dispatch resolves it
//! ([`BlockClass::declares`]). A class whose rule is written here answers;
//! any other class answers [`Rule::Unported`], and the caller does not
//! predict what it does.

use photon_core::ids::{BlockStateId, ItemId};
use std::collections::BTreeMap;

use photon_core::direction::{Axis, Direction};
use photon_data::block::classes::{BlockClass, Method};
use photon_data::block::registry::{self, BLOCKS, Block};
use photon_data::item::classes::{ItemClass, ItemMethod};
use photon_data::item::registry::{ITEMS, Item};

mod chest;
mod chorus;
mod connect;
mod directional;
mod dripleaf;
mod furnishings;
mod ground;
mod growth;
mod item;
mod mounted;
mod note;
mod paired;
mod placement;
mod replace;
mod shape;
mod signal;
mod speleothem;
mod sulfur;
mod survive;
mod tall;
mod vine;

pub use item::{
    ItemPlacement, PlaceWrites, Write, item_placement, place_writes, placement_state,
    update_context,
};

/// A rule's answer, or that the class's rule is not written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule<T> {
    Known(T),
    Unported,
}

impl<T> Rule<T> {
    /// The answer turned into another, when it is known.
    fn map<U>(self, convert: impl FnOnce(T) -> U) -> Rule<U> {
        match self {
            Rule::Known(value) => Rule::Known(convert(value)),
            Rule::Unported => Rule::Unported,
        }
    }
}

fn block_index(state: BlockStateId) -> Option<(usize, u32)> {
    registry::block_index_for_state(state)
}

fn block(state: BlockStateId) -> Option<&'static Block> {
    block_index(state).map(|(index, _)| &BLOCKS[index])
}

/// The class whose own `method` the block runs; `None` for the one
/// `BlockBehaviour` itself has.
fn owner(block: &Block, method: Method) -> Option<BlockClass> {
    block
        .class
        .lineage()
        .find(|class| class.declares().contains(&method))
}

/// The class whose own `method` a state's block runs; `None` for
/// `BlockBehaviour`'s own or a state outside the registry.
fn state_owner(state: BlockStateId, method: Method) -> Option<BlockClass> {
    block(state).and_then(|block| owner(block, method))
}

/// A property's value in a state.
pub fn property(state: BlockStateId, name: &str) -> Option<&'static str> {
    let (block, offset) = registry::block_for_state(state)?;
    block
        .properties_at(offset)?
        .into_iter()
        .find(|(property, _)| *property == name)
        .map(|(_, value)| value)
}

/// `BlockStateParser.serialize`: the block's name, and its properties in the state definition's
/// order (`name=value`, comma-separated) unless it has none.
pub fn serialize_state(state: BlockStateId) -> String {
    let Some((block, offset)) = registry::block_for_state(state) else {
        return "air".to_owned();
    };
    let properties = block.properties_at(offset).unwrap_or_default();
    if properties.is_empty() {
        return block.name.to_owned();
    }
    let values: Vec<String> = properties
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    format!("{}[{}]", block.name, values.join(","))
}

/// `setValue`: the same state with one property changed.
pub fn with_property(state: BlockStateId, name: &str, value: &str) -> Option<BlockStateId> {
    let (block, offset) = registry::block_for_state(state)?;
    let mut chosen: BTreeMap<String, String> = block
        .properties_at(offset)?
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    let slot = chosen.get_mut(name)?;
    *slot = value.to_owned();
    block.state_id(&chosen)
}

fn default_state(block: usize) -> BlockStateId {
    registry::BLOCKS[block].default_state_id()
}

fn is_air(state: BlockStateId) -> bool {
    matches!(
        registry::block_for_state(state).map(|(block, _)| block.name),
        Some("minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
    )
}

/// `getFluidState(pos).is(Fluids.WATER)`: a water source, loose or held.
pub fn is_water_source(state: BlockStateId) -> bool {
    crate::block::state_or_air(state).fluid.fluid == photon_data::block::definition::Fluid::Water
}

/// `"true"` or `"false"`, as a boolean property's value.
fn flag(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

/// `FluidState.is(FluidTags.WATER)`.
fn is_water(state: BlockStateId) -> bool {
    use photon_data::block::definition::Fluid;
    matches!(
        crate::block::state_or_air(state).fluid.fluid,
        Fluid::Water | Fluid::FlowingWater
    )
}

/// The height and the world border a placement is checked against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    /// `Level.getMaxY`.
    pub max_y: i32,
    /// The border's west, east, north and south edges (`getMinX`, `getMaxX`, `getMinZ`, `getMaxZ`).
    pub border: [f64; 4],
}

impl Limits {
    /// The overworld's height and a border at its widest.
    pub const OPEN: Self = Self {
        max_y: 319,
        border: [-29_999_984.0, 29_999_984.0, -29_999_984.0, 29_999_984.0],
    };

    /// `WorldBorder.isWithinBounds(BlockPos)`.
    pub fn within_border(&self, x: i32, z: i32) -> bool {
        let (x, z) = (f64::from(x), f64::from(z));
        x >= self.border[0] && x < self.border[1] && z >= self.border[2] && z < self.border[3]
    }
}

/// What a block reads of the level and of the use that places it.
#[derive(Clone, Copy)]
pub struct PlaceContext<'a> {
    pub state_at: &'a dyn Fn(i32, i32, i32) -> BlockStateId,
    /// Where the aim hit, and on which face.
    pub hit_pos: (i32, i32, i32),
    pub face: Direction,
    pub location: (f64, f64, f64),
    /// The held item's registry id; `None` is an empty hand.
    pub item: Option<ItemId>,
    /// `Player.getDirection`, from the yaw.
    pub horizontal_direction: Direction,
    /// The player's `(yRot, xRot)` in degrees.
    pub rotation: (f32, f32),
    /// Whether the clicked block itself is replaced (`replaceClicked`).
    pub replace_clicked: bool,
    /// `isSecondaryUseActive`: the player sneaks.
    pub secondary_use: bool,
    /// `isInside` of the hit: the aim started inside the block.
    pub inside: bool,
    /// `Player.canUseGameMasterBlocks`: an operator in creative mode.
    pub game_master: bool,
    /// The block tags the server sent.
    pub tags: &'a super::tags::BlockTags,
    pub limits: Limits,
    /// `getRawBrightness(pos, 0)`: the brighter of the two light layers at a position. A context
    /// without it cannot answer the rules that read the light.
    pub brightness: Option<&'a dyn Fn(i32, i32, i32) -> i32>,
}

impl<'a> PlaceContext<'a> {
    /// A use of `item` on the block at `aimed.0`, its face `aimed.1` hit at `aimed.2`, by a
    /// player looking along `rotation`. The clicked block is taken as replaced until
    /// [`can_be_replaced`] says otherwise.
    pub fn new(
        state_at: &'a dyn Fn(i32, i32, i32) -> BlockStateId,
        aimed: ((i32, i32, i32), Direction, (f64, f64, f64)),
        item: Option<ItemId>,
        rotation: (f32, f32),
        tags: &'a super::tags::BlockTags,
    ) -> Self {
        Self {
            state_at,
            hit_pos: aimed.0,
            face: aimed.1,
            location: aimed.2,
            item,
            horizontal_direction: Direction::from_y_rot(f64::from(rotation.0)),
            rotation,
            replace_clicked: true,
            secondary_use: false,
            inside: false,
            game_master: false,
            tags,
            limits: Limits::OPEN,
            brightness: None,
        }
    }

    /// `getClickedPos`: the clicked block if it is replaced, otherwise the
    /// one in front of the clicked face.
    pub fn clicked_pos(&self) -> (i32, i32, i32) {
        if self.replace_clicked {
            self.hit_pos
        } else {
            let (dx, dy, dz) = self.face.step();
            (
                self.hit_pos.0 + dx,
                self.hit_pos.1 + dy,
                self.hit_pos.2 + dz,
            )
        }
    }

    fn state(&self, pos: (i32, i32, i32)) -> BlockStateId {
        (self.state_at)(pos.0, pos.1, pos.2)
    }

    /// Whether the item in use is the block's own (`asItem`).
    fn holds_item_of(&self, state: BlockStateId) -> bool {
        block(state)
            .and_then(|block| photon_data::item::registry::index(block.item_name()))
            .is_some_and(|item| self.item == i32::try_from(item).ok().map(ItemId))
    }
}

/// `BlockBehaviour.canBeReplaced(state, context)` as the block's class
/// decides it.
pub fn can_be_replaced(state: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<bool> {
    let Some(block) = block(state) else {
        return Rule::Known(false);
    };
    match owner(block, Method::CanBeReplaced) {
        None => Rule::Known(replace::default_replaced(state, ctx)),
        Some(BlockClass::SlabBlock) => Rule::Known(slab_replaced(state, ctx)),
        Some(BlockClass::SnowLayerBlock) => Rule::Known(snow_layer_replaced(state, ctx)),
        Some(class) => replace::replaced(class, state, ctx).map_or(Rule::Unported, Rule::Known),
    }
}

/// `SlabBlock.canBeReplaced`: a single slab gives way to its own item from the side it would
/// join.
fn slab_replaced(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    let kind = property(state, "type");
    if kind == Some("double") || !ctx.holds_item_of(state) {
        return false;
    }
    if !ctx.replace_clicked {
        return true;
    }
    let above = ctx.location.1 - f64::from(ctx.clicked_pos().1) > 0.5;
    let horizontal = ctx.face.axis() != Axis::Y;
    if kind == Some("bottom") {
        ctx.face == Direction::Up || above && horizontal
    } else {
        ctx.face == Direction::Down || !above && horizontal
    }
}

/// `SnowLayerBlock.canBeReplaced`: one layer gives way to any block; snow in the hand adds to a
/// layer that is not full, from above when it is the block clicked.
fn snow_layer_replaced(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    let layers: i32 = property(state, "layers")
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    if !ctx.holds_item_of(state) || layers >= 8 {
        layers == 1
    } else {
        !ctx.replace_clicked || ctx.face == Direction::Up
    }
}

/// `BlockBehaviour.canSurvive` as the block's class decides it, for the state standing at `pos`.
pub fn can_survive(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    survive::can_survive(state, pos, ctx)
}

/// `Block.getStateForPlacement` for the block the item places; `None` is a
/// refusal.
pub fn state_for_placement(block: usize, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    placement::state_for_placement(block, ctx)
}

/// `instanceof StairBlock`.
fn is_stairs(state: BlockStateId) -> bool {
    block(state).is_some_and(|block| block.class.is(BlockClass::StairBlock))
}

fn facing(state: BlockStateId) -> Option<Direction> {
    property(state, "facing").and_then(Direction::from_name)
}

/// `StairBlock.getStairsShape`.
fn stairs_shape(
    state: BlockStateId,
    state_at: &dyn Fn(i32, i32, i32) -> BlockStateId,
    pos: (i32, i32, i32),
) -> &'static str {
    let Some(facing_dir) = facing(state) else {
        return "straight";
    };
    let half = property(state, "half");
    let at = |direction: Direction| {
        let (dx, dy, dz) = direction.step();
        state_at(pos.0 + dx, pos.1 + dy, pos.2 + dz)
    };
    let can_take_shape = |neighbour: Direction| {
        let other = at(neighbour);
        !is_stairs(other) || facing(other) != Some(facing_dir) || property(other, "half") != half
    };
    let behind = at(facing_dir);
    if is_stairs(behind)
        && property(behind, "half") == half
        && let Some(behind_facing) = facing(behind)
        && behind_facing.axis() != facing_dir.axis()
        && can_take_shape(behind_facing.opposite())
    {
        return if behind_facing == facing_dir.counter_clockwise(Axis::Y) {
            "outer_left"
        } else {
            "outer_right"
        };
    }
    let front = at(facing_dir.opposite());
    if is_stairs(front)
        && property(front, "half") == half
        && let Some(front_facing) = facing(front)
        && front_facing.axis() != facing_dir.axis()
        && can_take_shape(front_facing)
    {
        return if front_facing == facing_dir.counter_clockwise(Axis::Y) {
            "inner_left"
        } else {
            "inner_right"
        };
    }
    "straight"
}

/// `BlockBehaviour.updateShape`: the state after the neighbour in
/// `direction` changed, as the block's class decides it.
pub fn update_shape(
    state: BlockStateId,
    state_at: &dyn Fn(i32, i32, i32) -> BlockStateId,
    tags: &super::tags::BlockTags,
    pos: (i32, i32, i32),
    direction: Direction,
) -> Rule<BlockStateId> {
    let ctx = PlaceContext::new(
        state_at,
        (pos, direction, (0.0, 0.0, 0.0)),
        None,
        (0.0, 0.0),
        tags,
    );
    shape::update_shape(state, pos, direction, &ctx)
}

/// What a use on a block or with an item came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseResult {
    /// It happened; `swing` is whether the client swings its arm.
    Success {
        swing: bool,
    },
    Fail,
    Pass,
    /// `TRY_WITH_EMPTY_HAND`: the block would rather be used without the item.
    TryEmptyHand,
}

impl UseResult {
    pub fn consumes_action(self) -> bool {
        matches!(self, UseResult::Success { .. } | UseResult::Fail)
    }
}

/// `BlockBehaviour.useItemOn`.
pub fn use_item_on(state: BlockStateId) -> Rule<UseResult> {
    match state_owner(state, Method::UseItemOn) {
        None => Rule::Known(UseResult::TryEmptyHand),
        Some(_) => Rule::Unported,
    }
}

/// `BlockBehaviour.useWithoutItem`.
pub fn use_without_item(state: BlockStateId) -> Rule<UseResult> {
    match state_owner(state, Method::UseWithoutItem) {
        None => Rule::Known(UseResult::Pass),
        // `CommandBlock.useWithoutItem` opens the screen for a game master, which the game mode
        // decides with the player's permission; anyone else passes.
        Some(BlockClass::CommandBlock) => Rule::Known(UseResult::Pass),
        Some(_) => Rule::Unported,
    }
}

/// An item by registry id.
pub fn item(item: ItemId) -> Option<&'static Item> {
    usize::try_from(item.0)
        .ok()
        .and_then(|index| ITEMS.get(index))
}

/// The class whose own `useOn` the item runs; `Item` for an unknown item.
pub fn item_use_on_owner(item_id: ItemId) -> ItemClass {
    item(item_id).map_or(ItemClass::Item, |item| {
        item.class.owner_of(ItemMethod::UseOn)
    })
}

/// The class whose own `use` the item runs; `Item` for an unknown item.
pub fn item_use_owner(item_id: ItemId) -> ItemClass {
    item(item_id).map_or(ItemClass::Item, |item| item.class.owner_of(ItemMethod::Use))
}

/// Whether a state is air, for `updateOrDestroy`.
pub fn state_is_air(state: BlockStateId) -> bool {
    is_air(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::tags::BlockTags;
    use photon_core::identifier::Identifier;
    use photon_core::tags::{RegistryTags, TagEntry};

    fn snow_tag(blocks: &[&str]) -> BlockTags {
        let id = |text: &str| Identifier::parse(text).expect("an identifier");
        let members = blocks
            .iter()
            .map(|name| registry::block_index(name).expect("a block") as i32)
            .collect();
        BlockTags::from_registries(&[RegistryTags::new(
            id("minecraft:block"),
            vec![TagEntry::new(id("minecraft:snow"), members)],
        )])
        .expect("the block registry")
    }

    /// Grass placed on stone, under `above`, as the client predicts it.
    fn place_grass(above: &str, tags: &BlockTags) -> Option<&'static str> {
        let above = default_state(registry::block_index(above).expect("a block"));
        let state_at = |_: i32, y: i32, _: i32| {
            if y == 65 {
                above
            } else {
                crate::constants::VANILLA_AIR_BLOCK_STATE_ID
            }
        };
        let ctx = PlaceContext {
            state_at: &state_at,
            hit_pos: (0, 63, 0),
            face: Direction::Up,
            location: (0.5, 64.0, 0.5),
            item: None,
            horizontal_direction: Direction::North,
            rotation: (0.0, 0.0),
            replace_clicked: false,
            secondary_use: false,
            inside: false,
            game_master: false,
            tags,
            limits: Limits::OPEN,
            brightness: None,
        };
        let grass = registry::block_index("minecraft:grass_block").expect("grass");
        match state_for_placement(grass, &ctx) {
            Rule::Known(Some(state)) => property(state, "snowy"),
            _ => None,
        }
    }

    /// A trap door put beside `neighbour`, which may power it.
    fn place_trapdoor_beside(neighbour: &str) -> Rule<Option<BlockStateId>> {
        let beside = default_state(registry::block_index(neighbour).expect("a block"));
        let state_at = |x: i32, y: i32, z: i32| {
            if (x, y, z) == (1, 64, 0) {
                beside
            } else {
                crate::constants::VANILLA_AIR_BLOCK_STATE_ID
            }
        };
        let tags = BlockTags::default();
        let mut ctx = PlaceContext::new(
            &state_at,
            ((0, 63, 0), Direction::Up, (0.5, 64.0, 0.5)),
            None,
            (0.0, 0.0),
            &tags,
        );
        // The ground is not replaced, so the trap door goes above it.
        ctx.replace_clicked = false;
        let trapdoor = registry::block_index("minecraft:oak_trapdoor").expect("a trap door");
        state_for_placement(trapdoor, &ctx)
    }

    #[test]
    fn a_trap_door_beside_a_signal_source_opens_unless_it_is_not_written() {
        let opened = |neighbour| match place_trapdoor_beside(neighbour) {
            Rule::Known(Some(state)) => property(state, "open"),
            _ => None,
        };
        assert_eq!(opened("minecraft:redstone_block"), Some("true"));
        assert_eq!(place_trapdoor_beside("minecraft:target"), Rule::Unported);
        assert!(matches!(
            place_trapdoor_beside("minecraft:stone"),
            Rule::Known(Some(_))
        ));
    }

    #[test]
    fn a_support_that_moves_with_its_position_is_left_to_the_server() {
        let bamboo = default_state(registry::block_index("minecraft:bamboo").expect("bamboo"));
        let state_at = |x: i32, y: i32, z: i32| {
            if (x, y, z) == (0, 63, 0) {
                bamboo
            } else {
                crate::constants::VANILLA_AIR_BLOCK_STATE_ID
            }
        };
        let tags = BlockTags::default();
        let mut ctx = PlaceContext::new(
            &state_at,
            ((0, 63, 0), Direction::Up, (0.5, 64.0, 0.5)),
            None,
            (0.0, 0.0),
            &tags,
        );
        ctx.replace_clicked = false;
        let lantern = registry::block_index("minecraft:lantern").expect("a lantern");
        assert_eq!(state_for_placement(lantern, &ctx), Rule::Unported);
    }

    /// A chest put at the origin's height with a single chest beside it to the west facing north,
    /// by a player looking south.
    fn place_chest_beside_one(sneaking: bool) -> Option<&'static str> {
        let chest = registry::block_index("minecraft:chest").expect("a chest");
        let partner = with_property(default_state(chest), "facing", "north").expect("a facing");
        let state_at = |x: i32, y: i32, z: i32| {
            if (x, y, z) == (-1, 64, 0) {
                partner
            } else {
                crate::constants::VANILLA_AIR_BLOCK_STATE_ID
            }
        };
        let tags = BlockTags::default();
        let mut ctx = PlaceContext::new(
            &state_at,
            ((0, 63, 0), Direction::Up, (0.5, 64.0, 0.5)),
            None,
            (0.0, 0.0),
            &tags,
        );
        ctx.replace_clicked = false;
        ctx.secondary_use = sneaking;
        match state_for_placement(chest, &ctx) {
            Rule::Known(Some(state)) => property(state, "type"),
            _ => None,
        }
    }

    #[test]
    fn a_chest_joins_the_one_beside_it_unless_the_player_sneaks() {
        assert_eq!(place_chest_beside_one(false), Some("right"));
        assert_eq!(place_chest_beside_one(true), Some("single"));
    }

    #[test]
    fn sneaking_keeps_a_candle_from_taking_another() {
        let candle = registry::block_index("minecraft:candle").expect("a candle");
        let held = photon_game_item(candle);
        let state_at = |_: i32, _: i32, _: i32| crate::constants::VANILLA_AIR_BLOCK_STATE_ID;
        let tags = BlockTags::default();
        let mut ctx = PlaceContext::new(
            &state_at,
            ((0, 64, 0), Direction::Up, (0.5, 65.0, 0.5)),
            held,
            (0.0, 0.0),
            &tags,
        );
        let state = default_state(candle);
        assert_eq!(can_be_replaced(state, &ctx), Rule::Known(true));
        ctx.secondary_use = true;
        assert_eq!(can_be_replaced(state, &ctx), Rule::Known(false));
    }

    /// The registry id of the item a block places.
    fn photon_game_item(block: usize) -> Option<ItemId> {
        let index = photon_data::item::registry::index(BLOCKS[block].item_name())?;
        i32::try_from(index).ok().map(ItemId)
    }

    #[test]
    fn one_snow_layer_gives_way_and_more_do_not() {
        let snow = registry::block_index("minecraft:snow").expect("snow");
        let layers = |count: &str| {
            with_property(default_state(snow), "layers", count).expect("a layer count")
        };
        let state_at = |_: i32, _: i32, _: i32| crate::constants::VANILLA_AIR_BLOCK_STATE_ID;
        let tags = BlockTags::default();
        let ctx = PlaceContext {
            state_at: &state_at,
            hit_pos: (0, 64, 0),
            face: Direction::North,
            location: (0.5, 64.1, 0.0),
            item: None,
            horizontal_direction: Direction::North,
            rotation: (0.0, 0.0),
            replace_clicked: true,
            secondary_use: false,
            inside: false,
            game_master: false,
            tags: &tags,
            limits: Limits::OPEN,
            brightness: None,
        };
        assert_eq!(can_be_replaced(layers("1"), &ctx), Rule::Known(true));
        assert_eq!(can_be_replaced(layers("2"), &ctx), Rule::Known(false));
    }

    #[test]
    fn a_snowy_block_is_snowy_under_a_block_of_the_snow_tag() {
        let tags = snow_tag(&["minecraft:snow", "minecraft:snow_block"]);
        assert_eq!(place_grass("minecraft:snow", &tags), Some("true"));
        assert_eq!(place_grass("minecraft:stone", &tags), Some("false"));
        // Before the server sent tags, nothing is in the snow tag.
        assert_eq!(
            place_grass("minecraft:snow", &BlockTags::default()),
            Some("false")
        );
    }
}
