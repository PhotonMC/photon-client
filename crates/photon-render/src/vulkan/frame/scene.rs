use super::*;

const BOTH_STAGES: vk::ShaderStageFlags = vk::ShaderStageFlags::from_raw(
    vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw(),
);

impl VulkanBackend {
    /// The terrain shaders also take the lightmap's inputs, the camera and the fog; most runs
    /// push the same.
    fn terrain_push(&self, view_projection: [[f32; 4]; 4]) -> [f32; 46] {
        let mut push = [0.0_f32; 46];
        push[..16].copy_from_slice(&std::array::from_fn::<f32, 16, _>(|i| {
            view_projection[i / 4][i % 4]
        }));
        push[16..28].copy_from_slice(&self.lightmap[..12]);
        push[28..31].copy_from_slice(&self.sort_origin);
        push[31] = self.lightmap[12];
        push[32..40].copy_from_slice(&self.fog);
        push[44] = self.lightmap[13];
        push[45] = self.lightmap[14];
        push
    }

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
            if self.gui_only && self.blur_active() {
                self.record_blurred_panorama(scene);
            }
            self.begin_rendering(self.image_views[index], depth.view);
            let terrain_push = self.terrain_push(view_projection);
            for pass in PASSES {
                if self.gui_only && !matches!(pass, Pass::Panorama | Pass::Gui) {
                    continue;
                }
                match pass {
                    Pass::Sky => self.record_sky(scene),
                    Pass::Terrain => self.record_terrain(scene, &terrain_push, now),
                    Pass::Entities => {
                        // The portals are opaque and write the depth, so the entities that stand
                        // before them are drawn over them.
                        self.draw_portal(&terrain_push);
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
                    Pass::Weather => {
                        self.record_weather(scene, &terrain_push);
                        // The reference draws the world border in the same
                        // pass, right after the rain and snow
                        // (`LevelRenderer.addWeatherPass`).
                        self.record_world_border(scene, &terrain_push);
                    }
                    Pass::Hand => self.record_hand(scene, &terrain_push),
                    Pass::Panorama => self.record_panorama(scene),
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
            let Some((pipeline, set)) = self.sky_batch(scene, blend) else {
                continue;
            };
            // SAFETY: recording (see `# Safety`); the pipeline, set and buffer are live.
            unsafe {
                self.draw_vertices(pipeline, (Some(set), &push), buffer, (first, count));
            }
        }
    }

    /// The pipeline and the sampled set of a run of sky: the block atlas, or for the End's cube
    /// the End sky texture the end portal's layers also use (none until it is loaded).
    fn sky_batch<'a>(
        &'a self,
        scene: &'a SceneResources,
        blend: crate::sky::SkyBlend,
    ) -> Option<(&'a Pipeline, vk::DescriptorSet)> {
        let atlas = scene.descriptor_set;
        match blend {
            crate::sky::SkyBlend::Translucent => Some((&scene.sky_pipeline, atlas)),
            crate::sky::SkyBlend::Overlay => Some((&scene.sky_overlay_pipeline, atlas)),
            crate::sky::SkyBlend::EndSky => self
                .portal_scene
                .as_ref()
                .map(|portal| (&scene.sky_pipeline, portal.sets[0])),
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
        terrain_push: &[f32; 46],
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
        terrain_push: &[f32; 46],
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
        terrain_push: &[f32; 46],
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
                &[(BOTH_STAGES, 0, &as_bytes(terrain_push)[..184])],
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
    unsafe fn record_weather(&self, scene: &SceneResources, terrain_push: &[f32; 46]) {
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

    /// The world border overlay, over everything the weather pass already drew, leaving the depth
    /// alone except for its own small bias.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_world_border(&self, scene: &SceneResources, terrain_push: &[f32; 46]) {
        let (Some(buffer), Some(set)) = (self.world_border_buffer.as_ref(), scene.world_border_set)
        else {
            return;
        };
        let push = [(
            vk::ShaderStageFlags::VERTEX,
            0,
            &as_bytes(terrain_push)[..64],
        )];
        // SAFETY: recording (see `# Safety`).
        unsafe {
            self.draw_vertices(
                &scene.world_border_pipeline,
                (Some(set), &push),
                buffer,
                (0, self.world_border_count),
            );
        }
    }

    /// The first-person hand, then what it holds, over a cleared depth, with its own projection
    /// and the camera at the origin of its view space.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_hand(&self, scene: &SceneResources, terrain_push: &[f32; 46]) {
        if self.hand.buffer.as_ref().is_some() && self.hand.count + self.held_count != 0 {
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
        // SAFETY: recording (see `# Safety`).
        unsafe { self.record_screen_effects(scene, terrain_push) };
    }

    /// What covers the view from inside, over the hand: the block the eyes are in, the water, the
    /// fire, each blended over what is drawn. Nothing is lit or fogged.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    unsafe fn record_screen_effects(&self, scene: &SceneResources, terrain_push: &[f32; 46]) {
        let Some(buffer) = self.screen_effect_buffer.as_ref() else {
            return;
        };
        let [blocks, water, fire] = self.screen_effect_counts;
        if blocks + water + fire == 0 {
            return;
        }
        let mut push = [0.0_f32; 46];
        push[..40].copy_from_slice(&terrain_push[..40]);
        push[44] = terrain_push[44];
        push[45] = terrain_push[45];
        push[..16].copy_from_slice(&std::array::from_fn::<f32, 16, _>(|i| {
            self.screen_effect_projection[i / 4][i % 4]
        }));
        push[28..31].copy_from_slice(&[0.0; 3]);
        // No fog: the fog colour's alpha is how much of it there is.
        push[35] = 0.0;
        push[40] = 1.0;
        let push = [(BOTH_STAGES, 0, &as_bytes(&push)[..184])];
        let runs = [
            (Some(scene.descriptor_set), (0, blocks)),
            (scene.underwater_set, (blocks, water)),
            (Some(scene.descriptor_set), (blocks + water, fire)),
        ];
        for (set, range) in runs {
            if set.is_some() {
                // SAFETY: recording (see `# Safety`); the pipeline, set and buffer belong to `scene`
                // and `self`.
                unsafe {
                    self.draw_vertices(&scene.screen_effect_pipeline, (set, &push), buffer, range);
                }
            }
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
                crate::gui::GuiBlend::Translucent
                | crate::gui::GuiBlend::TranslucentBlur
                | crate::gui::GuiBlend::BlockSprite => (
                    &scene.gui_pipeline,
                    translucent_set(blend, set, scene),
                    buffer,
                ),
                crate::gui::GuiBlend::Invert => (&scene.gui_invert_pipeline, set, buffer),
                crate::gui::GuiBlend::Highlight => (&scene.gui_highlight_pipeline, set, buffer),
                crate::gui::GuiBlend::Vignette | crate::gui::GuiBlend::VignetteBlur => (
                    &scene.gui_vignette_pipeline,
                    translucent_set(blend, set, scene),
                    buffer,
                ),
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

/// The descriptor set a translucent GUI batch samples: the GUI atlas, or for a sprite of the block
/// atlas that atlas (the pipeline's layout is the same for both).
fn translucent_set(
    blend: crate::gui::GuiBlend,
    gui: vk::DescriptorSet,
    scene: &SceneResources,
) -> vk::DescriptorSet {
    match blend {
        crate::gui::GuiBlend::BlockSprite => scene.descriptor_set,
        crate::gui::GuiBlend::TranslucentBlur | crate::gui::GuiBlend::VignetteBlur => {
            scene.gui_blur.as_ref().map_or(gui, |(set, _)| *set)
        }
        _ => gui,
    }
}
