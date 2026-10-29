//! The occlusion flood's full update on a thread of its own, as the
//! reference runs it in the background (`scheduleFullUpdate`): the frames
//! keep the last result until the next is ready, and a new update starts
//! only once the last one is done.

use std::ops::Range;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread::JoinHandle;

use photon_render::section_workers::SectionKey;
use photon_render::visibility::{GraphWorld, OcclusionGraph, SectionState, VisibilitySet};

/// What a flood was started with, which later passes of it keep.
#[derive(Clone, Copy)]
pub struct FloodParams {
    pub camera: [f64; 3],
    pub view_distance: i32,
    pub smart_cull: bool,
}

/// What the flood reads of the world, copied for the sections within the
/// view distance of the camera's column, so the world can change while it
/// runs.
pub struct FloodInputs {
    camera: [f64; 3],
    view_distance: i32,
    /// `smartCull`: off only for a spectator inside blocks.
    smart_cull: bool,
    /// The dimension's sections, bottom to top.
    sections: Range<i32>,
    /// The view area's lowest column and its width in columns.
    low: (i32, i32),
    width: i32,
    /// Whether each column's chunk has arrived.
    loaded: Vec<bool>,
    /// Each section's state, column by column, bottom to top.
    states: Vec<SectionState>,
}

impl FloodInputs {
    /// Copy what the flood reads around `camera`: `column` fills a loaded
    /// column's states, bottom to top, and says whether it is loaded. The
    /// columns are copied one after the other.
    pub fn capture(
        camera: [f64; 3],
        view_distance: i32,
        smart_cull: bool,
        sections: Range<i32>,
        mut column: impl FnMut(i32, i32, &mut [SectionState]) -> bool,
    ) -> Self {
        let reach = view_distance.max(0);
        let [x, z] = [camera[0], camera[2]].map(|value| (value.floor() as i32).div_euclid(16));
        let low = (x - reach, z - reach);
        let width = 2 * reach + 1;
        let height = sections.len();
        // The flood never reads a section of a column not here.
        let mut states = vec![SectionState::Uncompiled; (width * width) as usize * height];
        let column_loaded: Vec<bool> = states
            .chunks_mut(height.max(1))
            .enumerate()
            .map(|(index, states)| {
                let index = index as i32;
                column(low.0 + index / width, low.1 + index % width, states)
            })
            .collect();
        Self {
            camera,
            view_distance,
            smart_cull,
            sections,
            low,
            width,
            loaded: column_loaded,
            states,
        }
    }

    /// Mark the compiled sections: each of `compiled` inside the copied area that is not empty.
    /// Walking the few compiled sections instead of asking for every section of the area is
    /// what keeps the copy cheap on the frame that makes it.
    pub fn mark_compiled(&mut self, compiled: impl Iterator<Item = (SectionKey, VisibilitySet)>) {
        let height = self.sections.len();
        for ((x, y, z), visibility) in compiled {
            let Some(column) = self.column(x, z).filter(|_| self.sections.contains(&y)) else {
                continue;
            };
            let state = &mut self.states[column * height + (y - self.sections.start) as usize];
            if !matches!(state, SectionState::Empty) {
                *state = SectionState::Compiled(visibility);
            }
        }
    }

    /// The camera, view distance and smart culling the copy was made for.
    pub fn params(&self) -> FloodParams {
        FloodParams {
            camera: self.camera,
            view_distance: self.view_distance,
            smart_cull: self.smart_cull,
        }
    }

    /// The column of a chunk inside the copied area.
    fn column(&self, x: i32, z: i32) -> Option<usize> {
        let (dx, dz) = (x - self.low.0, z - self.low.1);
        ((0..self.width).contains(&dx) && (0..self.width).contains(&dz))
            .then(|| (dx * self.width + dz) as usize)
    }

    /// The flood over the copy.
    fn run(&self) -> OcclusionGraph {
        let height = self.sections.len();
        let loaded = |x: i32, z: i32| self.column(x, z).is_some_and(|column| self.loaded[column]);
        let section = |(x, y, z): SectionKey| {
            self.column(x, z)
                .filter(|_| self.sections.contains(&y))
                .map_or(SectionState::Uncompiled, |column| {
                    self.states[column * height + (y - self.sections.start) as usize]
                })
        };
        OcclusionGraph::full(&GraphWorld {
            camera: self.camera,
            view_distance: self.view_distance,
            min_section_y: self.sections.start,
            max_section_y: self.sections.end - 1,
            loaded: &loaded,
            section: &section,
            smart_cull: self.smart_cull,
        })
    }
}

/// The thread that runs the full updates, one at a time.
pub struct Flood {
    jobs: Option<Sender<(u64, FloodInputs)>>,
    results: Receiver<(u64, OcclusionGraph)>,
    thread: Option<JoinHandle<()>>,
    /// The generation of the update running, if one is.
    running: Option<u64>,
    /// Bumped when the level goes, so an update over the old one is dropped.
    generation: u64,
}

impl Flood {
    pub fn new() -> Self {
        let (jobs, job_queue) = channel::<(u64, FloodInputs)>();
        let (result_queue, results) = channel();
        let thread = std::thread::Builder::new()
            .name("occlusion".to_owned())
            .spawn(move || {
                for (generation, inputs) in job_queue {
                    if result_queue.send((generation, inputs.run())).is_err() {
                        break;
                    }
                }
            })
            .expect("the occlusion thread starts");
        Self {
            jobs: Some(jobs),
            results,
            thread: Some(thread),
            running: None,
            generation: 0,
        }
    }

    /// Whether an update is running.
    pub fn busy(&self) -> bool {
        self.running.is_some()
    }

    /// Start an update; the caller waits until none is running.
    pub fn start(&mut self, inputs: FloodInputs) {
        if let Some(jobs) = &self.jobs
            && jobs.send((self.generation, inputs)).is_ok()
        {
            self.running = Some(self.generation);
        }
    }

    /// The update that finished since the last call, unless the level it
    /// flooded has gone.
    pub fn finished(&mut self) -> Option<OcclusionGraph> {
        match self.results.try_recv() {
            Ok((generation, graph)) => {
                self.running = None;
                (generation == self.generation).then_some(graph)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.running = None;
                None
            }
        }
    }

    /// The level went: whatever is running floods one that is no more.
    pub fn forget(&mut self) {
        self.generation += 1;
    }
}

impl Drop for Flood {
    fn drop(&mut self) {
        // Closing the queue ends the thread once its update is done.
        self.jobs = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world of mixed sections with some columns missing, flooded from
    /// above, inside and below it.
    #[test]
    fn the_copy_floods_as_the_world_does() {
        for smart_cull in [true, false] {
            copy_floods_as_the_world_does(smart_cull);
        }
    }

    /// Without smart culling a camera inside rock still floods through it.
    #[test]
    fn a_spectator_inside_rock_floods_through_it() {
        let rock = |_: SectionKey| SectionState::Compiled(VisibilitySet::NONE);
        let flood = |smart_cull| {
            FloodInputs::capture([8.5, 40.0, 8.5], 3, smart_cull, 0..8, |x, z, states| {
                for (y, state) in (0..8).zip(states) {
                    *state = rock((x, y, z));
                }
                true
            })
            .run()
            .reached()
            .len()
        };
        assert!(
            flood(true) <= 7,
            "smart culling stops at the camera's walls"
        );
        assert!(flood(false) > 7 * 7 * 4, "the spectator sees through rock");
    }

    /// Marking the compiled sections after the copy gives the copy that asks for each section.
    #[test]
    fn marking_the_compiled_sections_equals_asking_for_each() {
        let state = |(x, y, z): SectionKey| match (x * 7 + y * 3 + z * 5).rem_euclid(5) {
            0 | 1 => SectionState::Empty,
            2 => SectionState::Uncompiled,
            3 => SectionState::Compiled(VisibilitySet::ALL),
            _ => SectionState::Compiled(VisibilitySet::NONE),
        };
        let loaded = |x: i32, z: i32| (x + 2 * z).rem_euclid(9) != 0;
        let camera = [8.5, 300.0, -40.2];
        let asked = FloodInputs::capture(camera, 6, true, -4..20, |x, z, states| {
            for (y, slot) in (-4..20).zip(states) {
                *slot = state((x, y, z));
            }
            loaded(x, z)
        });
        let mut compiled = Vec::new();
        let mut marked = FloodInputs::capture(camera, 6, true, -4..20, |x, z, states| {
            for (y, slot) in (-4..20).zip(states) {
                *slot = match state((x, y, z)) {
                    SectionState::Empty => SectionState::Empty,
                    SectionState::Compiled(visibility) => {
                        compiled.push(((x, y, z), visibility));
                        SectionState::Uncompiled
                    }
                    other => other,
                };
            }
            loaded(x, z)
        });
        // Sections outside the area are in the list too and are ignored.
        compiled.push(((500, 0, 0), VisibilitySet::ALL));
        marked.mark_compiled(compiled.into_iter());
        assert_eq!(asked.states, marked.states);
        assert_eq!(asked.loaded, marked.loaded);
    }

    fn copy_floods_as_the_world_does(smart_cull: bool) {
        let state = |(x, y, z): SectionKey| match (x * 7 + y * 3 + z * 5).rem_euclid(5) {
            0 | 1 => SectionState::Empty,
            2 => SectionState::Uncompiled,
            3 => SectionState::Compiled(VisibilitySet::ALL),
            _ => SectionState::Compiled(VisibilitySet::NONE),
        };
        let loaded = |x: i32, z: i32| (x + 2 * z).rem_euclid(9) != 0;
        for camera in [[8.5, 300.0, -40.2], [-70.1, 20.4, 33.3], [5.0, -90.0, 5.0]] {
            let direct = photon_render::visibility::visible_sections(&GraphWorld {
                camera,
                view_distance: 6,
                min_section_y: -4,
                max_section_y: 19,
                loaded: &loaded,
                section: &state,
                smart_cull,
            });
            let copy = FloodInputs::capture(camera, 6, smart_cull, -4..20, |x, z, states| {
                if loaded(x, z) {
                    for (y, slot) in (-4..20).zip(states) {
                        *slot = state((x, y, z));
                    }
                }
                loaded(x, z)
            });
            assert!(!direct.is_empty());
            assert_eq!(copy.run().reached(), direct);
        }
    }
}
