//! Whether a block face is hidden by its neighbour, as the reference decides.
//!
//! The decision reads three tables captured from the reference (see
//! [`voxel_data::block::face_occlusion`]): each state's face occlusion
//! shapes, whether one shape sticks out past another, and the block pairs
//! that hide their shared face whatever their shapes. The few pairs whose
//! hiding depends on the states carry their rule here.

use voxel_core::direction::Direction;
use voxel_data::block::face_occlusion::{BLOCK_TUPLES, JOINS, SKIPS, TUPLES};
use voxel_data::block::registry;

fn slot(direction: Direction) -> usize {
    direction as usize
}

/// The face occlusion shape kind a state shows towards a direction: zero for
/// nothing, one for the whole block, higher for a partial shape.
pub fn occlusion_kind(state: u32, direction: Direction) -> Option<u8> {
    let tuple = voxel_data::block::state_value(&BLOCK_TUPLES, state)?;
    Some(TUPLES[usize::from(tuple)][slot(direction)])
}

fn property(state: u32, name: &str) -> Option<&'static str> {
    let (block, offset) = registry::block_for_state(state)?;
    block
        .properties_at(offset)?
        .into_iter()
        .find(|(property, _)| *property == name)
        .map(|(_, value)| value)
}

fn side_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

/// `BlockState.skipRendering` for the pair.
fn skip_rendering(state: u32, neighbour: u32, direction: Direction) -> bool {
    let (Some((block, _)), Some((other, _))) = (
        registry::block_index_for_state(state),
        registry::block_index_for_state(neighbour),
    ) else {
        return false;
    };
    let Some(pair) = SKIPS
        .iter()
        .find(|pair| usize::from(pair.block) == block && usize::from(pair.neighbour) == other)
    else {
        return false;
    };
    let listed = pair.directions & (1 << slot(direction)) != 0;
    if pair.always || !listed {
        return listed;
    }
    // Bars and panes: joined above and below, and across a side only when
    // both reach towards each other.
    if property(state, "north").is_some() && property(neighbour, "north").is_some() {
        if matches!(direction, Direction::Up | Direction::Down) {
            return true;
        }
        return property(state, side_name(direction)) == Some("true")
            && property(neighbour, side_name(direction.opposite())) == Some("true");
    }
    // The rest (mangrove roots) depend on the direction alone; the fluid
    // pairs never reach here, because fluids are not drawn from models.
    true
}

/// `Block.shouldRenderFace`: whether `state`'s face towards `direction` is
/// drawn with `neighbour` beside it. An unloaded neighbour hides nothing.
pub fn should_render_face(state: u32, neighbour: Option<u32>, direction: Direction) -> bool {
    let Some(neighbour) = neighbour else {
        return true;
    };
    let Some(occluder) = occlusion_kind(neighbour, direction.opposite()) else {
        return true;
    };
    if occluder == 1 {
        return false;
    }
    if skip_rendering(state, neighbour, direction) {
        return false;
    }
    if occluder == 0 {
        return true;
    }
    let Some(shape) = occlusion_kind(state, direction) else {
        return true;
    };
    if shape == 0 {
        return true;
    }
    JOINS[usize::from(shape)].as_bytes()[usize::from(occluder)] == b'1'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(name: &str) -> u32 {
        registry::block(name).expect(name).default_state_id()
    }

    #[test]
    fn leaves_hide_nothing_behind_them() {
        let log = state("minecraft:oak_log");
        let leaves = state("minecraft:oak_leaves");
        for direction in [Direction::Up, Direction::North, Direction::East] {
            assert!(should_render_face(log, Some(leaves), direction));
        }
    }

    #[test]
    fn full_blocks_and_matching_glass_hide_their_shared_face() {
        let stone = state("minecraft:stone");
        let glass = state("minecraft:glass");
        assert!(!should_render_face(stone, Some(stone), Direction::West));
        assert!(!should_render_face(glass, Some(glass), Direction::West));
        assert!(should_render_face(stone, Some(glass), Direction::West));
        assert!(should_render_face(stone, None, Direction::West));
    }

    #[test]
    fn a_bottom_slab_hides_only_the_face_it_covers() {
        let stone = state("minecraft:stone");
        let slab = state("minecraft:stone_slab");
        // A bottom slab above covers the stone's top; beside it, only half
        // of the side, so the side still shows.
        assert!(!should_render_face(stone, Some(slab), Direction::Up));
        assert!(should_render_face(stone, Some(slab), Direction::Down));
        assert!(should_render_face(stone, Some(slab), Direction::North));
    }
}
