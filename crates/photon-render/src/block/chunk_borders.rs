//! The F3+G chunk borders, as `ChunkBorderRenderer` emits them: red posts at the corners of the
//! chunk and its neighbours, a two-block grid on the walls of the chunk, rings every two blocks,
//! and blue posts, rings and a box for the chunk's own corners and section.

use super::outline::{LineVertex, pull_towards};

/// `ARGB.color(255, 0, 155, 155)`.
const CELL_BORDER: [f32; 4] = [0.0, 155.0 / 255.0, 155.0 / 255.0, 1.0];
/// `ARGB.color(255, 255, 255, 0)`.
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
/// `ARGB.colorFromFloat(1, 0.25, 0.25, 1)`: each channel is floored to a byte.
const MAJOR: [f32; 4] = [63.0 / 255.0, 63.0 / 255.0, 1.0, 1.0];
/// `ARGB.colorFromFloat(0.5, 1, 0, 0)`.
const NEIGHBOUR: [f32; 4] = [1.0, 0.0, 0.0, 127.0 / 255.0];

/// Where the borders are: the chunk's west-north corner and the section the player is in, in
/// blocks, the world's vertical bounds, and the mesh origin and camera the vertices are placed by.
pub struct Borders {
    pub chunk: [i32; 2],
    pub section_min_y: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub origin: [i32; 3],
    pub camera: [f32; 3],
}

struct Builder<'a> {
    borders: &'a Borders,
    opaque: Vec<LineVertex>,
    translucent: Vec<LineVertex>,
}

impl Builder<'_> {
    /// `Gizmos.line`: a strip of two triangles, in the group its alpha picks.
    fn line(&mut self, from: [i32; 3], to: [i32; 3], colour: [f32; 4], width: f32) {
        let place = |point: [i32; 3]| {
            let mesh: [f32; 3] =
                std::array::from_fn(|axis| (point[axis] - self.borders.origin[axis]) as f32);
            pull_towards(mesh, self.borders.camera)
        };
        let (start, end) = (place(from), place(to));
        let delta: [f32; 3] = std::array::from_fn(|axis| end[axis] - start[axis]);
        let length = delta.iter().map(|value| value * value).sum::<f32>().sqrt();
        let normal = delta.map(|value| value / length);
        let vertex = |position, side: f32| LineVertex {
            position,
            colour,
            normal,
            width: width * side,
        };
        let group = if colour[3] < 1.0 {
            &mut self.translucent
        } else {
            &mut self.opaque
        };
        group.extend([
            vertex(start, 1.0),
            vertex(start, -1.0),
            vertex(end, 1.0),
            vertex(end, -1.0),
            vertex(end, 1.0),
            vertex(start, -1.0),
        ]);
    }

    /// A ring of four lines round the chunk at height `y`.
    fn ring(&mut self, y: i32, colour: [f32; 4], width: f32) {
        let [x, z] = self.borders.chunk;
        let corners = [[x, z], [x, z + 16], [x + 16, z + 16], [x + 16, z], [x, z]];
        for pair in corners.windows(2) {
            self.line(
                [pair[0][0], y, pair[0][1]],
                [pair[1][0], y, pair[1][1]],
                colour,
                width,
            );
        }
    }

    fn post(&mut self, x: i32, z: i32, colour: [f32; 4], width: f32) {
        self.line(
            [x, self.borders.min_y, z],
            [x, self.borders.max_y + 1, z],
            colour,
            width,
        );
    }

    fn grid(&mut self) {
        let [x, z] = self.borders.chunk;
        for step in (2..16).step_by(2) {
            let colour = if step % 4 == 0 { CELL_BORDER } else { YELLOW };
            self.post(x + step, z, colour, 1.0);
            self.post(x + step, z + 16, colour, 1.0);
            self.post(x, z + step, colour, 1.0);
            self.post(x + 16, z + step, colour, 1.0);
        }
    }

    /// The section's box: the twelve edges of `Gizmos.cuboid`'s stroke.
    fn section_box(&mut self) {
        let [x, z] = self.borders.chunk;
        let (y0, y1) = (self.borders.section_min_y, self.borders.section_min_y + 16);
        let (x1, z1) = (x + 16, z + 16);
        let edges = [
            ([x, y0, z], [x1, y0, z]),
            ([x, y0, z], [x, y1, z]),
            ([x, y0, z], [x, y0, z1]),
            ([x1, y0, z], [x1, y1, z]),
            ([x1, y1, z], [x, y1, z]),
            ([x, y1, z], [x, y1, z1]),
            ([x, y1, z1], [x, y0, z1]),
            ([x, y0, z1], [x1, y0, z1]),
            ([x1, y0, z1], [x1, y0, z]),
            ([x, y1, z1], [x1, y1, z1]),
            ([x1, y0, z1], [x1, y1, z1]),
            ([x1, y1, z], [x1, y1, z1]),
        ];
        for (from, to) in edges {
            self.line(from, to, MAJOR, 1.0);
        }
    }
}

/// The strips of every border line, the opaque group first and then the translucent one, as the
/// reference submits them. The section's box is drawn in the reference without a depth test; here
/// it is tested like the rest.
pub fn chunk_border_vertices(borders: &Borders) -> Vec<LineVertex> {
    let mut builder = Builder {
        borders,
        opaque: Vec::new(),
        translucent: Vec::new(),
    };
    let [x, z] = borders.chunk;
    for dx in [-16, 0, 16] {
        for dz in [-16, 0, 16] {
            builder.post(x + dx, z + dz, NEIGHBOUR, 4.0);
        }
    }
    builder.grid();
    let mut y = borders.min_y;
    while y <= borders.max_y + 1 {
        let colour = if y % 8 == 0 { CELL_BORDER } else { YELLOW };
        builder.ring(y, colour, 1.0);
        y += 2;
    }
    for dx in [0, 16] {
        for dz in [0, 16] {
            builder.post(x + dx, z + dz, MAJOR, 4.0);
        }
    }
    builder.section_box();
    let mut y = borders.min_y;
    while y <= borders.max_y + 1 {
        builder.ring(y, MAJOR, 4.0);
        y += 16;
    }
    builder.opaque.extend(builder.translucent);
    builder.opaque
}

#[cfg(test)]
mod tests {
    use super::*;

    fn borders() -> Borders {
        Borders {
            chunk: [32, -16],
            section_min_y: 64,
            min_y: -64,
            max_y: 319,
            origin: [0, 0, 0],
            camera: [40.0, 70.0, -8.0],
        }
    }

    #[test]
    fn the_lines_are_counted_by_their_kind_and_the_red_ones_come_last() {
        let vertices = chunk_border_vertices(&borders());
        // 9 red posts, 28 grid posts, 193 two-block rings of 4, 4 corner posts, 12 box edges,
        // 25 section rings of 4; six vertices each.
        let lines = 9 + 28 + 193 * 4 + 4 + 12 + 25 * 4;
        assert_eq!(vertices.len(), lines * 6);
        let last = vertices.last().unwrap();
        assert_eq!(last.colour, NEIGHBOUR);
        assert_eq!(last.width.abs(), 4.0);
        assert_eq!(vertices[0].colour, YELLOW);
    }
}
