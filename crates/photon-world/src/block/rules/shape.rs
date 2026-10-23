//! `BlockBehaviour.updateShape`: how a block reshapes when a neighbour changes.

use super::*;

/// The state of the block at `pos` after the neighbour in `direction` changed, as the class that
/// declares `updateShape` decides it. A class whose rule is not written is `Unported`.
pub(super) fn update_shape(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    let Some(class) = state_owner(state, Method::UpdateShape) else {
        return Rule::Known(state);
    };
    match class {
        BlockClass::StairBlock => Rule::Known(stairs(state, pos, direction, ctx)),
        BlockClass::SpeleothemBlock => {
            Rule::Known(super::speleothem::update(state, pos, direction, ctx))
        }
        BlockClass::NoteBlock => Rule::Known(super::note::update(state, pos, direction, ctx)),
        BlockClass::PotentSulfurBlock => Rule::Known(super::sulfur::valid_state(state, pos, ctx)),
        BlockClass::SnowyBlock => Rule::Known(snowy(state, pos, direction, ctx)),
        _ => growths(class, state, pos, direction, ctx),
    }
}

/// The plants and chests.
fn growths(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    match class {
        BlockClass::ChorusPlantBlock => {
            Rule::Known(super::chorus::plant_update(state, pos, direction, ctx))
        }
        BlockClass::TripWireBlock => {
            Rule::Known(super::chorus::tripwire_update(state, pos, direction, ctx))
        }
        BlockClass::BigDripleafBlock => {
            Rule::Known(super::dripleaf::big_update(state, pos, direction, ctx))
        }
        BlockClass::ChestBlock | BlockClass::CopperChestBlock => {
            let (dx, dy, dz) = direction.step();
            let neighbour = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
            Rule::Known(super::chest::update(state, direction, neighbour, ctx))
        }
        // These only keep the water ticking, which the client does not run.
        BlockClass::EnderChestBlock => Rule::Known(state),
        _ => plants(class, state, pos, direction, ctx),
    }
}

/// The classes whose `updateShape` only schedules a tick or turns into air when the block can no
/// longer stand. The client schedules nothing, and it never applies an air result, so for the
/// client the state stays as it is.
const INERT: [BlockClass; 65] = [
    BlockClass::VegetationBlock,
    BlockClass::FallingBlock,
    BlockClass::BrushableBlock,
    BlockClass::MangroveRootsBlock,
    BlockClass::LeavesBlock,
    BlockClass::BaseTorchBlock,
    BlockClass::SoulFireBlock,
    BlockClass::CreakingHeartBlock,
    BlockClass::SugarCaneBlock,
    BlockClass::NetherPortalBlock,
    BlockClass::ComparatorBlock,
    BlockClass::BarrierBlock,
    BlockClass::CarpetBlock,
    BlockClass::ChorusFlowerBlock,
    BlockClass::CoralBlock,
    BlockClass::CoralPlantBlock,
    BlockClass::CoralFanBlock,
    BlockClass::CoralWallFanBlock,
    BlockClass::BubbleColumnBlock,
    BlockClass::SporeBlossomBlock,
    BlockClass::FrogspawnBlock,
    BlockClass::WallTorchBlock,
    BlockClass::RedstoneWallTorchBlock,
    BlockClass::AmethystClusterBlock,
    BlockClass::BannerBlock,
    BlockClass::BaseCoralPlantTypeBlock,
    BlockClass::BaseCoralWallFanBlock,
    BlockClass::BasePressurePlateBlock,
    BlockClass::BaseRailBlock,
    BlockClass::BigDripleafStemBlock,
    BlockClass::CactusBlock,
    BlockClass::CakeBlock,
    BlockClass::CandleBlock,
    BlockClass::CandleCakeBlock,
    BlockClass::CeilingHangingSignBlock,
    BlockClass::ChainBlock,
    BlockClass::CocoaBlock,
    BlockClass::ConduitBlock,
    BlockClass::DecoratedPotBlock,
    BlockClass::DirtPathBlock,
    BlockClass::DriedGhastBlock,
    BlockClass::FarmlandBlock,
    BlockClass::FlowerPotBlock,
    BlockClass::HangingRootsBlock,
    BlockClass::HeavyCoreBlock,
    BlockClass::LadderBlock,
    BlockClass::LanternBlock,
    BlockClass::LightBlock,
    BlockClass::LightningRodBlock,
    BlockClass::MangrovePropaguleBlock,
    BlockClass::ObserverBlock,
    BlockClass::RepeaterBlock,
    BlockClass::ScaffoldingBlock,
    BlockClass::SculkSensorBlock,
    BlockClass::SculkShriekerBlock,
    BlockClass::SeaPickleBlock,
    BlockClass::SeagrassBlock,
    BlockClass::SlabBlock,
    BlockClass::SnowLayerBlock,
    BlockClass::StandingSignBlock,
    BlockClass::TrapDoorBlock,
    BlockClass::WallBannerBlock,
    BlockClass::WallHangingSignBlock,
    BlockClass::WallSignBlock,
    BlockClass::WaterloggedTransparentBlock,
];

/// `HugeMushroomBlock.updateShape`: the face toward another block of the kind goes.
fn mushroom(
    state: BlockStateId,
    direction: Direction,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let (dx, dy, dz) = direction.step();
    let neighbour = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
    if block_index(neighbour).map(|(index, _)| index) == block_index(state).map(|(index, _)| index)
    {
        with_property(state, direction.name(), "false").unwrap_or(state)
    } else {
        state
    }
}

/// `HangingMossBlock.updateShape`: the tip is the one with no moss under it.
fn moss(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> BlockStateId {
    let below = ctx.state((pos.0, pos.1 - 1, pos.2));
    let same =
        block_index(below).map(|(index, _)| index) == block_index(state).map(|(index, _)| index);
    with_property(state, "tip", flag(!same)).unwrap_or(state)
}

/// The climbing and tall plants.
fn plants(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    match class {
        BlockClass::VineBlock => Rule::Known(super::vine::update(state, pos, direction, ctx)),
        BlockClass::HugeMushroomBlock => Rule::Known(mushroom(state, direction, pos, ctx)),
        BlockClass::HangingMossBlock => Rule::Known(moss(state, pos, ctx)),
        _ if INERT.contains(&class) => Rule::Known(state),
        _ => connecting(class, state, pos, direction, ctx),
    }
}

/// The classes that join their neighbours.
fn connecting(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    match class {
        BlockClass::FenceBlock => {
            Rule::Known(super::connect::fence_update(state, pos, direction, ctx))
        }
        BlockClass::IronBarsBlock => {
            Rule::Known(super::connect::bars_update(state, pos, direction, ctx))
        }
        BlockClass::WallBlock => super::connect::wall_update(state, pos, direction, ctx),
        BlockClass::FenceGateBlock => {
            Rule::Known(super::connect::gate_update(state, pos, direction, ctx))
        }
        _ => Rule::Unported,
    }
}

/// `StairBlock.updateShape`: a side neighbour reshapes the corner.
fn stairs(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction.axis() == Axis::Y {
        return state;
    }
    let shape = stairs_shape(state, ctx.state_at, pos);
    with_property(state, "shape", shape).unwrap_or(state)
}

/// `SnowyBlock.updateShape`: snowy under a block of the snow tag.
fn snowy(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction != Direction::Up {
        return state;
    }
    let above = ctx.state((pos.0, pos.1 + 1, pos.2));
    let snowy =
        block_index(above).is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:snow"));
    with_property(state, "snowy", flag(snowy)).unwrap_or(state)
}
