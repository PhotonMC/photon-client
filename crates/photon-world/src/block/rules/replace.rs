//! `canBeReplaced` of the classes that stack (candles, pickles, eggs, petals, litter, vines) or
//! refuse their own item.

use super::*;

fn count(state: BlockStateId, name: &str) -> Option<i32> {
    property(state, name).and_then(|value| value.parse().ok())
}

/// `BlockBehaviour.canBeReplaced`: the block's replaceable flag, unless the item in hand is the
/// block's own.
pub(super) fn default_replaced(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    block(state)
        .is_some_and(|block| block.replaceable && (ctx.item.is_none() || !ctx.holds_item_of(state)))
}

/// The classes with one more of the same to add: the property that counts them.
fn stack_property(class: BlockClass) -> Option<&'static str> {
    match class {
        BlockClass::CandleBlock => Some("candles"),
        BlockClass::SeaPickleBlock => Some("pickles"),
        BlockClass::TurtleEggBlock => Some("eggs"),
        BlockClass::FlowerBedBlock => Some("flower_amount"),
        BlockClass::LeafLitterBlock => Some("segment_amount"),
        _ => None,
    }
}

/// Whether the held item is the item named like `block_name`.
fn holds_item_named(ctx: &PlaceContext<'_>, block_name: &str) -> bool {
    photon_data::item::registry::index(block_name)
        .is_some_and(|item| ctx.item == i32::try_from(item).ok().map(ItemId))
}

/// A multiface block (glow lichen, sculk vein) has a face it does not cover yet.
fn has_vacant_face(state: BlockStateId) -> bool {
    ["north", "east", "south", "west", "up", "down"]
        .iter()
        .any(|name| property(state, name) == Some("false"))
}

/// `VineBlock.canBeReplaced`: a vine with a side still free, else the default.
fn vine_replaced(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    let clicked = ctx.state(ctx.clicked_pos());
    if block_index(clicked) != block_index(state) {
        return default_replaced(state, ctx);
    }
    ["up", "north", "east", "south", "west"]
        .iter()
        .any(|name| property(clicked, name) != Some("true"))
}

/// The stacking classes: one more of the held item fits unless the player sneaks.
fn stack_replaced(stacked: &str, state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    let room = count(state, stacked).is_some_and(|amount| amount < 4);
    !ctx.secondary_use && ctx.holds_item_of(state) && room || default_replaced(state, ctx)
}

/// `GrowingPlantBodyBlock.canBeReplaced`: not by the head of its own plant.
fn plant_body_replaced(state: BlockStateId, ctx: &PlaceContext<'_>) -> Option<bool> {
    let head = block(state)?.name.strip_suffix("_plant")?;
    Some(default_replaced(state, ctx) && !holds_item_named(ctx, head))
}

/// The answer of `class`'s own `canBeReplaced`, `None` for a class that is not written.
pub(super) fn replaced(
    class: BlockClass,
    state: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<bool> {
    if let Some(stacked) = stack_property(class) {
        return Some(stack_replaced(stacked, state, ctx));
    }
    match class {
        BlockClass::PitcherCropBlock => Some(false),
        BlockClass::ScaffoldingBlock => Some(ctx.holds_item_of(state)),
        BlockClass::MultifaceBlock => Some(!ctx.holds_item_of(state) || has_vacant_face(state)),
        BlockClass::VineBlock => Some(vine_replaced(state, ctx)),
        BlockClass::GrowingPlantBodyBlock => plant_body_replaced(state, ctx),
        _ => None,
    }
}

/// `getStateForPlacement` of the stacking classes: one more of the block clicked, or a first
/// one. A candle wraps round to one after four; the others stop at four.
pub(super) fn stack_placement(
    block_index_placed: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let class = BLOCKS[block_index_placed].class;
    let stacked = stack_property(class)?;
    let pos = ctx.clicked_pos();
    let clicked = ctx.state(pos);
    if block_index(clicked).map(|(index, _)| index) == Some(block_index_placed) {
        let amount = count(clicked, stacked)?;
        let next = if class == BlockClass::CandleBlock {
            amount % 4 + 1
        } else {
            (amount + 1).min(4)
        };
        return with_property(clicked, stacked, &next.to_string());
    }
    let water = flag(is_water_source(ctx.state(pos)));
    match class {
        BlockClass::CandleBlock | BlockClass::SeaPickleBlock => {
            with_property(default, "waterlogged", water)
        }
        BlockClass::FlowerBedBlock | BlockClass::LeafLitterBlock => with_property(
            default,
            "facing",
            ctx.horizontal_direction.opposite().name(),
        ),
        _ => Some(default),
    }
}
