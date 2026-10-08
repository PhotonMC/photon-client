//! Binary codecs for the pinned reference: Vec3, UUID, BlockPos, long arrays and BitSets.
//!
//! UUID and positions travel as big-endian longs with no length prefix.
//! Long arrays prefix a VarInt length; BitSets reuse that framing through
//! trimmed little-endian words. Each `readLong` is atomic, so a truncated
//! UUID can leave the first half consumed while a truncated position
//! consumes nothing.

use crate::aabb::{Aabb, BoundingBox};
use crate::axis_cycle::AxisCycle;
use crate::direction::{Axis, Direction};
use crate::math::{self, Vec3};
use crate::random::{RandomError, RandomSource};
use crate::wire;

mod bit_set;
pub use bit_set::*;
mod block_pos;
pub use block_pos::*;
mod chunk_pos;
pub use chunk_pos::*;
mod codec;
pub use codec::*;
mod mutable_block_pos;
pub use mutable_block_pos::*;
mod rotation;
pub use rotation::*;
mod section_pos;
pub use section_pos::*;
mod traversal;
pub use traversal::*;
mod uuid;
pub use uuid::*;
