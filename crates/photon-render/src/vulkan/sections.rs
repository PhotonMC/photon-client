//! Section meshes handed to the backend, and the retiring of the ones it replaces.

use super::*;
use crate::translucent::point_of_view;

impl VulkanBackend {
    /// Replace one section's mesh, or remove it when the mesh is empty.
    pub fn set_section(
        &mut self,
        key: (i32, i32, i32),
        mesh: &Mesh,
        centre: [f32; 3],
        fade: SectionFade,
    ) -> Result<(), VulkanError> {
        // SAFETY: live handles. The old section is retired, not freed, since the frames in flight
        // may still read it; `upload_section` needs only live handles and the exclusive access
        // `&mut self` gives.
        unsafe {
            // The frames in flight may still read the old mesh, so it is
            // freed after the last of them.
            if let Some(old) = self.sections.remove(&key) {
                self.retired
                    .push((self.frames_submitted, Retired::Section(old)));
            }
            if let Some(section) = self.upload_section(mesh, centre, fade)? {
                self.sections.insert(key, section);
            }
            Ok(())
        }
    }

    /// Give the arena a section's mesh, or nothing for an empty one.
    ///
    /// # Safety
    ///
    /// As [`Self::arena_upload`].
    pub(super) unsafe fn upload_section(
        &mut self,
        mesh: &Mesh,
        centre: [f32; 3],
        fade: SectionFade,
    ) -> Result<Option<SectionBuffers>, VulkanError> {
        // SAFETY: live handles; the bytes come from `as_bytes` over plain vertex and index data.
        unsafe {
            if mesh.indices.is_empty() && mesh.translucent_indices.is_empty() {
                return Ok(None);
            }
            let vertex_bytes = as_bytes(&mesh.vertices);
            let vertices = self.arena_upload(vertex_bytes)?;
            let mut all_indices: Vec<u32> = mesh
                .indices
                .iter()
                .chain(&mesh.translucent_indices)
                .copied()
                .collect();
            let sort = (!mesh.translucent_indices.is_empty()).then(|| {
                let mut sort = TranslucentSort::new(mesh, all_indices.clone());
                sort.order(self.sort_origin, centre);
                all_indices.clone_from(&sort.indices);
                Box::new(sort)
            });
            let index_bytes = as_bytes(&all_indices);
            let indices = self.arena_upload(index_bytes)?;
            Ok(Some(SectionBuffers {
                sort,
                vertices,
                indices,
                index_count: mesh.indices.len() as u32,
                translucent_index_count: mesh.translucent_indices.len() as u32,
                centre,
                fade,
            }))
        }
    }

    /// Hand over the next frame's moving blocks (the falling blocks): each a mesh with the place
    /// it is sorted from, drawn with the terrain.
    pub fn set_moving_blocks(&mut self, blocks: &[(&Mesh, [f32; 3])]) -> Result<(), VulkanError> {
        // SAFETY: live handles. The old meshes are retired, not freed, since the frames in flight
        // may still read them.
        unsafe {
            // The frames in flight may still read the old meshes.
            let old = std::mem::take(&mut self.moving_sections);
            let submitted = self.frames_submitted;
            self.retired.extend(
                old.into_iter()
                    .map(|section| (submitted, Retired::Section(section))),
            );
            for (mesh, centre) in blocks {
                if let Some(section) = self.upload_section(mesh, *centre, SectionFade::none())? {
                    self.moving_sections.push(section);
                }
            }
            Ok(())
        }
    }

    /// Draw only these sections from now on (the ones in view), or every
    /// section for `None`.
    pub fn set_visible_sections(&mut self, visible: Option<Vec<(i32, i32, i32)>>) {
        self.visible_sections = visible;
    }

    /// The sections drawn this frame: the ones in view, or all of them.
    pub(super) fn drawn_sections(&self) -> Vec<&SectionBuffers> {
        let mut drawn: Vec<&SectionBuffers> = match &self.visible_sections {
            Some(visible) => visible
                .iter()
                .filter_map(|key| self.sections.get(key))
                .collect(),
            None => self.sections.values().collect(),
        };
        drawn.extend(&self.moving_sections);
        drawn
    }

    /// Drop every section mesh.
    /// Draw only the GUI (a menu, no level), or everything again.
    pub fn set_gui_only(&mut self, gui_only: bool) {
        self.gui_only = gui_only;
    }

    pub fn clear_sections(&mut self) -> Result<(), VulkanError> {
        // SAFETY: the fence wait leaves the GPU done with every section, and `take` empties each
        // list, so every range goes back to the arena once.
        unsafe {
            self.wait_in_flight()?;
            for (_, section) in std::mem::take(&mut self.sections) {
                self.destroy_section(section);
            }
            for section in std::mem::take(&mut self.moving_sections) {
                self.destroy_section(section);
            }
            for (_, retired) in std::mem::take(&mut self.retired) {
                self.destroy_retired(retired);
            }
            // Their copies have nowhere to go now.
            self.staged.clear();
            self.staging_filling.clear();
            Ok(())
        }
    }

    /// What the section meshes hold: how many sections, the bytes of their vertices and of their
    /// indices, and the bytes of the arena blocks that carry them.
    pub fn section_memory(&self) -> SectionMemory {
        let sections = self.sections.values().chain(&self.moving_sections);
        let (mut count, mut vertex_bytes, mut index_bytes) = (0, 0, 0);
        for section in sections {
            count += 1;
            vertex_bytes += section.vertices.size;
            index_bytes += section.indices.size;
        }
        SectionMemory {
            sections: count,
            vertex_bytes,
            index_bytes,
            arena_bytes: self.arena.iter().flatten().map(|block| block.size).sum(),
        }
    }

    /// Where translucent sections are ordered from, usually the camera.
    pub fn set_sort_origin(&mut self, origin: [f32; 3]) {
        let moved = origin.map(f32::floor) != self.sort_origin.map(f32::floor);
        self.sort_origin = origin;
        // SAFETY: live handles and the exclusive access `&mut self` gives.
        unsafe { self.resort_translucent(moved) };
    }

    /// `LevelRenderer.scheduleResort`: order the translucent faces of a
    /// section again when the camera stands elsewhere among its neighbours,
    /// or, once it has moved to another block, when it stands level with the
    /// section on some axis.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    unsafe fn resort_translucent(&mut self, moved: bool) {
        let origin = self.sort_origin;
        let due: Vec<(i32, i32, i32)> = self
            .sections
            .iter()
            .filter_map(|(key, section)| {
                let sort = section.sort.as_ref()?;
                let view = point_of_view(origin, section.centre);
                let aligned = view.contains(&0);
                (view != sort.point_of_view || (moved && aligned)).then_some(*key)
            })
            .collect();
        for key in due {
            let Some(section) = self.sections.get_mut(&key) else {
                continue;
            };
            let Some(mut sort) = section.sort.take() else {
                continue;
            };
            sort.order(origin, section.centre);
            // SAFETY: live handles; the bytes are plain indices.
            let Ok(indices) = (unsafe { self.arena_upload(as_bytes(&sort.indices)) }) else {
                continue;
            };
            let Some(section) = self.sections.get_mut(&key) else {
                continue;
            };
            let old = std::mem::replace(&mut section.indices, indices);
            section.sort = Some(sort);
            self.retired
                .push((self.frames_submitted, Retired::Indices(old)));
        }
    }

    /// Give a retired range back to the arena.
    ///
    /// # Safety
    ///
    /// The GPU must be done with the ranges.
    pub(super) unsafe fn destroy_retired(&mut self, retired: Retired) {
        match retired {
            // SAFETY: the caller's contract.
            Retired::Section(section) => unsafe { self.destroy_section(section) },
            Retired::Indices(slice) => self.arena_free(slice),
        }
    }

    /// Give a section's ranges back to the arena; the GPU must be done
    /// with them.
    ///
    /// # Safety
    ///
    /// The GPU must be done with the section's ranges.
    pub(super) unsafe fn destroy_section(&mut self, section: SectionBuffers) {
        self.arena_free(section.vertices);
        self.arena_free(section.indices);
        // SAFETY: the caller guarantees the GPU is done with the section's ranges, and
        // `trim_arena` has the same contract.
        unsafe { self.trim_arena() };
    }
}

impl TranslucentSort {
    /// The faces of a mesh, whose indices (`indices`) lead with its opaque ones.
    fn new(mesh: &Mesh, indices: Vec<u32>) -> Self {
        let centroids = mesh
            .translucent_indices
            .as_chunks::<6>()
            .0
            .iter()
            .map(|face| {
                // `decodeQuadCentroids`: the middle of the first and third corners.
                let (first, third) = (
                    &mesh.vertices[face[0] as usize],
                    &mesh.vertices[face[2] as usize],
                );
                std::array::from_fn(|axis| (first.position[axis] + third.position[axis]) / 2.0)
            })
            .collect();
        Self {
            opaque: mesh.indices.len(),
            indices,
            centroids,
            point_of_view: [0; 3],
        }
    }

    /// Put the translucent faces in the order they are drawn.
    fn order(&mut self, camera: [f32; 3], centre: [f32; 3]) {
        self.point_of_view = point_of_view(camera, centre);
        let order = crate::translucent::order(camera, &self.centroids);
        let faces: Vec<[u32; 6]> = self.indices[self.opaque..].as_chunks::<6>().0.to_vec();
        self.indices.truncate(self.opaque);
        self.indices
            .extend(order.into_iter().flat_map(|index| faces[index]));
    }
}
