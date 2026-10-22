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
    pub(in super::super) unsafe fn bind_state(
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
    pub(super) unsafe fn draw_run(&self, scene: &SceneResources, run: Run, base: &[f32; 46]) {
        let (Some(buffer), first, count) = self.run_range(run) else {
            return;
        };
        let sampled = run.sampled();
        let set = sampled.set(scene);
        if sampled != Sampled::Unbound && set.is_none() {
            return;
        }
        let mut push = [0.0_f32; 47];
        push[..40].copy_from_slice(&base[..40]);
        push[44] = base[44];
        push[45] = base[45];
        push[46] = self.glint_strength;
        let bytes = match run {
            Run::Ground | Run::Held | Run::BlockParticle => {
                push[40] = 1.0;
                184
            }
            Run::Dissolve => {
                push[40..42].copy_from_slice(&self.dissolve_mask_offset);
                184
            }
            Run::Outline | Run::Gizmo => {
                let extent = self.description.extent;
                push[16] = extent.width as f32;
                push[17] = extent.height as f32;
                184
            }
            // The glint shader reads its strength after the boss's two values.
            Run::Glint | Run::HandGlint => 188,
            _ => 184,
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

    /// Draw the end portal and gateway surfaces: the end sky in the first set and the portal's
    /// picture in the second, with how far through the day the game is after the frame's push
    /// constants.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside the scene's rendering scope.
    pub(super) unsafe fn draw_portal(&self, base: &[f32; 46]) {
        let Some(buffer) = self.portal.buffer.as_ref() else {
            return;
        };
        let (first, count) = (0, self.portal.count);
        let Some(portal) = self.portal_scene.as_ref() else {
            return;
        };
        if count == 0 {
            return;
        }
        let mut push = [0.0_f32; 46];
        push[..40].copy_from_slice(&base[..40]);
        push[40] = self.portal_time;
        push[44] = base[44];
        push[45] = base[45];
        let (pipeline, sets) = (&portal.pipeline, portal.sets);
        // SAFETY: recording (see `# Safety`); the pipeline, both sets and the buffer are live, and
        // the push constants fill a prefix of the range the pipeline's layout has.
        unsafe {
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.handle,
            );
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &sets,
                &[],
            );
            self.device.cmd_push_constants(
                self.command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &as_bytes(&push)[..184],
            );
            self.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
            self.device
                .cmd_draw(self.command_buffer, count, 1, first, 0);
        }
    }

    /// The items the hand holds follow the arm in its buffer, and their glints follow the items.
    fn hand_range(&self, run: Run) -> (Option<&HostBuffer>, u32, u32) {
        let [items, glints] = self.held_counts;
        let arm = self.hand.count;
        let buffer = self.hand.buffer.as_ref();
        match run {
            Run::HandGlint => (buffer, arm + items, glints),
            _ => (buffer, arm, items),
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
            Run::Particle => list(&self.particle),
            Run::BlockParticle => list(&self.block_particle),
            Run::Map => list(&self.map),
            Run::MapDecoration => list(&self.map_decoration),
            Run::NameTagSeeThrough => list(&self.name_tag_see),
            Run::NameTag => list(&self.name_tag),
            Run::Outline | Run::Gizmo => list(self.lines.of(run)),
            Run::Cracks => list(&self.breaking),
            Run::Hand => list(&self.hand),
            // The items follow the arm in the hand's buffer.
            Run::Held | Run::HandGlint => self.hand_range(run),
        }
    }

    /// Draw the glowing entities' silhouettes with `pipeline`, over the entity textures, as
    /// the entities are drawn.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording inside a rendering scope over the outline chain's
    /// image, and `scene` must be the live scene.
    pub(in super::super) unsafe fn draw_silhouettes(
        &self,
        (scene, pipeline, list): (&SceneResources, &Pipeline, &DrawList),
        base: &[f32; 46],
    ) {
        let (Some(buffer), Some(set)) = (list.buffer.as_ref(), scene.entity_set) else {
            return;
        };
        let mut push = [0.0_f32; 47];
        push[..40].copy_from_slice(&base[..40]);
        push[44] = base[44];
        push[45] = base[45];
        // SAFETY: recording (see `# Safety`); the pipeline, set and buffer are live, and the push
        // constants are a prefix of `push`.
        unsafe {
            self.draw_vertices(
                pipeline,
                (
                    Some(set),
                    &[(
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        &as_bytes(&push)[..184],
                    )],
                ),
                buffer,
                (0, list.count),
            );
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

impl LineDraws {
    /// The list a line run draws.
    fn of(&self, run: Run) -> &DrawList {
        match run {
            Run::Gizmo => &self.gizmo,
            _ => &self.outline,
        }
    }
}
