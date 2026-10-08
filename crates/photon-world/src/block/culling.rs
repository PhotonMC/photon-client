//! Whether a block face is hidden by its neighbour (`Block.shouldRenderFace`
//! and the classes' own `skipRendering`).

use std::sync::atomic::{AtomicBool, Ordering};

use photon_core::direction::{Axis, Direction};
use photon_data::block::blocks::id;
use photon_data::block::classes::BlockClass;
use photon_data::block::properties::{EAST, NORTH, SOUTH, WEST};

use super::state::{EMPTY_FACE, FULL_FACE, StateInfo, states};
use super::tags::BlockTags;

/// `LeavesBlock.cutoutLeaves`, which the graphics options set; leaves hide
/// their shared faces only when it is off.
static CUTOUT_LEAVES: AtomicBool = AtomicBool::new(true);

/// `LeavesBlock.setCutoutLeaves`.
pub fn set_cutout_leaves(cutout: bool) {
    CUTOUT_LEAVES.store(cutout, Ordering::Relaxed);
}

/// `CrossCollisionBlock.PROPERTY_BY_DIRECTION`, which has only the sides.
fn side_property(direction: Direction) -> Option<&'static photon_data::block::registry::Property> {
    match direction {
        Direction::North => Some(&NORTH),
        Direction::East => Some(&EAST),
        Direction::South => Some(&SOUTH),
        Direction::West => Some(&WEST),
        Direction::Up | Direction::Down => None,
    }
}

/// `BlockState.skipRendering`: whether the pair hides its shared face
/// whatever their shapes.
pub fn skip_rendering(
    state: &StateInfo,
    neighbour: &StateInfo,
    direction: Direction,
    tags: &BlockTags,
) -> bool {
    let block = state.block();
    for class in block.class.lineage() {
        match class {
            BlockClass::HalfTransparentBlock | BlockClass::PowderSnowBlock => {
                if neighbour.block == state.block {
                    return true;
                }
            }
            BlockClass::LeavesBlock => {
                if !CUTOUT_LEAVES.load(Ordering::Relaxed)
                    && neighbour.block().class.is(BlockClass::LeavesBlock)
                {
                    return true;
                }
            }
            BlockClass::IronBarsBlock => {
                if iron_bars_join(state, neighbour, direction, tags) {
                    return true;
                }
            }
            BlockClass::LiquidBlock => return neighbour.fluid.is_same(block.fluid),
            BlockClass::MangroveRootsBlock => {
                return usize::from(neighbour.block) == id::MANGROVE_ROOTS
                    && direction.axis() == Axis::Y;
            }
            _ => {}
        }
    }
    false
}

/// `IronBarsBlock.skipRendering`'s own test: bars joined above and below,
/// or across a side both reach towards.
fn iron_bars_join(
    state: &StateInfo,
    neighbour: &StateInfo,
    direction: Direction,
    tags: &BlockTags,
) -> bool {
    let block = state.block();
    let other = neighbour.block();
    let opposite = side_property(direction.opposite());
    let joins = neighbour.block == state.block
        || tags.contains(usize::from(neighbour.block), "minecraft:bars")
            && tags.contains(usize::from(state.block), "minecraft:bars")
            && opposite.is_some_and(|property| {
                other.value(u32::from(neighbour.offset), property).is_some()
            });
    if !joins {
        return false;
    }
    if direction.axis() == Axis::Y {
        return true;
    }
    side_property(direction).is_some_and(|property| block.flag(u32::from(state.offset), property))
        && opposite.is_some_and(|property| other.flag(u32::from(neighbour.offset), property))
}

/// `Block.shouldRenderFace`: whether `state`'s face towards `direction` is
/// drawn with `neighbour` beside it. An unloaded neighbour hides nothing.
pub fn should_render_face(
    state: u32,
    neighbour: Option<u32>,
    direction: Direction,
    tags: &BlockTags,
) -> bool {
    let states = states();
    let (Some(own), Some(neighbour)) = (states.get(state), neighbour.and_then(|id| states.get(id)))
    else {
        return true;
    };
    let occluder = neighbour.face(direction.opposite());
    if occluder == FULL_FACE {
        return false;
    }
    if skip_rendering(own, neighbour, direction, tags) {
        return false;
    }
    if occluder == EMPTY_FACE {
        return true;
    }
    let shape = own.face(direction);
    if shape == EMPTY_FACE {
        return true;
    }
    states.joins(shape, occluder)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(name: &str) -> u32 {
        photon_data::block::registry::block(name)
            .expect(name)
            .default_state_id()
    }

    #[test]
    fn leaves_hide_nothing_behind_them() {
        let log = state("minecraft:oak_log");
        let leaves = state("minecraft:oak_leaves");
        for direction in [Direction::Up, Direction::North, Direction::East] {
            assert!(should_render_face(
                log,
                Some(leaves),
                direction,
                &BlockTags::default()
            ));
        }
    }

    #[test]
    fn full_blocks_and_matching_glass_hide_their_shared_face() {
        let stone = state("minecraft:stone");
        let glass = state("minecraft:glass");
        assert!(!should_render_face(
            stone,
            Some(stone),
            Direction::West,
            &BlockTags::default()
        ));
        assert!(!should_render_face(
            glass,
            Some(glass),
            Direction::West,
            &BlockTags::default()
        ));
        assert!(should_render_face(
            stone,
            Some(glass),
            Direction::West,
            &BlockTags::default()
        ));
        assert!(should_render_face(
            stone,
            None,
            Direction::West,
            &BlockTags::default()
        ));
    }

    #[test]
    fn a_bottom_slab_hides_only_the_face_it_covers() {
        let stone = state("minecraft:stone");
        let slab = state("minecraft:stone_slab");
        // A bottom slab above covers the stone's top; beside it, only half
        // of the side, so the side still shows.
        assert!(!should_render_face(
            stone,
            Some(slab),
            Direction::Up,
            &BlockTags::default()
        ));
        assert!(should_render_face(
            stone,
            Some(slab),
            Direction::Down,
            &BlockTags::default()
        ));
        assert!(should_render_face(
            stone,
            Some(slab),
            Direction::North,
            &BlockTags::default()
        ));
    }
}
