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

/// What a block reads of the level and of the use that places it.
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
    /// Whether the clicked block itself is replaced (`replaceClicked`).
    pub replace_clicked: bool,
    /// The block tags the server sent.
    pub tags: &'a super::tags::BlockTags,
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
pub fn state_for_placement(block: usize, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let default = default_state(block);
    match owner(&BLOCKS[block], Method::GetStateForPlacement) {
        Some(BlockClass::Block) => Rule::Known(Some(default)),
        Some(BlockClass::SnowyBlock) => Rule::Known(snowy_placement(default, ctx)),
        Some(BlockClass::RotatedPillarBlock) => {
            let axis = match ctx.face.axis() {
                Axis::X => "x",
                Axis::Y => "y",
                Axis::Z => "z",
            };
            Rule::Known(with_property(default, "axis", axis))
        }
        Some(BlockClass::SlabBlock) => Rule::Known(slab_placement(block, default, ctx)),
        Some(BlockClass::StairBlock) => Rule::Known(stair_placement(default, ctx)),
        _ => Rule::Unported,
    }
}

/// `"true"` or `"false"`, as a boolean property's value.
fn flag(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

/// Whether a slab or stair goes in the lower half: not against a downward face, and an upward
/// face or a hit in the lower half of the block.
#[expect(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "the reference's `!(y > 0.5)`, which a NaN also passes"
)]
fn lower_half(ctx: &PlaceContext<'_>) -> bool {
    let pos = ctx.clicked_pos();
    ctx.face != Direction::Down
        && (ctx.face == Direction::Up || !(ctx.location.1 - f64::from(pos.1) > 0.5))
}

/// `SnowyBlock.getStateForPlacement`: snowy when the block above is in the snow tag
/// (`isSnowySetting`).
fn snowy_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let (x, y, z) = ctx.clicked_pos();
    let above = ctx.state((x, y + 1, z));
    let snowy =
        block_index(above).is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:snow"));
    with_property(default, "snowy", flag(snowy))
}

/// `SlabBlock.getStateForPlacement`: a slab of the same block becomes a double one.
fn slab_placement(
    block: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let replaced = ctx.state(ctx.clicked_pos());
    if block_index(replaced).map(|(index, _)| index) == Some(block) {
        return with_property(replaced, "type", "double")
            .and_then(|state| with_property(state, "waterlogged", "false"));
    }
    let half = if lower_half(ctx) { "bottom" } else { "top" };
    with_property(default, "type", half)
        .and_then(|state| with_property(state, "waterlogged", flag(is_water_source(replaced))))
}

/// `StairBlock.getStateForPlacement`.
fn stair_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let water = flag(is_water_source(ctx.state(pos)));
    let half = if lower_half(ctx) { "bottom" } else { "top" };
    let state = with_property(default, "facing", ctx.horizontal_direction.name())
        .and_then(|state| with_property(state, "half", half))
        .and_then(|state| with_property(state, "waterlogged", water))?;
    let shape = stairs_shape(state, ctx.state_at, pos);
    with_property(state, "shape", shape)
}

/// `BlockBehaviour.canSurvive` as the block's class decides it.
pub fn can_survive(state: BlockStateId) -> Rule<bool> {
    match state_owner(state, Method::CanSurvive) {
        None => Rule::Known(true),
        Some(_) => Rule::Unported,
    }
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
    pos: (i32, i32, i32),
    direction: Direction,
) -> Rule<BlockStateId> {
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
            replace_clicked: false,
            tags,
        };
        let grass = registry::block_index("minecraft:grass_block").expect("grass");
        match state_for_placement(grass, &ctx) {
            Rule::Known(Some(state)) => property(state, "snowy"),
            _ => None,
        }
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
