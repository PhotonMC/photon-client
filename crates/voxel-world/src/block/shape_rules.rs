//! A block's shapes as its class works them out (`getShape` and
//! `getCollisionShape`): the nearest class in the block's lineage that
//! declares the method answers, and a block no class answers for has the
//! reference's defaults.

/// A shape or a map of them built the first time it is asked for.
macro_rules! cached {
    ($ty:ty, $make:expr) => {{
        static CELL: std::sync::OnceLock<$ty> = std::sync::OnceLock::new();
        CELL.get_or_init(|| $make)
    }};
}

mod connected;
mod hinged;
mod singles;

use voxel_core::direction::Direction;
use voxel_core::shape::{Shapes, VoxelShape};
use voxel_data::block::classes::{BlockClass, Method};
use voxel_data::block::definition::OffsetType;
use voxel_data::block::registry::Block;

/// The shape a `Shapes.rotate*` map holds for a key.
fn arm<K: PartialEq>(shapes: &[(K, VoxelShape)], key: K) -> &VoxelShape {
    &shapes
        .iter()
        .find(|(own, _)| *own == key)
        .expect("the map holds every key asked for")
        .1
}

/// A property's value by its name; `None` when the block has no such one.
fn prop(block: &Block, offset: u32, name: &str) -> Option<&'static str> {
    let property = block.properties.iter().find(|own| own.name == name)?;
    block.value(offset, property)
}

fn flag(block: &Block, offset: u32, name: &str) -> bool {
    prop(block, offset, name) == Some("true")
}

/// An integer property's value; zero when the block has no such one.
fn number(block: &Block, offset: u32, name: &str) -> usize {
    prop(block, offset, name).map_or(0, |value| value.parse().expect("an integer property"))
}

/// A direction property's value.
fn facing(block: &Block, offset: u32, name: &str) -> Direction {
    prop(block, offset, name)
        .and_then(Direction::from_name)
        .expect("a facing property holds a direction")
}

/// The class whose own `method` the block runs; `None` for the one
/// `BlockBehaviour` itself has.
fn owner(block: &Block, method: Method) -> Option<BlockClass> {
    block
        .class
        .lineage()
        .find(|class| class.declares().contains(&method))
}

/// Whether the class moves its shapes by the block's random offset
/// (`shape.move(state.getOffset(pos))`); the others keep them where the
/// block's cell is, though the block is drawn off the grid.
pub fn moves_with_position(block: &Block) -> bool {
    block.offset_type != OffsetType::None
        && matches!(
            owner(block, Method::GetShape),
            Some(
                BlockClass::FlowerBlock
                    | BlockClass::MangrovePropaguleBlock
                    | BlockClass::SpeleothemBlock
                    | BlockClass::BambooStalkBlock
                    | BlockClass::BambooSaplingBlock
            )
        )
}

/// `getShape` for the state at `offset` of `block`, with no level and no
/// entity and before the block's random offset.
pub fn outline(block: &Block, offset: u32) -> VoxelShape {
    match owner(block, Method::GetShape) {
        // `BlockBehaviour.getShape`: the whole block.
        None => Shapes::block(),
        Some(class) => class_outline(class, block, offset)
            .unwrap_or_else(|| panic!("no getShape is written for {class:?}")),
    }
}

/// `getCollisionShape`, which is the outline unless the block has no
/// collision, for the state at `offset` of `block`, before the block's
/// random offset where it moves with position.
pub fn collision(block: &Block, offset: u32) -> VoxelShape {
    match owner(block, Method::GetCollisionShape) {
        // `BlockBehaviour.getCollisionShape`.
        None if block.has_collision => outline(block, offset),
        None => Shapes::empty(),
        Some(class) => class_collision(class, block, offset)
            .unwrap_or_else(|| panic!("no getCollisionShape is written for {class:?}")),
    }
}

fn class_outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    Some(match class {
        BlockClass::WallBlock => connected::wall_outline(block, offset),
        BlockClass::CrossCollisionBlock => {
            connected::cross_outline(block, offset, block.class.is(BlockClass::FenceBlock))
        }
        BlockClass::StairBlock => hinged::stairs_outline(block, offset),
        BlockClass::DoorBlock => hinged::door_outline(block, offset),
        BlockClass::TrapDoorBlock => hinged::trapdoor_outline(block, offset),
        _ => return singles::outline(class, block, offset),
    })
}

fn class_collision(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    Some(match class {
        BlockClass::WallBlock => connected::wall_collision(block, offset),
        BlockClass::CrossCollisionBlock => {
            connected::cross_collision(block, offset, block.class.is(BlockClass::FenceBlock))
        }
        _ => return singles::collision(class, block, offset),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_data::block::registry::{BLOCKS, block};

    fn state(name: &str, properties: &[(&str, &str)]) -> (&'static Block, u32) {
        let block = block(name).expect("a block");
        let chosen = properties
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        let id = block.state_id(&chosen).expect("a state");
        (block, id - block.first_state_id())
    }

    /// Every state of every block has both shapes; a class whose shape is
    /// not written panics.
    #[test]
    fn every_state_has_shapes() {
        for block in &BLOCKS {
            for offset in 0..block.state_count() {
                outline(block, offset);
                collision(block, offset);
            }
        }
    }

    /// Bamboo collides where it stands, which is off the grid by its own
    /// random offset: the shape differs by what the two positions' offsets do.
    #[test]
    fn bamboo_collides_where_its_offset_puts_it() {
        let (bamboo, offset) = state("minecraft:bamboo", &[]);
        let id = bamboo.first_state_id() + offset;
        let at = |position: [i32; 3]| {
            crate::block_state::collision_shape_at(id, position)
                .bounds()
                .expect("a box")
        };
        let (near, far) = ([0, 64, 0], [13, 64, -7]);
        let (near_shift, far_shift) = (
            super::super::offset(bamboo, near),
            super::super::offset(bamboo, far),
        );
        assert_ne!(near_shift, far_shift);
        let gap = at(far).min_x - at(near).min_x;
        assert!((gap - (far_shift[0] - near_shift[0])).abs() < 1.0e-12);
    }

    #[test]
    fn a_bottom_slab_is_the_lower_half() {
        let (slab, offset) = state("minecraft:stone_slab", &[("type", "bottom")]);
        let bounds = outline(slab, offset).bounds().expect("a box");
        assert_eq!((bounds.min_y, bounds.max_y), (0.0, 0.5));
    }

    #[test]
    fn a_fence_collides_higher_than_it_shows() {
        let (fence, offset) = state("minecraft:oak_fence", &[]);
        let outline = outline(fence, offset).bounds().expect("a box");
        let collision = collision(fence, offset).bounds().expect("a box");
        assert_eq!((outline.max_y, collision.max_y), (1.0, 1.5));
    }

    #[test]
    fn a_top_stair_is_the_bottom_one_turned_over() {
        let (bottom, at) = state("minecraft:oak_stairs", &[("half", "bottom")]);
        let (top, above) = state("minecraft:oak_stairs", &[("half", "top")]);
        let bottom = outline(bottom, at).bounds().expect("a box");
        let top = outline(top, above).bounds().expect("a box");
        assert_eq!((bottom.min_y, bottom.max_y), (0.0, 1.0));
        assert_eq!((top.min_y, top.max_y), (0.0, 1.0));
        assert_eq!(
            outline_area(&state("minecraft:oak_stairs", &[("half", "top")])),
            0.75
        );
    }

    fn outline_area((block, offset): &(&'static Block, u32)) -> f64 {
        let shape = outline(block, *offset);
        shape
            .to_aabbs()
            .iter()
            .map(|aabb| {
                (aabb.max_x - aabb.min_x) * (aabb.max_y - aabb.min_y) * (aabb.max_z - aabb.min_z)
            })
            .sum()
    }
}
