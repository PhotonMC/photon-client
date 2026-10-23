//! The blocks that hang from a wall or a ceiling, or stand on the ground by their own rules:
//! torches, ladders, signs, banners, skulls and coral fans. Their `getStateForPlacement` looks
//! for the first side the player looks along that can carry them, and their `canSurvive` asks
//! the block behind, below or above.

use super::placement::nearest_looking_directions;
use super::survive::can_support_center;
use super::*;
use crate::block::SupportType;

/// `isSolid` of the state at `pos`.
fn solid_at(ctx: &PlaceContext<'_>, pos: (i32, i32, i32)) -> bool {
    crate::block::state(ctx.state(pos)).is_some_and(|info| info.solid)
}

fn step(pos: (i32, i32, i32), direction: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

/// The block a wall block leans on: `pos.relative(facing.getOpposite())`.
fn behind(state: BlockStateId, pos: (i32, i32, i32)) -> ((i32, i32, i32), Direction) {
    let facing = facing(state).unwrap_or(Direction::North);
    (step(pos, facing.opposite()), facing)
}

/// `Block.canSurvive` of the classes written here; `Unported` for any other.
pub(super) fn survives(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    let below = step(pos, Direction::Down);
    Rule::Known(match class {
        // `BaseTorchBlock.canSurvive`.
        BlockClass::BaseTorchBlock => can_support_center(ctx, below, Direction::Up),
        // A torch, ladder or coral fan on a wall: a full face towards it.
        BlockClass::WallTorchBlock
        | BlockClass::RedstoneWallTorchBlock
        | BlockClass::LadderBlock
        | BlockClass::BaseCoralWallFanBlock => {
            let (at, facing) = behind(state, pos);
            crate::block::state(ctx.state(at)).is_some_and(|info| info.is_face_sturdy(facing))
        }
        BlockClass::WallSignBlock | BlockClass::WallBannerBlock => {
            solid_at(ctx, behind(state, pos).0)
        }
        BlockClass::StandingSignBlock | BlockClass::BannerBlock => solid_at(ctx, below),
        BlockClass::CeilingHangingSignBlock => {
            can_support_center(ctx, step(pos, Direction::Up), Direction::Down)
        }
        // `BaseCoralPlantTypeBlock.canSurvive`.
        BlockClass::BaseCoralPlantTypeBlock => crate::block::state(ctx.state(below))
            .is_some_and(|info| info.is_face_sturdy(Direction::Up)),
        _ => return Rule::Unported,
    })
}

/// The first side the player looks along, nearest first, whose wall-leaning state survives.
fn first_wall_state(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
    accepts: impl Fn(Direction, BlockStateId) -> bool,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    for direction in nearest_looking_directions(ctx) {
        if direction.axis() == Axis::Y {
            continue;
        }
        let Some(state) = with_property(default, "facing", direction.opposite().name()) else {
            continue;
        };
        match can_survive(state, pos, ctx) {
            Rule::Known(true) if accepts(direction, state) => return Rule::Known(Some(state)),
            Rule::Known(_) => {}
            Rule::Unported => return Rule::Unported,
        }
    }
    Rule::Known(None)
}

fn waterlogged(state: Option<BlockStateId>, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let water = is_water_source(ctx.state(ctx.clicked_pos()));
    with_property(state?, "waterlogged", flag(water))
}

/// `WallTorchBlock.getStateForPlacement`, which the redstone one asks of the plain wall torch.
pub(super) fn wall_torch(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    first_wall_state(default, ctx, |_, _| true)
}

/// `LadderBlock.getStateForPlacement`: never against a ladder that already faces the clicked face.
pub(super) fn ladder(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    if !ctx.replace_clicked {
        let against = ctx.state(step(ctx.clicked_pos(), ctx.face.opposite()));
        if block_index(against) == block_index(default) && facing(against) == Some(ctx.face) {
            return Rule::Known(None);
        }
    }
    first_wall_state(default, ctx, |_, _| true).map(|state| waterlogged(state, ctx))
}

/// `WallSignBlock.getStateForPlacement`.
pub(super) fn wall_sign(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    first_wall_state(default, ctx, |_, _| true).map(|state| waterlogged(state, ctx))
}

/// `WallBannerBlock.getStateForPlacement`.
pub(super) fn wall_banner(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    first_wall_state(default, ctx, |_, _| true)
}

/// `BaseCoralWallFanBlock.getStateForPlacement`: waterlogged by full water, then a wall.
pub(super) fn coral_wall_fan(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let Some(state) = coral_placement(default, ctx) else {
        return Rule::Known(None);
    };
    first_wall_state(state, ctx, |_, _| true)
}

/// `BaseCoralPlantTypeBlock.getStateForPlacement`: waterlogged where full water was.
pub(super) fn coral_placement(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let there = ctx.state(ctx.clicked_pos());
    let full = is_water(there) && crate::block::state_or_air(there).fluid.amount == 8;
    with_property(default, "waterlogged", flag(full))
}

/// `WallHangingSignBlock.canAttachTo`.
fn hanging_attaches(
    ctx: &PlaceContext<'_>,
    state: BlockStateId,
    at: (i32, i32, i32),
    face: Direction,
) -> bool {
    let attached = ctx.state(at);
    let on_wall_sign = block_index(attached)
        .is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:wall_hanging_signs"));
    if on_wall_sign {
        return facing(attached).map(Direction::axis) == facing(state).map(Direction::axis);
    }
    crate::block::state(attached)
        .is_some_and(|info| info.is_face_sturdy_for(face, SupportType::Full))
}

/// `WallHangingSignBlock.canPlace`: a side to hang from, or a sign on either side.
pub(super) fn hanging_can_place(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let facing = facing(state).unwrap_or(Direction::North);
    let (clockwise, counter) = (facing.clockwise(Axis::Y), facing.counter_clockwise(Axis::Y));
    hanging_attaches(ctx, state, step(pos, clockwise), counter)
        || hanging_attaches(ctx, state, step(pos, counter), clockwise)
}

/// `WallHangingSignBlock.getStateForPlacement`: not along the face clicked.
pub(super) fn wall_hanging_sign(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    let state = first_wall_state(default, ctx, |direction, state| {
        direction.axis() != ctx.face.axis() && hanging_can_place(state, pos, ctx)
    });
    state.map(|state| waterlogged(state, ctx))
}

/// `RotationSegment.convertToSegment(float)`: sixteen segments to a turn.
pub(super) fn segment(degrees: f32) -> i32 {
    let angle = degrees * (16.0_f32 / 360.0_f32);
    (((f64::from(angle) + 0.5).floor()) as i32) & 15
}

/// A rotation property set from the way the player faces, turned half a circle.
fn facing_player(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let turned = segment(ctx.rotation.0 + 180.0_f32);
    with_property(default, "rotation", &turned.to_string())
}

/// `StandingSignBlock.getStateForPlacement`.
pub(super) fn standing_sign(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    waterlogged(facing_player(default, ctx), ctx)
}

/// `BannerBlock.getStateForPlacement`.
pub(super) fn standing_banner(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    facing_player(default, ctx)
}

/// `RotationSegment.convertToSegment(Direction)`.
fn segment_of(direction: Direction) -> i32 {
    if direction.axis() == Axis::Y {
        0
    } else {
        direction.data2d() << 2
    }
}

/// `RotationSegment.convertToDirection`: the four turns that face a side.
fn direction_of_segment(segment: i32) -> Option<Direction> {
    match segment {
        0 => Some(Direction::North),
        4 => Some(Direction::East),
        8 => Some(Direction::South),
        12 => Some(Direction::West),
        _ => None,
    }
}

/// `CeilingHangingSignBlock.getStateForPlacement`: attached to the middle of the block above
/// unless a hanging sign over it already runs along the way the player faces.
pub(super) fn ceiling_hanging_sign(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    let above_pos = step(pos, Direction::Up);
    let above = ctx.state(above_pos);
    if block(above).is_some_and(|block| block.dynamic_shape) {
        return Rule::Unported;
    }
    let direction = ctx.horizontal_direction;
    let full_face = crate::block::state(above).is_some_and(|info| {
        let shape = crate::block::states()
            .collision(info)
            .face_shape(Direction::Down);
        crate::block::state::is_shape_full_block(&shape)
    });
    let mut middle = !full_face || ctx.secondary_use;
    let below_hanging_sign = block_index(above)
        .is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:all_hanging_signs"));
    if below_hanging_sign && !ctx.secondary_use {
        let along = facing(above)
            .or_else(|| {
                property(above, "rotation")
                    .and_then(|turn| turn.parse().ok())
                    .and_then(direction_of_segment)
            })
            .is_some_and(|facing| facing.axis() == direction.axis());
        if along {
            middle = false;
        }
    }
    let turn = if middle {
        segment(ctx.rotation.0 + 180.0_f32)
    } else {
        segment_of(direction.opposite())
    };
    let state = with_property(default, "attached", flag(middle))
        .and_then(|state| with_property(state, "rotation", &turn.to_string()));
    Rule::Known(waterlogged(state, ctx))
}

/// `AbstractSkullBlock.getStateForPlacement`: powered by a signal beside it.
fn powered_skull(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    super::signal::has_neighbor_signal(ctx.clicked_pos(), ctx)
        .map(|powered| with_property(default, "powered", flag(powered)))
}

/// `SkullBlock.getStateForPlacement`.
pub(super) fn standing_skull(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    powered_skull(default, ctx).map(|state| {
        state.and_then(|state| {
            with_property(state, "rotation", &segment(ctx.rotation.0).to_string())
        })
    })
}

/// `WallSkullBlock.getStateForPlacement`: facing the player, on the first side that holds
/// something that does not give way.
pub(super) fn wall_skull(
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let state = match powered_skull(default, ctx) {
        Rule::Known(Some(state)) => state,
        Rule::Known(None) => return Rule::Known(None),
        Rule::Unported => return Rule::Unported,
    };
    let pos = ctx.clicked_pos();
    for direction in nearest_looking_directions(ctx) {
        if direction.axis() == Axis::Y {
            continue;
        }
        let Some(facing_state) = with_property(state, "facing", direction.opposite().name()) else {
            continue;
        };
        match can_be_replaced(ctx.state(step(pos, direction)), ctx) {
            Rule::Known(false) => return Rule::Known(Some(facing_state)),
            Rule::Known(true) => {}
            Rule::Unported => return Rule::Unported,
        }
    }
    Rule::Known(None)
}
