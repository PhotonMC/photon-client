//! Section meshes handed to the backend, and the retiring of the ones it replaces.

use super::*;

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
                self.retired.push((self.frames_submitted, old));
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
            let all_indices: Vec<u32> = mesh
                .indices
                .iter()
                .chain(&mesh.translucent_indices)
                .copied()
                .collect();
            let index_bytes = as_bytes(&all_indices);
            let indices = self.arena_upload(index_bytes)?;
            Ok(Some(SectionBuffers {
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
            self.retired
                .extend(old.into_iter().map(|section| (submitted, section)));
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
    pub fn set_visible_sections(&mut self, visible: Option<&[(i32, i32, i32)]>) {
        self.visible_sections = visible.map(<[_]>::to_vec);
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
            for (_, section) in std::mem::take(&mut self.retired) {
                self.destroy_section(section);
            }
            // Their copies have nowhere to go now.
            self.staged.clear();
            self.staging_filling.clear();
            Ok(())
        }
    }

    /// Where translucent sections are ordered from, usually the camera.
    pub fn set_sort_origin(&mut self, origin: [f32; 3]) {
        self.sort_origin = origin;
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
