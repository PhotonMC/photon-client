//! Screen layouts (`GridLayout`, `LinearLayout`, `FrameLayout`,
//! `LayoutSettings`): elements put in cells of a grid, each with padding and
//! an alignment in its cell, and the grid put in a screen. The elements'
//! places live in an arena, so a layout can move them as it arranges.

/// An element's place and size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// `LayoutSettings`: the padding around an element in its cell, and where it
/// stands in what is left (0 is the left or top, 1 the right or bottom).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Settings {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub x_align: f32,
    pub y_align: f32,
}

impl Settings {
    /// `padding(left, top, right, bottom)`.
    pub fn padding(mut self, left: i32, top: i32, right: i32, bottom: i32) -> Self {
        (self.left, self.top, self.right, self.bottom) = (left, top, right, bottom);
        self
    }

    pub fn padding_top(mut self, top: i32) -> Self {
        self.top = top;
        self
    }

    pub fn align_horizontally(mut self, x_align: f32) -> Self {
        self.x_align = x_align;
        self
    }
}

/// An element: a place in the arena, or a layout of its own.
#[derive(Debug, Clone)]
pub enum Item {
    Leaf(usize),
    Grid(Box<Grid>),
}

#[derive(Debug, Clone, Default)]
pub struct Arena(pub Vec<Rect>);

impl Arena {
    /// A new element of a size, at the origin.
    pub fn add(&mut self, width: i32, height: i32) -> usize {
        self.0.push(Rect {
            x: 0,
            y: 0,
            width,
            height,
        });
        self.0.len() - 1
    }
}

impl Item {
    pub fn width(&self, arena: &Arena) -> i32 {
        match self {
            Self::Leaf(index) => arena.0[*index].width,
            Self::Grid(grid) => grid.width,
        }
    }

    pub fn height(&self, arena: &Arena) -> i32 {
        match self {
            Self::Leaf(index) => arena.0[*index].height,
            Self::Grid(grid) => grid.height,
        }
    }

    pub fn x(&self, arena: &Arena) -> i32 {
        match self {
            Self::Leaf(index) => arena.0[*index].x,
            Self::Grid(grid) => grid.x,
        }
    }

    pub fn y(&self, arena: &Arena) -> i32 {
        match self {
            Self::Leaf(index) => arena.0[*index].y,
            Self::Grid(grid) => grid.y,
        }
    }

    /// `setX`: a layout moves its children by as much as it moved.
    pub fn set_x(&mut self, arena: &mut Arena, x: i32) {
        match self {
            Self::Leaf(index) => arena.0[*index].x = x,
            Self::Grid(grid) => grid.set_x(arena, x),
        }
    }

    pub fn set_y(&mut self, arena: &mut Arena, y: i32) {
        match self {
            Self::Leaf(index) => arena.0[*index].y = y,
            Self::Grid(grid) => grid.set_y(arena, y),
        }
    }
}

#[derive(Debug, Clone)]
struct Cell {
    item: Item,
    row: usize,
    column: usize,
    rows: usize,
    columns: usize,
    settings: Settings,
}

impl Cell {
    fn last_row(&self) -> usize {
        self.row + self.rows - 1
    }

    fn last_column(&self) -> usize {
        self.column + self.columns - 1
    }

    fn width(&self, arena: &Arena) -> i32 {
        self.item.width(arena) + self.settings.left + self.settings.right
    }

    fn height(&self, arena: &Arena) -> i32 {
        self.item.height(arena) + self.settings.top + self.settings.bottom
    }

    /// `AbstractChildWrapper.setX`: `(int) lerp` truncates.
    fn place_x(&mut self, arena: &mut Arena, x: i32, available: i32) {
        let least = self.settings.left as f32;
        let most = (available - self.item.width(arena) - self.settings.right) as f32;
        let offset = (least + self.settings.x_align * (most - least)) as i32;
        self.item.set_x(arena, offset + x);
    }

    /// `AbstractChildWrapper.setY`: `Math.round(lerp)`.
    fn place_y(&mut self, arena: &mut Arena, y: i32, available: i32) {
        let least = self.settings.top as f32;
        let most = (available - self.item.height(arena) - self.settings.bottom) as f32;
        let offset = (least + self.settings.y_align * (most - least) + 0.5).floor() as i32;
        self.item.set_y(arena, offset + y);
    }
}

/// `com.mojang.math.Divisor`: a number cut into parts as even as whole
/// numbers allow.
struct Divisor {
    denominator: i32,
    quotient: i32,
    modulus: i32,
    remainder: i32,
}

impl Divisor {
    fn new(numerator: i32, denominator: i32) -> Self {
        let (quotient, modulus) = if denominator > 0 {
            (numerator / denominator, numerator % denominator)
        } else {
            (0, 0)
        };
        Self {
            denominator,
            quotient,
            modulus,
            remainder: 0,
        }
    }

    fn next(&mut self) -> i32 {
        let mut next = self.quotient;
        self.remainder += self.modulus;
        if self.remainder >= self.denominator {
            self.remainder -= self.denominator;
            next += 1;
        }
        next
    }
}

/// `GridLayout`.
#[derive(Debug, Clone, Default)]
pub struct Grid {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub column_spacing: i32,
    pub row_spacing: i32,
    /// `defaultCellSetting`: what a new cell is given.
    pub defaults: Settings,
    cells: Vec<Cell>,
    /// `RowHelper`'s place, in cells.
    next_index: usize,
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    /// `addChild(child, row, column, rows, columns, settings)`.
    pub fn add(
        &mut self,
        item: Item,
        (row, column): (usize, usize),
        (rows, columns): (usize, usize),
        settings: Settings,
    ) {
        self.cells.push(Cell {
            item,
            row,
            column,
            rows,
            columns,
            settings,
        });
    }

    /// `RowHelper.addChild`: the next free place of a row of `columns` cells,
    /// moving to the next row when `span` cells do not fit.
    pub fn add_in_row(&mut self, item: Item, columns: usize, span: usize, settings: Settings) {
        let mut row = self.next_index / columns;
        let mut begin = self.next_index % columns;
        if begin + span > columns {
            row += 1;
            begin = 0;
            // `Mth.roundToward(index, columns)`: up to a multiple.
            self.next_index = self.next_index.div_ceil(columns) * columns;
        }
        self.next_index += span;
        self.add(item, (row, begin), (1, span), settings);
    }

    /// `arrangeElements`.
    pub fn arrange(&mut self, arena: &mut Arena) {
        let (mut max_row, mut max_column) = (0, 0);
        for cell in &mut self.cells {
            if let Item::Grid(inner) = &mut cell.item {
                inner.arrange(arena);
            }
            max_row = max_row.max(cell.last_row());
            max_column = max_column.max(cell.last_column());
        }
        let mut column_widths = vec![0; max_column + 1];
        let mut row_heights = vec![0; max_row + 1];
        for cell in &self.cells {
            let height = cell.height(arena) - (cell.rows as i32 - 1) * self.row_spacing;
            let mut divisor = Divisor::new(height, cell.rows as i32);
            for slot in &mut row_heights[cell.row..=cell.last_row()] {
                *slot = (*slot).max(divisor.next());
            }
            let width = cell.width(arena) - (cell.columns as i32 - 1) * self.column_spacing;
            let mut divisor = Divisor::new(width, cell.columns as i32);
            for slot in &mut column_widths[cell.column..=cell.last_column()] {
                *slot = (*slot).max(divisor.next());
            }
        }
        let mut column_offsets = vec![0; max_column + 1];
        let mut row_offsets = vec![0; max_row + 1];
        for column in 1..=max_column {
            column_offsets[column] =
                column_offsets[column - 1] + column_widths[column - 1] + self.column_spacing;
        }
        for row in 1..=max_row {
            row_offsets[row] = row_offsets[row - 1] + row_heights[row - 1] + self.row_spacing;
        }
        let (x, y) = (self.x, self.y);
        for cell in &mut self.cells {
            let available_width: i32 = (cell.column..=cell.last_column())
                .map(|c| column_widths[c])
                .sum::<i32>()
                + self.column_spacing * (cell.columns as i32 - 1);
            cell.place_x(arena, x + column_offsets[cell.column], available_width);
            let available_height: i32 = (cell.row..=cell.last_row())
                .map(|r| row_heights[r])
                .sum::<i32>()
                + self.row_spacing * (cell.rows as i32 - 1);
            cell.place_y(arena, y + row_offsets[cell.row], available_height);
        }
        self.width = column_offsets[max_column] + column_widths[max_column];
        self.height = row_offsets[max_row] + row_heights[max_row];
    }

    fn set_x(&mut self, arena: &mut Arena, x: i32) {
        let delta = x - self.x;
        for cell in &mut self.cells {
            let moved = cell.item.x(arena) + delta;
            cell.item.set_x(arena, moved);
        }
        self.x = x;
    }

    fn set_y(&mut self, arena: &mut Arena, y: i32) {
        let delta = y - self.y;
        for cell in &mut self.cells {
            let moved = cell.item.y(arena) + delta;
            cell.item.set_y(arena, moved);
        }
        self.y = y;
    }

    /// `FrameLayout.alignInRectangle`: the grid in a rectangle, `align` of
    /// the way along each side (`(int) lerp` truncates).
    pub fn align_in_rectangle(
        &mut self,
        arena: &mut Arena,
        (x, y, width, height): (i32, i32, i32, i32),
        (align_x, align_y): (f32, f32),
    ) {
        let offset_x = (align_x * (width - self.width) as f32) as i32;
        let offset_y = (align_y * (height - self.height) as f32) as i32;
        self.set_x(arena, x + offset_x);
        self.set_y(arena, y + offset_y);
    }

    /// Every leaf, in the order the cells were added.
    pub fn leaves(&self, out: &mut Vec<usize>) {
        for cell in &self.cells {
            match &cell.item {
                Item::Leaf(index) => out.push(*index),
                Item::Grid(inner) => inner.leaves(out),
            }
        }
    }
}

/// A child of a `FrameLayout` of a given place and size: put in it by its
/// alignment (`x` truncates, `y` rounds, as `AbstractChildWrapper` has it).
pub fn place_in_frame(
    arena: &mut Arena,
    item: Item,
    (x, y, width, height): (i32, i32, i32, i32),
    (x_align, y_align): (f32, f32),
) {
    let mut cell = Cell {
        item,
        row: 0,
        column: 0,
        rows: 1,
        columns: 1,
        settings: Settings {
            x_align,
            y_align,
            ..Settings::default()
        },
    };
    cell.place_x(arena, x, width);
    cell.place_y(arena, y, height);
}

/// `LinearLayout.horizontal().spacing(spacing)` of leaves.
pub fn horizontal(arena: &Arena, leaves: &[usize], spacing: i32) -> Grid {
    let _ = arena;
    let mut grid = Grid::new();
    grid.column_spacing = spacing;
    for (index, leaf) in leaves.iter().enumerate() {
        grid.add(Item::Leaf(*leaf), (0, index), (1, 1), Settings::default());
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_column_of_wide_buttons_is_centred_in_its_cells() {
        let mut arena = Arena::default();
        let mut grid = Grid::new();
        grid.defaults = Settings::default().padding(4, 4, 4, 0);
        let wide = arena.add(204, 20);
        let left = arena.add(98, 20);
        let right = arena.add(98, 20);
        grid.add_in_row(Item::Leaf(wide), 2, 2, grid.defaults.padding_top(50));
        grid.add_in_row(Item::Leaf(left), 2, 1, grid.defaults);
        grid.add_in_row(Item::Leaf(right), 2, 1, grid.defaults);
        grid.arrange(&mut arena);
        grid.align_in_rectangle(&mut arena, (0, 0, 427, 240), (0.5, 0.25));
        let (wide, left, right) = (arena.0[wide], arena.0[left], arena.0[right]);
        assert_eq!((wide.x, left.x, right.x), (111, 111, 217));
        // Each row is 24 down: its height and the padding above it.
        assert_eq!((left.y - wide.y, right.y - wide.y), (24, 24));
    }

    #[test]
    fn a_number_is_cut_into_even_parts() {
        let mut divisor = Divisor::new(7, 3);
        let parts: Vec<i32> = (0..3).map(|_| divisor.next()).collect();
        assert_eq!(parts.iter().sum::<i32>(), 7);
    }
}
