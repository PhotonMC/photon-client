//! The world's sections meshed on worker threads as they arrive and change,
//! and which of them are in view: `LevelRenderer.compileSections`,
//! `SectionOcclusionGraph`, the culling frustum and the section fade.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

use photon_net::connection::PlaySession;
use photon_render::VulkanBackend;
use photon_render::section_workers::{SectionKey, SectionWorkers};
use photon_render::visibility::{Frustum, OcclusionGraph, VisibilitySet};
use photon_render::vulkan::SectionFade;

use super::flood::{Flood, FloodInputs, FloodParams};

mod fade;
mod graph;
use crate::profile::{self, Span};

/// The time a frame may spend copying regions for the workers: at several
/// hundred frames a second this still hands out hundreds of sections a
/// second, and a frame spending more shows as a spike.
const COPY_BUDGET: Duration = Duration::from_millis(1);

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
    /// Sections waiting to be built, each with whether the player changed it
    /// (`SectionDirtyState.isDirtyFromPlayer`, which stays set while it waits).
    pending: HashMap<SectionKey, bool>,
    /// `prioritizeChunkUpdates`: which dirty sections are built before the frame ends.
    prioritize: photon_game::options::PrioritizeChunkUpdates,
    /// Sections dirtied since they were last looked at for being empty;
    /// only a change dirties one, so only these can have turned empty.
    unchecked: Vec<SectionKey>,
    /// The generation of each section's newest hand-out; a mesh from an
    /// older one is stale.
    latest: HashMap<SectionKey, u64>,
    next_generation: u64,
    fades: fade::Fades,
    /// Each built section's face visibility (`CompiledSectionMesh`).
    compiled: HashMap<SectionKey, VisibilitySet>,
    /// What the flood last read of each dirtied section: whether its chunk
    /// was here and whether it held only air. A section dirtied by a change
    /// that leaves both as they were (light, a block swapped for another)
    /// does not make the flood run again.
    flood_inputs: HashMap<SectionKey, (bool, bool)>,
    /// The occlusion flood's result, and whether a full one has to run
    /// again.
    graph: Option<(OcclusionGraph, FloodParams)>,
    /// Sections that changed since the flood passed them on: a mesh that
    /// finished, a section that stopped being empty, a chunk that arrived
    /// (`schedulePropagationFrom`).
    propagate: Vec<SectionKey>,
    /// What was passed on while a full flood ran, to pass on again from the
    /// result.
    propagated_meanwhile: Vec<SectionKey>,
    running: Option<FloodParams>,
    /// The graph's sections in the frustum they were last filtered with,
    /// and the graph's generation; still cameras filter once.
    in_frustum: Option<(photon_render::visibility::Frustum, u64, Vec<SectionKey>)>,
    graph_generation: u64,
    graph_stale: bool,
    /// The thread the flood runs on.
    flood: Flood,
    /// The camera's eight-block cell and the view distance the flood last
    /// ran with, and its smart culling (`invalidateIfNeeded`).
    graph_from: Option<([i64; 3], i32, bool)>,
    workers: SectionWorkers,
    worker_count: usize,
    /// Whether the occlusion graph and the frustum choose the sections;
    /// `PHOTON_CULLING=0` turns them off to compare against.
    culling: bool,
    /// Counts the sections that blink out of view, in a bench run.
    blinks: Option<crate::profile::BlinkWatch>,
    /// The dimension's sections, bottom to top.
    sections: Range<i32>,
    stats: MeshStats,
    /// The `ao` option.
    ambient_occlusion: bool,
}

impl SectionMeshes {
    /// Whether the view chooses what is drawn (`PHOTON_CULLING` is not `0`).
    pub fn culling(&self) -> bool {
        self.culling
    }

    /// The sections the session has dirtied so far, and workers to mesh
    /// them, leaving two cores: one for the game's frames and one for the
    /// connection's reader and the sound, so a burst of meshing does not
    /// keep the frame waiting for a core.
    pub fn new(
        session: &mut PlaySession,
        renderer: Arc<photon_render::block::world_mesh::BlockRenderer>,
        tints: Arc<photon_render::block::biome_tint::BiomeTints>,
        chunk_fade: Duration,
    ) -> Self {
        let pending: HashMap<SectionKey, bool> =
            session.take_dirty_sections().into_iter().collect();
        let unchecked = pending.keys().copied().collect();
        let worker_count = std::thread::available_parallelism()
            .map_or(2, |count| count.get().saturating_sub(2))
            .clamp(1, 8);
        let dimension = session.world().dimension();
        let low = dimension.min_y.div_euclid(16);
        Self {
            pending,
            prioritize: photon_game::options::PrioritizeChunkUpdates::None,
            unchecked,
            latest: HashMap::new(),
            next_generation: 0,
            fades: fade::Fades::new(chunk_fade),
            compiled: HashMap::new(),
            flood_inputs: HashMap::new(),
            graph: None,
            propagate: Vec::new(),
            propagated_meanwhile: Vec::new(),
            running: None,
            in_frustum: None,
            graph_generation: 0,
            graph_stale: true,
            flood: Flood::new(),
            graph_from: None,
            workers: SectionWorkers::new(worker_count, renderer, tints),
            worker_count,
            blinks: crate::profile::BlinkWatch::from_env(),
            culling: std::env::var("PHOTON_CULLING").map_or(true, |value| value != "0"),
            sections: low..low + dimension.height / 16,
            stats: MeshStats::default(),
            ambient_occlusion: true,
        }
    }

    /// The `ao` option: a change drops every section and builds them again
    /// (`LevelExtractor.allChanged`, `invalidateCompiledGeometry`).
    pub fn set_ambient_occlusion(
        &mut self,
        value: bool,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.ambient_occlusion == value {
            return Ok(());
        }
        self.ambient_occlusion = value;
        self.rebuild_all(backend)
    }

    /// The block models changed (the resource packs): the workers mesh with the new renderer and
    /// every section is built again.
    pub fn set_renderer(
        &mut self,
        renderer: Arc<photon_render::block::world_mesh::BlockRenderer>,
        tints: Arc<photon_render::block::biome_tint::BiomeTints>,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.workers = SectionWorkers::new(self.worker_count, renderer, tints);
        self.rebuild_all(backend)
    }

    /// `LevelExtractor.allChanged`: every section is dropped and built again (the `ao` option,
    /// F3+A).
    pub fn rebuild_all(
        &mut self,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Built, in flight and waiting: the sections that held something.
        let mut again: Vec<SectionKey> = self.compiled.keys().copied().collect();
        again.extend(self.latest.keys().copied());
        again.extend(self.pending.keys().copied());
        backend.clear_sections()?;
        self.clear();
        self.unchecked.extend(again.iter().copied());
        self.pending
            .extend(again.into_iter().map(|key| (key, false)));
        Ok(())
    }

    /// F3+A, `LevelExtractor.allChanged`: besides what was built, queued or in flight, every
    /// section of every loaded chunk is built again, so one that was lost on the way comes back.
    pub fn rebuild_all_loaded(
        &mut self,
        session: &PlaySession,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rebuild_all(backend)?;
        for (chunk_x, chunk_z) in session.world().loaded_chunk_positions() {
            for section_y in self.sections.clone() {
                let key = (chunk_x, section_y, chunk_z);
                self.pending.entry(key).or_insert(false);
                self.unchecked.push(key);
            }
        }
        Ok(())
    }

    /// The sections of the level the session is in now (a respawn can bring a dimension of
    /// another height, such as the overworld after a lobby in the end).
    pub fn set_dimension(&mut self, session: &PlaySession) {
        let dimension = session.world().dimension();
        let low = dimension.min_y.div_euclid(16);
        self.sections = low..low + dimension.height / 16;
    }

    /// A respawn or a change of dimension: nothing of the old level stays.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.unchecked.clear();
        self.latest.clear();
        self.fades.clear();
        self.compiled.clear();
        self.flood_inputs.clear();
        self.graph = None;
        self.in_frustum = None;
        self.graph_generation += 1;
        self.propagate.clear();
        self.propagated_meanwhile.clear();
        self.graph_stale = true;
        self.flood.forget();
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
                self.fades.remove(key);
                self.compiled.remove(&key);
                self.flood_inputs.remove(&key);
                backend.set_section(
                    key,
                    &photon_render::Mesh::default(),
                    [0.0; 3],
                    SectionFade::none(),
                )?;
            }
        }
        Ok(())
    }

    /// `LevelRenderer.isSectionCompiledAndVisible`: the section has shown and is at least
    /// three tenths into its fade.
    pub fn compiled_and_visible(&self, key: SectionKey) -> bool {
        self.fades.compiled_and_visible(key)
    }

    /// The chunk fade option changed: sections that show for the first time from now on fade in
    /// over this long.
    pub fn set_chunk_fade(&mut self, length: Duration) {
        self.fades.set_option(length);
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
        profile::timed(Span::Light, || session.update_light());
        let dirtied = session.take_dirty_sections();
        let keys: Vec<SectionKey> = dirtied.iter().map(|&(key, _)| key).collect();
        self.note_flood_inputs(session, &keys);
        self.unchecked.extend(keys);
        for (key, by_player) in dirtied {
            let waiting = self.pending.entry(key).or_insert(false);
            *waiting |= by_player;
        }
        let camera_block = view.camera.map(|value| value.floor() as i64);
        loop {
            profile::timed(Span::ShowBuilt, || self.show_built(backend, camera_block))?;
            profile::timed(Span::ShowEmpty, || {
                self.show_empty(session, backend, camera_block)
            })?;
            let visible = profile::timed(Span::Visible, || self.visible(session, view));
            let ordered =
                profile::timed(Span::Ready, || self.ready(session, &visible, camera_block));
            backend.set_visible_sections(self.culling.then_some(visible));
            let nothing_to_hand_out = ordered.is_empty();
            let built_now = profile::timed(Span::HandOut, || {
                self.hand_out(session, ordered, (view.origin, camera_block))
            });
            if built_now {
                profile::timed(Span::ShowBuilt, || self.show_built(backend, camera_block))?;
            }
            let flood_settled = !self.graph_stale && !self.flood.busy();
            if !script_done
                || (nothing_to_hand_out && self.workers.in_flight() == 0 && flood_settled)
            {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// A chunk arriving or a section that stopped being empty passes the
    /// flood on from it.
    fn note_flood_inputs(&mut self, session: &PlaySession, dirtied: &[SectionKey]) {
        let world = session.world();
        for &(x, y, z) in dirtied {
            let inputs = (world.contains_chunk(x, z), world.section_is_empty(x, y, z));
            let before = self.flood_inputs.insert((x, y, z), inputs);
            // `updateEmptySections` and `loadedExpectedChunks`: a section
            // that stopped being empty, or whose chunk arrived.
            let (was_here, was_empty) = before.unwrap_or((false, true));
            if (inputs.0 && !was_here) || (was_empty && !inputs.1) {
                self.propagate.push((x, y, z));
            }
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
                // Most rebuilds leave which faces see each other as it was.
                if self.compiled.insert(built.key, built.visibility) != Some(built.visibility) {
                    self.propagate.push(built.key);
                }
                let fade = self.fades.built(built.key, camera_block);
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
        // A section whose chunk is not here yet is dirtied again when it comes.
        let empty: Vec<SectionKey> = std::mem::take(&mut self.unchecked)
            .into_iter()
            .filter(|key| self.pending.contains_key(key))
            .filter(|&(x, y, z)| world.contains_chunk(x, z) && world.section_is_empty(x, y, z))
            .collect();
        for key in empty {
            self.pending.remove(&key);
            self.latest.remove(&key);
            self.compiled.remove(&key);
            // An empty section has shown too, so what is built in it later
            // does not fade.
            let fade = self.fades.empty(key, camera_block);
            backend.set_section(key, &photon_render::Mesh::default(), [0.0; 3], fade)?;
        }
        Ok(())
    }

    /// The sections in view: the occlusion flood's, run again when the
    /// camera crosses into another eight-block cell or what it floods
    /// through changed, and then the ones in the frustum.
    fn visible(&mut self, session: &PlaySession, view: &SectionView) -> Vec<SectionKey> {
        if !self.culling {
            // Every dirty section within the view distance, in view or not.
            let camera = view
                .camera
                .map(|value| (value.floor() as i32).div_euclid(16));
            return self
                .pending
                .keys()
                .copied()
                .filter(|&(x, _, z)| {
                    (x - camera[0]).abs() <= view.view_distance
                        && (z - camera[2]).abs() <= view.view_distance
                })
                .collect();
        }
        let cell = view.camera.map(|value| (value / 8.0).floor() as i64);
        let smart_cull = smart_cull(session, view.camera);
        if self.graph_from != Some((cell, view.view_distance, smart_cull)) {
            self.graph_from = Some((cell, view.view_distance, smart_cull));
            self.graph_stale = true;
        }
        self.refresh_graph(session, view, smart_cull);
        self.graph_in_view(&view.frustum.offset_to_fully_include_camera_cube(8))
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
        if self.pending.is_empty() {
            return Vec::new();
        }
        let world = session.world();
        let lit = |x: i32, z: i32| {
            world.contains_chunk(x, z) && world.light_engine().light_on_in_column(x, z)
        };
        // A column has up to a few dozen sections in view, and all of them ask about the same
        // eight neighbours, so each column is asked once.
        let mut surrounded: HashMap<(i32, i32), bool> = HashMap::new();
        let mut ordered: Vec<SectionKey> = visible
            .iter()
            .copied()
            .filter(|key| self.pending.contains_key(key))
            .filter(|&(x, y, z)| {
                self.compiled.contains_key(&(x, y, z))
                    || *surrounded.entry((x, z)).or_insert_with(|| {
                        (-1..=1).all(|dx| {
                            (-1..=1).all(|dz| (dx == 0 && dz == 0) || lit(x + dx, z + dz))
                        })
                    })
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
    /// little queued behind it, and spending at most [`COPY_BUDGET`] of the
    /// frame copying.
    /// Sections `prioritizeChunkUpdates` has built on the spot come back as `true`: they show
    /// the frame they are dirtied.
    fn hand_out(
        &mut self,
        session: &PlaySession,
        ordered: Vec<SectionKey>,
        (origin, camera_block): ([i32; 3], [i64; 3]),
    ) -> bool {
        let world = session.world();
        let budget = Instant::now();
        let mut built_now = false;
        for key in ordered {
            let now = self.builds_now(key, camera_block);
            if !now
                && (self.workers.in_flight() >= self.worker_count * 4
                    || budget.elapsed() > COPY_BUDGET)
            {
                // A later section the player changed is still built on the spot.
                continue;
            }
            built_now |= now;
            self.pending.remove(&key);
            let (min, size, quart_min, quart_size) = SectionWorkers::region(key);
            let copy = world.copy_region(min, size, quart_min, quart_size);
            self.next_generation += 1;
            self.latest.insert(key, self.next_generation);
            let cardinal = world.dimension_environment().cardinal_light_type();
            let job = (key, self.next_generation);
            if now {
                self.workers
                    .compile_now(job, copy, origin, cardinal, self.ambient_occlusion);
            } else {
                self.workers
                    .submit(job.0, job.1, copy, origin, cardinal, self.ambient_occlusion);
            }
        }
        built_now
    }

    /// `LevelRenderer.compileSections`: whether a dirty section is compiled before the frame
    /// goes on rather than by a worker.
    fn builds_now(&self, key: SectionKey, camera_block: [i64; 3]) -> bool {
        use photon_game::options::PrioritizeChunkUpdates as Priority;
        let by_player = self.pending.get(&key).copied().unwrap_or(false);
        match self.prioritize {
            Priority::None => false,
            Priority::PlayerAffected => by_player,
            Priority::Nearby => by_player || fade::near(key, camera_block),
        }
    }

    /// `prioritizeChunkUpdates`.
    pub fn set_prioritize(&mut self, prioritize: photon_game::options::PrioritizeChunkUpdates) {
        self.prioritize = prioritize;
    }

    /// How many sections wait to be built.
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// `PHOTON_PROFILE`'s line about the meshes built since the last one.
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

/// `Camera.extractRenderState`: smart culling is off for a spectator whose
/// camera is inside a block that fills its cube, who sees through the rock.
fn smart_cull(session: &PlaySession, camera: [f64; 3]) -> bool {
    let [x, y, z] = camera.map(|value| value.floor() as i32);
    !(session.game_mode().is_spectator()
        && session
            .world()
            .block_state_at(x, y, z)
            .and_then(photon_world::block::state)
            .is_some_and(|state| state.solid_render))
}
