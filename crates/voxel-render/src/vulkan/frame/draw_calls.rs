use super::*;

impl VulkanBackend {
    /// Record the draw for one swapchain image.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording, and the section's ranges must be live arena ranges.
    pub(in super::super) unsafe fn bind_section(
        &self,
        section: &SectionBuffers,
        layout: vk::PipelineLayout,
        now: std::time::Instant,
    ) {
        // SAFETY: recording on `command_buffer`; the section's ranges are in arena blocks that
        // exist, which indexing `arena` checks.
        unsafe {
            self.device.cmd_push_constants(
                self.command_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                160,
                &section.fade.visibility(now).to_ne_bytes(),
            );
            self.device.cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                &[self.arena[section.vertices.block].buffer],
                &[section.vertices.offset],
            );
            self.device.cmd_bind_index_buffer(
                self.command_buffer,
                self.arena[section.indices.block].buffer,
                section.indices.offset,
                vk::IndexType::UINT32,
            );
        }
    }

    /// Dissolving entities (a dying dragon's body), over the entities' own draw: its push constants
    /// carry where the masks are.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope, and `scene` must be
    /// the live scene. `push` needs at least 40 floats (a shorter one panics).
    pub(in super::super) unsafe fn record_dissolve(
        &self,
        scene: &SceneResources,
        terrain_push: &[f32],
    ) {
        // SAFETY: recording; the pipeline, layout and set belong to `scene` and the vertex buffer
        // to `self`, all live. The push constants are a bounds-checked prefix of `push`.
        unsafe {
            if let (Some(buffer), Some(set)) = (self.dissolve.buffer.as_ref(), scene.entity_set)
                && self.dissolve.count > 0
            {
                let mut push = [0.0_f32; 44];
                push[..40].copy_from_slice(&terrain_push[..40]);
                push[40] = self.dissolve_mask_offset[0];
                push[41] = self.dissolve_mask_offset[1];
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.dissolve_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.dissolve_pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.dissolve_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&push)[..176],
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.dissolve.count, 1, 0, 0);
            }
        }
    }

    /// The maps in item frames: quads of the maps' image, drawn as text is.
    ///
    /// # Safety
    ///
    /// As [`Self::draw_triangles`].
    pub(in super::super) unsafe fn record_maps(
        &self,
        scene: &SceneResources,
        terrain_push: &[f32],
    ) {
        // SAFETY: the callee's contract is the recording one, which this function's own contract
        // passes on.
        unsafe {
            if let (Some(buffer), Some(set)) = (self.map.buffer.as_ref(), scene.map_set) {
                self.draw_triangles(
                    (scene.text_pipeline, scene.text_pipeline_layout),
                    set,
                    (buffer, self.map.count),
                    terrain_push,
                );
            }
            if let (Some(buffer), Some(set)) =
                (self.map_decoration.buffer.as_ref(), scene.entity_set)
            {
                self.draw_triangles(
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    set,
                    (buffer, self.map_decoration.count),
                    terrain_push,
                );
            }
        }
    }

    /// The shadows, experience orbs and name tags: flat shapes drawn after the
    /// entities, each from its own buffer.
    ///
    /// # Safety
    ///
    /// As [`Self::draw_triangles`].
    pub(in super::super) unsafe fn record_flat_shapes(
        &self,
        scene: &SceneResources,
        terrain_push: &[f32],
    ) {
        // SAFETY: the callees' contract is the recording one, which this function's own contract
        // passes on.
        unsafe {
            // Shadows under the entities, before them: blended quads of the
            // shadow texture over the blocks' tops.
            if let (Some(buffer), Some(set)) = (self.shadow.buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    set,
                    (buffer, self.shadow.count),
                    terrain_push,
                );
            }
            self.record_dissolve(scene, terrain_push);
            // The glowing layers (the eyes of spiders and endermen): the
            // `EYES` pipeline is the shadow's shaders and state, the texel
            // times the colour with no light and no cut-out, then fog.
            if let (Some(buffer), Some(set)) = (self.glow.buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    set,
                    (buffer, self.glow.count),
                    terrain_push,
                );
            }
            // The energy swirls (a charged creeper's), the enchantments' glints (over the
            // depth the entities wrote) and a dying dragon's rays (by their alpha): added over
            // what is behind.
            if let Some(set) = scene.entity_set {
                for (buffer, count, pipeline) in [
                    (
                        self.swirl.buffer.as_ref(),
                        self.swirl.count,
                        (scene.swirl_pipeline, scene.swirl_pipeline_layout),
                    ),
                    (
                        self.glint.buffer.as_ref(),
                        self.glint.count,
                        (scene.glint_pipeline, scene.glint_pipeline_layout),
                    ),
                    (
                        self.rays.buffer.as_ref(),
                        self.rays.count,
                        (scene.rays_pipeline, scene.rays_pipeline_layout),
                    ),
                ] {
                    if let Some(buffer) = buffer {
                        self.draw_triangles(pipeline, set, (buffer, count), terrain_push);
                    }
                }
            }
            // Experience orbs: blended quads over the entity textures.
            if let (Some(buffer), Some(set)) = (self.orb.buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    set,
                    (buffer, self.orb.count),
                    terrain_push,
                );
            }
            self.record_maps(scene, terrain_push);
            // Name tags after the entities: the text seen through blocks
            // first, then the text tested against the depth.
            if let Some(set) = scene.gui_set {
                for (buffer, count, pipeline) in [
                    (
                        self.name_tag_see.buffer.as_ref(),
                        self.name_tag_see.count,
                        (
                            scene.text_see_through_pipeline,
                            scene.text_see_through_pipeline_layout,
                        ),
                    ),
                    (
                        self.name_tag.buffer.as_ref(),
                        self.name_tag.count,
                        (scene.text_pipeline, scene.text_pipeline_layout),
                    ),
                ] {
                    if let Some(buffer) = buffer {
                        self.draw_triangles(pipeline, set, (buffer, count), terrain_push);
                    }
                }
            }
        }
    }

    /// Draw a buffer's triangles with a pipeline over a descriptor set and the
    /// frame's push constants. Nothing is drawn for an empty buffer.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope. `push` needs at least
    /// 40 floats (a shorter one panics).
    pub(in super::super) unsafe fn draw_triangles(
        &self,
        (pipeline, layout): (vk::Pipeline, vk::PipelineLayout),
        set: vk::DescriptorSet,
        (buffer, count): (&HostBuffer, u32),
        push: &[f32],
    ) {
        if count == 0 {
            return;
        }
        // SAFETY: recording; the pipeline, layout, set and buffer are live, and the push constants
        // are a bounds-checked prefix of `push`.
        unsafe {
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                &[set],
                &[],
            );
            self.device.cmd_push_constants(
                self.command_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &as_bytes(push)[..160],
            );
            self.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
            self.device.cmd_draw(self.command_buffer, count, 1, 0, 0);
        }
    }
}
