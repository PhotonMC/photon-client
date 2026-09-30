//! The world's sections meshed on worker threads as they arrive and change,
//! and which of them are in view: `LevelRenderer.compileSections`,
//! `SectionOcclusionGraph`, the culling frustum and the section fade.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

use voxel_net::connection::PlaySession;
use voxel_render::VulkanBackend;
use voxel_render::section_workers::{SectionKey, SectionWorkers};
use voxel_render::visibility::{
    Frustum, GraphWorld, SectionState, VisibilitySet, visible_sections,
};
use voxel_render::vulkan::SectionFade;

/// The squared distance, in blocks, within which a section counts as near
/// the camera: it does not fade in.
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

/// Where the sections are seen from this frame.
pub struct SectionView {
    /// The camera's place in the world.
    pub camera: [f64; 3],
    /// The culling frustum at the camera.
    pub frustum: Frustum,
    /// The render distance, in chunks.
    pub view_distance: i32,
    /// The point meshes are measured from.
    pub origin: [i32; 3],
}

/// The dirty sections, the meshes handed out for them, which are in view,
/// and when each first showed.
pub struct SectionMeshes {
    /// Sections waiting to be built.
    pending: HashSet<SectionKey>,
    /// The generation of each section's newest hand-out; a mesh from an
    /// older one is stale.
    latest: HashMap<SectionKey, u64>,
    next_generation: u64,
    fades: HashMap<SectionKey, SectionFade>,
    /// Each built section's face visibility (`CompiledSectionMesh`).
    compiled: HashMap<SectionKey, VisibilitySet>,
    /// The occlusion flood's sections, and whether it has to run again.
    graph: Vec<SectionKey>,
    graph_stale: bool,
    /// The camera's eight-block cell and the view distance the flood last
    /// ran with (`invalidateIfNeeded`).
    graph_from: Option<([i64; 3], i32)>,
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
        let pending = session.take_dirty_sections().into_iter().collect();
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
            compiled: HashMap::new(),
            graph: Vec::new(),
            graph_stale: true,
            graph_from: None,
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
        self.compiled.clear();
        self.graph.clear();
        self.graph_stale = true;
    }

    /// Drop the sections of chunks the server forgot.
    pub fn forget_chunks(
        &mut self,
        chunks: impl IntoIterator<Item = (i32, i32)>,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (chunk_x, chunk_z) in chunks {
            self.graph_stale = true;
            for section_y in self.sections.clone() {
                let key = (chunk_x, section_y, chunk_z);
                self.pending.remove(&key);
                self.latest.remove(&key);
                self.fades.remove(&key);
                self.compiled.remove(&key);
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

    /// Update the light, show what the workers have finished, work out which
    /// sections are in view, and hand the dirty ones in view to the workers,
    /// nearest first. A finished script waits until every section in view is
    /// built, so its capture has them all.
    pub fn update(
        &mut self,
        session: &mut PlaySession,
        backend: &mut VulkanBackend,
        view: &SectionView,
        script_done: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // `ClientLevel.update`: queued light and the light engine, each frame.
        session.update_light();
        let dirtied = session.take_dirty_sections();
        // A chunk arriving or a section turning empty changes the flood.
        self.graph_stale |= !dirtied.is_empty();
        self.pending.extend(dirtied);
        let camera_block = view.camera.map(|value| value.floor() as i64);
        loop {
            self.show_built(backend, camera_block)?;
            self.show_empty(session, backend, camera_block)?;
            let visible = self.visible(session, view);
            backend.set_visible_sections(&visible);
            let ordered = self.ready(session, &visible, camera_block);
            let nothing_to_hand_out = ordered.is_empty();
            self.hand_out(session, ordered, view.origin);
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
                self.compiled.insert(built.key, built.visibility);
                self.graph_stale = true;
                let fade = self.fade(built.key, camera_block);
                backend.set_section(built.key, &built.mesh, built.centre, fade)?;
            }
        }
        Ok(())
    }

    /// Dirty sections holding only air need no worker: they show as nothing
    /// at once, and the flood passes through them.
    fn show_empty(
        &mut self,
        session: &PlaySession,
        backend: &mut VulkanBackend,
        camera_block: [i64; 3],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let world = session.world();
        let empty: Vec<SectionKey> = self
            .pending
            .iter()
            .copied()
            .filter(|&(x, y, z)| world.contains_chunk(x, z) && world.section_is_empty(x, y, z))
            .collect();
        for key in empty {
            self.pending.remove(&key);
            self.latest.remove(&key);
            self.compiled.remove(&key);
            // An empty section has shown too, so what is built in it later
            // does not fade.
            let fade = self.fade(key, camera_block);
            backend.set_section(key, &voxel_render::Mesh::default(), [0.0; 3], fade)?;
        }
        Ok(())
    }

    /// The sections in view: the occlusion flood's, run again when the
    /// camera crosses into another eight-block cell or what it floods
    /// through changed, and then the ones in the frustum.
    fn visible(&mut self, session: &PlaySession, view: &SectionView) -> Vec<SectionKey> {
        let cell = view.camera.map(|value| (value / 8.0).floor() as i64);
        if self.graph_from != Some((cell, view.view_distance)) {
            self.graph_from = Some((cell, view.view_distance));
            self.graph_stale = true;
        }
        if self.graph_stale {
            self.graph_stale = false;
            let world = session.world();
            let compiled = &self.compiled;
            let section = |(x, y, z): SectionKey| {
                if world.section_is_empty(x, y, z) {
                    SectionState::Empty
                } else {
                    compiled
                        .get(&(x, y, z))
                        .map_or(SectionState::Uncompiled, |visibility| {
                            SectionState::Compiled(*visibility)
                        })
                }
            };
            self.graph = visible_sections(&GraphWorld {
                camera: view.camera,
                view_distance: view.view_distance,
                min_section_y: self.sections.start,
                max_section_y: self.sections.end - 1,
                loaded: &|x, z| world.contains_chunk(x, z),
                section: &section,
                smart_cull: true,
            });
        }
        let frustum = view.frustum.offset_to_fully_include_camera_cube(8);
        self.graph
            .iter()
            .copied()
            .filter(|key| frustum.section_visible(*key))
            .collect()
    }

    /// The dirty sections in view that may be built now, nearest first: one
    /// built before is rebuilt at once, one never built waits until the
    /// chunks around it and their light have arrived
    /// (`SectionUpdateTracker.hasAllNeighbors`), so it is built once rather
    /// than again for every neighbour.
    fn ready(
        &self,
        session: &PlaySession,
        visible: &[SectionKey],
        camera_block: [i64; 3],
    ) -> Vec<SectionKey> {
        let world = session.world();
        let lit = |x: i32, z: i32| {
            world.contains_chunk(x, z) && world.light_engine().light_on_in_column(x, z)
        };
        let mut ordered: Vec<SectionKey> = visible
            .iter()
            .copied()
            .filter(|key| self.pending.contains(key))
            .filter(|&(x, y, z)| {
                self.compiled.contains_key(&(x, y, z))
                    || (-1..=1)
                        .all(|dx| (-1..=1).all(|dz| (dx == 0 && dz == 0) || lit(x + dx, z + dz)))
            })
            .collect();
        let distance = |&(x, y, z): &SectionKey| {
            [x, y, z]
                .iter()
                .zip(camera_block)
                .map(|(&section, block)| (i64::from(section) * 16 + 8 - block).pow(2))
                .sum::<i64>()
        };
        ordered.sort_by_key(distance);
        ordered
    }

    /// Hand sections to the workers, keeping every worker busy with a
    /// little queued behind it, and spending at most a few milliseconds of
    /// the frame copying.
    fn hand_out(&mut self, session: &PlaySession, ordered: Vec<SectionKey>, origin: [i32; 3]) {
        let world = session.world();
        let budget = Instant::now();
        for key in ordered {
            if self.workers.in_flight() >= self.worker_count * 4
                || budget.elapsed() > Duration::from_millis(4)
            {
                break;
            }
            self.pending.remove(&key);
            let (min, size, quart_min, quart_size) = SectionWorkers::region(key);
            let copy = world.copy_region(min, size, quart_min, quart_size);
            self.next_generation += 1;
            self.latest.insert(key, self.next_generation);
            self.workers.submit(key, self.next_generation, copy, origin);
        }
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
