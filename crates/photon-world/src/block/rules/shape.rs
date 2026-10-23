//! `BlockBehaviour.updateShape`: how a block reshapes when a neighbour changes.

use super::*;

/// The classes whose `updateShape` only schedules a tick or turns into air when the block can no
/// longer stand. The client schedules nothing, and it never applies an air result, so for the
/// client the state stays as it is.
const INERT: [BlockClass; 75] = [
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
    BlockClass::DoublePlantBlock,
    BlockClass::SmallDripleafBlock,
    BlockClass::BeehiveBlock,
    BlockClass::FaceAttachedHorizontalDirectionalBlock,
    BlockClass::PistonHeadBlock,
    BlockClass::PitcherCropBlock,
    BlockClass::TripWireHookBlock,
    BlockClass::LiquidBlock,
    BlockClass::ShelfBlock,
    BlockClass::CopperGolemStatueBlock,
];

/// How one class reshapes to the neighbour in a direction.
type Update = fn(BlockStateId, (i32, i32, i32), Direction, &PlaceContext<'_>) -> Rule<BlockStateId>;

/// The classes whose `updateShape` is written, by the class that declares it.
const RULES: [(BlockClass, Update); 31] = [
    (BlockClass::StairBlock, |s, p, d, c| {
        Rule::Known(stairs(s, p, d, c))
    }),
    (BlockClass::SpeleothemBlock, |s, p, d, c| {
        Rule::Known(super::speleothem::update(s, p, d, c))
    }),
    (BlockClass::NoteBlock, |s, p, d, c| {
        Rule::Known(super::note::update(s, p, d, c))
    }),
    (BlockClass::PotentSulfurBlock, |s, p, _, c| {
        Rule::Known(super::sulfur::valid_state(s, p, c))
    }),
    (BlockClass::SnowyBlock, |s, p, d, c| {
        Rule::Known(snowy(s, p, d, c))
    }),
    (BlockClass::ChorusPlantBlock, |s, p, d, c| {
        Rule::Known(super::chorus::plant_update(s, p, d, c))
    }),
    (BlockClass::TripWireBlock, |s, p, d, c| {
        Rule::Known(super::chorus::tripwire_update(s, p, d, c))
    }),
    (BlockClass::BigDripleafBlock, |s, p, d, c| {
        Rule::Known(super::dripleaf::big_update(s, p, d, c))
    }),
    (BlockClass::ChestBlock, chest),
    (BlockClass::CopperChestBlock, chest),
    (BlockClass::VineBlock, |s, p, d, c| {
        Rule::Known(super::vine::update(s, p, d, c))
    }),
    (BlockClass::HugeMushroomBlock, |s, p, d, c| {
        Rule::Known(mushroom(s, d, p, c))
    }),
    (BlockClass::HangingMossBlock, |s, p, _, c| {
        Rule::Known(moss(s, p, c))
    }),
    (BlockClass::BedBlock, |s, p, d, c| {
        Rule::Known(super::paired::bed(s, p, d, c))
    }),
    (BlockClass::CampfireBlock, |s, p, d, c| {
        Rule::Known(super::paired::campfire(s, p, d, c))
    }),
    (BlockClass::DoorBlock, super::paired::door),
    (BlockClass::MultifaceBlock, |s, p, d, c| {
        Rule::Known(super::faces::multiface(s, p, d, c))
    }),
    (BlockClass::MossyCarpetBlock, |s, p, _, c| {
        Rule::Known(super::faces::carpet(s, p, c))
    }),
    (
        BlockClass::BambooSaplingBlock,
        super::paired::bamboo_sapling,
    ),
    (BlockClass::BambooStalkBlock, |s, p, d, c| {
        Rule::Known(super::paired::bamboo_stalk(s, p, d, c))
    }),
    (BlockClass::BellBlock, |s, p, d, c| {
        Rule::Known(super::paired::bell(s, p, d, c))
    }),
    (BlockClass::GrowingPlantHeadBlock, |s, p, d, c| {
        Rule::Known(super::growing::head(s, p, d, c))
    }),
    (BlockClass::GrowingPlantBodyBlock, super::growing::body),
    (BlockClass::ConcretePowderBlock, |s, p, _, c| {
        super::placement::concrete_update(s, p, c)
    }),
    (BlockClass::AttachedStemBlock, |s, p, d, c| {
        Rule::Known(super::paired::attached_stem(s, d, p, c))
    }),
    (BlockClass::FenceBlock, |s, p, d, c| {
        Rule::Known(super::connect::fence_update(s, p, d, c))
    }),
    (BlockClass::IronBarsBlock, |s, p, d, c| {
        Rule::Known(super::connect::bars_update(s, p, d, c))
    }),
    (BlockClass::WallBlock, super::connect::wall_update),
    (BlockClass::FenceGateBlock, |s, p, d, c| {
        Rule::Known(super::connect::gate_update(s, p, d, c))
    }),
    // These only keep the water ticking, which the client does not run.
    (BlockClass::RedStoneWireBlock, super::redstone::update),
    (BlockClass::EnderChestBlock, |s, _, _, _| Rule::Known(s)),
];

fn chest(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    let (dx, dy, dz) = direction.step();
    let neighbour = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
    Rule::Known(super::chest::update(state, direction, neighbour, ctx))
}

/// The state of the block at `pos` after the neighbour in `direction` changed, as the class that
/// declares `updateShape` decides it: the table first, then the inert classes. A class whose rule
/// is not written is `Unported`.
pub(super) fn update_shape(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    let Some(class) = state_owner(state, Method::UpdateShape) else {
        return Rule::Known(state);
    };
    if let Some((_, update)) = RULES.iter().find(|(listed, _)| *listed == class) {
        return update(state, pos, direction, ctx);
    }
    if INERT.contains(&class) {
        Rule::Known(state)
    } else {
        Rule::Unported
    }
}

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
