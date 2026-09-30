//! The block rules using an item runs into: may the block be replaced, what
//! state a placed block takes, may it stand there, and how a block reshapes
//! when a neighbour changes.
//!
//! Each block runs the rule of the class whose own method it inherits, as
//! the reference's class hierarchy resolves it; `block_behaviour_table` names
//! that class. A class whose rule is written here answers; any other class
//! answers [`Rule::Unported`], and the caller does not predict what it does.

use std::collections::BTreeMap;

use voxel_core::direction::{Axis, Direction};
use voxel_data::block::behaviour::{BLOCKS, CLASSES, ITEMS};
use voxel_data::block::registry;

/// A rule's answer, or that the class's rule is not written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule<T> {
    Known(T),
    Unported,
}

fn class_name(index: u16) -> &'static str {
    CLASSES[usize::from(index)]
}

fn block_index(state: i32) -> Option<(usize, u32)> {
    registry::block_index_for_state(u32::try_from(state).ok()?)
}

/// The block's table row.
fn behaviour(state: i32) -> Option<&'static voxel_data::block::behaviour::BlockBehaviour> {
    block_index(state).map(|(index, _)| &BLOCKS[index])
}

/// A property's value in a state.
pub fn property(state: i32, name: &str) -> Option<&'static str> {
    let (block, offset) = registry::block_for_state(u32::try_from(state).ok()?)?;
    block
        .properties_at(offset)?
        .into_iter()
        .find(|(property, _)| *property == name)
        .map(|(_, value)| value)
}

/// `setValue`: the same state with one property changed.
pub fn with_property(state: i32, name: &str, value: &str) -> Option<i32> {
    let (block, offset) = registry::block_for_state(u32::try_from(state).ok()?)?;
    let mut chosen: BTreeMap<String, String> = block
        .properties_at(offset)?
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    let slot = chosen.get_mut(name)?;
    *slot = value.to_owned();
    block.state_id(&chosen).map(|id| id as i32)
}

fn default_state(block: usize) -> i32 {
    registry::BLOCKS[block].default_state_id() as i32
}

fn is_air(state: i32) -> bool {
    matches!(
        registry::block_for_state(state as u32).map(|(block, _)| block.name),
        Some("minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
    )
}

/// `getFluidState(pos).is(Fluids.WATER)`: a water source, loose or held.
pub fn is_water_source(state: i32) -> bool {
    static SOURCE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    let source = *SOURCE.get_or_init(|| {
        registry::block("minecraft:water").map_or(u32::MAX, |block| block.default_state_id())
    });
    let (index, offset) = match block_index(state) {
        Some(found) => found,
        None => return false,
    };
    let id = voxel_data::block::per_state(voxel_data::block::destroy::BLOCKS[index], offset)
        .expect("a destroy trait per state");
    voxel_data::block::destroy::TRAITS[usize::from(id)].fluid_block == source
}

/// What a block reads of the level and of the use that places it.
pub struct PlaceContext<'a> {
    pub state_at: &'a dyn Fn(i32, i32, i32) -> i32,
    /// Where the aim hit, and on which face.
    pub hit_pos: (i32, i32, i32),
    pub face: Direction,
    pub location: (f64, f64, f64),
    /// The held item's registry id; `None` is an empty hand.
    pub item: Option<i32>,
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

    fn holds_item_of(&self, state: i32) -> bool {
        behaviour(state).is_some_and(|row| self.item == Some(i32::from(row.item)))
    }
}

/// `BlockBehaviour.canBeReplaced(state, context)` as the block's class
/// decides it.
pub fn can_be_replaced(state: i32, ctx: &PlaceContext<'_>) -> Rule<bool> {
    let Some(row) = behaviour(state) else {
        return Rule::Known(false);
    };
    match class_name(row.can_be_replaced) {
        "BlockBehaviour" => {
            let (_, offset) = block_index(state).expect("row found");
            let replaceable = row.replaceable(offset);
            Rule::Known(replaceable && (ctx.item.is_none() || !ctx.holds_item_of(state)))
        }
        "SlabBlock" => {
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
    let row = &BLOCKS[block];
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
    match class_name(row.placement) {
        "Block" => Rule::Known(Some(default)),
        "RotatedPillarBlock" => {
            let axis = match ctx.face.axis() {
                Axis::X => "x",
                Axis::Y => "y",
                Axis::Z => "z",
            };
            Rule::Known(with_property(default, "axis", axis))
        }
        "SlabBlock" => {
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
        "StairBlock" => {
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
    match behaviour(state).map(|row| class_name(row.can_survive)) {
        Some("BlockBehaviour") | None => Rule::Known(true),
        _ => Rule::Unported,
    }
}

fn is_stairs(state: i32) -> bool {
    behaviour(state).is_some_and(|row| row.stairs)
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
    match behaviour(state).map(|row| class_name(row.update_shape)) {
        Some("BlockBehaviour") | None => Rule::Known(state),
        Some("StairBlock") => {
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
    match behaviour(state).map(|row| class_name(row.use_item_on)) {
        Some("BlockBehaviour") | None => Rule::Known(UseResult::TryEmptyHand),
        _ => Rule::Unported,
    }
}

/// `BlockBehaviour.useWithoutItem`.
pub fn use_without_item(state: i32) -> Rule<UseResult> {
    match behaviour(state).map(|row| class_name(row.use_without_item)) {
        Some("BlockBehaviour") | None => Rule::Known(UseResult::Pass),
        _ => Rule::Unported,
    }
}

/// An item's table row, by registry id.
pub fn item_behaviour(item: i32) -> Option<&'static voxel_data::block::behaviour::ItemBehaviour> {
    usize::try_from(item)
        .ok()
        .and_then(|index| ITEMS.get(index))
}

pub fn item_class(item: i32) -> &'static str {
    item_behaviour(item).map_or("Item", |row| class_name(row.class))
}

pub fn item_use_on_class(item: i32) -> &'static str {
    item_behaviour(item).map_or("Item", |row| class_name(row.use_on))
}

pub fn item_use_class(item: i32) -> &'static str {
    item_behaviour(item).map_or("Item", |row| class_name(row.use_))
}

/// Whether a state is air, for `updateOrDestroy`.
pub fn state_is_air(state: i32) -> bool {
    is_air(state)
}
