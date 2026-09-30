//! Blocks as the reference builds them: the declarations in `voxel-data`
//! and what each class works out from them, the way `Block`, its subclasses
//! and `BlockStateBase` do.

pub mod culling;
pub mod fluid;
pub mod shapes;
pub mod state;
pub mod tags;

pub use culling::should_render_face;
pub use fluid::FluidState;
pub use state::{StateInfo, state, state_or_air, states};

use voxel_data::block::classes::BlockClass;
use voxel_data::block::registry::Block;

/// Whether a fluid's side shows its overlay sprite against the block:
/// `LiquidBlockRenderer` checks for `HalfTransparentBlock` and `LeavesBlock`.
pub fn shows_fluid_overlay(block: &Block) -> bool {
    block.class.is(BlockClass::HalfTransparentBlock) || block.class.is(BlockClass::LeavesBlock)
}
