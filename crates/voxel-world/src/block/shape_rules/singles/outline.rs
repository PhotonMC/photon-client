use super::*;

mod furniture;
mod mounts;
mod plants;
mod redstone;

/// `getShape` of a class that has one shape it always gives, or a few it
/// picks from by the state; `None` for a class this does not cover.
pub fn outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    mounts::outline(class, block, offset)
        .or_else(|| plants::outline(class, block, offset))
        .or_else(|| redstone::outline(class, block, offset))
        .or_else(|| furniture::outline(class, block, offset))
}
