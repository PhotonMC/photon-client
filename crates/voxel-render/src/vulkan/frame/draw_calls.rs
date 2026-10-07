use super::*;

/// A range of push constants: its stages, its offset and its bytes.
pub(super) type PushRange<'a> = (vk::ShaderStageFlags, u32, &'a [u8]);

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
                &[self.arena[section.vertices.block]
                    .as_ref()
                    .expect("a section's block is live")
                    .buffer
                    .buffer],
                &[section.vertices.offset],
            );
            self.device.cmd_bind_index_buffer(
                self.command_buffer,
                self.arena[section.indices.block]
                    .as_ref()
                    .expect("a section's block is live")
                    .buffer
                    .buffer,
                section.indices.offset,
                vk::IndexType::UINT32,
            );
        }
    }

    /// Bind `pipeline`, the set it samples (if it has one) and its push constants.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope.
    pub(super) unsafe fn bind_state(
        &self,
        pipeline: &Pipeline,
        set: Option<vk::DescriptorSet>,
        push: &[PushRange],
    ) {
        // SAFETY: recording; the pipeline and its layout are live and the set belongs to the
        // scene.
        unsafe {
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.handle,
            );
            if let Some(set) = set {
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.layout,
                    0,
                    &[set],
                    &[],
                );
            }
            for &(stages, offset, bytes) in push {
                self.device.cmd_push_constants(
                    self.command_buffer,
                    pipeline.layout,
                    stages,
                    offset,
                    bytes,
                );
            }
        }
    }

    /// Draw `count` vertices of `buffer` from `first` with `pipeline`, over `set` and with `push`.
    /// Nothing is drawn for none.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope.
    pub(super) unsafe fn draw_vertices(
        &self,
        pipeline: &Pipeline,
        (set, push): (Option<vk::DescriptorSet>, &[PushRange]),
        buffer: &HostBuffer,
        (first, count): (u32, u32),
    ) {
        if count == 0 {
            return;
        }
        // SAFETY: recording; `bind_state` has the contract of its arguments, and the buffer is
        // live.
        unsafe {
            self.bind_state(pipeline, set, push);
            self.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
            self.device
                .cmd_draw(self.command_buffer, count, 1, first, 0);
        }
    }

    /// Draw one run of vertices the frame was handed, with the frame's push constants `base` (the
    /// run adds its own).
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope, and `scene` must be
    /// the live scene.
    pub(super) unsafe fn draw_run(&self, scene: &SceneResources, run: Run, base: &[f32; 40]) {
        let (Some(buffer), first, count) = self.run_range(run) else {
            return;
        };
        let sampled = run.sampled();
        let set = sampled.set(scene);
        if sampled != Sampled::Unbound && set.is_none() {
            return;
        }
        let mut push = [0.0_f32; 44];
        push[..40].copy_from_slice(base);
        let bytes = match run {
            Run::Ground | Run::Held => {
                push[40] = 1.0;
                164
            }
            Run::Dissolve => {
                push[40..42].copy_from_slice(&self.dissolve_mask_offset);
                176
            }
            Run::Outline => {
                let extent = self.description.extent;
                push[16] = extent.width as f32;
                push[17] = extent.height as f32;
                160
            }
            _ => 160,
        };
        // SAFETY: recording (see `# Safety`); the pipeline, set and buffer belong to `scene` and
        // `self`, and the push constants are a prefix of `push`.
        unsafe {
            self.draw_vertices(
                run.pipeline(scene),
                (
                    set,
                    &[(
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        &as_bytes(&push)[..bytes],
                    )],
                ),
                buffer,
                (first, count),
            );
        }
    }

    /// The buffer a run's vertices are in, and which of them it draws.
    fn run_range(&self, run: Run) -> (Option<&HostBuffer>, u32, u32) {
        fn list(list: &DrawList) -> (Option<&HostBuffer>, u32, u32) {
            (list.buffer.as_ref(), 0, list.count)
        }
        match run {
            Run::Ground => list(&self.ground),
            Run::Entity => list(&self.entity),
            Run::Shadow => list(&self.shadow),
            Run::Dissolve => list(&self.dissolve),
            Run::Glow => list(&self.glow),
            Run::Swirl => list(&self.swirl),
            Run::Glint => list(&self.glint),
            Run::Rays => list(&self.rays),
            Run::Orb => list(&self.orb),
            Run::Map => list(&self.map),
            Run::MapDecoration => list(&self.map_decoration),
            Run::NameTagSeeThrough => list(&self.name_tag_see),
            Run::NameTag => list(&self.name_tag),
            Run::Outline => list(&self.outline),
            Run::Cracks => list(&self.breaking),
            Run::Hand => list(&self.hand),
            // The items follow the arm in the hand's buffer.
            Run::Held => (self.hand.buffer.as_ref(), self.hand.count, self.held_count),
        }
    }

    /// Clear the depth of the whole target.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope.
    pub(super) unsafe fn clear_depth(&self) {
        // SAFETY: recording inside the scope, on an attachment that has a depth aspect.
        unsafe {
            self.device.cmd_clear_attachments(
                self.command_buffer,
                &[vk::ClearAttachment {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    color_attachment: 0,
                    clear_value: vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 0.0,
                            stencil: 0,
                        },
                    },
                }],
                &[vk::ClearRect {
                    rect: vk::Rect2D::default().extent(self.description.extent),
                    base_array_layer: 0,
                    layer_count: 1,
                }],
            );
        }
    }
}
