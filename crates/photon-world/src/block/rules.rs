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

/// A rule's answer, or that the class's rule is not written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule<T> {
    Known(T),
    Unported,
}

fn block_index(state: i32) -> Option<(usize, u32)> {
    registry::block_index_for_state(BlockStateId(state))
}

fn block(state: i32) -> Option<&'static Block> {
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
fn state_owner(state: i32, method: Method) -> Option<BlockClass> {
    block(state).and_then(|block| owner(block, method))
}

/// A property's value in a state.
pub fn property(state: i32, name: &str) -> Option<&'static str> {
    let (block, offset) = registry::block_for_state(BlockStateId(state))?;
    block
        .properties_at(offset)?
        .into_iter()
        .find(|(property, _)| *property == name)
        .map(|(_, value)| value)
}

/// `setValue`: the same state with one property changed.
pub fn with_property(state: i32, name: &str, value: &str) -> Option<i32> {
    let (block, offset) = registry::block_for_state(BlockStateId(state))?;
    let mut chosen: BTreeMap<String, String> = block
        .properties_at(offset)?
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    let slot = chosen.get_mut(name)?;
    *slot = value.to_owned();
    block.state_id(&chosen).map(|id| id.0)
}

fn default_state(block: usize) -> i32 {
    registry::BLOCKS[block].default_state_id().0
}

fn is_air(state: i32) -> bool {
    matches!(
        registry::block_for_state(BlockStateId(state)).map(|(block, _)| block.name),
        Some("minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
    )
}

/// `getFluidState(pos).is(Fluids.WATER)`: a water source, loose or held.
pub fn is_water_source(state: i32) -> bool {
    crate::block::state_or_air(i64::from(state)).fluid.fluid
        == photon_data::block::definition::Fluid::Water
}

/// What a block reads of the level and of the use that places it.
pub struct PlaceContext<'a> {
    pub state_at: &'a dyn Fn(i32, i32, i32) -> i32,
    /// Where the aim hit, and on which face.
    pub hit_pos: (i32, i32, i32),
    pub face: Direction,
    pub location: (f64, f64, f64),
    /// The held item's registry id; `None` is an empty hand.
    pub item: Option<ItemId>,
    /// `Player.getDirection`, from the yaw.
    pub horizontal_direction: Direction,
    /// Whether the clicked block itself is replaced (`replaceClicked`).
    pub replace_clicked: bool,
}

impl PlaceContext<'_> {
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

    fn state(&self, pos: (i32, i32, i32)) -> i32 {
        (self.state_at)(pos.0, pos.1, pos.2)
    }

    /// Whether the item in use is the block's own (`asItem`).
    fn holds_item_of(&self, state: i32) -> bool {
        block(state)
            .and_then(|block| photon_data::item::registry::index(block.item_name()))
            .is_some_and(|item| self.item == i32::try_from(item).ok().map(ItemId))
    }
}

/// `BlockBehaviour.canBeReplaced(state, context)` as the block's class
/// decides it.
pub fn can_be_replaced(state: i32, ctx: &PlaceContext<'_>) -> Rule<bool> {
    let Some(block) = block(state) else {
        return Rule::Known(false);
    };
    match owner(block, Method::CanBeReplaced) {
        None => Rule::Known(block.replaceable && (ctx.item.is_none() || !ctx.holds_item_of(state))),
        Some(BlockClass::SlabBlock) => {
            let kind = property(state, "type");
            if kind == Some("double") || !ctx.holds_item_of(state) {
                return Rule::Known(false);
            }
            if !ctx.replace_clicked {
                return Rule::Known(true);
            }
            let above = ctx.location.1 - f64::from(ctx.clicked_pos().1) > 0.5;
            let horizontal = ctx.face.axis() != Axis::Y;
            Rule::Known(if kind == Some("bottom") {
                ctx.face == Direction::Up || above && horizontal
            } else {
                ctx.face == Direction::Down || !above && horizontal
            })
        }
        _ => Rule::Unported,
    }
}

/// `Block.getStateForPlacement` for the block the item places; `None` is a
/// refusal.
pub fn state_for_placement(block: usize, ctx: &PlaceContext<'_>) -> Rule<Option<i32>> {
    let default = default_state(block);
    let pos = ctx.clicked_pos();
    #[expect(
        clippy::neg_cmp_op_on_partial_ord,
        reason = "the reference's `!(y > 0.5)`, which a NaN also passes"
    )]
    let lower_half = |ctx: &PlaceContext<'_>| {
        ctx.face != Direction::Down
            && (ctx.face == Direction::Up || !(ctx.location.1 - f64::from(pos.1) > 0.5))
    };
    match owner(&BLOCKS[block], Method::GetStateForPlacement) {
        Some(BlockClass::Block) => Rule::Known(Some(default)),
        Some(BlockClass::RotatedPillarBlock) => {
            let axis = match ctx.face.axis() {
                Axis::X => "x",
                Axis::Y => "y",
                Axis::Z => "z",
            };
            Rule::Known(with_property(default, "axis", axis))
        }
        Some(BlockClass::SlabBlock) => {
            let replaced = ctx.state(pos);
            if block_index(replaced).map(|(index, _)| index) == Some(block) {
                let double = with_property(replaced, "type", "double")
                    .and_then(|state| with_property(state, "waterlogged", "false"));
                return Rule::Known(double);
            }
            let water = if is_water_source(replaced) {
                "true"
            } else {
                "false"
            };
            let half = if lower_half(ctx) { "bottom" } else { "top" };
            Rule::Known(
                with_property(default, "type", half)
                    .and_then(|state| with_property(state, "waterlogged", water)),
            )
        }
        Some(BlockClass::StairBlock) => {
            let water = if is_water_source(ctx.state(pos)) {
                "true"
            } else {
                "false"
            };
            let half = if lower_half(ctx) { "bottom" } else { "top" };
            let Some(state) = with_property(default, "facing", ctx.horizontal_direction.name())
                .and_then(|state| with_property(state, "half", half))
                .and_then(|state| with_property(state, "waterlogged", water))
            else {
                return Rule::Known(None);
            };
            let shape = stairs_shape(state, ctx.state_at, pos);
            Rule::Known(with_property(state, "shape", shape))
        }
        _ => Rule::Unported,
    }
}

/// `BlockBehaviour.canSurvive` as the block's class decides it.
pub fn can_survive(state: i32) -> Rule<bool> {
    match state_owner(state, Method::CanSurvive) {
        None => Rule::Known(true),
        Some(_) => Rule::Unported,
    }
}

/// `instanceof StairBlock`.
fn is_stairs(state: i32) -> bool {
    block(state).is_some_and(|block| block.class.is(BlockClass::StairBlock))
}

fn facing(state: i32) -> Option<Direction> {
    property(state, "facing").and_then(Direction::from_name)
}

/// `StairBlock.getStairsShape`.
fn stairs_shape(
    state: i32,
    state_at: &dyn Fn(i32, i32, i32) -> i32,
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
    state: i32,
    state_at: &dyn Fn(i32, i32, i32) -> i32,
    pos: (i32, i32, i32),
    direction: Direction,
) -> Rule<i32> {
    match state_owner(state, Method::UpdateShape) {
        None => Rule::Known(state),
        Some(BlockClass::StairBlock) => {
            if direction.axis() == Axis::Y {
                return Rule::Known(state);
            }
            let shape = stairs_shape(state, state_at, pos);
            Rule::Known(with_property(state, "shape", shape).unwrap_or(state))
        }
        _ => Rule::Unported,
    }
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
pub fn use_item_on(state: i32) -> Rule<UseResult> {
    match state_owner(state, Method::UseItemOn) {
        None => Rule::Known(UseResult::TryEmptyHand),
        Some(_) => Rule::Unported,
    }
}

/// `BlockBehaviour.useWithoutItem`.
pub fn use_without_item(state: i32) -> Rule<UseResult> {
    match state_owner(state, Method::UseWithoutItem) {
        None => Rule::Known(UseResult::Pass),
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
pub fn state_is_air(state: i32) -> bool {
    is_air(state)
}
