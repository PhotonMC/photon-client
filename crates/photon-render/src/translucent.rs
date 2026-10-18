//! How translucent faces are ordered: the farthest from the camera first
//! (`VertexSorting.byDistance`), and when the camera stands where a section's
//! faces have to be ordered again (`TranslucencyPointOfView`).

/// `Vector3f.distanceSquared`.
fn distance_squared(from: [f32; 3], to: [f32; 3]) -> f32 {
    let [dx, dy, dz] = std::array::from_fn(|axis| from[axis] - to[axis]);
    dx.mul_add(dx, dy.mul_add(dy, dz * dz))
}

/// The order faces are drawn in, by the middle of each: the farthest from
/// `camera` first, faces at one distance as they were.
pub fn order(camera: [f32; 3], centroids: &[[f32; 3]]) -> Vec<usize> {
    let keys: Vec<f32> = centroids
        .iter()
        .map(|at| distance_squared(camera, *at))
        .collect();
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_by(|a, b| keys[*b].total_cmp(&keys[*a]));
    order
}

/// Which of the three neighbouring sections along each axis the camera
/// stands in, from the camera and the section's centre.
pub fn point_of_view(camera: [f32; 3], centre: [f32; 3]) -> [i32; 3] {
    std::array::from_fn(|axis| {
        (((camera[axis] - (centre[axis] - 8.0)) / 16.0).floor() as i32).clamp(-1, 1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_farthest_comes_first_and_ties_keep_their_order() {
        let centroids = [
            [1.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];
        assert_eq!(order([0.0; 3], &centroids), vec![1, 3, 0, 2]);
    }

    #[test]
    fn the_point_of_view_is_clamped_to_the_neighbours() {
        let centre = [8.0, 8.0, 8.0];
        assert_eq!(point_of_view([8.0, 8.0, 8.0], centre), [0, 0, 0]);
        assert_eq!(point_of_view([-1.0, 40.0, 17.0], centre), [-1, 1, 1]);
        assert_eq!(point_of_view([15.9, 0.0, 16.0], centre), [0, 0, 1]);
    }
}
