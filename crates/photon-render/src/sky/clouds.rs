//! Clouds, the way the reference's cloud renderer lays them out.
//!
//! The cloud texture is a grid of cells, one per pixel: a pixel that is not
//! nearly transparent is a cloud 12 blocks wide and 4 tall. The grid wraps,
//! drifts towards positive x as game time passes, and is laid out in rings of
//! cells around the camera out to the cloud range. In the default (fancy)
//! style each cell is a box: its top when the camera is not below the
//! clouds, its bottom when not above, and a side only where the neighbouring
//! cell is empty and the side faces the camera. The cells right around the
//! camera also get inward faces, so the inside of a cloud shows.
//!
//! Vertices are placed relative to the cell the camera is in; the shader adds
//! the camera's offset inside that cell every frame, so the faces are only
//! rebuilt when the camera crosses into another cell.

use crate::texture::png::DecodedImage;

/// Blocks along one side of a cell.
pub const CELL_SIZE: f32 = 12.0;
/// Blocks from a cloud's bottom to its top.
pub const CELL_HEIGHT: f32 = 4.0;

/// Face directions in the reference's 3D data order.
const DOWN: usize = 0;
const UP: usize = 1;
const NORTH: usize = 2;
const SOUTH: usize = 3;
const WEST: usize = 4;
const EAST: usize = 5;

/// Each direction's four corners, in unit-cell coordinates and in the order
/// the reference's cloud shader lists them.
const CORNERS: [[[f32; 3]; 4]; 6] = [
    [
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
    ],
    [
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0],
    ],
    [
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
    ],
    [
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
    ],
    [
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
    [
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
    ],
];

/// How bright each direction's faces are: bottom, top, the z sides, the x
/// sides.
const SHADES: [f32; 6] = [0.7, 1.0, 0.8, 0.8, 0.9, 0.9];

/// One cloud vertex: its place relative to the camera's cell corner at the
/// cloud's bottom, and its face's brightness.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct CloudVertex {
    pub position: [f32; 3],
    pub shade: f32,
}

/// Where the camera is against the cloud layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeCamera {
    Above,
    Inside,
    Below,
}

impl RelativeCamera {
    /// From the cloud layer's bottom measured from the camera.
    pub fn from_bottom(relative_bottom: f32) -> Self {
        if relative_bottom + CELL_HEIGHT < 0.0 {
            Self::Above
        } else if relative_bottom > 0.0 {
            Self::Below
        } else {
            Self::Inside
        }
    }
}

/// The cell grid read from the cloud texture.
#[derive(Debug, Clone)]
pub struct CloudCells {
    width: i32,
    height: i32,
    /// Per cell: whether it is a cloud, and whether its north, east, south
    /// and west neighbours are empty.
    cells: Vec<Option<[bool; 4]>>,
}

impl CloudCells {
    pub fn from_image(image: &DecodedImage) -> Self {
        let width = image.width as i32;
        let height = image.height as i32;
        let empty = |x: i32, y: i32| image.rgba[((x + y * width) * 4 + 3) as usize] < 10;
        let mut cells = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                cells.push((!empty(x, y)).then(|| {
                    // The reference wraps the x neighbours by the height too;
                    // the texture is square, so it makes no difference.
                    [
                        empty(x, (y - 1).rem_euclid(height)),
                        empty((x + 1).rem_euclid(height), y),
                        empty(x, (y + 1).rem_euclid(height)),
                        empty((x - 1).rem_euclid(height), y),
                    ]
                }));
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// The width and height of the grid in blocks.
    pub fn size_in_blocks(&self) -> (f64, f64) {
        (
            f64::from(self.width) * f64::from(CELL_SIZE),
            f64::from(self.height) * f64::from(CELL_SIZE),
        )
    }

    /// The width of the grid in cells, which is how long the drift takes
    /// to wrap, in multiples of 400 ticks.
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    fn cell(&self, x: i32, z: i32) -> Option<[bool; 4]> {
        let index = x.rem_euclid(self.width) + z.rem_euclid(self.height) * self.width;
        self.cells[index as usize]
    }

    /// The fancy clouds' triangles around the camera's cell, in the
    /// reference's ring order (nearest first), two triangles per face.
    pub fn build(
        &self,
        relative: RelativeCamera,
        centre_x: i32,
        centre_z: i32,
        radius_cells: i32,
    ) -> Vec<CloudVertex> {
        let mut vertices = Vec::new();
        let mut face = |x: i32, z: i32, direction: usize, inside: bool| {
            let corners = CORNERS[direction];
            let corner = |quad_vertex: usize| {
                let [u, v, w] = corners[if inside { 3 - quad_vertex } else { quad_vertex }];
                CloudVertex {
                    position: [
                        (u + x as f32) * CELL_SIZE,
                        v * CELL_HEIGHT,
                        (w + z as f32) * CELL_SIZE,
                    ],
                    shade: SHADES[direction],
                }
            };
            for quad_vertex in [0, 1, 2, 0, 2, 3] {
                vertices.push(corner(quad_vertex));
            }
        };
        let mut cell = |x: i32, z: i32| {
            let Some([north, east, south, west]) = self.cell(centre_x + x, centre_z + z) else {
                return;
            };
            if relative != RelativeCamera::Below {
                face(x, z, UP, false);
            }
            if relative != RelativeCamera::Above {
                face(x, z, DOWN, false);
            }
            if north && z > 0 {
                face(x, z, NORTH, false);
            }
            if south && z < 0 {
                face(x, z, SOUTH, false);
            }
            if west && x > 0 {
                face(x, z, WEST, false);
            }
            if east && x < 0 {
                face(x, z, EAST, false);
            }
            if x.abs() <= 1 && z.abs() <= 1 {
                for direction in 0..6 {
                    face(x, z, direction, true);
                }
            }
        };
        for ring in 0..=2 * radius_cells {
            for x in -ring..=ring {
                let z = ring - x.abs();
                if z >= 0 && z <= radius_cells && x * x + z * z <= radius_cells * radius_cells {
                    if z != 0 {
                        cell(x, -z);
                    }
                    cell(x, z);
                }
            }
        }
        vertices
    }
}

/// Where the clouds are this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloudPlacement {
    /// The cell the camera is in.
    pub cell: (i32, i32),
    /// Where the camera's cell corner sits from the camera, at the clouds'
    /// bottom.
    pub offset: [f32; 3],
    pub relative: RelativeCamera,
}

/// `CloudRenderer.render`'s placement: the grid drifts 0.03 blocks a tick
/// along x, and is shifted 3.96 blocks along z.
pub fn place(
    cells: &CloudCells,
    bottom_y: f32,
    camera: [f64; 3],
    game_time: i64,
    partial_tick: f32,
) -> CloudPlacement {
    let relative_bottom = (f64::from(bottom_y) - camera[1]) as f32;
    let drift = game_time.rem_euclid(i64::from(cells.width()) * 400) as f32 + partial_tick;
    let mut cloud_x = camera[0] + f64::from(drift * 0.030_000_001_f32);
    let mut cloud_z = camera[2] + f64::from(3.96_f32);
    let (width_blocks, height_blocks) = cells.size_in_blocks();
    cloud_x -= (cloud_x / width_blocks).floor() * width_blocks;
    cloud_z -= (cloud_z / height_blocks).floor() * height_blocks;
    let cell_x = (cloud_x / 12.0).floor() as i32;
    let cell_z = (cloud_z / 12.0).floor() as i32;
    let in_cell_x = (cloud_x - f64::from(cell_x as f32 * CELL_SIZE)) as f32;
    let in_cell_z = (cloud_z - f64::from(cell_z as f32 * CELL_SIZE)) as f32;
    CloudPlacement {
        cell: (cell_x, cell_z),
        offset: [-in_cell_x, relative_bottom, -in_cell_z],
        relative: RelativeCamera::from_bottom(relative_bottom),
    }
}

/// How many cells out the clouds reach for a cloud range in chunks.
pub fn radius_cells(range_chunks: i32) -> i32 {
    ((range_chunks * 16) as f32 / CELL_SIZE).ceil() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: u32, height: u32, solid: &[(u32, u32)]) -> DecodedImage {
        let mut rgba = vec![0; (width * height * 4) as usize];
        for &(x, y) in solid {
            rgba[((x + y * width) * 4 + 3) as usize] = 255;
        }
        DecodedImage {
            width,
            height,
            rgba,
        }
    }

    #[test]
    fn a_lone_cell_below_the_camera_shows_its_top_and_the_facing_sides() {
        let cells = CloudCells::from_image(&image(8, 8, &[(4, 4)]));
        // The cell two to the east and two to the south of the camera's.
        let vertices = cells.build(RelativeCamera::Above, 2, 2, 4);
        // Top, plus the west and north sides that face back at the camera.
        assert_eq!(vertices.len(), 3 * 6);
        assert!(vertices.iter().all(|vertex| vertex.position[0] >= 24.0));
    }

    #[test]
    fn cells_around_the_camera_get_inward_faces() {
        let cells = CloudCells::from_image(&image(8, 8, &[(0, 0)]));
        let vertices = cells.build(RelativeCamera::Inside, 0, 0, 2);
        // Top and bottom, no outward sides at the centre, six inward faces.
        assert_eq!(vertices.len(), 8 * 6);
    }

    #[test]
    fn placement_drifts_along_x() {
        let cells = CloudCells::from_image(&image(256, 256, &[]));
        let still = place(&cells, 192.33, [0.0, 64.0, 0.0], 0, 0.0);
        let later = place(&cells, 192.33, [0.0, 64.0, 0.0], 100, 0.0);
        assert_eq!(still.cell, (0, 0));
        assert!((later.offset[0] + 3.0).abs() < 1e-4);
        assert_eq!(still.relative, RelativeCamera::Below);
        assert_eq!(radius_cells(128), 171);
    }
}
