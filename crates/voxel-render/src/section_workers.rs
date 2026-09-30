//! Section meshing off the main thread.
//!
//! The main thread copies a section and the blocks around it out of the
//! world and hands the copy to a pool of workers; each worker meshes it and
//! sends the mesh back. A section dirtied again while its mesh is being built
//! gets a newer generation, and the older mesh is dropped when it returns.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use crate::biome_tint::BiomeTints;
use crate::scene::Mesh;
use crate::world_mesh::BlockRenderer;
use voxel_world::RegionCopy;

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
    /// How long the worker took.
    pub took: std::time::Duration,
}

pub struct SectionWorkers {
    jobs: Option<Sender<Job>>,
    built: Receiver<Built>,
    threads: Vec<std::thread::JoinHandle<()>>,
    in_flight: usize,
}

impl SectionWorkers {
    /// Start `count` workers sharing the block models and biome colours.
    pub fn new(count: usize, renderer: Arc<BlockRenderer>, tints: Arc<BiomeTints>) -> Self {
        let (job_sender, job_receiver) = channel::<Job>();
        let job_receiver = Arc::new(Mutex::new(job_receiver));
        let (built_sender, built) = channel();
        let threads = (0..count.max(1))
            .map(|index| {
                let jobs = Arc::clone(&job_receiver);
                let built = built_sender.clone();
                let renderer = Arc::clone(&renderer);
                let tints = Arc::clone(&tints);
                std::thread::Builder::new()
                    .name(format!("section-mesher-{index}"))
                    .spawn(move || {
                        loop {
                            let job = match jobs.lock() {
                                Ok(receiver) => receiver.recv(),
                                Err(_) => return,
                            };
                            let Ok(job) = job else { return };
                            if built.send(mesh_section(&renderer, &tints, job)).is_err() {
                                return;
                            }
                        }
                    })
                    .expect("a section mesher thread starts")
            })
            .collect();
        Self {
            jobs: Some(job_sender),
            built,
            threads,
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

    /// Hand a copied section to the workers.
    pub fn submit(&mut self, key: SectionKey, generation: u64, copy: RegionCopy, origin: [i32; 3]) {
        if let Some(jobs) = &self.jobs
            && jobs
                .send(Job {
                    key,
                    generation,
                    copy,
                    origin,
                })
                .is_ok()
        {
            self.in_flight += 1;
        }
    }

    /// The meshes finished since the last call.
    pub fn collect(&mut self) -> Vec<Built> {
        let built: Vec<Built> = self.built.try_iter().collect();
        self.in_flight -= built.len();
        built
    }
}

impl Drop for SectionWorkers {
    fn drop(&mut self) {
        self.jobs = None;
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
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
        &crate::world_mesh::MeshWorld {
            read: &read,
            light: &light,
            biome_tint: &tint,
            tags: copy.tags(),
        },
        corner,
        [16, 16, 16],
        job.origin,
    );
    Built {
        key: job.key,
        generation: job.generation,
        mesh: region.mesh,
        centre: [0, 1, 2].map(|axis| (corner[axis] + 8 - job.origin[axis]) as f32),
        took: started.elapsed(),
    }
}
