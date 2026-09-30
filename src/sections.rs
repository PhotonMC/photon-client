//! The world's sections meshed on worker threads as they arrive and change:
//! `LevelRenderer.compileSections` and its section fade.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

use voxel_net::connection::PlaySession;
use voxel_render::VulkanBackend;
use voxel_render::section_workers::{SectionKey, SectionWorkers};
use voxel_render::vulkan::SectionFade;

/// How long a section away from the camera waits for its neighbouring
/// chunks before it is built anyway.
const SETTLE: Duration = Duration::from_secs(2);

/// The squared distance, in blocks, within which a section counts as near
/// the camera: it is built at once and does not fade in.
const NEAR: i64 = 768;

/// Whether a section's centre is near the camera's block.
fn near(key: SectionKey, camera_block: [i64; 3]) -> bool {
    [key.0, key.1, key.2]
        .iter()
        .zip(camera_block)
        .map(|(&section, block)| (i64::from(section) * 16 + 8 - block).pow(2))
        .sum::<i64>()
        < NEAR
}

/// What the workers have built since the profile last printed.
#[derive(Default)]
pub struct MeshStats {
    built: u32,
    took: Duration,
    /// How often each section has been built over the run.
    counts: HashMap<SectionKey, u32>,
}

/// The dirty sections, the meshes handed out for them, and when each first
/// showed.
pub struct SectionMeshes {
    /// Dirty sections, with when each was last dirtied.
    pending: HashMap<SectionKey, Instant>,
    /// The generation of each section's newest hand-out; a mesh from an
    /// older one is stale.
    latest: HashMap<SectionKey, u64>,
    next_generation: u64,
    fades: HashMap<SectionKey, SectionFade>,
    workers: SectionWorkers,
    worker_count: usize,
    /// The chunk fade option.
    chunk_fade: Duration,
    /// The dimension's sections, bottom to top.
    sections: Range<i32>,
    stats: MeshStats,
}

impl SectionMeshes {
    /// The sections the session has dirtied so far, and workers to mesh
    /// them, leaving one core for the game.
    pub fn new(
        session: &mut PlaySession,
        renderer: Arc<voxel_render::world_mesh::BlockRenderer>,
        tints: Arc<voxel_render::biome_tint::BiomeTints>,
        chunk_fade: Duration,
    ) -> Self {
        let pending = session
            .take_dirty_sections()
            .into_iter()
            .map(|key| (key, Instant::now()))
            .collect();
        let worker_count = std::thread::available_parallelism()
            .map_or(2, |count| count.get().saturating_sub(1))
            .clamp(1, 8);
        let dimension = session.world().dimension();
        let low = dimension.min_y.div_euclid(16);
        Self {
            pending,
            latest: HashMap::new(),
            next_generation: 0,
            fades: HashMap::new(),
            workers: SectionWorkers::new(worker_count, renderer, tints),
            worker_count,
            chunk_fade,
            sections: low..low + dimension.height / 16,
            stats: MeshStats::default(),
        }
    }

    /// A respawn or a change of dimension: nothing of the old level stays.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.latest.clear();
        self.fades.clear();
    }

    /// Drop the sections of chunks the server forgot.
    pub fn forget_chunks(
        &mut self,
        chunks: impl IntoIterator<Item = (i32, i32)>,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (chunk_x, chunk_z) in chunks {
            for section_y in self.sections.clone() {
                let key = (chunk_x, section_y, chunk_z);
                self.pending.remove(&key);
                self.latest.remove(&key);
                self.fades.remove(&key);
                backend.set_section(
                    key,
                    &voxel_render::Mesh::default(),
                    [0.0; 3],
                    SectionFade::none(),
                )?;
            }
        }
        Ok(())
    }

    /// A section fades in from the fog the first time it shows, unless it is
    /// near the camera; its later rebuilds keep that first time.
    fn fade(&mut self, key: SectionKey, camera_block: [i64; 3]) -> SectionFade {
        let chunk_fade = self.chunk_fade;
        *self.fades.entry(key).or_insert_with(|| SectionFade {
            since: Instant::now(),
            length: if near(key, camera_block) {
                Duration::ZERO
            } else {
                chunk_fade
            },
        })
    }

    /// Update the light, then mesh the dirty sections nearest the player
    /// first, and show what the workers have finished. A finished script
    /// waits until every section is built, so its capture has them all.
    pub fn update(
        &mut self,
        session: &mut PlaySession,
        backend: &mut VulkanBackend,
        feet: voxel_core::math::Vec3,
        camera_block: [i64; 3],
        script_done: bool,
        origin: [i32; 3],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // `ClientLevel.update`: queued light and the light engine, each frame.
        session.update_light();
        let dirtied = Instant::now();
        self.pending.extend(
            session
                .take_dirty_sections()
                .into_iter()
                .map(|key| (key, dirtied)),
        );
        let player_section = (
            feet.x.floor() as i32 >> 4,
            feet.y.floor() as i32 >> 4,
            feet.z.floor() as i32 >> 4,
        );
        loop {
            self.show_built(backend, camera_block)?;
            let ordered = self.ready(session, player_section, camera_block, script_done);
            let nothing_to_hand_out = ordered.is_empty();
            self.hand_out(session, backend, ordered, camera_block, origin)?;
            if !script_done || (nothing_to_hand_out && self.workers.in_flight() == 0) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// The meshes the workers finished, unless a newer hand-out replaced
    /// them.
    fn show_built(
        &mut self,
        backend: &mut VulkanBackend,
        camera_block: [i64; 3],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for built in self.workers.collect() {
            self.stats.built += 1;
            self.stats.took += built.took;
            *self.stats.counts.entry(built.key).or_insert(0_u32) += 1;
            if self.latest.get(&built.key) == Some(&built.generation) {
                self.latest.remove(&built.key);
                let fade = self.fade(built.key, camera_block);
                backend.set_section(built.key, &built.mesh, built.centre, fade)?;
            }
        }
        Ok(())
    }

    /// The dirty sections within the view distance that may be built now,
    /// nearest first. A section away from the camera waits until the chunks
    /// around it have arrived, so it is built once rather than again for
    /// every neighbour (or, at the edge of what the server sends, until it
    /// has waited long enough); one near the camera, where the player
    /// changes blocks, is built at once.
    fn ready(
        &self,
        session: &PlaySession,
        player_section: (i32, i32, i32),
        camera_block: [i64; 3],
        script_done: bool,
    ) -> Vec<SectionKey> {
        let world = session.world();
        let view = session.view_distance().clamp(2, 12);
        let now = Instant::now();
        let mut ordered: Vec<SectionKey> = self
            .pending
            .iter()
            .filter(|&(&(x, y, z), &dirtied)| {
                (x - player_section.0).abs() <= view
                    && (z - player_section.2).abs() <= view
                    && (near((x, y, z), camera_block)
                        || script_done
                        || now - dirtied >= SETTLE
                        || (-1..=1)
                            .all(|dx| (-1..=1).all(|dz| world.contains_chunk(x + dx, z + dz))))
            })
            .map(|(&key, _)| key)
            .collect();
        ordered.sort_by_key(|&(x, y, z)| {
            (x - player_section.0).pow(2)
                + (y - player_section.1).pow(2)
                + (z - player_section.2).pow(2)
        });
        ordered
    }

    /// Hand sections to the workers, keeping every worker busy with a
    /// little queued behind it, and spending at most a few milliseconds of
    /// the frame copying. An empty section shows at once.
    fn hand_out(
        &mut self,
        session: &PlaySession,
        backend: &mut VulkanBackend,
        ordered: Vec<SectionKey>,
        camera_block: [i64; 3],
        origin: [i32; 3],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let world = session.world();
        let budget = Instant::now();
        for key in ordered {
            if self.workers.in_flight() >= self.worker_count * 4
                || budget.elapsed() > Duration::from_millis(4)
            {
                break;
            }
            self.pending.remove(&key);
            if world.section_is_empty(key.0, key.1, key.2) {
                self.latest.remove(&key);
                // An empty section has shown too, so what is built in it
                // later does not fade.
                let fade = self.fade(key, camera_block);
                backend.set_section(key, &voxel_render::Mesh::default(), [0.0; 3], fade)?;
                continue;
            }
            let (min, size, quart_min, quart_size) = SectionWorkers::region(key);
            let copy = world.copy_region(min, size, quart_min, quart_size);
            self.next_generation += 1;
            self.latest.insert(key, self.next_generation);
            self.workers.submit(key, self.next_generation, copy, origin);
        }
        Ok(())
    }

    /// How many sections wait to be built.
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// `VOXEL_PROFILE`'s line about the meshes built since the last one.
    pub fn report(&mut self) {
        let stats = &self.stats;
        if stats.built > 0 {
            let repeated = stats.counts.values().filter(|count| **count > 1).count();
            println!(
                "profile meshes={} avg_ms={:.2} worker_ms={:.0} sections={} rebuilt={}",
                stats.built,
                stats.took.as_secs_f64() * 1000.0 / f64::from(stats.built),
                stats.took.as_secs_f64() * 1000.0,
                stats.counts.len(),
                repeated,
            );
        }
        self.stats.built = 0;
        self.stats.took = Duration::ZERO;
    }
}
