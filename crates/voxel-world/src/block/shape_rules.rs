//! A block's shapes as its class works them out (`getShape` and
//! `getCollisionShape`): the nearest class in the block's lineage that
//! declares the method answers, and a block no class answers for has the
//! reference's defaults. A class not written here answers `None`, and the
//! state cache takes that state's shape from the reference's table until it
//! is.

mod connected;
mod hinged;

use voxel_core::direction::Direction;
use voxel_core::shape::{Shapes, VoxelShape};
use voxel_data::block::classes::{BlockClass, Method};
use voxel_data::block::definition::Property;
use voxel_data::block::registry::Block;

/// The shape a `Shapes.rotate*` map holds for a direction.
fn arm(shapes: &[(Direction, VoxelShape)], direction: Direction) -> &VoxelShape {
    &shapes
        .iter()
        .find(|(own, _)| *own == direction)
        .expect("the map holds every direction asked for")
        .1
}

/// A direction property's value.
fn facing(block: &Block, offset: u32, property: &Property) -> Direction {
    block
        .value(offset, property)
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

/// `getShape` for the state at `offset` of `block`, with no level and no
/// entity; `None` when its class is not written.
pub fn outline(block: &Block, offset: u32) -> Option<VoxelShape> {
    match owner(block, Method::GetShape) {
        // `BlockBehaviour.getShape`: the whole block.
        None => Some(Shapes::block()),
        Some(class) => class_outline(class, block, offset),
    }
}

/// `getCollisionShape`, which is the outline unless the block has no
/// collision, for the state at `offset` of `block`; `None` when its class
/// is not written.
pub fn collision(block: &Block, offset: u32) -> Option<VoxelShape> {
    match owner(block, Method::GetCollisionShape) {
        // `BlockBehaviour.getCollisionShape`.
        None if block.has_collision => outline(block, offset),
        None => Some(Shapes::empty()),
        Some(class) => class_collision(class, block, offset),
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
        _ => return None,
    })
}

fn class_collision(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    Some(match class {
        BlockClass::WallBlock => connected::wall_collision(block, offset),
        BlockClass::CrossCollisionBlock => {
            connected::cross_collision(block, offset, block.class.is(BlockClass::FenceBlock))
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::{table_collision_shape, table_outline_shape};
    use voxel_data::block::registry::BLOCKS;

    /// A shape's boxes as bits, and whether it is the reference's own whole
    /// block or empty shape, which it tells from another by identity.
    fn fingerprint(shape: &VoxelShape) -> (u8, Vec<u64>) {
        let kind = if shape.equals_java(&Shapes::empty()) {
            1
        } else if shape.equals_java(&Shapes::block()) {
            2
        } else {
            0
        };
        let boxes = shape
            .to_aabbs()
            .iter()
            .flat_map(|aabb| {
                [
                    aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
                ]
                .map(f64::to_bits)
            })
            .collect();
        (kind, boxes)
    }

    /// Every state a class computes has the shape the reference's table
    /// holds for it; the ones no class computes yet are counted.
    #[test]
    fn computed_shapes_are_the_tables() {
        let (mut outlines, mut collisions, mut states) = (0, 0, 0);
        let mut wrong = Vec::new();
        for block in &BLOCKS {
            for offset in 0..block.state_count() {
                states += 1;
                let id = block.first_state_id() + offset;
                if let Some(shape) = outline(block, offset) {
                    outlines += 1;
                    if fingerprint(&shape) != fingerprint(&table_outline_shape(id)) {
                        wrong.push(format!("{} outline at {offset}", block.name));
                    }
                }
                if let Some(shape) = collision(block, offset) {
                    collisions += 1;
                    if fingerprint(&shape) != fingerprint(&table_collision_shape(id)) {
                        wrong.push(format!("{} collision at {offset}", block.name));
                    }
                }
            }
        }
        println!("shapes computed: outline {outlines}/{states}, collision {collisions}/{states}");
        assert!(
            wrong.is_empty(),
            "{} differ, first {:?}",
            wrong.len(),
            &wrong[..wrong.len().min(8)]
        );
    }
}
