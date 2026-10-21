//! The fluid a block state holds (`BlockBehaviour.getFluidState` and the
//! classes that override it), and what the reference reads of it.

use photon_core::ids::BlockStateId;
use photon_data::block::classes::BlockClass;
use photon_data::block::definition::Fluid;
use photon_data::block::properties::{LEVEL, WATERLOGGED};
use photon_data::block::registry::{BLOCKS, Block};

/// `FluidState`: a fluid type, its amount (eight for a source) and whether
/// a flowing fluid is falling.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FluidState {
    pub fluid: Fluid,
    pub amount: u8,
    pub falling: bool,
}

impl FluidState {
    pub const EMPTY: Self = Self {
        fluid: Fluid::Empty,
        amount: 0,
        falling: false,
    };

    /// `FlowingFluid.getSource(falling)`.
    const fn source(fluid: Fluid, falling: bool) -> Self {
        Self {
            fluid,
            amount: 8,
            falling,
        }
    }

    /// `FlowingFluid.getFlowing(amount, falling)`.
    const fn flowing(source: Fluid, amount: u8, falling: bool) -> Self {
        let fluid = match source {
            Fluid::Water => Fluid::FlowingWater,
            Fluid::Lava => Fluid::FlowingLava,
            other => other,
        };
        Self {
            fluid,
            amount,
            falling,
        }
    }

    pub fn is_empty(self) -> bool {
        self.fluid == Fluid::Empty
    }

    /// `FluidState.isSource`: the still fluid rather than the flowing one.
    pub fn is_source(self) -> bool {
        matches!(self.fluid, Fluid::Water | Fluid::Lava)
    }

    /// `Fluid.isSame`: water and flowing water are one fluid, and so are
    /// lava and flowing lava.
    pub fn is_same(self, other: Fluid) -> bool {
        family(self.fluid) == family(other)
    }

    /// `FluidState.getOwnHeight`.
    pub fn own_height(self) -> f32 {
        if self.is_empty() {
            0.0
        } else {
            f32::from(self.amount) / 9.0
        }
    }

    /// The fluid's registered name (`Fluids.EMPTY` and the four others).
    pub fn registered_name(self) -> &'static str {
        match self.fluid {
            Fluid::Empty => "minecraft:empty",
            Fluid::Water => "minecraft:water",
            Fluid::FlowingWater => "minecraft:flowing_water",
            Fluid::Lava => "minecraft:lava",
            Fluid::FlowingLava => "minecraft:flowing_lava",
        }
    }

    /// The state's properties and their values as `StateHolder.getValues` lists them (sorted by
    /// name): `falling` for any fluid, and the amount as `level` for a flowing one.
    pub fn properties(self) -> Vec<(&'static str, String)> {
        match self.fluid {
            Fluid::Empty => Vec::new(),
            Fluid::Water | Fluid::Lava => vec![("falling", self.falling.to_string())],
            Fluid::FlowingWater | Fluid::FlowingLava => vec![
                ("falling", self.falling.to_string()),
                ("level", self.amount.to_string()),
            ],
        }
    }

    /// `FluidState.createLegacyBlock`: the liquid block state that stands for
    /// this fluid, or air for none.
    pub fn legacy_block(self) -> BlockStateId {
        let block = match family(self.fluid) {
            Fluid::Water => photon_data::block::blocks::id::WATER,
            Fluid::Lava => photon_data::block::blocks::id::LAVA,
            _ => return crate::constants::VANILLA_AIR_BLOCK_STATE_ID,
        };
        // `FlowingFluid.getLegacyLevel`.
        let level = if self.is_source() {
            0
        } else {
            8 - u32::from(self.amount.min(8)) + if self.falling { 8 } else { 0 }
        };
        BLOCKS[block].state_at(level)
    }
}

/// The still fluid of a fluid's pair.
fn family(fluid: Fluid) -> Fluid {
    match fluid {
        Fluid::FlowingWater => Fluid::Water,
        Fluid::FlowingLava => Fluid::Lava,
        other => other,
    }
}

/// `getFluidState` for the state at `offset` of `block`, from the nearest
/// class in its lineage that overrides it.
pub fn fluid_state(block: &Block, offset: u32) -> FluidState {
    let water = FluidState::source(Fluid::Water, false);
    for class in block.class.lineage() {
        match class {
            BlockClass::LiquidBlock => {
                // `LiquidBlock.stateCache`: the source, then flowing amounts
                // seven down to one, then a falling flow of eight.
                let level: u8 = block
                    .value(offset, &LEVEL)
                    .and_then(|level| level.parse().ok())
                    .unwrap_or(0);
                return match level.min(8) {
                    0 => FluidState::source(block.fluid, false),
                    8 => FluidState::flowing(block.fluid, 8, true),
                    level => FluidState::flowing(block.fluid, 8 - level, false),
                };
            }
            BlockClass::BubbleColumnBlock
            | BlockClass::KelpBlock
            | BlockClass::KelpPlantBlock
            | BlockClass::SeagrassBlock
            | BlockClass::TallSeagrassBlock => return water,
            // Waterlogged: water; otherwise the parent's.
            BlockClass::WaterloggedTransparentBlock if block.flag(offset, &WATERLOGGED) => {
                return FluidState::source(Fluid::Water, true);
            }
            class if WATERLOGGABLE.contains(&class) && block.flag(offset, &WATERLOGGED) => {
                return water;
            }
            _ => {}
        }
    }
    FluidState::EMPTY
}

/// The classes whose `getFluidState` is water while waterlogged and their
/// parent's otherwise.
const WATERLOGGABLE: &[BlockClass] = &[
    BlockClass::AmethystClusterBlock,
    BlockClass::BarrierBlock,
    BlockClass::BaseCoralPlantTypeBlock,
    BlockClass::BaseRailBlock,
    BlockClass::BigDripleafBlock,
    BlockClass::BigDripleafStemBlock,
    BlockClass::CampfireBlock,
    BlockClass::CandleBlock,
    BlockClass::ChainBlock,
    BlockClass::ChestBlock,
    BlockClass::ConduitBlock,
    BlockClass::CopperGolemStatueBlock,
    BlockClass::CrossCollisionBlock,
    BlockClass::DecoratedPotBlock,
    BlockClass::DriedGhastBlock,
    BlockClass::EnderChestBlock,
    BlockClass::HangingRootsBlock,
    BlockClass::HeavyCoreBlock,
    BlockClass::LadderBlock,
    BlockClass::LanternBlock,
    BlockClass::LeavesBlock,
    BlockClass::LightBlock,
    BlockClass::LightningRodBlock,
    BlockClass::MangrovePropaguleBlock,
    BlockClass::MangroveRootsBlock,
    BlockClass::MultifaceBlock,
    BlockClass::ScaffoldingBlock,
    BlockClass::SculkSensorBlock,
    BlockClass::SculkShriekerBlock,
    BlockClass::SeaPickleBlock,
    BlockClass::ShelfBlock,
    BlockClass::SignBlock,
    BlockClass::SlabBlock,
    BlockClass::SmallDripleafBlock,
    BlockClass::SpeleothemBlock,
    BlockClass::StairBlock,
    BlockClass::TrapDoorBlock,
    BlockClass::WallBlock,
];
