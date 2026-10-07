use super::*;

impl VulkanBackend {
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
    pub(in super::super) unsafe fn record_scene(
        &self,
        index: usize,
        view_projection: [[f32; 4]; 4],
    ) {
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
}
