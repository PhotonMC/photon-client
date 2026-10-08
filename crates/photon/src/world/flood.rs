//! The occlusion flood's full update on a thread of its own, as the
//! reference runs it in the background (`scheduleFullUpdate`): the frames
//! keep the last result until the next is ready, and a new update starts
//! only once the last one is done.

use std::ops::Range;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread::JoinHandle;

use photon_render::section_workers::SectionKey;
use photon_render::visibility::{GraphWorld, SectionState, visible_sections};

/// What the flood reads of the world, copied for the sections within the
/// view distance of the camera's column, so the world can change while it
/// runs.
pub struct FloodInputs {
    camera: [f64; 3],
    view_distance: i32,
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
    /// Copy what the flood reads around `camera`: `column` adds a loaded
    /// column's states, bottom to top, and says whether it is loaded.
    pub fn capture(
        camera: [f64; 3],
        view_distance: i32,
        sections: Range<i32>,
        column: impl Fn(i32, i32, &mut Vec<SectionState>) -> bool,
    ) -> Self {
        let reach = view_distance.max(0);
        let [x, z] = [camera[0], camera[2]].map(|value| (value.floor() as i32).div_euclid(16));
        let low = (x - reach, z - reach);
        let width = 2 * reach + 1;
        let mut column_loaded = Vec::with_capacity((width * width) as usize);
        let mut states = Vec::with_capacity((width * width) as usize * sections.len());
        for column_x in low.0..low.0 + width {
            for column_z in low.1..low.1 + width {
                let here = column(column_x, column_z, &mut states);
                column_loaded.push(here);
                if !here {
                    // The flood never reads a section of a column not here.
                    states.extend(sections.clone().map(|_| SectionState::Uncompiled));
                }
            }
        }
        Self {
            camera,
            view_distance,
            sections,
            low,
            width,
            loaded: column_loaded,
            states,
        }
    }

    /// The column of a chunk inside the copied area.
    fn column(&self, x: i32, z: i32) -> Option<usize> {
        let (dx, dz) = (x - self.low.0, z - self.low.1);
        ((0..self.width).contains(&dx) && (0..self.width).contains(&dz))
            .then(|| (dx * self.width + dz) as usize)
    }

    /// The flood over the copy.
    fn run(&self) -> Vec<SectionKey> {
        let height = self.sections.len();
        let loaded = |x: i32, z: i32| self.column(x, z).is_some_and(|column| self.loaded[column]);
        let section = |(x, y, z): SectionKey| {
            self.column(x, z)
                .filter(|_| self.sections.contains(&y))
                .map_or(SectionState::Uncompiled, |column| {
                    self.states[column * height + (y - self.sections.start) as usize]
                })
        };
        visible_sections(&GraphWorld {
            camera: self.camera,
            view_distance: self.view_distance,
            min_section_y: self.sections.start,
            max_section_y: self.sections.end - 1,
            loaded: &loaded,
            section: &section,
            smart_cull: true,
        })
    }
}

/// The thread that runs the full updates, one at a time.
pub struct Flood {
    jobs: Option<Sender<(u64, FloodInputs)>>,
    results: Receiver<(u64, Vec<SectionKey>)>,
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
    pub fn finished(&mut self) -> Option<Vec<SectionKey>> {
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
    use photon_render::visibility::VisibilitySet;

    use super::*;

    /// A world of mixed sections with some columns missing, flooded from
    /// above, inside and below it.
    #[test]
    fn the_copy_floods_as_the_world_does() {
        let state = |(x, y, z): SectionKey| match (x * 7 + y * 3 + z * 5).rem_euclid(5) {
            0 | 1 => SectionState::Empty,
            2 => SectionState::Uncompiled,
            3 => SectionState::Compiled(VisibilitySet::ALL),
            _ => SectionState::Compiled(VisibilitySet::NONE),
        };
        let loaded = |x: i32, z: i32| (x + 2 * z).rem_euclid(9) != 0;
        for camera in [[8.5, 300.0, -40.2], [-70.1, 20.4, 33.3], [5.0, -90.0, 5.0]] {
            let direct = visible_sections(&GraphWorld {
                camera,
                view_distance: 6,
                min_section_y: -4,
                max_section_y: 19,
                loaded: &loaded,
                section: &state,
                smart_cull: true,
            });
            let copy = FloodInputs::capture(camera, 6, -4..20, |x, z, states| {
                if loaded(x, z) {
                    states.extend((-4..20).map(|y| state((x, y, z))));
                }
                loaded(x, z)
            });
            assert!(!direct.is_empty());
            assert_eq!(copy.run(), direct);
        }
    }
}
