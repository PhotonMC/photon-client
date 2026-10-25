//! `SelectableSlotContainer.getHitSlot`: the slot of a shelf or chiseled bookshelf a click lands in.

use super::*;

/// `SelectableSlotContainer.getSection`: the part of the face, `0..sections`, a coordinate in a
/// block lies in.
fn section(coordinate: f32, sections: i32) -> i32 {
    let pixel = coordinate * 16.0_f32;
    let size = 16.0_f32 / sections as f32;
    ((pixel / size).floor() as i32).clamp(0, sections - 1)
}

/// The slot hit on the face a block faces, row by row from the top; `None` for a click on another
/// face. `block` is the block's position, `location` the click and `(rows, columns)` its slots.
pub(super) fn hit_slot(
    state: BlockStateId,
    block: (i32, i32, i32),
    (face, location): (Direction, [f64; 3]),
    (rows, columns): (i32, i32),
) -> Option<i32> {
    let facing = Direction::from_name(property(state, "facing")?)?;
    if facing != face {
        return None;
    }
    let (dx, dy, dz) = face.step();
    let relative = [
        location[0] - f64::from(block.0 + dx),
        location[1] - f64::from(block.1 + dy),
        location[2] - f64::from(block.2 + dz),
    ];
    let x = match face {
        Direction::North => (1.0 - relative[0]) as f32,
        Direction::South => relative[0] as f32,
        Direction::West => relative[2] as f32,
        Direction::East => (1.0 - relative[2]) as f32,
        Direction::Down | Direction::Up => return None,
    };
    let y = relative[1] as f32;
    let row = section(1.0_f32 - y, rows);
    let column = section(x, columns);
    Some(column + row * columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shelf(facing: &str) -> BlockStateId {
        let index = registry::block_index("minecraft:chiseled_bookshelf").expect("a block");
        let chosen = [("facing".to_owned(), facing.to_owned())]
            .into_iter()
            .chain((0..6).map(|slot| (format!("slot_{slot}_occupied"), "false".to_owned())))
            .collect();
        BLOCKS[index].state_id(&chosen).expect("a state")
    }

    /// The north face is read from its east side, the first slot at the top left.
    #[test]
    fn the_slot_follows_the_face_and_the_row() {
        let state = shelf("north");
        let at = (0, 64, 2);
        let hit = |x: f64, y: f64| hit_slot(state, at, (Direction::North, [x, y, 2.0]), (2, 3));
        assert_eq!(hit(0.9, 64.9), Some(0));
        assert_eq!(hit(0.5, 64.9), Some(1));
        assert_eq!(hit(0.1, 64.9), Some(2));
        assert_eq!(hit(0.9, 64.1), Some(3));
        assert_eq!(hit(0.1, 64.1), Some(5));
        assert_eq!(
            hit_slot(state, at, (Direction::South, [0.5, 64.5, 3.0]), (2, 3)),
            None
        );
        assert_eq!(
            hit_slot(state, at, (Direction::Up, [0.5, 65.0, 2.5]), (2, 3)),
            None
        );
    }
}
