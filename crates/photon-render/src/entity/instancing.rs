//! Entities drawn from a mesh they share ([ADR 0016](../../../../docs/adr/0016-instanced-entity-drawing.md)).
//!
//! A mob's posed parts are built once into a mesh in model space (the root pose is the
//! identity), and each entity that stands in that pose is one [`Instance`]: its placement
//! matrices, light, overlay code and colour. The GPU places the mesh; [`Instance::place`] does the
//! same on the CPU, for the entities that are alone in their pose and for the tests.

use std::collections::HashMap;

use crate::joml::Pose;
use crate::scene::Vertex;

/// One entity's placement, as the instanced entity pipeline reads it: 100 bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Instance {
    /// The four columns of the position matrix, each three rows (the last row is 0, 0, 0, 1).
    pub position: [[f32; 3]; 4],
    /// The three columns of the normal matrix.
    pub normal: [[f32; 3]; 3],
    /// The overlay code the vertices' shade carries.
    pub shade: f32,
    /// The colour (`0xRRGGBB`) the texels are multiplied by, or zero.
    pub colour: f32,
    /// Block and sky light, as the lightmap reads them.
    pub light: [f32; 2],
}

impl Instance {
    /// The instance that places a model-space mesh as `pose` does, or none when the pose does
    /// not keep normals unit length (a non-uniform scale), which the CPU path draws.
    pub fn new(pose: &Pose, (shade, colour): (f32, f32), light: [f32; 2]) -> Option<Self> {
        if !pose.keeps_normals_unit() {
            return None;
        }
        let m = &pose.pose.m;
        let n = &pose.normal.m;
        Some(Self {
            position: std::array::from_fn(|column| [m[column][0], m[column][1], m[column][2]]),
            normal: *n,
            shade,
            colour,
            light,
        })
    }

    /// A vertex of a model-space mesh, placed: what the instanced pipeline's vertex shader makes
    /// of it.
    pub fn place(&self, vertex: &Vertex) -> Vertex {
        let [x, y, z] = vertex.position;
        let [nx, ny, nz, _] = vertex.tint;
        let p = &self.position;
        let n = &self.normal;
        let at = |row: usize| p[0][row] * x + p[1][row] * y + p[2][row] * z + p[3][row];
        let turned = |row: usize| n[0][row] * nx + n[1][row] * ny + n[2][row] * nz;
        Vertex {
            position: [at(0), at(1), at(2)],
            uv: vertex.uv,
            shade: self.shade,
            tint: [turned(0), turned(1), turned(2), self.colour],
            light: self.light,
        }
    }
}

/// One mesh's run of instances in a frame's lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Batch {
    pub first_vertex: u32,
    pub vertex_count: u32,
    pub first_instance: u32,
    pub instance_count: u32,
}

/// What a frame draws from shared meshes: the meshes in use, the instances sorted by mesh, and the
/// batches that say which instances use which mesh.
#[derive(Debug, Default)]
pub struct InstancedFrame {
    pub vertices: Vec<Vertex>,
    pub instances: Vec<Instance>,
    pub batches: Vec<Batch>,
}

/// How many meshes the cache keeps before it starts again.
const MESHES_KEPT: usize = 4096;

/// The meshes built so far, by the key of the pose they were built from, and the entities placed
/// this frame.
#[derive(Default)]
pub struct Batcher {
    meshes: HashMap<u64, Vec<Vertex>>,
    /// The key each slot of this frame's list stands for, in the order of first use.
    slots: Vec<u64>,
    slot_of: HashMap<u64, usize>,
    placed: Vec<(usize, Instance)>,
}

impl Batcher {
    /// Forget the meshes (the models or textures they were built from changed).
    pub fn clear(&mut self) {
        self.meshes.clear();
        self.begin_frame();
    }

    /// Start a frame: nothing is placed yet.
    pub fn begin_frame(&mut self) {
        self.slots.clear();
        self.slot_of.clear();
        self.placed.clear();
    }

    /// Whether the cache has the mesh of `key`.
    pub fn has(&self, key: u64) -> bool {
        self.meshes.contains_key(&key)
    }

    /// Place an entity whose posed parts hash to `key`; `build` makes the model-space mesh when
    /// the cache lacks it.
    pub fn add(&mut self, key: u64, instance: Instance, build: impl FnOnce() -> Vec<Vertex>) {
        if self.meshes.len() >= MESHES_KEPT && !self.meshes.contains_key(&key) {
            // Every mesh this frame uses is in the list below by its key; forgetting the rest
            // costs a rebuild when one comes back.
            let used: std::collections::HashSet<u64> = self.slots.iter().copied().collect();
            self.meshes.retain(|key, _| used.contains(key));
        }
        self.meshes.entry(key).or_insert_with(build);
        let slots = &mut self.slots;
        let slot = *self.slot_of.entry(key).or_insert_with(|| {
            slots.push(key);
            slots.len() - 1
        });
        self.placed.push((slot, instance));
    }

    /// The frame's lists. The meshes only one entity uses are placed on the CPU into `alone`
    /// instead: a draw each would cost more than they save.
    pub fn finish(&mut self, alone: &mut Vec<Vertex>) -> InstancedFrame {
        let mut counts = vec![0_u32; self.slots.len()];
        for &(slot, _) in &self.placed {
            counts[slot] += 1;
        }
        let mut frame = InstancedFrame::default();
        let mut start = vec![u32::MAX; self.slots.len()];
        for (slot, key) in self.slots.iter().enumerate() {
            if counts[slot] > 1 {
                let mesh = &self.meshes[key];
                start[slot] = frame.instances.len() as u32;
                frame.batches.push(Batch {
                    first_vertex: frame.vertices.len() as u32,
                    vertex_count: mesh.len() as u32,
                    first_instance: start[slot],
                    instance_count: counts[slot],
                });
                frame.vertices.extend_from_slice(mesh);
                frame.instances.resize(
                    frame.instances.len() + counts[slot] as usize,
                    self.placed[0].1,
                );
            }
        }
        let mut next = start.clone();
        for &(slot, instance) in &self.placed {
            if counts[slot] > 1 {
                frame.instances[next[slot] as usize] = instance;
                next[slot] += 1;
            } else {
                let mesh = &self.meshes[&self.slots[slot]];
                alone.extend(mesh.iter().map(|vertex| instance.place(vertex)));
            }
        }
        self.begin_frame();
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joml::Quaternion;

    fn vertex(position: [f32; 3], normal: [f32; 3]) -> Vertex {
        Vertex {
            position,
            uv: [0.25, 0.5],
            shade: 0.0,
            tint: [normal[0], normal[1], normal[2], 0.0],
            light: [0.0; 2],
        }
    }

    fn pose() -> Pose {
        let mut pose = Pose::default();
        pose.translate(3.5, 64.0, -2.25);
        pose.rotate(&Quaternion::rotation_y(1.3));
        pose.scale(-1.0, -1.0, 1.0);
        pose.translate(0.0, -1.501, 0.0);
        pose
    }

    #[test]
    fn a_placed_vertex_is_where_the_pose_puts_it_within_the_tolerance() {
        let pose = pose();
        let instance = Instance::new(&pose, (7.0, 255.0), [48.0, 240.0]).expect("unit normals");
        let mesh = vertex([0.3, 0.7, -0.2], [0.0, 1.0, 0.0]);
        let placed = instance.place(&mesh);
        let expected = pose.pose.transform_position(0.3, 0.7, -0.2);
        let normal = pose.transform_normal([0.0, 1.0, 0.0]);
        for axis in 0..3 {
            // A sixteenth of a model pixel, which is 1/16 block.
            assert!((placed.position[axis] - expected[axis]).abs() <= 1.0 / 4096.0);
            assert!((placed.tint[axis] - normal[axis]).abs() <= 1e-5);
        }
        assert_eq!(
            (placed.shade, placed.tint[3], placed.light, placed.uv),
            (7.0, 255.0, [48.0, 240.0], [0.25, 0.5])
        );
    }

    #[test]
    fn a_non_uniform_scale_is_not_an_instance() {
        let mut pose = Pose::default();
        pose.scale(1.0, 2.0, 1.0);
        assert_eq!(Instance::new(&pose, (0.0, 0.0), [0.0; 2]), None);
    }

    #[test]
    fn instances_are_sorted_by_mesh_and_a_mesh_used_once_is_placed_on_the_cpu() {
        let mesh = |x: f32| move || vec![vertex([x, 0.0, 0.0], [0.0, 1.0, 0.0])];
        let at = |x: f32| {
            let mut pose = Pose::default();
            pose.translate(x, 0.0, 0.0);
            Instance::new(&pose, (0.0, 0.0), [0.0; 2]).expect("unit normals")
        };
        let mut batcher = Batcher::default();
        batcher.add(1, at(10.0), mesh(1.0));
        batcher.add(2, at(20.0), mesh(2.0));
        batcher.add(1, at(30.0), mesh(1.0));
        let mut alone = Vec::new();
        let frame = batcher.finish(&mut alone);
        assert_eq!(frame.batches.len(), 1);
        assert_eq!(frame.batches[0].instance_count, 2);
        assert_eq!(frame.vertices.len(), 1);
        assert_eq!(frame.instances[0].position[3][0], 10.0);
        assert_eq!(frame.instances[1].position[3][0], 30.0);
        assert_eq!(alone.len(), 1);
        assert_eq!(alone[0].position[0], 22.0);
        // The next frame starts empty but keeps the meshes.
        let again = batcher.finish(&mut alone);
        assert!(again.batches.is_empty());
    }
}
