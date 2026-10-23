//! `getStateForPlacement` of the blocks that take their facing, waterlogging or neighbours'
//! pairing from the click: leaves, chests, fence gates, observers, pistons, hoppers, chains,
//! lightning rods, amethyst buds, shulker boxes and end rods.

use super::*;

fn waterlogged_at(ctx: &PlaceContext<'_>) -> &'static str {
    flag(is_water_source(ctx.state(ctx.clicked_pos())))
}

fn in_tag(state: BlockStateId, ctx: &PlaceContext<'_>, tag: &str) -> bool {
    block_index(state).is_some_and(|(index, _)| ctx.tags.contains(index, tag))
}

/// `LeavesBlock.getDistanceAt`: a log (the tag) is 0, a leaf block is its own distance, any other
/// block is 7.
fn leaf_distance(state: BlockStateId, ctx: &PlaceContext<'_>) -> i32 {
    if in_tag(state, ctx, "minecraft:prevents_nearby_leaf_decay") {
        return 0;
    }
    let is_leaves = block(state).is_some_and(|block| block.class.is(BlockClass::LeavesBlock));
    if !is_leaves {
        return 7;
    }
    property(state, "distance")
        .and_then(|value| value.parse().ok())
        .unwrap_or(7)
}

/// `LeavesBlock.getStateForPlacement`: persistent, waterlogged by the water it replaces, at the
/// distance its neighbours give.
pub(super) fn leaves(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let mut distance = 7;
    for direction in Direction::stream() {
        let (dx, dy, dz) = direction.step();
        let neighbour = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
        distance = distance.min(leaf_distance(neighbour, ctx) + 1);
        if distance == 1 {
            break;
        }
    }
    with_property(default, "persistent", "true")
        .and_then(|state| with_property(state, "waterlogged", waterlogged_at(ctx)))
        .and_then(|state| with_property(state, "distance", &distance.to_string()))
}

/// What a chest connects to: its own block, or any copper chest for a copper one.
fn chest_connects(block: usize, other: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    if BLOCKS[block].class.is(BlockClass::CopperChestBlock) {
        in_tag(other, ctx, "minecraft:copper_chests") && property(other, "type").is_some()
    } else {
        block_index(other).map(|(index, _)| index) == Some(block)
    }
}

/// `ChestBlock.candidatePartnerFacing`: the facing of a single chest of the same kind on that
/// side.
fn partner_facing(block: usize, ctx: &PlaceContext<'_>, toward: Direction) -> Option<Direction> {
    let pos = ctx.clicked_pos();
    let (dx, dy, dz) = toward.step();
    let other = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
    (chest_connects(block, other, ctx) && property(other, "type") == Some("single"))
        .then(|| facing(other))
        .flatten()
}

/// `ChestBlock.getStateForPlacement`'s first step: sneaking against the side of a single chest
/// joins it from that side, the new chest facing as it does.
fn join_from_side(block: usize, ctx: &PlaceContext<'_>) -> Option<(Direction, &'static str)> {
    if ctx.face.axis() == Axis::Y || !ctx.secondary_use {
        return None;
    }
    let neighbour = partner_facing(block, ctx, ctx.face.opposite())?;
    if neighbour.axis() == ctx.face.axis() {
        return None;
    }
    let right = neighbour.counter_clockwise(Axis::Y) == ctx.face.opposite();
    Some((neighbour, if right { "right" } else { "left" }))
}

/// `ChestBlock.getChestType`: left of a single chest that faces the same way on the clockwise
/// side, right of one on the other.
fn pair_with_neighbour(
    block: usize,
    ctx: &PlaceContext<'_>,
    facing_direction: Direction,
) -> &'static str {
    let beside = |side: Direction| partner_facing(block, ctx, side) == Some(facing_direction);
    if beside(facing_direction.clockwise(Axis::Y)) {
        "left"
    } else if beside(facing_direction.counter_clockwise(Axis::Y)) {
        "right"
    } else {
        "single"
    }
}

/// `ChestBlock.getStateForPlacement`: facing the player, joined to a single chest beside it
/// unless sneaking; sneaking against the side of a chest joins it from that side.
pub(super) fn chest(
    block: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let (facing_direction, kind) = join_from_side(block, ctx).unwrap_or_else(|| {
        let facing_direction = ctx.horizontal_direction.opposite();
        let kind = if ctx.secondary_use {
            "single"
        } else {
            pair_with_neighbour(block, ctx, facing_direction)
        };
        (facing_direction, kind)
    });
    // A copper chest joined to another takes the least oxidised of the pair: not written.
    if kind != "single" && BLOCKS[block].class.is(BlockClass::CopperChestBlock) {
        return Rule::Unported;
    }
    Rule::Known(
        with_property(default, "facing", facing_direction.name())
            .and_then(|state| with_property(state, "type", kind))
            .and_then(|state| with_property(state, "waterlogged", waterlogged_at(ctx))),
    )
}

/// `EnderChestBlock.getStateForPlacement`.
pub(super) fn ender_chest(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    with_property(
        default,
        "facing",
        ctx.horizontal_direction.opposite().name(),
    )
    .and_then(|state| with_property(state, "waterlogged", waterlogged_at(ctx)))
}

/// `FenceGateBlock.getStateForPlacement`: facing the way the player looks, open where a signal
/// reaches it, in a wall when a wall stands beside it across its opening.
pub(super) fn fence_gate(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    let Rule::Known(powered) = super::signal::has_neighbor_signal(pos, ctx) else {
        return Rule::Unported;
    };
    let direction = ctx.horizontal_direction;
    let is_wall = |direction: Direction| {
        let (dx, dy, dz) = direction.step();
        in_tag(
            ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz)),
            ctx,
            "minecraft:walls",
        )
    };
    let in_wall = match direction.axis() {
        Axis::Z => is_wall(Direction::West) || is_wall(Direction::East),
        Axis::X => is_wall(Direction::North) || is_wall(Direction::South),
        Axis::Y => false,
    };
    Rule::Known(
        with_property(default, "facing", direction.name())
            .and_then(|state| with_property(state, "open", flag(powered)))
            .and_then(|state| with_property(state, "powered", flag(powered)))
            .and_then(|state| with_property(state, "in_wall", flag(in_wall))),
    )
}

/// `ObserverBlock.getStateForPlacement`: facing the way the player looks.
pub(super) fn observer(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let nearest = super::placement::nearest_direction(ctx);
    with_property(default, "facing", nearest.name())
}

/// `PistonBaseBlock.getStateForPlacement`: its head toward the way the player looks, retracted.
pub(super) fn piston(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let nearest = super::placement::nearest_direction(ctx);
    with_property(default, "facing", nearest.opposite().name())
        .and_then(|state| with_property(state, "extended", "false"))
}

/// `HopperBlock.getStateForPlacement`: out of the side clicked, down for the top and bottom.
pub(super) fn hopper(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let out = ctx.face.opposite();
    let out = if out.axis() == Axis::Y {
        Direction::Down
    } else {
        out
    };
    with_property(default, "facing", out.name())
        .and_then(|state| with_property(state, "enabled", "true"))
}

/// `ChainBlock.getStateForPlacement`: along the axis of the face clicked, waterlogged.
pub(super) fn chain(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let axis = match ctx.face.axis() {
        Axis::X => "x",
        Axis::Y => "y",
        Axis::Z => "z",
    };
    with_property(default, "axis", axis)
        .and_then(|state| with_property(state, "waterlogged", waterlogged_at(ctx)))
}

/// `LightningRodBlock` and `AmethystClusterBlock.getStateForPlacement`: pointing out of the face
/// clicked, waterlogged by the water they replace.
pub(super) fn faced_waterlogged(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    with_property(default, "facing", ctx.face.name())
        .and_then(|state| with_property(state, "waterlogged", waterlogged_at(ctx)))
}

/// `ShulkerBoxBlock.getStateForPlacement`: opening out of the face clicked.
pub(super) fn shulker_box(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    with_property(default, "facing", ctx.face.name())
}

/// `EndRodBlock.getStateForPlacement`: pointing out of the face clicked, or back into the rod
/// behind that already points at the click.
pub(super) fn end_rod(
    block: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let (dx, dy, dz) = ctx.face.opposite().step();
    let behind = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
    let continues = block_index(behind).map(|(index, _)| index) == Some(block)
        && facing(behind) == Some(ctx.face);
    let toward = if continues {
        ctx.face.opposite()
    } else {
        ctx.face
    };
    with_property(default, "facing", toward.name())
}
