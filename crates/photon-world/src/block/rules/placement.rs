//! `Block.getStateForPlacement`: the state a block takes where it is placed.

use super::*;
use photon_core::math::{mth_cos_f64, mth_sin_f64};

/// What a class's `getStateForPlacement` answers; `None` is a refusal.
type Placement = fn(usize, BlockStateId, &PlaceContext<'_>) -> Rule<Option<BlockStateId>>;

/// The placement rules written so far, by the class that declares them.
const PLACEMENTS: &[(BlockClass, Placement)] = &[
    (BlockClass::Block, |_, default, _| {
        Rule::Known(Some(default))
    }),
    (BlockClass::SnowyBlock, |_, default, ctx| {
        Rule::Known(snowy_placement(default, ctx))
    }),
    (BlockClass::RotatedPillarBlock, |_, default, ctx| {
        Rule::Known(pillar_placement(default, ctx))
    }),
    (BlockClass::SlabBlock, |block, default, ctx| {
        Rule::Known(slab_placement(block, default, ctx))
    }),
    (BlockClass::StairBlock, |_, default, ctx| {
        Rule::Known(stair_placement(default, ctx))
    }),
    (BlockClass::GlazedTerracottaBlock, |_, default, ctx| {
        Rule::Known(glazed_placement(default, ctx))
    }),
    (BlockClass::ConcretePowderBlock, concrete_placement),
    (BlockClass::TrapDoorBlock, |_, default, ctx| {
        trapdoor_placement(default, ctx)
    }),
    (BlockClass::FenceBlock, |block, default, ctx| {
        Rule::Known(super::connect::fence_placement(block, default, ctx))
    }),
    (BlockClass::IronBarsBlock, |_, default, ctx| {
        Rule::Known(super::connect::bars_placement(default, ctx))
    }),
    (BlockClass::WallBlock, |_, default, ctx| {
        super::connect::wall_placement(default, ctx)
    }),
    (BlockClass::CandleBlock, stacked),
    (BlockClass::SeaPickleBlock, stacked),
    (BlockClass::TurtleEggBlock, stacked),
    (BlockClass::FlowerBedBlock, stacked),
    (BlockClass::LeafLitterBlock, stacked),
    (BlockClass::LanternBlock, |_, default, ctx| {
        if super::survive::has_dynamic_neighbour(ctx.clicked_pos(), ctx) {
            return Rule::Unported;
        }
        Rule::Known(lantern_placement(default, ctx))
    }),
    (BlockClass::BellBlock, |_, default, ctx| {
        if super::survive::has_dynamic_neighbour(ctx.clicked_pos(), ctx) {
            return Rule::Unported;
        }
        Rule::Known(bell_placement(default, ctx))
    }),
    (BlockClass::DiodeBlock, |_, default, ctx| {
        Rule::Known(diode_facing(default, ctx))
    }),
    (BlockClass::RepeaterBlock, repeater_placement),
    (BlockClass::WallTorchBlock, |_, default, ctx| {
        super::mounted::wall_torch(default, ctx)
    }),
    (BlockClass::RedstoneWallTorchBlock, |_, default, ctx| {
        super::mounted::wall_torch(default, ctx)
    }),
    (BlockClass::LadderBlock, |_, default, ctx| {
        super::mounted::ladder(default, ctx)
    }),
    (BlockClass::WallSignBlock, |_, default, ctx| {
        super::mounted::wall_sign(default, ctx)
    }),
    (BlockClass::WallHangingSignBlock, |_, default, ctx| {
        super::mounted::wall_hanging_sign(default, ctx)
    }),
    (BlockClass::StandingSignBlock, |_, default, ctx| {
        Rule::Known(super::mounted::standing_sign(default, ctx))
    }),
    (BlockClass::CeilingHangingSignBlock, |_, default, ctx| {
        super::mounted::ceiling_hanging_sign(default, ctx)
    }),
    (BlockClass::BannerBlock, |_, default, ctx| {
        Rule::Known(super::mounted::standing_banner(default, ctx))
    }),
    (BlockClass::WallBannerBlock, |_, default, ctx| {
        super::mounted::wall_banner(default, ctx)
    }),
    (BlockClass::SkullBlock, |_, default, ctx| {
        super::mounted::standing_skull(default, ctx)
    }),
    (BlockClass::WallSkullBlock, |_, default, ctx| {
        super::mounted::wall_skull(default, ctx)
    }),
    (BlockClass::BaseCoralPlantTypeBlock, |_, default, ctx| {
        Rule::Known(super::mounted::coral_placement(default, ctx))
    }),
    (BlockClass::BaseCoralWallFanBlock, |_, default, ctx| {
        super::mounted::coral_wall_fan(default, ctx)
    }),
    (BlockClass::DoorBlock, |_, default, ctx| {
        super::tall::door(default, ctx)
    }),
    (BlockClass::DoublePlantBlock, |_, default, ctx| {
        super::tall::double_plant(default, ctx)
    }),
    (BlockClass::BedBlock, |_, default, ctx| {
        super::tall::bed(default, ctx)
    }),
    (BlockClass::LeavesBlock, |_, default, ctx| {
        Rule::Known(super::directional::leaves(default, ctx))
    }),
    (BlockClass::ChestBlock, super::directional::chest),
    (BlockClass::CopperChestBlock, super::directional::chest),
    (BlockClass::EnderChestBlock, |_, default, ctx| {
        Rule::Known(super::directional::ender_chest(default, ctx))
    }),
    (BlockClass::FenceGateBlock, |_, default, ctx| {
        super::directional::fence_gate(default, ctx)
    }),
    (BlockClass::ObserverBlock, |_, default, ctx| {
        Rule::Known(super::directional::observer(default, ctx))
    }),
    (BlockClass::PistonBaseBlock, |_, default, ctx| {
        Rule::Known(super::directional::piston(default, ctx))
    }),
    (BlockClass::CommandBlock, |_, default, ctx| {
        Rule::Known(super::directional::command_block(default, ctx))
    }),
    (BlockClass::JigsawBlock, |_, default, ctx| {
        Rule::Known(super::directional::jigsaw(default, ctx))
    }),
    (BlockClass::HopperBlock, |_, default, ctx| {
        Rule::Known(super::directional::hopper(default, ctx))
    }),
    (BlockClass::ChainBlock, |_, default, ctx| {
        Rule::Known(super::directional::chain(default, ctx))
    }),
    (BlockClass::LightningRodBlock, |_, default, ctx| {
        Rule::Known(super::directional::faced_waterlogged(default, ctx))
    }),
    (BlockClass::AmethystClusterBlock, |_, default, ctx| {
        Rule::Known(super::directional::faced_waterlogged(default, ctx))
    }),
    (BlockClass::ShulkerBoxBlock, |_, default, ctx| {
        Rule::Known(super::directional::shulker_box(default, ctx))
    }),
    (BlockClass::EndRodBlock, |block, default, ctx| {
        Rule::Known(super::directional::end_rod(block, default, ctx))
    }),
    (BlockClass::BambooStalkBlock, |_, default, ctx| {
        Rule::Known(super::growth::bamboo(default, ctx))
    }),
    (BlockClass::GrowingPlantBlock, |_, default, ctx| {
        Rule::Known(super::growth::growing_plant(default, default, ctx))
    }),
    (BlockClass::KelpBlock, |_, default, ctx| {
        Rule::Known(super::growth::kelp(default, default, ctx))
    }),
    (BlockClass::VineBlock, super::growth::vine),
    (BlockClass::MultifaceBlock, super::growth::multiface),
    (BlockClass::ScaffoldingBlock, |_, default, ctx| {
        Rule::Known(super::growth::scaffolding(default, ctx))
    }),
    (BlockClass::BaseRailBlock, |_, default, ctx| {
        Rule::Known(super::growth::rail(default, ctx))
    }),
    (BlockClass::RedStoneWireBlock, |_, default, ctx| {
        super::growth::wire(default, ctx)
    }),
    (
        BlockClass::FaceAttachedHorizontalDirectionalBlock,
        |_, default, ctx| Rule::Known(face_attached_placement(default, ctx)),
    ),
];

/// `Block.getStateForPlacement` for the block the item places; `None` is a
/// refusal.
pub(super) fn state_for_placement(
    block: usize,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let owner = owner(&BLOCKS[block], Method::GetStateForPlacement);
    match PLACEMENTS.iter().find(|(class, _)| Some(*class) == owner) {
        Some((_, placement)) => placement(block, default_state(block), ctx),
        None => Rule::Unported,
    }
}

/// `RotatedPillarBlock.getStateForPlacement`: the axis of the face clicked.
fn pillar_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let axis = match ctx.face.axis() {
        Axis::X => "x",
        Axis::Y => "y",
        Axis::Z => "z",
    };
    with_property(default, "axis", axis)
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

/// `GlazedTerracottaBlock.getStateForPlacement`: facing the player.
fn glazed_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    with_property(
        default,
        "facing",
        ctx.horizontal_direction.opposite().name(),
    )
}

/// `ConcretePowderBlock.touchesLiquid`, including the order it reads the neighbours in: the
/// block below is only looked at when the position itself holds water.
fn touches_liquid(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let mut tested = pos;
    for direction in Direction::stream() {
        let mut state = ctx.state(tested);
        if direction != Direction::Down || is_water(state) {
            let (dx, dy, dz) = direction.step();
            tested = (pos.0 + dx, pos.1 + dy, pos.2 + dz);
            state = ctx.state(tested);
            let sturdy = crate::block::state(state)
                .is_some_and(|info| info.is_face_sturdy(direction.opposite()));
            if is_water(state) && !sturdy {
                return true;
            }
        }
    }
    false
}

/// `ConcretePowderBlock.getStateForPlacement`: concrete at once where it meets water.
fn concrete_placement(
    block: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    if !is_water(ctx.state(pos)) && !touches_liquid(pos, ctx) {
        return Rule::Known(Some(default));
    }
    let concrete = BLOCKS[block]
        .name
        .strip_suffix("_powder")
        .and_then(registry::block_index);
    concrete.map_or(Rule::Unported, |index| {
        Rule::Known(Some(default_state(index)))
    })
}

/// `TrapDoorBlock.getStateForPlacement`.
fn trapdoor_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    let (facing, half) = if !ctx.replace_clicked && ctx.face.axis() != Axis::Y {
        let upper = ctx.location.1 - f64::from(pos.1) > 0.5;
        (ctx.face, if upper { "top" } else { "bottom" })
    } else {
        let half = if ctx.face == Direction::Up {
            "bottom"
        } else {
            "top"
        };
        (ctx.horizontal_direction.opposite(), half)
    };
    let Rule::Known(powered) = super::signal::has_neighbor_signal(pos, ctx) else {
        return Rule::Unported;
    };
    let open = flag(powered);
    let water = flag(is_water_source(ctx.state(pos)));
    let state = with_property(default, "facing", facing.name())
        .and_then(|state| with_property(state, "half", half))
        .and_then(|state| with_property(state, "open", open))
        .and_then(|state| with_property(state, "powered", open))
        .and_then(|state| with_property(state, "waterlogged", water));
    Rule::Known(state)
}

/// `FaceAttachedHorizontalDirectionalBlock.getStateForPlacement`: the first direction the player
/// looks along (the face clicked first) that the block can hang from.
fn face_attached_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    nearest_looking_directions(ctx)
        .into_iter()
        .find_map(|direction| {
            let state = if direction.axis() == Axis::Y {
                let face = if direction == Direction::Up {
                    "ceiling"
                } else {
                    "floor"
                };
                with_property(default, "face", face).and_then(|state| {
                    with_property(state, "facing", ctx.horizontal_direction.name())
                })
            } else {
                with_property(default, "face", "wall")
                    .and_then(|state| with_property(state, "facing", direction.opposite().name()))
            }?;
            super::survive::face_attached_survives(state, pos, ctx).then_some(state)
        })
}

/// The three axis directions, the nearest first, from the sizes of the look vector's parts.
fn axis_order(
    [x_yaw, y_mag, z_yaw]: [f32; 3],
    pitch_cos: f32,
    [axis_x, axis_y, axis_z]: [Direction; 3],
) -> (Direction, Direction, Direction) {
    let (x_mag, z_mag) = (x_yaw * pitch_cos, z_yaw * pitch_cos);
    if x_yaw > z_yaw {
        if y_mag > x_mag {
            (axis_y, axis_x, axis_z)
        } else if z_mag > y_mag {
            (axis_x, axis_z, axis_y)
        } else {
            (axis_x, axis_y, axis_z)
        }
    } else if y_mag > z_mag {
        (axis_y, axis_z, axis_x)
    } else if x_mag > y_mag {
        (axis_z, axis_x, axis_y)
    } else {
        (axis_z, axis_y, axis_x)
    }
}

/// `Direction.orderedByNearest`: the six directions, the one the player looks along most first.
fn ordered_by_nearest((y_rot, x_rot): (f32, f32)) -> [Direction; 6] {
    let radians = (std::f64::consts::PI / 180.0) as f32;
    let pitch = f64::from(x_rot * radians);
    let yaw = f64::from(-y_rot * radians);
    let (pitch_sin, pitch_cos) = (mth_sin_f64(pitch), mth_cos_f64(pitch));
    let (yaw_sin, yaw_cos) = (mth_sin_f64(yaw), mth_cos_f64(yaw));
    let x_yaw = yaw_sin.abs();
    let y_mag = pitch_sin.abs();
    let z_yaw = yaw_cos.abs();
    let axis_x = [Direction::West, Direction::East][usize::from(yaw_sin > 0.0)];
    let axis_y = [Direction::Down, Direction::Up][usize::from(pitch_sin < 0.0)];
    let axis_z = [Direction::North, Direction::South][usize::from(yaw_cos > 0.0)];
    let magnitudes = [x_yaw, y_mag, z_yaw];
    let (first, second, third) = axis_order(magnitudes, pitch_cos, [axis_x, axis_y, axis_z]);
    [
        first,
        second,
        third,
        third.opposite(),
        second.opposite(),
        first.opposite(),
    ]
}

/// `BlockPlaceContext.getNearestLookingDirection`.
pub(super) fn nearest_direction(ctx: &PlaceContext<'_>) -> Direction {
    ordered_by_nearest(ctx.rotation)[0]
}

/// `BlockPlaceContext.getNearestLookingDirections`: when the clicked block is not replaced, the
/// direction into it comes first.
pub(super) fn nearest_looking_directions(ctx: &PlaceContext<'_>) -> [Direction; 6] {
    let mut directions = ordered_by_nearest(ctx.rotation);
    if !ctx.replace_clicked {
        let into = ctx.face.opposite();
        if let Some(index) = directions.iter().position(|direction| *direction == into) {
            directions.copy_within(0..index, 1);
            directions[0] = into;
        }
    }
    directions
}

/// `LanternBlock.getStateForPlacement`: hanging from above or standing, whichever the look
/// reaches first and the block can survive.
fn lantern_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let water = flag(is_water_source(ctx.state(pos)));
    nearest_looking_directions(ctx)
        .into_iter()
        .filter(|direction| direction.axis() == Axis::Y)
        .find_map(|direction| {
            let state = with_property(default, "hanging", flag(direction == Direction::Up))?;
            super::survive::lantern_survives(state, pos, ctx).then_some(state)
        })
        .and_then(|state| with_property(state, "waterlogged", water))
}

/// `BellBlock.getStateForPlacement`.
fn bell_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let survives = |state: BlockStateId| super::survive::bell_survives(state, pos, ctx);
    let axis = ctx.face.axis();
    if axis == Axis::Y {
        let attachment = if ctx.face == Direction::Down {
            "ceiling"
        } else {
            "floor"
        };
        let state = with_property(default, "attachment", attachment)
            .and_then(|state| with_property(state, "facing", ctx.horizontal_direction.name()))?;
        return survives(state).then_some(state);
    }
    bell_on_wall(default, ctx, axis)
}

/// `BellBlock.getStateForPlacement` against a side face: hung from one wall or between two, else
/// standing on or hanging under whatever is below.
fn bell_on_wall(default: BlockStateId, ctx: &PlaceContext<'_>, axis: Axis) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let survives = |state: BlockStateId| super::survive::bell_survives(state, pos, ctx);
    let sturdy = |direction: Direction, face: Direction| {
        let (dx, dy, dz) = direction.step();
        crate::block::state(ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz)))
            .is_some_and(|info| info.is_face_sturdy(face))
    };
    let double = axis == Axis::X
        && sturdy(Direction::West, Direction::East)
        && sturdy(Direction::East, Direction::West)
        || axis == Axis::Z
            && sturdy(Direction::North, Direction::South)
            && sturdy(Direction::South, Direction::North);
    let attachment = if double { "double_wall" } else { "single_wall" };
    let state = with_property(default, "facing", ctx.face.opposite().name())
        .and_then(|state| with_property(state, "attachment", attachment))?;
    if survives(state) {
        return Some(state);
    }
    let below = sturdy(Direction::Down, Direction::Up);
    let state = with_property(state, "attachment", if below { "floor" } else { "ceiling" })?;
    survives(state).then_some(state)
}

/// `DiodeBlock.getStateForPlacement`: facing the player.
fn diode_facing(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    with_property(
        default,
        "facing",
        ctx.horizontal_direction.opposite().name(),
    )
}

/// `RepeaterBlock.getStateForPlacement`: locked by a repeater or comparator beside it that
/// gives a signal; unported when there is one.
fn repeater_placement(
    _: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let Some(state) = diode_facing(default, ctx) else {
        return Rule::Known(None);
    };
    let pos = ctx.clicked_pos();
    let facing = ctx.horizontal_direction.opposite();
    for side in [facing.clockwise(Axis::Y), facing.counter_clockwise(Axis::Y)] {
        let (dx, dy, dz) = side.step();
        let beside = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
        if block(beside).is_some_and(|block| block.class.is(BlockClass::DiodeBlock)) {
            return Rule::Unported;
        }
    }
    Rule::Known(with_property(state, "locked", "false"))
}

/// The stacking classes' placement.
fn stacked(
    block: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    Rule::Known(super::replace::stack_placement(block, default, ctx))
}
