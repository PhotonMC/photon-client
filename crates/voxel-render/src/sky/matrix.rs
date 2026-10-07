use super::*;

pub(super) type Matrix = [[f32; 4]; 4];

pub(super) fn unpack(colour: u32) -> [f32; 4] {
    [
        channel(colour, 16) as f32 / 255.0,
        channel(colour, 8) as f32 / 255.0,
        channel(colour, 0) as f32 / 255.0,
        channel(colour, 24) as f32 / 255.0,
    ]
}

/// `left * right`, both row-major as `m[row][column]`.
pub(super) fn mul(left: Matrix, right: Matrix) -> Matrix {
    let mut out = [[0.0; 4]; 4];
    for (row, out_row) in out.iter_mut().enumerate() {
        for (column, cell) in out_row.iter_mut().enumerate() {
            *cell = (0..4).map(|k| left[row][k] * right[k][column]).sum();
        }
    }
    out
}

pub(super) fn rotate_x(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn rotate_y(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn rotate_z(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn translate(x: f32, y: f32, z: f32) -> Matrix {
    [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn scale(x: f32, y: f32, z: f32) -> Matrix {
    [
        [x, 0.0, 0.0, 0.0],
        [0.0, y, 0.0, 0.0],
        [0.0, 0.0, z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn transform(matrix: Matrix, point: [f32; 3]) -> [f32; 3] {
    let row = |r: usize| {
        matrix[r][0] * point[0] + matrix[r][1] * point[1] + matrix[r][2] * point[2] + matrix[r][3]
    };
    [row(0), row(1), row(2)]
}
