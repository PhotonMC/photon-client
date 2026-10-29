//! The occlusion graph's upkeep for the section meshes: the full floods on
//! their thread, and the passes of the flood from sections that changed.

use photon_net::connection::PlaySession;
use photon_render::section_workers::SectionKey;
use photon_render::visibility::{GraphWorld, OcclusionGraph, SectionState};

use super::{FloodInputs, FloodParams, SectionMeshes, SectionView};
use crate::profile::{self, Span};

impl SectionMeshes {
    /// Take the flood's full result when it is done, pass the flood on from
    /// the sections that changed, and start another full one if the camera
    /// went elsewhere.
    pub(super) fn refresh_graph(
        &mut self,
        session: &PlaySession,
        view: &SectionView,
        smart_cull: bool,
    ) {
        if let Some(blinks) = self.blinks.as_mut() {
            blinks.frame();
        }
        if let Some(mut graph) = self.flood.finished() {
            let params = self
                .running
                .take()
                .expect("a flood that ran has its params");
            let again = std::mem::take(&mut self.propagated_meanwhile);
            self.pass_on(session, &mut graph, params, &again);
            if let Some(blinks) = self.blinks.as_mut() {
                let frustum = view.frustum.offset_to_fully_include_camera_cube(8);
                let old = self
                    .graph
                    .as_ref()
                    .map_or(&[][..], |(graph, _)| graph.reached());
                let same_cull = self
                    .graph
                    .as_ref()
                    .is_none_or(|(_, before)| before.smart_cull == params.smart_cull);
                blinks.swapped(old, graph.reached(), same_cull, |key| {
                    frustum.section_visible(*key)
                });
            }
            self.graph = Some((graph, params));
            self.graph_generation += 1;
        }
        if !self.propagate.is_empty() {
            let from = std::mem::take(&mut self.propagate);
            if let Some((mut graph, params)) = self.graph.take() {
                if self.pass_on(session, &mut graph, params, &from) > 0 {
                    self.graph_generation += 1;
                }
                self.graph = Some((graph, params));
            }
            if self.flood.busy() {
                self.propagated_meanwhile.extend(from);
            }
        }
        // The flood runs on its own thread; the frames keep the last one
        // meanwhile, and one that went stale starts again when it is done.
        if self.graph_stale && !self.flood.busy() {
            self.graph_stale = false;
            let inputs =
                profile::timed(Span::Graph, || self.flood_inputs(session, view, smart_cull));
            self.running = Some(inputs.params());
            self.propagated_meanwhile.clear();
            self.flood.start(inputs);
        }
    }

    /// `runPartialUpdate`: the flood goes on from `from` over the world as
    /// it is now; how many sections joined.
    fn pass_on(
        &self,
        session: &PlaySession,
        graph: &mut OcclusionGraph,
        params: FloodParams,
        from: &[SectionKey],
    ) -> usize {
        let world = session.world();
        let state = |key: SectionKey| {
            let (x, y, z) = key;
            if world.section_is_empty(x, y, z) {
                SectionState::Empty
            } else {
                self.compiled
                    .get(&key)
                    .map_or(SectionState::Uncompiled, |visibility| {
                        SectionState::Compiled(*visibility)
                    })
            }
        };
        graph.propagate(
            &GraphWorld {
                camera: params.camera,
                view_distance: params.view_distance,
                min_section_y: self.sections.start,
                max_section_y: self.sections.end - 1,
                loaded: &|x, z| world.contains_chunk(x, z),
                section: &state,
                smart_cull: params.smart_cull,
            },
            from,
        )
    }

    /// The flood's sections in the frustum; the same answer while neither
    /// moves.
    pub(super) fn graph_in_view(
        &mut self,
        frustum: &photon_render::visibility::Frustum,
    ) -> Vec<SectionKey> {
        if let Some((seen, generation, drawn)) = &self.in_frustum
            && seen == frustum
            && *generation == self.graph_generation
        {
            return drawn.clone();
        }
        let drawn: Vec<SectionKey> = self
            .graph
            .iter()
            .flat_map(|(graph, _)| graph.reached())
            .copied()
            .filter(|key| frustum.section_visible(*key))
            .collect();
        self.in_frustum = Some((*frustum, self.graph_generation, drawn.clone()));
        drawn
    }

    /// What the flood reads of the world around the camera, copied.
    fn flood_inputs(
        &self,
        session: &PlaySession,
        view: &SectionView,
        smart_cull: bool,
    ) -> FloodInputs {
        let world = session.world();
        let compiled = &self.compiled;
        let sections = self.sections.clone();
        let column = |x: i32, z: i32, states: &mut [SectionState]| {
            let Some(empty) = world.column_is_empty(x, z, sections.clone()) else {
                return false;
            };
            for (empty, state) in empty.zip(states) {
                *state = if empty {
                    SectionState::Empty
                } else {
                    SectionState::Uncompiled
                };
            }
            true
        };
        let mut inputs = FloodInputs::capture(
            view.camera,
            view.view_distance,
            smart_cull,
            self.sections.clone(),
            column,
        );
        inputs.mark_compiled(compiled.iter().map(|(key, visibility)| (*key, *visibility)));
        inputs
    }
}
