//! Moving the focus with the arrow keys (`ContainerEventHandler`'s
//! `nextFocusPathInDirection`, `ScreenRectangle`, `ScreenDirection`): the
//! widget next to the focused one in the direction, found by where their
//! rectangles lie.

/// `ScreenDirection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    fn positive(self) -> bool {
        matches!(self, Self::Down | Self::Right)
    }

    /// `isAfter`.
    fn is_after(self, a: i32, b: i32) -> bool {
        if self.positive() { a > b } else { b > a }
    }

    /// `coordinateValueComparator`.
    fn compare(self, a: i32, b: i32) -> std::cmp::Ordering {
        if a == b {
            std::cmp::Ordering::Equal
        } else if self.positive() == (a < b) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

/// `ScreenRectangle`: where a widget is, and how big.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    /// `getBoundInDirection`: the last row or column inside on that side.
    fn bound(self, direction: Direction) -> i32 {
        let (start, length) = if direction.horizontal() {
            (self.x, self.width)
        } else {
            (self.y, self.height)
        };
        if direction.positive() {
            start + length - 1
        } else {
            start
        }
    }

    /// `overlapsInAxis`: the two reach the same rows (or columns) of the
    /// other axis, `horizontal` naming the axis they are compared along.
    fn overlaps(self, other: Self, horizontal: bool) -> bool {
        let (negative, positive) = if horizontal {
            (Direction::Left, Direction::Right)
        } else {
            (Direction::Up, Direction::Down)
        };
        self.bound(negative).max(other.bound(negative))
            <= self.bound(positive).min(other.bound(positive))
    }

    fn center(self, horizontal: bool) -> i32 {
        let (negative, positive) = if horizontal {
            (Direction::Left, Direction::Right)
        } else {
            (Direction::Up, Direction::Down)
        };
        (self.bound(positive) + self.bound(negative)) / 2
    }

    /// `getBorder`: a line one unit thick just outside the side.
    pub fn border(self, direction: Direction) -> Self {
        let first = self.bound(direction);
        let (second, length) = if direction.horizontal() {
            (self.bound(Direction::Up), self.height)
        } else {
            (self.bound(Direction::Left), self.width)
        };
        let (step_x, step_y) = match direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        if direction.horizontal() {
            Self {
                x: first + step_x,
                y: second + step_y,
                width: 1,
                height: length,
            }
        } else {
            Self {
                x: second + step_x,
                y: first + step_y,
                width: length,
                height: 1,
            }
        }
    }
}

/// A child of a container that arrow navigation looks at.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    pub rectangle: Rectangle,
    /// Whether the child can take the focus (`nextFocusPath` is not null).
    pub focusable: bool,
}

/// `handleArrowNavigation` of a container whose children are in this order:
/// the one to focus next, given the one that has the focus (if any) and the
/// container's own rectangle.
pub fn next_in_direction(
    children: &[Candidate],
    focused: Option<usize>,
    direction: Direction,
    container: Rectangle,
) -> Option<usize> {
    let focused_rectangle = match focused {
        Some(index) => children[index].rectangle.border(direction),
        None => container.border(direction.opposite()),
    };
    in_direction(children, focused, direction, focused_rectangle)
}

fn in_direction(
    children: &[Candidate],
    excluded: Option<usize>,
    direction: Direction,
    focused: Rectangle,
) -> Option<usize> {
    let horizontal = direction.horizontal();
    let focused_first = focused.bound(direction.opposite());
    let mut potential: Vec<usize> = children
        .iter()
        .enumerate()
        .filter(|(index, child)| {
            Some(*index) != excluded && child.rectangle.overlaps(focused, !horizontal) && {
                let first = child.rectangle.bound(direction.opposite());
                direction.is_after(first, focused_first)
                    || (first == focused_first
                        && direction
                            .is_after(child.rectangle.bound(direction), focused.bound(direction)))
            }
        })
        .map(|(index, _)| index)
        .collect();
    // By the near edge along the direction, then by the near edge across it
    // towards the positive side (a stable sort, as the reference's).
    let across = if horizontal {
        Direction::Down
    } else {
        Direction::Right
    };
    potential.sort_by(|&a, &b| {
        let (a, b) = (children[a].rectangle, children[b].rectangle);
        direction
            .compare(a.bound(direction.opposite()), b.bound(direction.opposite()))
            .then_with(|| across.compare(a.bound(across.opposite()), b.bound(across.opposite())))
    });
    if let Some(found) = potential
        .into_iter()
        .find(|&index| children[index].focusable)
    {
        return Some(found);
    }
    vaguely_in_direction(children, excluded, direction, focused)
}

/// `nextFocusPathVaguelyInDirection`: the nearest child by the distance of
/// the facing sides' middles, among those that lie beyond.
fn vaguely_in_direction(
    children: &[Candidate],
    excluded: Option<usize>,
    direction: Direction,
    focused: Rectangle,
) -> Option<usize> {
    let horizontal = direction.horizontal();
    let side = |rectangle: Rectangle, towards: Direction| {
        let along = rectangle.bound(towards);
        let across = rectangle.center(!horizontal);
        if horizontal {
            (along, across)
        } else {
            (across, along)
        }
    };
    let coordinate = |point: (i32, i32)| if horizontal { point.0 } else { point.1 };
    let from = side(focused, direction);
    let mut potential: Vec<(usize, i64)> = children
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != excluded)
        .filter_map(|(index, child)| {
            let to = side(child.rectangle, direction.opposite());
            direction
                .is_after(coordinate(to), coordinate(from))
                .then(|| {
                    let (dx, dy) = (i64::from(from.0 - to.0), i64::from(from.1 - to.1));
                    (index, dx * dx + dy * dy)
                })
        })
        .collect();
    potential.sort_by_key(|&(_, distance)| distance);
    potential
        .into_iter()
        .find(|&(index, _)| children[index].focusable)
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i32, y: i32, width: i32, height: i32) -> Candidate {
        Candidate {
            rectangle: Rectangle {
                x,
                y,
                width,
                height,
            },
            focusable: true,
        }
    }

    #[test]
    fn the_focus_goes_to_the_neighbour_that_shares_rows_or_columns() {
        // Two buttons over a wide one.
        let children = [at(10, 10, 98, 20), at(116, 10, 98, 20), at(10, 34, 204, 20)];
        let screen = Rectangle {
            x: 0,
            y: 0,
            width: 427,
            height: 240,
        };
        let go = |from, direction| next_in_direction(&children, from, direction, screen);
        assert_eq!(go(Some(0), Direction::Right), Some(1));
        assert_eq!(go(Some(0), Direction::Down), Some(2));
        assert_eq!(go(Some(1), Direction::Down), Some(2));
        assert_eq!(go(Some(2), Direction::Up), Some(0));
        assert_eq!(go(Some(0), Direction::Left), None);
        // No focus yet: down enters from the top, the first by position.
        assert_eq!(go(None, Direction::Down), Some(0));
    }
}
