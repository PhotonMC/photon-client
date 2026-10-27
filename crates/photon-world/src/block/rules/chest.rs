//! `ChestBlock.updateShape`: the halves of a double chest follow each other.

use super::directional::chest_joins;
use super::*;

/// `ChestBlock.getConnectedDirection`: the side the other half stands on.
pub(super) fn connected_direction(state: BlockStateId) -> Option<Direction> {
    let facing = facing(state)?;
    Some(if property(state, "type") == Some("left") {
        facing.clockwise(Axis::Y)
    } else {
        facing.counter_clockwise(Axis::Y)
    })
}

/// `Block.withPropertiesOf`: the block of `like`, with the properties `state` has.
pub(super) fn with_properties_of(like: BlockStateId, state: BlockStateId) -> Option<BlockStateId> {
    let (target, _) = registry::block_for_state(like)?;
    let (source, offset) = registry::block_for_state(state)?;
    let chosen: BTreeMap<String, String> = source
        .properties_at(offset)?
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    target.state_id(&chosen)
}

/// The type a single chest takes from a half beside it that faces the same way and reaches back
/// to it.
fn pairing(
    state: BlockStateId,
    direction: Direction,
    neighbour: BlockStateId,
) -> Option<&'static str> {
    let other = property(neighbour, "type")?;
    let joins = property(state, "type") == Some("single")
        && other != "single"
        && facing(state) == facing(neighbour)
        && connected_direction(neighbour) == Some(direction.opposite());
    joins.then_some(if other == "left" { "right" } else { "left" })
}

/// `ChestBlock.updateShape`: a chest beside a single one of the same facing becomes the other
/// half of the pair, and one whose partner is gone becomes single.
fn chest_shape(
    state: BlockStateId,
    direction: Direction,
    joins: bool,
    neighbour: BlockStateId,
) -> BlockStateId {
    if joins {
        let Some(kind) = pairing(state, direction, neighbour) else {
            return state;
        };
        with_property(state, "type", kind).unwrap_or(state)
    } else if connected_direction(state) == Some(direction) {
        with_property(state, "type", "single").unwrap_or(state)
    } else {
        state
    }
}

/// `ChestBlock.updateShape` with `CopperChestBlock.updateShape` over it: a copper chest next to
/// its partner takes the partner's block.
pub(super) fn update(
    state: BlockStateId,
    direction: Direction,
    neighbour: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let Some((index, _)) = block_index(state) else {
        return state;
    };
    let joins = chest_joins(index, neighbour, direction, ctx);
    let updated = chest_shape(state, direction, joins, neighbour);
    let copper = BLOCKS[index].class.is(BlockClass::CopperChestBlock);
    let halved = property(updated, "type") != Some("single");
    if copper && joins && halved && connected_direction(updated) == Some(direction) {
        return with_properties_of(neighbour, updated).unwrap_or(updated);
    }
    updated
}
