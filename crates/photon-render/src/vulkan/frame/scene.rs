use super::*;

const BOTH_STAGES: vk::ShaderStageFlags = vk::ShaderStageFlags::from_raw(
    vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw(),
);

impl VulkanBackend {
    /// Record the scene's draws for one swapchain image.
    ///
    /// # Safety
    ///
    /// Must be called between the begin and the end of `command_buffer`, with swapchain image
    /// `index` acquired.
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
            self.stamp(0);
            self.begin_rendering(self.image_views[index], depth.view);
            // The terrain shaders also take the lightmap's inputs, the camera and the fog; most
            // runs push the same.
            let mut terrain_push = [0.0_f32; 40];
            let matrix: [f32; 16] = std::array::from_fn(|i| view_projection[i / 4][i % 4]);
            terrain_push[..16].copy_from_slice(&matrix);
            terrain_push[16..28].copy_from_slice(&self.lightmap);
            terrain_push[28..31].copy_from_slice(&self.sort_origin);
            terrain_push[32..].copy_from_slice(&self.fog);
            for pass in PASSES {
                if self.gui_only && !matches!(pass, Pass::Gui) {
                    continue;
                }
                match pass {
                    Pass::Sky => self.record_sky(scene),
                    Pass::Terrain => self.record_terrain(scene, &terrain_push, now),
                    Pass::Entities => {
                        for run in ENTITY_RUNS {
                            self.draw_run(scene, run, &terrain_push);
                        }
                    }
                    Pass::Outline => self.draw_run(scene, Run::Outline, &terrain_push),
                    Pass::Cracks => self.draw_run(scene, Run::Cracks, &terrain_push),
                    Pass::Translucent => self.record_translucent(scene, &terrain_push, now),
                    Pass::Particles => {
                        for run in PARTICLE_RUNS {
                            self.draw_run(scene, run, &terrain_push);
                        }
                    }
                    Pass::Clouds => self.record_clouds(scene),
                    Pass::Weather => self.record_weather(scene, &terrain_push),
                    Pass::Hand => self.record_hand(scene, &terrain_push),
                    Pass::Gui => self.record_gui(scene),
                }
                if let Some(query) = pass.stamp_after() {
                    self.stamp(query);
                }
            }
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
            self.stamp(8);
        }
    }

    /// Begin the scene's rendering over the already cleared colour target, clearing the depth, and
    /// set the viewport and the scissor.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; both views are live.
    unsafe fn begin_rendering(&self, colour: vk::ImageView, depth: vk::ImageView) {
        // SAFETY: recording, outside a scope, with live views.
        unsafe {
            let colour_attachment = [vk::RenderingAttachmentInfo::default()
                .image_view(colour)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                // The clear already happened, so the colour target is loaded.
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(depth)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                });
            let extent = self.description.extent;
            let rendering = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(extent))
                .layer_count(1)
                .color_attachments(&colour_attachment)
                .depth_attachment(&depth_attachment);
            self.dynamic_rendering
                .cmd_begin_rendering(self.command_buffer, &rendering);
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
        }
    }

    /// The sky, batch by batch in its blends.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_sky(&self, scene: &SceneResources) {
        let (Some(sky), Some(buffer)) = (self.pending_sky.as_ref(), self.sky_buffer.as_ref())
        else {
            return;
        };
        if sky.vertex_count == 0 {
            return;
        }
        let matrix: [f32; 16] = std::array::from_fn(|i| sky.view_projection[i / 4][i % 4]);
        let mut fragment = [0.0_f32; 8];
        fragment[..4].copy_from_slice(&sky.fog_colour);
        fragment[4] = sky.sky_end;
        let push = [
            (vk::ShaderStageFlags::VERTEX, 0, &as_bytes(&matrix)[..64]),
            (
                vk::ShaderStageFlags::FRAGMENT,
                64,
                &as_bytes(&fragment)[..32],
            ),
        ];
        for &(blend, first, count) in &sky.batches {
            let pipeline = match blend {
                crate::sky::SkyBlend::Translucent => &scene.sky_pipeline,
                crate::sky::SkyBlend::Overlay => &scene.sky_overlay_pipeline,
            };
            // SAFETY: recording (see `# Safety`); the pipeline, set and buffer are live.
            unsafe {
                self.draw_vertices(
                    pipeline,
                    (Some(scene.descriptor_set), &push),
                    buffer,
                    (first, count),
                );
            }
        }
    }

    /// The opaque sections of the terrain.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_terrain(
        &self,
        scene: &SceneResources,
        terrain_push: &[f32; 40],
        now: std::time::Instant,
    ) {
        let sections: Vec<(&SectionBuffers, u32, u32)> = self
            .drawn_sections()
            .into_iter()
            .filter(|section| section.index_count > 0)
            .map(|section| (section, 0, section.index_count))
            .collect();
        // SAFETY: recording (see `# Safety`).
        unsafe { self.draw_sections(&scene.pipeline, scene, terrain_push, &sections, now) };
    }

    /// The translucent sections, the farthest first, so nearer water blends over farther.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_translucent(
        &self,
        scene: &SceneResources,
        terrain_push: &[f32; 40],
        now: std::time::Instant,
    ) {
        let distance = |section: &SectionBuffers| {
            (0..3)
                .map(|axis| (section.centre[axis] - self.sort_origin[axis]).powi(2))
                .sum::<f32>()
        };
        let mut sections: Vec<(&SectionBuffers, u32, u32)> = self
            .drawn_sections()
            .into_iter()
            .filter(|section| section.translucent_index_count > 0)
            .map(|section| {
                (
                    section,
                    section.index_count,
                    section.translucent_index_count,
                )
            })
            .collect();
        sections.sort_by(|a, b| distance(b.0).total_cmp(&distance(a.0)));
        // SAFETY: recording (see `# Safety`).
        unsafe {
            self.draw_sections(
                &scene.translucent_pipeline,
                scene,
                terrain_push,
                &sections,
                now,
            );
        }
    }

    /// Draw indexed ranges of sections (a section, its first index and its index count) with
    /// `pipeline`, over the atlas, with the frame's push constants. Nothing is bound for none.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live, and the sections' ranges are
    /// live arena ranges.
    unsafe fn draw_sections(
        &self,
        pipeline: &Pipeline,
        scene: &SceneResources,
        terrain_push: &[f32; 40],
        sections: &[(&SectionBuffers, u32, u32)],
        now: std::time::Instant,
    ) {
        if sections.is_empty() {
            return;
        }
        // SAFETY: recording (see `# Safety`).
        unsafe {
            self.bind_state(
                pipeline,
                Some(scene.descriptor_set),
                &[(BOTH_STAGES, 0, &as_bytes(terrain_push)[..160])],
            );
            for &(section, first, count) in sections {
                self.bind_section(section, pipeline.layout, now);
                self.device
                    .cmd_draw_indexed(self.command_buffer, count, 1, first, 0, 0);
            }
        }
    }

    /// The clouds, tested against and writing depth.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_clouds(&self, scene: &SceneResources) {
        let Some(clouds) = self.cloud_buffer.as_ref().filter(|clouds| clouds.visible) else {
            return;
        };
        // SAFETY: recording (see `# Safety`).
        unsafe {
            self.draw_vertices(
                &scene.cloud_pipeline,
                (None, &[(BOTH_STAGES, 0, &as_bytes(&clouds.push)[..96])]),
                &clouds.host,
                (0, clouds.vertex_count),
            );
        }
    }

    /// Rain, then snow, each with its texture, over everything and leaving the depth alone.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_weather(&self, scene: &SceneResources, terrain_push: &[f32; 40]) {
        let Some(buffer) = self.weather_buffer.as_ref() else {
            return;
        };
        if scene.weather_sets.len() != 2 {
            return;
        }
        let (rain, snow) = self.weather_counts;
        let push = [(BOTH_STAGES, 0, &as_bytes(terrain_push)[..160])];
        for (set, first, count) in [
            (scene.weather_sets[0], 0, rain),
            (scene.weather_sets[1], rain, snow),
        ] {
            // SAFETY: recording (see `# Safety`).
            unsafe {
                self.draw_vertices(
                    &scene.weather_pipeline,
                    (Some(set), &push),
                    buffer,
                    (first, count),
                );
            }
        }
    }

    /// The first-person hand, then what it holds, over a cleared depth, with its own projection
    /// and the camera at the origin of its view space.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_hand(&self, scene: &SceneResources, terrain_push: &[f32; 40]) {
        if self.hand.buffer.as_ref().is_none() || self.hand.count + self.held_count == 0 {
            return;
        }
        let mut hand_push = *terrain_push;
        hand_push[..16].copy_from_slice(&std::array::from_fn::<f32, 16, _>(|i| {
            self.hand_projection[i / 4][i % 4]
        }));
        hand_push[28..31].copy_from_slice(&[0.0; 3]);
        // SAFETY: recording (see `# Safety`).
        unsafe {
            self.clear_depth();
            self.draw_run(scene, Run::Hand, &hand_push);
            self.draw_run(scene, Run::Held, &hand_push);
        }
    }

    /// The GUI, run by run in its blends.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_gui(&self, scene: &SceneResources) {
        let (Some(buffer), Some(set)) = (self.gui_buffer.as_ref(), scene.gui_set) else {
            return;
        };
        let push = [2.0 / self.gui_size[0], 2.0 / self.gui_size[1], 0.0, 0.0];
        let push = [(BOTH_STAGES, 0, &as_bytes(&push)[..16])];
        for &(blend, first, count) in &self.gui_batches {
            let (pipeline, set, vertices) = match blend {
                crate::gui::GuiBlend::Translucent => (&scene.gui_pipeline, set, buffer),
                crate::gui::GuiBlend::Invert => (&scene.gui_invert_pipeline, set, buffer),
                crate::gui::GuiBlend::Items | crate::gui::GuiBlend::Entity => {
                    let Some(items) = self.gui_item_buffer.as_ref() else {
                        continue;
                    };
                    // The player's model samples the skin; the items sample the block atlas.
                    let samples = if blend == crate::gui::GuiBlend::Entity {
                        let Some(skin) = scene.entity_set else {
                            continue;
                        };
                        skin
                    } else {
                        scene.descriptor_set
                    };
                    // Each item is drawn over its own cleared depth in the reference's item
                    // atlas; the slots never overlap, so one clear serves them all.
                    // SAFETY: recording (see `# Safety`).
                    unsafe { self.clear_depth() };
                    (&scene.gui_item_pipeline, samples, items)
                }
            };
            // SAFETY: recording (see `# Safety`); the pipeline, set and buffer are live.
            unsafe {
                self.draw_vertices(pipeline, (Some(set), &push), vertices, (first, count));
            }
        }
    }
}
