//! Section meshing off the main thread.
//!
//! The main thread copies a section and the blocks around it out of the
//! world and hands the copy to a pool of workers; each worker meshes it and
//! sends the mesh back. A section dirtied again while its mesh is being built
//! gets a newer generation, and the older mesh is dropped when it returns.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::block::biome_tint::BiomeTints;
use crate::block::world_mesh::BlockRenderer;
use crate::scene::Mesh;
use photon_world::RegionCopy;

/// A section: chunk x, section y, chunk z.
pub type SectionKey = (i32, i32, i32);

/// How far past its own blocks a section's mesh reads the world.
const PADDING: i32 = 2;

struct Job {
    key: SectionKey,
    generation: u64,
    copy: RegionCopy,
    origin: [i32; 3],
}

/// A finished section mesh.
pub struct Built {
    pub key: SectionKey,
    pub generation: u64,
    pub mesh: Mesh,
    /// The section's centre, from the mesh origin.
    pub centre: [f32; 3],
    /// Which of its faces see each other, for the occlusion flood.
    pub visibility: crate::visibility::VisibilitySet,
    /// How long the worker took.
    pub took: std::time::Duration,
}

pub struct SectionWorkers {
    pool: rayon::ThreadPool,
    renderer: Arc<BlockRenderer>,
    tints: Arc<BiomeTints>,
    sender: Sender<Built>,
    built: Receiver<Built>,
    in_flight: usize,
}

impl SectionWorkers {
    /// Start `count` workers sharing the block models and biome colours.
    pub fn new(count: usize, renderer: Arc<BlockRenderer>, tints: Arc<BiomeTints>) -> Self {
        let (sender, built) = channel();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(count.max(1))
            .thread_name(|index| format!("section-mesher-{index}"))
            .build()
            .expect("a section mesher pool starts");
        Self {
            pool,
            renderer,
            tints,
            sender,
            built,
            in_flight: 0,
        }
    }

    /// Sections handed out and not yet collected.
    pub fn in_flight(&self) -> usize {
        self.in_flight
    }

    /// The span of blocks and quart biomes a section's mesh reads.
    pub fn region(key: SectionKey) -> ([i32; 3], [i32; 3], [i32; 3], [i32; 3]) {
        let corner = [key.0 * 16, key.1 * 16, key.2 * 16];
        let min = corner.map(|value| value - PADDING);
        let size = [16 + 2 * PADDING; 3];
        // The tint blends two blocks either way, and the biome zoom looks
        // from two blocks back to one quart ahead.
        let quart_min = corner.map(|value| (value - PADDING - 2 - 2) >> 2);
        let quart_max = corner.map(|value| ((value + 15 + PADDING + 2 - 2) >> 2) + 1);
        let quart_size = [0, 1, 2].map(|axis| quart_max[axis] - quart_min[axis] + 1);
        (min, size, quart_min, quart_size)
    }

    /// Hand a copied section to the workers, which take them in the order
    /// they were handed over.
    pub fn submit(&mut self, key: SectionKey, generation: u64, copy: RegionCopy, origin: [i32; 3]) {
        let job = Job {
            key,
            generation,
            copy,
            origin,
        };
        let renderer = Arc::clone(&self.renderer);
        let tints = Arc::clone(&self.tints);
        let sender = self.sender.clone();
        self.pool.spawn_fifo(move || {
            // The receiver is gone when the workers are being dropped.
            let _ = sender.send(mesh_section(&renderer, &tints, job));
        });
        self.in_flight += 1;
    }

    /// The meshes finished since the last call.
    pub fn collect(&mut self) -> Vec<Built> {
        let built: Vec<Built> = self.built.try_iter().collect();
        self.in_flight -= built.len();
        built
    }
}

fn mesh_section(renderer: &BlockRenderer, tints: &BiomeTints, job: Job) -> Built {
    let started = std::time::Instant::now();
    let copy = &job.copy;
    let corner = [job.key.0 * 16, job.key.1 * 16, job.key.2 * 16];
    let read = |x: i32, y: i32, z: i32| copy.block_state_at(x, y, z);
    let light = |x: i32, y: i32, z: i32| copy.light_at(x, y, z);
    let noise_biome = |x: i32, y: i32, z: i32| copy.noise_biome(x, y, z);
    // Neighbouring blocks blend over mostly the same columns, so the zoomed
    // biome of each position is worked out once.
    let biomes: RefCell<HashMap<[i32; 3], usize>> = RefCell::new(HashMap::new());
    let biome_at = |x: i32, y: i32, z: i32| {
        if let Some(biome) = biomes.borrow().get(&[x, y, z]) {
            return *biome;
        }
        let biome = tints.biome_at(x, y, z, &noise_biome);
        biomes.borrow_mut().insert([x, y, z], biome);
        biome
    };
    let tint = |kind, position| tints.tint_from(kind, position, &biome_at);
    let region = renderer.mesh_region(
        &crate::block::world_mesh::MeshWorld {
            read: &read,
            light: &light,
            biome_tint: &tint,
            tags: copy.tags(),
        },
        corner,
        [16, 16, 16],
        job.origin,
    );
    // `SectionCompiler`: every block that renders solid hides what is
    // behind it.
    let mut graph = crate::visibility::VisGraph::default();
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let state = read(corner[0] + x, corner[1] + y, corner[2] + z)
                    .unwrap_or(photon_world::VANILLA_AIR_BLOCK_STATE_ID);
                if photon_world::block::state_or_air(state).solid_render {
                    graph.set_opaque(x as usize, y as usize, z as usize);
                }
            }
        }
    }
    Built {
        key: job.key,
        generation: job.generation,
        visibility: graph.resolve(),
        mesh: region.mesh,
        centre: [0, 1, 2].map(|axis| (corner[axis] + 8 - job.origin[axis]) as f32),
        took: started.elapsed(),
    }
}
