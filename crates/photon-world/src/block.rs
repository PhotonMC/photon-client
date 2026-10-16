//! Blocks as the reference builds them: the declarations in `photon-data`
//! and what each class works out from them, the way `Block`, its subclasses
//! and `BlockStateBase` do.

pub(crate) mod chest;
pub(crate) mod entity;
pub(crate) mod piston;
pub mod registry;
pub mod rules;

pub mod culling;
pub mod fluid;
pub mod shape_rules;
pub use shape_rules::moves_with_position;
pub mod shapes;
pub mod state;
pub mod tags;

pub use culling::should_render_face;
pub use fluid::FluidState;
pub use state::{StateInfo, state, state_or_air, states};

use photon_data::block::classes::BlockClass;
use photon_data::block::definition::OffsetType;
use photon_data::block::registry::Block;

/// Whether a fluid's side shows its overlay sprite against the block:
/// `LiquidBlockRenderer` checks for `HalfTransparentBlock` and `LeavesBlock`.
pub fn shows_fluid_overlay(block: &Block) -> bool {
    block.class.is(BlockClass::HalfTransparentBlock) || block.class.is(BlockClass::LeavesBlock)
}

/// `getMaxHorizontalOffset`.
pub fn max_horizontal_offset(block: &Block) -> f32 {
    if block.class.is(BlockClass::SpeleothemBlock) {
        shapes::speleothem_max_horizontal_offset()
    } else {
        0.25
    }
}

/// `getMaxVerticalOffset`.
pub fn max_vertical_offset(block: &Block) -> f32 {
    if block.class.is(BlockClass::SmallDripleafBlock) {
        0.1
    } else {
        0.2
    }
}

/// `BlockState.getOffset`: the random nudge a block with an offset type
/// (plants, bamboo, dripstone) takes off the grid at a position.
pub fn offset(block: &Block, position: [i32; 3]) -> [f64; 3] {
    if block.offset_type == OffsetType::None {
        return [0.0; 3];
    }
    let horizontal_max = f64::from(max_horizontal_offset(block));
    let seed = photon_core::math::mth_get_seed(position[0], 0, position[2]);
    let axis = |bits: i64| {
        let value = (f64::from((bits & 15) as f32 / 15.0) - 0.5) * 0.5;
        if value < -horizontal_max {
            -horizontal_max
        } else {
            value.min(horizontal_max)
        }
    };
    let y = if block.offset_type == OffsetType::Xyz {
        (f64::from((seed >> 4 & 15) as f32 / 15.0) - 1.0) * f64::from(max_vertical_offset(block))
    } else {
        0.0
    };
    [axis(seed), y, axis(seed >> 8)]
}

/// `BlockBehaviour.getSeed`: the number a block's model randomness is
/// seeded from. Two-block plants and doors share their lower half's, and a
/// bed its head's column.
pub fn seed(state: &StateInfo, position: [i32; 3]) -> i64 {
    use photon_data::block::properties::{BED_PART, DOUBLE_BLOCK_HALF, HORIZONTAL_FACING};
    let block = state.block();
    let offset = u32::from(state.offset);
    let [x, y, z] = position;
    for class in block.class.lineage() {
        match class {
            BlockClass::DoublePlantBlock | BlockClass::DoorBlock => {
                let below = i32::from(block.value(offset, &DOUBLE_BLOCK_HALF) != Some("lower"));
                return photon_core::math::mth_get_seed(x, y - below, z);
            }
            BlockClass::BedBlock => {
                let step = i32::from(block.value(offset, &BED_PART) != Some("head"));
                let (dx, _, dz) = block
                    .value(offset, &HORIZONTAL_FACING)
                    .and_then(photon_core::direction::Direction::from_name)
                    .map_or((0, 0, 0), photon_core::direction::Direction::step);
                return photon_core::math::mth_get_seed(x + dx * step, y, z + dz * step);
            }
            _ => {}
        }
    }
    photon_core::math::mth_get_seed(x, y, z)
}
