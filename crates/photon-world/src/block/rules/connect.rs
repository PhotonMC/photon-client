//! `getStateForPlacement` of the blocks that connect to their neighbours: fences, iron bars and
//! panes, and walls.

use super::*;
use photon_core::shape::{BooleanOp, Shapes, VoxelShape};
use std::sync::OnceLock;

const SIDES: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

fn tagged(state: BlockStateId, tag: &str, ctx: &PlaceContext<'_>) -> bool {
    block_index(state).is_some_and(|(index, _)| ctx.tags.contains(index, tag))
}

fn class_is(state: BlockStateId, class: BlockClass) -> bool {
    block(state).is_some_and(|block| block.class.is(class))
}

/// `Block.isExceptionForConnection`.
fn is_exception(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    const NAMES: [&str; 5] = [
        "minecraft:barrier",
        "minecraft:carved_pumpkin",
        "minecraft:jack_o_lantern",
        "minecraft:melon",
        "minecraft:pumpkin",
    ];
    class_is(state, BlockClass::LeavesBlock)
        || block(state).is_some_and(|block| NAMES.contains(&block.name))
        || tagged(state, "minecraft:shulker_boxes", ctx)
}

/// `FenceGateBlock.connectsToDirection`.
fn gate_connects(state: BlockStateId, direction: Direction) -> bool {
    facing(state).is_some_and(|facing| facing.axis() == direction.clockwise(Axis::Y).axis())
}

/// Whether the neighbour on `side` has a full face towards the block.
fn face_solid(ctx: &PlaceContext<'_>, pos: (i32, i32, i32), side: Direction) -> bool {
    crate::block::state(ctx.state(neighbour(pos, side)))
        .is_some_and(|info| info.is_face_sturdy(side.opposite()))
}

fn neighbour(pos: (i32, i32, i32), side: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = side.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

/// `FenceBlock.connectsTo`.
fn fence_connects(
    block_index_placed: usize,
    state: BlockStateId,
    solid: bool,
    toward: Direction,
    ctx: &PlaceContext<'_>,
) -> bool {
    let wooden = |index: usize| ctx.tags.contains(index, "minecraft:wooden_fences");
    let same_fence = block_index(state).is_some_and(|(index, _)| {
        ctx.tags.contains(index, "minecraft:fences") && wooden(index) == wooden(block_index_placed)
    });
    let gate = class_is(state, BlockClass::FenceGateBlock) && gate_connects(state, toward);
    !is_exception(state, ctx) && solid || same_fence || gate
}

/// `IronBarsBlock`'s attach rule.
fn bars_attach(state: BlockStateId, solid: bool, ctx: &PlaceContext<'_>) -> bool {
    !is_exception(state, ctx) && solid
        || class_is(state, BlockClass::IronBarsBlock)
        || tagged(state, "minecraft:walls", ctx)
}

/// `WallBlock.connectsTo`.
fn wall_connects(
    state: BlockStateId,
    solid: bool,
    toward: Direction,
    ctx: &PlaceContext<'_>,
) -> bool {
    tagged(state, "minecraft:walls", ctx)
        || !is_exception(state, ctx) && solid
        || class_is(state, BlockClass::IronBarsBlock)
        || class_is(state, BlockClass::FenceGateBlock) && gate_connects(state, toward)
}

/// Sets the four sides of a fence or bars, and its waterlogging.
fn with_sides(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
    connects: impl Fn(BlockStateId, bool, Direction) -> bool,
) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let mut state = default;
    for side in SIDES {
        let beside = ctx.state(neighbour(pos, side));
        let connected = connects(beside, face_solid(ctx, pos, side), side.opposite());
        state = with_property(state, side.name(), flag(connected))?;
    }
    with_property(state, "waterlogged", flag(is_water_source(ctx.state(pos))))
}

pub(super) fn fence_placement(
    block: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    with_sides(default, ctx, |state, solid, toward| {
        fence_connects(block, state, solid, toward, ctx)
    })
}

pub(super) fn bars_placement(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    with_sides(default, ctx, |state, solid, _| {
        bars_attach(state, solid, ctx)
    })
}

/// `WallBlock.TEST_SHAPE_POST` and `TEST_SHAPES_WALL`.
fn test_shapes() -> &'static (VoxelShape, [(Direction, VoxelShape); 4]) {
    static SHAPES: OnceLock<(VoxelShape, [(Direction, VoxelShape); 4])> = OnceLock::new();
    SHAPES.get_or_init(|| {
        (
            crate::block::shapes::column_xz(2.0, 0.0, 16.0),
            Shapes::rotate_horizontal(&crate::block::shapes::box_z_xy(2.0, 0.0, 9.0)),
        )
    })
}

/// `WallBlock.isCovered`.
fn covered(above: &VoxelShape, test: &VoxelShape) -> bool {
    !Shapes::join_is_not_empty(test, above, BooleanOp::OnlyFirst)
}

/// `WallBlock.getStateForPlacement`; unported above a block whose collision shape depends on
/// where it stands.
pub(super) fn wall_placement(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    let top = ctx.state(neighbour(pos, Direction::Up));
    let Some(above) = above_shape(top) else {
        return Rule::Unported;
    };
    let sides = SIDES.map(|side| {
        let beside = ctx.state(neighbour(pos, side));
        wall_connects(beside, face_solid(ctx, pos, side), side.opposite(), ctx)
    });
    let water = flag(is_water_source(ctx.state(pos)));
    Rule::Known(
        with_property(default, "waterlogged", water)
            .and_then(|state| wall_shape(state, top, sides, &above, ctx)),
    )
}

/// The lower face of the collision shape of the block above a wall; `None` for a block whose
/// shape depends on where it stands.
fn above_shape(top: BlockStateId) -> Option<VoxelShape> {
    if block(top)
        .is_some_and(|block| block.dynamic_shape || crate::block::moves_with_position(block))
    {
        return None;
    }
    let info = crate::block::state_or_air(top);
    Some(
        crate::block::states()
            .collision(info)
            .face_shape(Direction::Down),
    )
}

/// `FenceBlock.updateShape`: the side towards the changed neighbour.
pub(super) fn fence_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if !SIDES.contains(&direction) {
        return state;
    }
    let Some((index, _)) = block_index(state) else {
        return state;
    };
    let beside = ctx.state(neighbour(pos, direction));
    let connected = fence_connects(
        index,
        beside,
        face_solid(ctx, pos, direction),
        direction.opposite(),
        ctx,
    );
    with_property(state, direction.name(), flag(connected)).unwrap_or(state)
}

/// `IronBarsBlock.updateShape`: the side towards the changed neighbour.
pub(super) fn bars_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if !SIDES.contains(&direction) {
        return state;
    }
    let beside = ctx.state(neighbour(pos, direction));
    let attached = bars_attach(beside, face_solid(ctx, pos, direction), ctx);
    with_property(state, direction.name(), flag(attached)).unwrap_or(state)
}

/// `WallBlock.updateShape`: a change above reshapes the arms and the post; one at the side
/// replaces that arm's connection first. Unported above a block whose shape moves.
pub(super) fn wall_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    if direction == Direction::Down {
        return Rule::Known(state);
    }
    let top = ctx.state(neighbour(pos, Direction::Up));
    let Some(above) = above_shape(top) else {
        return Rule::Unported;
    };
    let sides = SIDES.map(|side| {
        if side == direction {
            let beside = ctx.state(neighbour(pos, side));
            wall_connects(beside, face_solid(ctx, pos, side), side.opposite(), ctx)
        } else {
            property(state, side.name()).is_some_and(|height| height != "none")
        }
    });
    Rule::Known(wall_shape(state, top, sides, &above, ctx).unwrap_or(state))
}

/// `FenceGateBlock.updateShape`: in a wall when the block beside it along its line, or the one
/// opposite, is a wall.
pub(super) fn gate_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let along = facing(state).map(|facing| facing.clockwise(Axis::Y).axis());
    if along != Some(direction.axis()) {
        return state;
    }
    let is_wall = |at| tagged(ctx.state(at), "minecraft:walls", ctx);
    let in_wall =
        is_wall(neighbour(pos, direction)) || is_wall(neighbour(pos, direction.opposite()));
    with_property(state, "in_wall", flag(in_wall)).unwrap_or(state)
}

/// `WallBlock.updateShape` with the four connections and the shape above.
fn wall_shape(
    state: BlockStateId,
    top: BlockStateId,
    sides: [bool; 4],
    above: &VoxelShape,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let (post, arms) = test_shapes();
    let mut state = state;
    let mut heights = Vec::with_capacity(4);
    for (side, connected) in SIDES.into_iter().zip(sides) {
        let test = &arms.iter().find(|(direction, _)| *direction == side)?.1;
        let height = match (connected, connected && covered(above, test)) {
            (false, _) => "none",
            (true, true) => "tall",
            (true, false) => "low",
        };
        heights.push(height);
        state = with_property(state, side.name(), height)?;
    }
    let raised = raise_post(&heights, top, above, post, ctx);
    with_property(state, "up", flag(raised))
}

/// `WallBlock.shouldRaisePost`.
fn raise_post(
    heights: &[&str],
    top: BlockStateId,
    above: &VoxelShape,
    post: &VoxelShape,
    ctx: &PlaceContext<'_>,
) -> bool {
    if class_is(top, BlockClass::WallBlock) && property(top, "up") == Some("true") {
        return true;
    }
    let [north, east, south, west] = [heights[0], heights[1], heights[2], heights[3]];
    if has_corner([north, east, south, west]) {
        return true;
    }
    let high = north == "tall" && south == "tall" || east == "tall" && west == "tall";
    !high && (tagged(top, "minecraft:wall_post_override", ctx) || covered(above, post))
}

/// Whether the arms leave a corner or a lone post: none at all, or an unpaired arm.
fn has_corner([north, east, south, west]: [&str; 4]) -> bool {
    let none = |height: &str| height == "none";
    none(north) && none(south) && none(west) && none(east)
        || none(north) != none(south)
        || none(west) != none(east)
}
