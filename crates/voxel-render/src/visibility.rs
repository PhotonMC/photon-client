//! Which sections are worth compiling and drawing, as the reference decides:
//! each section's face-to-face visibility (`VisGraph`), the flood from the
//! camera through them (`SectionOcclusionGraph`) and the camera's view cone
//! (`Frustum`).

use std::collections::VecDeque;

use voxel_core::direction::Direction;

use crate::section_workers::SectionKey;

const DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `VisibilitySet`: whether one face of a section can be seen through it
/// from another.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisibilitySet(u64);

impl VisibilitySet {
    pub const ALL: Self = Self(u64::MAX);
    pub const NONE: Self = Self(0);

    fn bit(first: Direction, second: Direction) -> u64 {
        1 << (first as u32 + second as u32 * 6)
    }

    /// Every face in the set sees every other.
    fn add(&mut self, faces: u8) {
        for first in DIRECTIONS {
            for second in DIRECTIONS {
                if faces & (1 << first as u8) != 0 && faces & (1 << second as u8) != 0 {
                    self.0 |= Self::bit(first, second) | Self::bit(second, first);
                }
            }
        }
    }

    pub fn visibility_between(self, first: Direction, second: Direction) -> bool {
        self.0 & Self::bit(first, second) != 0
    }
}

/// `VisGraph`: which blocks of a section are opaque, flood-filled from its
/// edges to find which faces see each other.
pub struct VisGraph {
    opaque: Vec<bool>,
    empty: usize,
}

impl Default for VisGraph {
    fn default() -> Self {
        Self {
            opaque: vec![false; 4096],
            empty: 4096,
        }
    }
}

impl VisGraph {
    /// A block that hides what is behind it (`isSolidRender`), in the
    /// section's own coordinates.
    pub fn set_opaque(&mut self, x: usize, y: usize, z: usize) {
        self.opaque[index(x, y, z)] = true;
        self.empty -= 1;
    }

    /// `VisGraph.resolve`.
    pub fn resolve(mut self) -> VisibilitySet {
        let mut visibility = VisibilitySet::NONE;
        if 4096 - self.empty < 256 {
            return VisibilitySet::ALL;
        }
        if self.empty == 0 {
            return visibility;
        }
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let edge = x == 0 || x == 15 || y == 0 || y == 15 || z == 0 || z == 15;
                    let start = index(x, y, z);
                    if edge && !self.opaque[start] {
                        let faces = self.flood_fill(start);
                        visibility.add(faces);
                    }
                }
            }
        }
        visibility
    }

    /// The faces the open cells joined to `start` reach, marking them done.
    fn flood_fill(&mut self, start: usize) -> u8 {
        let mut faces = 0_u8;
        let mut queue = VecDeque::from([start]);
        self.opaque[start] = true;
        while let Some(cell) = queue.pop_front() {
            let (x, y, z) = (cell & 15, cell >> 8 & 15, cell >> 4 & 15);
            for (coordinate, low, high) in [
                (x, Direction::West, Direction::East),
                (y, Direction::Down, Direction::Up),
                (z, Direction::North, Direction::South),
            ] {
                if coordinate == 0 {
                    faces |= 1 << low as u8;
                } else if coordinate == 15 {
                    faces |= 1 << high as u8;
                }
            }
            for direction in DIRECTIONS {
                let neighbour = match direction {
                    Direction::Down => (y > 0).then(|| cell - 256),
                    Direction::Up => (y < 15).then(|| cell + 256),
                    Direction::North => (z > 0).then(|| cell - 16),
                    Direction::South => (z < 15).then(|| cell + 16),
                    Direction::West => (x > 0).then(|| cell - 1),
                    Direction::East => (x < 15).then(|| cell + 1),
                };
                if let Some(neighbour) = neighbour
                    && !self.opaque[neighbour]
                {
                    self.opaque[neighbour] = true;
                    queue.push_back(neighbour);
                }
            }
        }
        faces
    }
}

/// JOML's `FrustumIntersection` on a clip matrix: six planes, each
/// `(x, y, z, w)` with the point inside where `x·p + w >= 0`.
#[derive(Clone, Copy, Debug)]
struct FrustumIntersection {
    planes: [[f32; 4]; 6],
}

/// `intersectAab`'s answers.
const INSIDE: i32 = -2;
const INTERSECT: i32 = -1;

impl FrustumIntersection {
    /// `set(m, true)`: the planes of a column-major clip matrix, normalised.
    fn new(m: &[[f32; 4]; 4]) -> Self {
        let row = |r: usize| [m[0][r], m[1][r], m[2][r], m[3][r]];
        let (x, y, z, w) = (row(0), row(1), row(2), row(3));
        let plane = |sign: f32, axis: [f32; 4]| {
            let plane: [f32; 4] = std::array::from_fn(|i| w[i] + sign * axis[i]);
            let inverse =
                1.0 / (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            plane.map(|value| value * inverse)
        };
        Self {
            planes: [
                plane(1.0, x),
                plane(-1.0, x),
                plane(1.0, y),
                plane(-1.0, y),
                plane(1.0, z),
                plane(-1.0, z),
            ],
        }
    }

    /// `intersectAab`: inside, straddling, or the first plane it is
    /// wholly outside of.
    fn intersect_aab(&self, min: [f32; 3], max: [f32; 3]) -> i32 {
        let mut inside = true;
        for (number, [a, b, c, d]) in self.planes.iter().enumerate() {
            let far = |normal: f32, low: f32, high: f32| if normal < 0.0 { low } else { high };
            let near = |normal: f32, low: f32, high: f32| if normal < 0.0 { high } else { low };
            let reaches = a * far(*a, min[0], max[0])
                + b * far(*b, min[1], max[1])
                + c * far(*c, min[2], max[2])
                >= -d;
            if !reaches {
                return number as i32;
            }
            inside &= a * near(*a, min[0], max[0])
                + b * near(*b, min[1], max[1])
                + c * near(*c, min[2], max[2])
                >= -d;
        }
        if inside { INSIDE } else { INTERSECT }
    }
}

/// `Frustum`: the camera's view cone, planes in the camera's own frame and
/// the camera's place in the world.
#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    intersection: FrustumIntersection,
    /// The view direction, `matrix.transformTranspose(0, 0, 1, 0)`.
    view: [f32; 3],
    camera: [f64; 3],
}

impl Frustum {
    /// `Frustum(modelView, projection)` prepared at `camera`: the view's
    /// rotation (no translation) under the culling projection.
    pub fn new(
        view_rotation: &[[f32; 4]; 4],
        projection: &[[f32; 4]; 4],
        camera: [f64; 3],
    ) -> Self {
        let clip = crate::scene::multiply(*projection, *view_rotation);
        Self {
            intersection: FrustumIntersection::new(&clip),
            // `transformTranspose(0, 0, 1, 0)`: the clip matrix's z row.
            view: [clip[0][2], clip[1][2], clip[2][2]],
            camera,
        }
    }

    /// `offsetToFullyIncludeCameraCube`: back the camera away along the view
    /// until the cube of `size` blocks around it is wholly inside.
    pub fn offset_to_fully_include_camera_cube(mut self, size: i32) -> Self {
        let size = f64::from(size);
        let low = self.camera.map(|value| (value / size).floor() * size);
        let high = self.camera.map(|value| (value / size).ceil() * size);
        // A degenerate view could never take the cube in; the reference
        // would spin, this gives up after far more steps than any real view
        // needs.
        for _ in 0..1024 {
            let min: [f32; 3] = std::array::from_fn(|axis| (low[axis] - self.camera[axis]) as f32);
            let max: [f32; 3] = std::array::from_fn(|axis| (high[axis] - self.camera[axis]) as f32);
            if self.intersection.intersect_aab(min, max) == INSIDE {
                break;
            }
            for axis in 0..3 {
                self.camera[axis] -= f64::from(self.view[axis] * 4.0);
            }
        }
        self
    }

    /// `isVisible`: whether any of a box is in view.
    pub fn is_visible(&self, min: [f64; 3], max: [f64; 3]) -> bool {
        let relative = |corner: [f64; 3]| -> [f32; 3] {
            std::array::from_fn(|axis| (corner[axis] - self.camera[axis]) as f32)
        };
        matches!(
            self.intersection
                .intersect_aab(relative(min), relative(max)),
            INSIDE | INTERSECT
        )
    }

    /// Whether a section's box is in view.
    pub fn section_visible(&self, (x, y, z): SectionKey) -> bool {
        let min = [x, y, z].map(|value| f64::from(value * 16));
        self.is_visible(min, min.map(|value| value + 16.0))
    }
}

/// What a section is to the flood: not yet built, built with a visibility,
/// or empty (air only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionState {
    Uncompiled,
    Compiled(VisibilitySet),
    Empty,
}

/// What the occlusion flood reads of the world.
pub struct GraphWorld<'a> {
    /// The camera's position in the world.
    pub camera: [f64; 3],
    /// The render distance, in chunks.
    pub view_distance: i32,
    /// The world's sections, bottom and top inclusive.
    pub min_section_y: i32,
    pub max_section_y: i32,
    /// Whether a chunk has arrived.
    pub loaded: &'a dyn Fn(i32, i32) -> bool,
    /// What a section is to the flood.
    pub section: &'a dyn Fn(SectionKey) -> SectionState,
    /// `smartCull`: off only for a spectator inside blocks.
    pub smart_cull: bool,
}

/// A section reached by the flood, with the sides it was reached through
/// and every side the path to it went.
#[derive(Clone, Copy, Default)]
struct Node {
    sources: u8,
    directions: u8,
}

impl GraphWorld<'_> {
    fn camera_section(&self) -> SectionKey {
        let [x, y, z] = self
            .camera
            .map(|value| (value.floor() as i32).div_euclid(16));
        (x, y, z)
    }

    /// `ViewArea.getRenderSection`: a section inside the view area around
    /// the camera's section.
    fn exists(&self, camera: SectionKey, key: SectionKey) -> bool {
        (self.min_section_y..=self.max_section_y).contains(&key.1)
            && (key.0 - camera.0).abs() <= self.view_distance
            && (key.2 - camera.2).abs() <= self.view_distance
    }

    /// `getRelativeFrom`.
    fn relative(
        &self,
        camera: SectionKey,
        from: SectionKey,
        direction: Direction,
    ) -> Option<SectionKey> {
        let next = step(from, direction);
        let in_range = in_view_distance((camera.0, camera.2), self.view_distance, (next.0, next.2))
            && (camera.1 - next.1).abs() <= self.view_distance;
        (in_range && self.exists(camera, next)).then_some(next)
    }
}

mod index;
pub use index::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_section_sees_through_every_face_and_a_solid_one_none() {
        assert_eq!(VisGraph::default().resolve(), VisibilitySet::ALL);
        let mut solid = VisGraph::default();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    solid.set_opaque(x, y, z);
                }
            }
        }
        assert_eq!(solid.resolve(), VisibilitySet::NONE);
    }

    #[test]
    fn a_wall_across_a_section_parts_its_two_sides() {
        let mut graph = VisGraph::default();
        for y in 0..16 {
            for z in 0..16 {
                graph.set_opaque(8, y, z);
            }
        }
        let visibility = graph.resolve();
        assert!(!visibility.visibility_between(Direction::West, Direction::East));
        assert!(visibility.visibility_between(Direction::West, Direction::North));
        assert!(visibility.visibility_between(Direction::East, Direction::Up));
    }

    /// Looking north (towards -z) from the origin.
    fn north_frustum() -> Frustum {
        let view = crate::scene::look_at([0.0; 3], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]);
        Frustum::new(
            &view,
            &culling_projection(70.0, 16.0 / 9.0, 256.0),
            [0.5, 64.5, 0.5],
        )
    }

    #[test]
    fn the_frustum_holds_what_is_ahead_and_not_what_is_behind() {
        let frustum = north_frustum();
        assert!(frustum.section_visible((0, 4, -3)));
        assert!(!frustum.section_visible((0, 4, 3)));
        assert!(!frustum.section_visible((20, 4, -1)));
    }

    #[test]
    fn backing_off_takes_the_camera_cube_in() {
        let frustum = north_frustum().offset_to_fully_include_camera_cube(8);
        assert!(
            frustum.camera[2] > 0.5,
            "the camera backs away from its view"
        );
        assert!(frustum.is_visible([0.0, 64.0, 0.0], [8.0, 72.0, 8.0]));
    }

    fn flood(section: &dyn Fn(SectionKey) -> SectionState, camera: [f64; 3]) -> Vec<SectionKey> {
        visible_sections(&GraphWorld {
            camera,
            view_distance: 2,
            min_section_y: 0,
            max_section_y: 3,
            loaded: &|_, _| true,
            section,
            smart_cull: true,
        })
    }

    #[test]
    fn open_sections_are_all_reached() {
        let reached = flood(
            &|_| SectionState::Compiled(VisibilitySet::ALL),
            [8.0, 24.0, 8.0],
        );
        // Five by five columns within two chunks (the view distance counts
        // from one chunk out, so even the corners are in), four sections
        // tall.
        assert_eq!(reached.len(), 25 * 4);
        assert_eq!(reached[0], (0, 1, 0));
    }

    #[test]
    fn a_closed_section_is_reached_but_stops_the_flood() {
        // Everything but the camera's own section is solid rock.
        let reached = flood(
            &|key| {
                if key == (0, 1, 0) {
                    SectionState::Compiled(VisibilitySet::ALL)
                } else {
                    SectionState::Compiled(VisibilitySet::NONE)
                }
            },
            [8.0, 24.0, 8.0],
        );
        assert_eq!(
            reached.len(),
            7,
            "the camera's section and its six neighbours"
        );
    }

    #[test]
    fn empty_sections_carry_the_flood_but_are_not_drawn() {
        let reached = flood(&|_| SectionState::Empty, [8.0, 24.0, 8.0]);
        assert!(reached.is_empty());
    }

    #[test]
    fn a_camera_above_the_world_starts_from_the_top_layer() {
        let reached = flood(
            &|_| SectionState::Compiled(VisibilitySet::ALL),
            [8.0, 200.0, 8.0],
        );
        assert_eq!(reached.first().map(|key| key.1), Some(3));
        assert!(!reached.is_empty());
    }
}
