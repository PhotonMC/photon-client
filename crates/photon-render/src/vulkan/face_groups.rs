//! A section's opaque faces grouped by the way they face, so the groups that face away from the
//! camera are left out of the draw instead of being fetched, shaded and culled one by one.
//!
//! The groups follow each other in the index buffer in this order: faces looking along -X, -Y,
//! -Z, the faces that line up with no axis, then those looking along +X, +Y, +Z. With the camera
//! on one side of a section on every axis, the groups to draw are then one run. Within a group
//! the faces keep the order the mesh gave them (coplanar overlays rely on it).

use crate::scene::BlockVertex;

const GROUPS: usize = 7;
/// The group of faces that line up with no axis, which is always drawn.
const OTHER: usize = 3;
/// How far apart the corners of an aligned face may be on its axis, and how much of its normal
/// the other axes may hold, before it counts as tilted.
const TOLERANCE: f32 = 1e-4;
/// A group is left out only when the camera is this far behind all of its planes, so a face seen
/// edge-on is never dropped.
const MARGIN: f32 = 1e-3;
/// Whether a face whose corners wind counter-clockwise seen from a point looks at that point. The
/// pipeline culls the back; see `cull_mode`.
const WINDING_FACES_VIEWER: bool = true;

/// Where one group sits in the index buffer, and the planes its faces lie in along its axis.
#[derive(Debug, Clone, Copy)]
struct Group {
    first: u32,
    count: u32,
    low: f32,
    high: f32,
}

impl Group {
    const EMPTY: Self = Self {
        first: 0,
        count: 0,
        low: f32::INFINITY,
        high: f32::NEG_INFINITY,
    };
}

/// The ranges of a section's opaque indices, one per direction group.
#[derive(Debug, Clone, Copy)]
pub(super) struct FaceGroups {
    groups: [Group; GROUPS],
}

/// The index ranges to draw: `(first, count)`, in index-buffer order.
#[derive(Debug, Clone, Copy)]
pub(super) struct Runs {
    ranges: [(u32, u32); GROUPS],
    len: usize,
}

impl Runs {
    /// One range.
    pub(super) fn single(first: u32, count: u32) -> Self {
        let mut ranges = [(0, 0); GROUPS];
        ranges[0] = (first, count);
        Self { ranges, len: 1 }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.ranges[..self.len].iter().copied()
    }

    fn push(&mut self, first: u32, count: u32) {
        if let Some(last) = self.ranges[..self.len].last_mut()
            && last.0 + last.1 == first
        {
            last.1 += count;
            return;
        }
        self.ranges[self.len] = (first, count);
        self.len += 1;
    }
}

impl FaceGroups {
    /// All of `count` indices in the group that is always drawn.
    pub(super) fn ungrouped(count: u32) -> Self {
        let mut groups = [Group::EMPTY; GROUPS];
        groups[OTHER].count = count;
        Self { groups }
    }

    /// The opaque `indices` of a mesh put in group order, with the ranges of the groups.
    pub(super) fn group(vertices: &[BlockVertex], indices: &[u32]) -> (Vec<u32>, Self) {
        let mut buckets: [Vec<u32>; GROUPS] = Default::default();
        let mut groups = [Group::EMPTY; GROUPS];
        let (faces, rest) = indices.as_chunks::<6>();
        for face in faces {
            let (group, plane) = classify(vertices, face).unwrap_or((OTHER, 0.0));
            buckets[group].extend_from_slice(face);
            groups[group].low = groups[group].low.min(plane);
            groups[group].high = groups[group].high.max(plane);
        }
        buckets[OTHER].extend_from_slice(rest);
        let mut grouped = Vec::with_capacity(indices.len());
        for (bucket, group) in buckets.iter().zip(&mut groups) {
            group.first = grouped.len() as u32;
            group.count = bucket.len() as u32;
            grouped.extend_from_slice(bucket);
        }
        (grouped, Self { groups })
    }

    /// The ranges to draw for a camera at `camera`: the groups that do not face away from it.
    pub(super) fn visible(&self, camera: [f32; 3]) -> Runs {
        let mut runs = Runs {
            ranges: [(0, 0); GROUPS],
            len: 0,
        };
        for (index, group) in self.groups.iter().enumerate() {
            if group.count > 0 && !faces_away(index, group, camera) {
                runs.push(group.first, group.count);
            }
        }
        runs
    }
}

/// Whether every face of the group has the camera behind it.
fn faces_away(index: usize, group: &Group, camera: [f32; 3]) -> bool {
    if index == OTHER {
        return false;
    }
    if index < OTHER {
        // Looking along -axis: seen from where the axis is less than the plane.
        camera[index] >= group.high + MARGIN
    } else {
        camera[index - OTHER - 1] <= group.low - MARGIN
    }
}

/// The group of a face of two triangles and the plane it lies in, when it lines up with an axis
/// and both triangles look the same way.
fn classify(vertices: &[BlockVertex], face: &[u32]) -> Option<(usize, f32)> {
    let corner = |at: usize| vertices.get(face[at] as usize).map(|v| v.position);
    let first = facing(corner(0)?, corner(1)?, corner(2)?)?;
    let second = facing(corner(3)?, corner(4)?, corner(5)?)?;
    if first != second {
        return None;
    }
    let (axis, positive) = first;
    let plane = corner(0)?[axis];
    for at in 1..6 {
        if (corner(at)?[axis] - plane).abs() > TOLERANCE {
            return None;
        }
    }
    let group = if positive == WINDING_FACES_VIEWER {
        OTHER + 1 + axis
    } else {
        axis
    };
    Some((group, plane))
}

/// The axis a triangle's normal lines up with and whether the winding points along it, or `None`
/// for a tilted or empty triangle.
fn facing(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<(usize, bool)> {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let axis = (0..3).max_by(|&x, &y| normal[x].abs().total_cmp(&normal[y].abs()))?;
    let size = normal[axis].abs();
    if size <= f32::EPSILON {
        return None;
    }
    let aligned = (0..3)
        .filter(|&other| other != axis)
        .all(|other| normal[other].abs() <= TOLERANCE * size);
    aligned.then_some((axis, normal[axis] > 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(position: [f32; 3]) -> BlockVertex {
        BlockVertex::new(position, [0.0; 2], [1.0; 4], [0.0; 2])
    }

    /// A quad in the plane `axis = plane`, wound so its cross-product normal points along the
    /// axis when `positive`.
    fn quad(axis: usize, plane: f32, positive: bool, at: f32) -> [BlockVertex; 4] {
        let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
        let corner = |u: f32, v: f32| {
            let mut position = [0.0; 3];
            position[axis] = plane;
            position[a] = at + u;
            position[b] = at + v;
            vertex(position)
        };
        let mut corners = [
            corner(0.0, 0.0),
            corner(1.0, 0.0),
            corner(1.0, 1.0),
            corner(0.0, 1.0),
        ];
        if !positive {
            corners.swap(1, 3);
        }
        corners
    }

    fn mesh(quads: &[[BlockVertex; 4]]) -> (Vec<BlockVertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for quad in quads {
            let base = vertices.len() as u32;
            vertices.extend_from_slice(quad);
            indices.extend([0, 1, 2, 0, 2, 3].map(|index| base + index));
        }
        (vertices, indices)
    }

    #[test]
    fn faces_are_grouped_by_direction_keeping_their_order() {
        let up_a = quad(1, 5.0, true, 0.0);
        let up_b = quad(1, 6.0, true, 3.0);
        let east = quad(0, 2.0, true, 0.0);
        let (vertices, indices) = mesh(&[up_a, east, up_b]);
        let (grouped, groups) = FaceGroups::group(&vertices, &indices);
        assert_eq!(grouped.len(), indices.len());
        let group = |positive_axis: usize| groups.groups[OTHER + 1 + positive_axis];
        assert_eq!(group(1).count, 12, "both up faces");
        assert_eq!((group(1).low, group(1).high), (5.0, 6.0));
        assert_eq!(group(0).count, 6);
        // The first up face comes before the second, as in the mesh.
        let first_up = group(1).first as usize;
        assert_eq!(grouped[first_up], 0);
        assert_eq!(grouped[first_up + 6], 8);
    }

    #[test]
    fn a_tilted_or_bent_face_is_always_drawn() {
        let tilted = [
            vertex([0.0, 0.0, 0.0]),
            vertex([1.0, 0.0, 0.0]),
            vertex([1.0, 1.0, 1.0]),
            vertex([0.0, 1.0, 1.0]),
        ];
        let mut bent = quad(1, 5.0, true, 0.0);
        bent[3] = vertex([bent[3].position[0], 6.0, bent[3].position[2]]);
        let (vertices, indices) = mesh(&[tilted, bent]);
        let (_, groups) = FaceGroups::group(&vertices, &indices);
        assert_eq!(groups.groups[OTHER].count, 12);
        assert_eq!(groups.visible([100.0; 3]).iter().count(), 1);
    }

    #[test]
    fn groups_behind_the_camera_are_left_out_and_runs_merge() {
        let quads: Vec<_> = (0..3)
            .flat_map(|axis| [quad(axis, 4.0, true, 0.0), quad(axis, 4.0, false, 2.0)])
            .collect();
        let (vertices, indices) = mesh(&quads);
        let (_, groups) = FaceGroups::group(&vertices, &indices);
        let drawn = |camera: [f32; 3]| groups.visible(camera).iter().map(|(_, n)| n).sum::<u32>();
        // Past the planes on every axis: only the three groups looking back at it.
        assert_eq!(drawn([9.0; 3]), 18);
        assert_eq!(groups.visible([9.0; 3]).iter().count(), 1, "one run");
        assert_eq!(drawn([-9.0; 3]), 18);
        assert_eq!(groups.visible([-9.0; 3]).iter().count(), 1, "one run");
        // In the plane's range both ways are seen: nothing is left out.
        assert_eq!(drawn([4.0; 3]), 36);
    }

    #[test]
    fn faces_with_a_stray_tail_keep_the_tail() {
        let (vertices, mut indices) = mesh(&[quad(1, 5.0, true, 0.0)]);
        indices.extend([0, 1]);
        let (grouped, groups) = FaceGroups::group(&vertices, &indices);
        assert_eq!(grouped.len(), 8);
        assert_eq!(groups.groups[OTHER].count, 2);
    }
}
