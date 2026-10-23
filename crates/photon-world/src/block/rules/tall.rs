//! The blocks of two parts: doors, tall plants and beds. Placing one decides the state of the
//! part it puts down and what the other part will be; the item and the block's `setPlacedBy`
//! write the rest (see [`super::item`]).

use super::*;

fn step(pos: (i32, i32, i32), direction: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

/// `level.getBlockState(pos.above()).canBeReplaced(context)` below the build height.
fn room_above(ctx: &PlaceContext<'_>) -> Rule<bool> {
    let pos = ctx.clicked_pos();
    if pos.1 >= ctx.limits.max_y {
        return Rule::Known(false);
    }
    can_be_replaced(ctx.state(step(pos, Direction::Up)), ctx)
}

fn is_lower_door(state: BlockStateId) -> bool {
    block(state).is_some_and(|block| block.class.is(BlockClass::DoorBlock))
        && property(state, "half") == Some("lower")
}

fn full_collision(ctx: &PlaceContext<'_>, pos: (i32, i32, i32)) -> bool {
    crate::block::state(ctx.state(pos)).is_some_and(|info| info.collision_full_block)
}

/// How many of the two blocks beside the door on one side, at its foot and head, have a full
/// collision shape, each counting `amount`.
fn side_weight(ctx: &PlaceContext<'_>, side: Direction, amount: i32) -> i32 {
    let pos = ctx.clicked_pos();
    let foot = step(pos, side);
    let head = step(step(pos, Direction::Up), side);
    amount * (i32::from(full_collision(ctx, foot)) + i32::from(full_collision(ctx, head)))
}

/// `DoorBlock.getHinge`'s last resort: the hinge on the side of the half of the block clicked.
fn hinge_by_click(ctx: &PlaceContext<'_>) -> bool {
    let pos = ctx.clicked_pos();
    let (step_x, _, step_z) = ctx.horizontal_direction.step();
    let click_x = ctx.location.0 - f64::from(pos.0);
    let click_z = ctx.location.2 - f64::from(pos.2);
    (step_x >= 0 || click_z >= 0.5)
        && (step_x <= 0 || click_z <= 0.5)
        && (step_z >= 0 || click_x <= 0.5)
        && (step_z <= 0 || click_x >= 0.5)
}

/// `DoorBlock.getHinge`: against the side that holds more solid blocks or a door, else by the
/// half of the block the click was in.
fn hinge(ctx: &PlaceContext<'_>) -> &'static str {
    let pos = ctx.clicked_pos();
    let place = ctx.horizontal_direction;
    let (left, right) = (place.counter_clockwise(Axis::Y), place.clockwise(Axis::Y));
    let balance = side_weight(ctx, left, -1) + side_weight(ctx, right, 1);
    let door_left = is_lower_door(ctx.state(step(pos, left)));
    let door_right = is_lower_door(ctx.state(step(pos, right)));
    let left_hinge = if (!door_left || door_right) && balance <= 0 {
        if (!door_right || door_left) && balance >= 0 {
            hinge_by_click(ctx)
        } else {
            true
        }
    } else {
        false
    };
    if left_hinge { "left" } else { "right" }
}

/// `DoorBlock.getStateForPlacement`: the lower half, powered by a signal at either half.
pub(super) fn door(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    match room_above(ctx) {
        Rule::Known(true) => {}
        Rule::Known(false) => return Rule::Known(None),
        Rule::Unported => return Rule::Unported,
    }
    let pos = ctx.clicked_pos();
    let Rule::Known(here) = super::signal::has_neighbor_signal(pos, ctx) else {
        return Rule::Unported;
    };
    let powered = if here {
        true
    } else {
        match super::signal::has_neighbor_signal(step(pos, Direction::Up), ctx) {
            Rule::Known(above) => above,
            Rule::Unported => return Rule::Unported,
        }
    };
    let power = flag(powered);
    Rule::Known(
        with_property(default, "facing", ctx.horizontal_direction.name())
            .and_then(|state| with_property(state, "hinge", hinge(ctx)))
            .and_then(|state| with_property(state, "powered", power))
            .and_then(|state| with_property(state, "open", power))
            .and_then(|state| with_property(state, "half", "lower")),
    )
}

/// `DoublePlantBlock.getStateForPlacement`: the plain state where the block above gives way.
pub(super) fn double_plant(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    match room_above(ctx) {
        Rule::Known(room) => Rule::Known(room.then_some(default)),
        Rule::Unported => Rule::Unported,
    }
}

/// `BedBlock.getStateForPlacement`: facing away from the player, where the head part fits.
pub(super) fn bed(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let facing = ctx.horizontal_direction;
    let head = step(ctx.clicked_pos(), facing);
    match can_be_replaced(ctx.state(head), ctx) {
        Rule::Known(replaced) => Rule::Known(
            (replaced && ctx.limits.within_border(head.0, head.2))
                .then(|| with_property(default, "facing", facing.name()))
                .flatten(),
        ),
        Rule::Unported => Rule::Unported,
    }
}

/// `DoorBlock.canSurvive`: a lower half needs a full face to stand on, an upper half the block
/// below it.
pub(super) fn door_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let below = ctx.state(step(pos, Direction::Down));
    if property(state, "half") == Some("lower") {
        crate::block::state(below).is_some_and(|info| info.is_face_sturdy(Direction::Up))
    } else {
        block_index(below).map(|(index, _)| index) == block_index(state).map(|(index, _)| index)
    }
}

/// `DoublePlantBlock.canSurvive`: an upper half needs the lower half of its own block; a lower
/// half stands as any plant does.
pub(super) fn double_plant_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    if property(state, "half") != Some("upper") {
        return super::survive::vegetation(state, pos, ctx);
    }
    let below = ctx.state(step(pos, Direction::Down));
    Rule::Known(
        block_index(below).map(|(index, _)| index) == block_index(state).map(|(index, _)| index)
            && property(below, "half") == Some("lower"),
    )
}
