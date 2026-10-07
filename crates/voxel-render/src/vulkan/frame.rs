//! Recording and submitting one frame, and what it measures.

use super::*;

impl VulkanBackend {
    /// Record the draw for one swapchain image.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording, and the section's ranges must be live arena ranges.
    pub(super) unsafe fn bind_section(
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
    pub(super) unsafe fn record_dissolve(&self, scene: &SceneResources, terrain_push: &[f32]) {
        // SAFETY: recording; the pipeline, layout and set belong to `scene` and the vertex buffer
        // to `self`, all live. The push constants are a bounds-checked prefix of `push`.
        unsafe {
            if let (Some(buffer), Some(set)) = (self.dissolve_buffer.as_ref(), scene.entity_set)
                && self.dissolve_count > 0
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
                    .cmd_draw(self.command_buffer, self.dissolve_count, 1, 0, 0);
            }
        }
    }

    /// The maps in item frames: quads of the maps' image, drawn as text is.
    ///
    /// # Safety
    ///
    /// As [`Self::draw_triangles`].
    pub(super) unsafe fn record_maps(&self, scene: &SceneResources, terrain_push: &[f32]) {
        // SAFETY: the callee's contract is the recording one, which this function's own contract
        // passes on.
        unsafe {
            if let (Some(buffer), Some(set)) = (self.map_buffer.as_ref(), scene.map_set) {
                self.draw_triangles(
                    (scene.text_pipeline, scene.text_pipeline_layout),
                    set,
                    (buffer, self.map_count),
                    terrain_push,
                );
            }
            if let (Some(buffer), Some(set)) =
                (self.map_decoration_buffer.as_ref(), scene.entity_set)
            {
                self.draw_triangles(
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    set,
                    (buffer, self.map_decoration_count),
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
    pub(super) unsafe fn record_flat_shapes(&self, scene: &SceneResources, terrain_push: &[f32]) {
        // SAFETY: the callees' contract is the recording one, which this function's own contract
        // passes on.
        unsafe {
            // Shadows under the entities, before them: blended quads of the
            // shadow texture over the blocks' tops.
            if let (Some(buffer), Some(set)) = (self.shadow_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    set,
                    (buffer, self.shadow_count),
                    terrain_push,
                );
            }
            self.record_dissolve(scene, terrain_push);
            // The glowing layers (the eyes of spiders and endermen): the
            // `EYES` pipeline is the shadow's shaders and state, the texel
            // times the colour with no light and no cut-out, then fog.
            if let (Some(buffer), Some(set)) = (self.glow_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    set,
                    (buffer, self.glow_count),
                    terrain_push,
                );
            }
            // The energy swirls (a charged creeper's), the enchantments' glints (over the
            // depth the entities wrote) and a dying dragon's rays (by their alpha): added over
            // what is behind.
            if let Some(set) = scene.entity_set {
                for (buffer, count, pipeline) in [
                    (
                        self.swirl_buffer.as_ref(),
                        self.swirl_count,
                        (scene.swirl_pipeline, scene.swirl_pipeline_layout),
                    ),
                    (
                        self.glint_buffer.as_ref(),
                        self.glint_count,
                        (scene.glint_pipeline, scene.glint_pipeline_layout),
                    ),
                    (
                        self.rays_buffer.as_ref(),
                        self.rays_count,
                        (scene.rays_pipeline, scene.rays_pipeline_layout),
                    ),
                ] {
                    if let Some(buffer) = buffer {
                        self.draw_triangles(pipeline, set, (buffer, count), terrain_push);
                    }
                }
            }
            // Experience orbs: blended quads over the entity textures.
            if let (Some(buffer), Some(set)) = (self.orb_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    set,
                    (buffer, self.orb_count),
                    terrain_push,
                );
            }
            self.record_maps(scene, terrain_push);
            // Name tags after the entities: the text seen through blocks
            // first, then the text tested against the depth.
            if let Some(set) = scene.gui_set {
                for (buffer, count, pipeline) in [
                    (
                        self.name_tag_see_buffer.as_ref(),
                        self.name_tag_see_count,
                        (
                            scene.text_see_through_pipeline,
                            scene.text_see_through_pipeline_layout,
                        ),
                    ),
                    (
                        self.name_tag_buffer.as_ref(),
                        self.name_tag_count,
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
    pub(super) unsafe fn draw_triangles(
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

    /// Record the scene's draws for one swapchain image.
    ///
    /// # Safety
    ///
    /// Must be called between the begin and the end of `command_buffer`, with swapchain image
    /// `index` acquired.
    #[expect(
        clippy::too_many_lines,
        reason = "one command buffer recorded in draw order"
    )]
    pub(super) unsafe fn record_scene(&self, index: usize, view_projection: [[f32; 4]; 4]) {
        // SAFETY: recording, inside the rendering scope this function begins and ends; the image
        // views, pipelines, sets and buffers are owned by `self` or `scene` and live.
        unsafe {
            let now = std::time::Instant::now();
            let (Some(scene), Some(depth)) = (self.scene.as_ref(), self.depth.as_ref()) else {
                return;
            };
            let colour_attachment = [vk::RenderingAttachmentInfo::default()
                .image_view(self.image_views[index])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                // The clear already happened, so the colour target is loaded.
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(depth.view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                });
            let rendering = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(self.description.extent))
                .layer_count(1)
                .color_attachments(&colour_attachment)
                .depth_attachment(&depth_attachment);
            self.stamp(0);
            self.dynamic_rendering
                .cmd_begin_rendering(self.command_buffer, &rendering);
            let extent = self.description.extent;
            self.device.cmd_set_viewport(
                self.command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(extent.width as f32)
                    .height(extent.height as f32)
                    .max_depth(1.0)],
            );
            self.device.cmd_set_scissor(
                self.command_buffer,
                0,
                &[vk::Rect2D::default().extent(extent)],
            );
            // The sky first, over the cleared fog colour and under everything.
            if let (Some(sky), Some(buffer)) = (self.pending_sky.as_ref(), self.sky_buffer.as_ref())
                && sky.vertex_count > 0
            {
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                let matrix: [f32; 16] = std::array::from_fn(|i| sky.view_projection[i / 4][i % 4]);
                let mut fragment = [0.0_f32; 8];
                fragment[..4].copy_from_slice(&sky.fog_colour);
                fragment[4] = sky.sky_end;
                for &(blend, first, count) in &sky.batches {
                    let (layout, pipeline) = match blend {
                        crate::sky::SkyBlend::Translucent => {
                            (scene.sky_pipeline_layout, scene.sky_pipeline)
                        }
                        crate::sky::SkyBlend::Overlay => (
                            scene.sky_overlay_pipeline_layout,
                            scene.sky_overlay_pipeline,
                        ),
                    };
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
                        &[scene.descriptor_set],
                        &[],
                    );
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        &as_bytes(&matrix)[..64],
                    );
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        64,
                        &as_bytes(&fragment)[..32],
                    );
                    self.device
                        .cmd_draw(self.command_buffer, count, 1, first, 0);
                }
            }
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                scene.pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                scene.pipeline_layout,
                0,
                &[scene.descriptor_set],
                &[],
            );
            let matrix: [f32; 16] = std::array::from_fn(|i| view_projection[i / 4][i % 4]);
            self.stamp(1);
            // The terrain shaders also take the lightmap's inputs, the
            // camera and the fog.
            let mut terrain_push = [0.0_f32; 40];
            terrain_push[..16].copy_from_slice(&matrix);
            terrain_push[16..28].copy_from_slice(&self.lightmap);
            terrain_push[28..31].copy_from_slice(&self.sort_origin);
            terrain_push[32..].copy_from_slice(&self.fog);
            self.device.cmd_push_constants(
                self.command_buffer,
                scene.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &as_bytes(&terrain_push)[..160],
            );
            for section in self
                .drawn_sections()
                .into_iter()
                .filter(|section| section.index_count > 0)
            {
                self.bind_section(section, scene.pipeline_layout, now);
                self.device
                    .cmd_draw_indexed(self.command_buffer, section.index_count, 1, 0, 0, 0);
            }
            self.stamp(2);
            // Dropped items and the blocks on entities, opaque, before the
            // entities so that a translucent shell (a sulfur cube's) blends
            // over the block inside it: the items' quads in the block atlas,
            // shaded as the held ones are, fully faded in.
            if let Some(buffer) = self.ground_buffer.as_ref()
                && self.ground_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.held_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.held_pipeline_layout,
                    0,
                    &[scene.descriptor_set],
                    &[],
                );
                let mut ground_push = [0.0_f32; 41];
                ground_push[..40].copy_from_slice(&terrain_push);
                ground_push[40] = 1.0;
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.held_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&ground_push)[..164],
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.ground_count, 1, 0, 0);
            }
            // Entities after the opaque terrain and before the water.
            if let (Some(buffer), Some(set)) = (self.entity_buffer.as_ref(), scene.entity_set)
                && self.entity_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.entity_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.entity_pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.entity_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&terrain_push)[..160],
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.entity_count, 1, 0, 0);
            }
            self.record_flat_shapes(scene, &terrain_push);
            self.stamp(3);
            // The aimed block's outline over the solid world.
            if let Some(buffer) = self.outline_buffer.as_ref()
                && self.outline_count > 0
            {
                let mut line_push = terrain_push;
                line_push[16] = extent.width as f32;
                line_push[17] = extent.height as f32;
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.lines_pipeline,
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.lines_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&line_push)[..160],
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.outline_count, 1, 0, 0);
            }
            // Cracks over the blocks being broken.
            if let (Some(buffer), Some(set)) = (self.breaking_buffer.as_ref(), scene.crack_set)
                && self.breaking_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.crumbling_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.crumbling_pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.crumbling_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&terrain_push)[..160],
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.breaking_count, 1, 0, 0);
            }
            self.stamp(4);
            // Translucent faces last, the farthest section first, so nearer
            // water blends over farther.
            let mut translucent: Vec<&SectionBuffers> = self
                .drawn_sections()
                .into_iter()
                .filter(|section| section.translucent_index_count > 0)
                .collect();
            let distance = |section: &SectionBuffers| {
                (0..3)
                    .map(|axis| (section.centre[axis] - self.sort_origin[axis]).powi(2))
                    .sum::<f32>()
            };
            translucent.sort_by(|a, b| distance(b).total_cmp(&distance(a)));
            if !translucent.is_empty() {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.translucent_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.translucent_pipeline_layout,
                    0,
                    &[scene.descriptor_set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.translucent_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&terrain_push)[..160],
                );
            }
            for section in translucent {
                self.bind_section(section, scene.translucent_pipeline_layout, now);
                self.device.cmd_draw_indexed(
                    self.command_buffer,
                    section.translucent_index_count,
                    1,
                    section.index_count,
                    0,
                    0,
                );
            }
            self.stamp(5);
            // The clouds after the terrain, tested against and writing depth.
            if let Some(clouds) = self.cloud_buffer.as_ref()
                && clouds.visible
                && clouds.vertex_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.cloud_pipeline,
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.cloud_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&clouds.push)[..96],
                );
                self.device.cmd_bind_vertex_buffers(
                    self.command_buffer,
                    0,
                    &[clouds.host.buffer],
                    &[0],
                );
                self.device
                    .cmd_draw(self.command_buffer, clouds.vertex_count, 1, 0, 0);
            }
            // Rain, then snow, each with its texture, over everything and
            // leaving the depth alone.
            if let Some(buffer) = self.weather_buffer.as_ref()
                && scene.weather_sets.len() == 2
                && self.weather_counts.0 + self.weather_counts.1 > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.weather_pipeline,
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.weather_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &as_bytes(&terrain_push)[..160],
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                let (rain, snow) = self.weather_counts;
                for (set, first, count) in [
                    (scene.weather_sets[0], 0, rain),
                    (scene.weather_sets[1], rain, snow),
                ] {
                    if count == 0 {
                        continue;
                    }
                    self.device.cmd_bind_descriptor_sets(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.weather_pipeline_layout,
                        0,
                        &[set],
                        &[],
                    );
                    self.device
                        .cmd_draw(self.command_buffer, count, 1, first, 0);
                }
            }
            self.stamp(6);
            // The first-person hand last, over a cleared depth, with its own
            // projection and the camera at the origin of its view space.
            if let Some(buffer) = self.hand_buffer.as_ref()
                && self.hand_count + self.held_count > 0
            {
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
                let mut hand_push = terrain_push;
                hand_push[..16].copy_from_slice(&std::array::from_fn::<f32, 16, _>(|i| {
                    self.hand_projection[i / 4][i % 4]
                }));
                hand_push[28..31].copy_from_slice(&[0.0; 3]);
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                if let Some(set) = scene.entity_set
                    && self.hand_count > 0
                {
                    self.device.cmd_bind_pipeline(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.entity_pipeline,
                    );
                    self.device.cmd_bind_descriptor_sets(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.entity_pipeline_layout,
                        0,
                        &[set],
                        &[],
                    );
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        scene.entity_pipeline_layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        &as_bytes(&hand_push)[..160],
                    );
                    self.device
                        .cmd_draw(self.command_buffer, self.hand_count, 1, 0, 0);
                }
                // The held items after the arm, from the block atlas the way
                // the terrain samples it, fully faded in.
                if self.held_count > 0 {
                    self.device.cmd_bind_pipeline(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.held_pipeline,
                    );
                    self.device.cmd_bind_descriptor_sets(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.held_pipeline_layout,
                        0,
                        &[scene.descriptor_set],
                        &[],
                    );
                    let mut held_push = [0.0_f32; 41];
                    held_push[..40].copy_from_slice(&hand_push);
                    held_push[40] = 1.0;
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        scene.held_pipeline_layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        &as_bytes(&held_push)[..164],
                    );
                    self.device.cmd_draw(
                        self.command_buffer,
                        self.held_count,
                        1,
                        self.hand_count,
                        0,
                    );
                }
            }
            self.stamp(7);
            // The GUI over everything, run by run in its blends.
            if let (Some(buffer), Some(set)) = (self.gui_buffer.as_ref(), scene.gui_set)
                && !self.gui_batches.is_empty()
            {
                let push = [2.0 / self.gui_size[0], 2.0 / self.gui_size[1], 0.0, 0.0];
                for &(blend, first, count) in &self.gui_batches {
                    let (pipeline, layout, set, vertices) = match blend {
                        crate::gui::GuiBlend::Translucent => {
                            (scene.gui_pipeline, scene.gui_pipeline_layout, set, buffer)
                        }
                        crate::gui::GuiBlend::Invert => (
                            scene.gui_invert_pipeline,
                            scene.gui_invert_pipeline_layout,
                            set,
                            buffer,
                        ),
                        crate::gui::GuiBlend::Items | crate::gui::GuiBlend::Entity => {
                            let Some(items) = self.gui_item_buffer.as_ref() else {
                                continue;
                            };
                            // The player's model samples the skin; the items
                            // sample the block atlas.
                            let samples = if blend == crate::gui::GuiBlend::Entity {
                                let Some(skin) = scene.entity_set else {
                                    continue;
                                };
                                skin
                            } else {
                                scene.descriptor_set
                            };
                            // Each item is drawn over its own cleared depth
                            // in the reference's item atlas; the slots never
                            // overlap, so one clear serves them all.
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
                            (
                                scene.gui_item_pipeline,
                                scene.gui_item_pipeline_layout,
                                samples,
                                items,
                            )
                        }
                    };
                    self.device.cmd_bind_vertex_buffers(
                        self.command_buffer,
                        0,
                        &[vertices.buffer],
                        &[0],
                    );
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
                        &as_bytes(&push)[..16],
                    );
                    self.device
                        .cmd_draw(self.command_buffer, count, 1, first, 0);
                }
            }
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
            self.stamp(8);
        }
    }

    /// Wait until every frame up to number `through` has finished on the GPU. The wait is counted
    /// against the frame being built, since that is the frame that stood still for it.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn complete_through(&self, through: u64) -> Result<(), VulkanError> {
        let through = through.min(self.frames_submitted);
        let start = std::time::Instant::now();
        for number in self.completed.get() + 1..=through {
            let slot = &self.frames[number as usize % FRAMES_IN_FLIGHT];
            // A slot that took a later frame had to finish this one first, so only a slot still
            // holding this frame has a fence to wait on.
            if slot.stamp == number {
                // SAFETY: live handles; the fence is waited on, never reset, here.
                let waited = unsafe {
                    self.device
                        .wait_for_fences(&[slot.in_flight], true, u64::MAX)
                };
                waited.map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            }
            self.completed.set(number);
        }
        self.fence_wait.set(self.fence_wait.get() + start.elapsed());
        Ok(())
    }

    /// Give back the section meshes whose last frame has finished, along with the copies still
    /// waiting to fill them.
    ///
    /// # Safety
    ///
    /// The GPU must be done with every frame up to `completed`.
    pub(super) unsafe fn free_retired(&mut self) {
        let completed = self.completed.get();
        let (done, waiting): (Vec<_>, Vec<_>) = std::mem::take(&mut self.retired)
            .into_iter()
            .partition(|(stamp, _)| *stamp <= completed);
        self.retired = waiting;
        for (_, section) in done {
            // A copy into a mesh nobody will draw has nowhere safe to go: its range is free now.
            self.staged
                .retain(|copy| copy.target != section.vertices && copy.target != section.indices);
            // SAFETY: the section's last frame has finished (see `# Safety`).
            unsafe { self.destroy_section(section) };
        }
    }

    /// Wait until every frame submitted so far has finished.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn wait_in_flight(&self) -> Result<(), VulkanError> {
        // SAFETY: the caller's contract.
        unsafe { self.complete_through(self.frames_submitted) }
    }

    /// Write a timestamp after the work recorded so far, if the device can.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording, with the timestamp queries reset in it.
    pub(super) unsafe fn stamp(&self, query: u32) {
        if let Some(queries) = &self.timestamps {
            let query = (self.frame_slot as u32) * TIMESTAMP_COUNT + query;
            // SAFETY: recording (see `# Safety`), and the pool is live and owned by `self`.
            unsafe {
                self.device.cmd_write_timestamp(
                    self.command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    queries.pool,
                    query,
                );
            }
        }
    }

    /// Read back the timestamps the last frame of this slot wrote, if it wrote any.
    ///
    /// # Safety
    ///
    /// The caller has waited for that frame, so it has finished.
    pub(super) unsafe fn read_timestamps(&mut self) {
        let slot = self.frame_slot;
        let Some(queries) = self
            .timestamps
            .as_mut()
            .filter(|queries| queries.pending[slot])
        else {
            return;
        };
        queries.pending[slot] = false;
        let mut ticks = [0_u64; TIMESTAMP_COUNT as usize];
        // SAFETY: live pool; the frame that wrote it has finished (see `# Safety`). A pass the
        // frame did not reach leaves the query unavailable and the call fails, which keeps the
        // previous reading.
        let read = unsafe {
            self.device.get_query_pool_results(
                queries.pool,
                slot as u32 * TIMESTAMP_COUNT,
                &mut ticks,
                vk::QueryResultFlags::TYPE_64,
            )
        };
        if read.is_ok() {
            queries.last = Some(GpuTimings::from_ticks(
                &ticks,
                queries.valid_bits,
                queries.period_ns,
            ));
        }
    }

    /// The CPU time the last frame spent in the backend, by span.
    pub fn timings(&self) -> BackendTimings {
        self.timings
    }

    /// The GPU's time in each pass of the last frame whose timestamps have
    /// been read back, or nothing when the device cannot write timestamps.
    pub fn gpu_timings(&self) -> Option<GpuTimings> {
        self.timestamps.as_ref().and_then(|queries| queries.last)
    }

    /// What the backend selected.
    pub fn description(&self) -> &BackendDescription {
        &self.description
    }

    /// Acquire, clear to one colour, and present a single frame.
    ///
    /// The colour is a placeholder: no reference pixel has been compared yet.
    pub fn draw_clear_frame(&mut self, colour: [f32; 4]) -> Result<(), VulkanError> {
        self.render_frame(colour, false, IDENTITY).map(|_| ())
    }

    /// Draw the loaded scene under one view-projection matrix.
    pub fn draw_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        self.render_frame(colour, false, view_projection)
            .map(|_| ())
    }

    /// Draw the loaded scene and read the presented image back.
    pub fn capture_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, view_projection)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    /// Render one frame and read the presented image back as RGBA8 rows.
    ///
    /// Readback is the same path frame comparison will need later, so it copies
    /// the image the compositor receives rather than re-rendering into an
    /// offscreen target.
    pub fn capture_frame(&mut self, colour: [f32; 4]) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, IDENTITY)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one frame submitted in the order the API requires"
    )]
    pub(super) fn render_frame(
        &mut self,
        colour: [f32; 4],
        capture: bool,
        view_projection: [[f32; 4]; 4],
    ) -> Result<Option<Vec<u8>>, VulkanError> {
        // SAFETY: live handles. The fence wait comes first, so the previous frame is done before
        // its resources are recycled; the command buffer is begun here and ended before the
        // submit, and the readback is mapped only after the fence has signalled.
        unsafe {
            // The frame about to be recorded takes the slot of the one before the last, which must
            // have finished: the last frame may still be drawing.
            let number = self.frames_submitted + 1;
            self.frame_slot = number as usize % FRAMES_IN_FLIGHT;
            self.command_buffer = self.frames[self.frame_slot].command_buffer;
            self.complete_through(self.frames[self.frame_slot].stamp)?;
            self.read_timestamps();
            self.free_retired();
            let completed = self.completed.get();
            self.staging_in_flight
                .retain(|&(stamp, _)| stamp > completed);
            self.trim_staging();
            if capture {
                self.make_readback()?;
            }

            let acquire_start = std::time::Instant::now();
            let (index, _) = match self.swapchain_device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.frames[self.frame_slot].image_available,
                vk::Fence::null(),
            ) {
                Ok(value) => value,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.out_of_date = true;
                    return Ok(None);
                }
                Err(result) => return Err(VulkanError::Call("vkAcquireNextImageKHR", result)),
            };
            self.timings.acquire = acquire_start.elapsed();
            self.device
                .reset_fences(&[self.frames[self.frame_slot].in_flight])
                .map_err(|result| VulkanError::Call("vkResetFences", result))?;

            let image = self.images[index as usize];
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1);

            self.device
                .begin_command_buffer(
                    self.command_buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;
            let record_start = std::time::Instant::now();
            if let Some(queries) = &self.timestamps {
                self.device.cmd_reset_query_pool(
                    self.command_buffer,
                    queries.pool,
                    self.frame_slot as u32 * TIMESTAMP_COUNT,
                    TIMESTAMP_COUNT,
                );
            }
            self.record_staged_copies();

            transition(
                &self.device,
                self.command_buffer,
                image,
                range,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );
            self.device.cmd_clear_color_image(
                self.command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue { float32: colour },
                &[range],
            );

            if self.scene.is_some() {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                );
                self.record_scene(index as usize, view_projection);
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
            }
            if capture {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                );
                let region = vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width: self.description.extent.width,
                        height: self.description.extent.height,
                        depth: 1,
                    });
                self.device.cmd_copy_image_to_buffer(
                    self.command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    self.readback.expect("made for the capture").0,
                    &[region],
                );
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            } else {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            }

            self.device
                .end_command_buffer(self.command_buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;
            self.timings.record = record_start.elapsed();

            let submit_start = std::time::Instant::now();
            let wait_semaphores = [self.frames[self.frame_slot].image_available];
            let wait_stages = [vk::PipelineStageFlags::TRANSFER];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished[index as usize]];
            let submit = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            self.device
                .queue_submit(
                    self.queue,
                    &[submit],
                    self.frames[self.frame_slot].in_flight,
                )
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;

            let swapchains = [self.swapchain];
            let indices = [index];
            let present = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);
            match self.swapchain_device.queue_present(self.queue, &present) {
                Ok(false) => {}
                Ok(true)
                | Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
                | Err(vk::Result::SUBOPTIMAL_KHR) => self.out_of_date = true,
                Err(result) => return Err(VulkanError::Call("vkQueuePresentKHR", result)),
            }
            self.timings.submit_present = submit_start.elapsed();
            self.timings.fence_wait = self.fence_wait.take();
            self.frames_submitted = number;
            self.frames[self.frame_slot].stamp = number;
            let (slot, wrote_scene) = (self.frame_slot, self.scene.is_some());
            if let Some(queries) = self.timestamps.as_mut() {
                queries.pending[slot] = wrote_scene;
            }

            if !capture {
                return Ok(None);
            }
            self.wait_in_flight()?;
            let byte_count =
                (self.description.extent.width * self.description.extent.height * 4) as usize;
            let memory = self.readback.expect("made for the capture").1;
            let mapped = self
                .device
                .map_memory(memory, 0, byte_count as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            let mut rgba = vec![0_u8; byte_count];
            std::ptr::copy_nonoverlapping(mapped, rgba.as_mut_ptr(), byte_count);
            self.device.unmap_memory(memory);
            self.free_readback();
            if self.description.surface_format == vk::Format::B8G8R8A8_UNORM {
                for pixel in rgba.as_chunks_mut::<4>().0 {
                    pixel.swap(0, 2);
                }
            }
            Ok(Some(rgba))
        }
    }
}

/// Insert the layout transition the clear and the present each need.
///
/// # Safety
///
/// `command_buffer` must be recording outside a render pass, and `image` must belong to `device`.
pub(super) unsafe fn transition(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    range: vk::ImageSubresourceRange,
    from: vk::ImageLayout,
    to: vk::ImageLayout,
) {
    // SAFETY: the caller's contract: `command_buffer` is recording and `image` belongs to
    // `device`.
    unsafe {
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(from)
            .new_layout(to)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(range)
            // Scaffolding barrier: make every prior write available and every
            // later read see it. Once the real pipeline exists these become the
            // specific stage/access pairs each pass needs.
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE);
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}
