//! Loading the atlas and the scene it makes, and destroying them.

use super::*;

impl VulkanBackend {
    /// # Safety
    ///
    /// The device and `layout` must be live.
    unsafe fn create_gui_pipelines(
        &self,
        layout: vk::DescriptorSetLayout,
    ) -> Result<GuiPipelines, VulkanError> {
        // SAFETY: the caller keeps the device and the layout live.
        unsafe {
            Ok(GuiPipelines {
                invert: self.create_pipeline(layout, PipelineKind::GuiInvert)?,
                highlight: self.create_pipeline(layout, PipelineKind::GuiHighlight)?,
                vignette: self.create_pipeline(layout, PipelineKind::GuiVignette)?,
                item: self.create_pipeline(layout, PipelineKind::GuiItem)?,
                glint: self.create_pipeline(layout, PipelineKind::GuiGlint)?,
            })
        }
    }

    /// The pipelines of the lines: the outline's and its backing's.
    ///
    /// # Safety
    ///
    /// The device and `layout` must be live.
    unsafe fn create_line_pipelines(
        &self,
        layout: vk::DescriptorSetLayout,
    ) -> Result<[Pipeline; 2], VulkanError> {
        // SAFETY: the caller keeps the device and the layout live.
        unsafe {
            Ok([
                self.create_pipeline(layout, PipelineKind::Lines)?,
                self.create_pipeline(layout, PipelineKind::LinesBacking)?,
            ])
        }
    }

    /// Build the depth target, texture, buffers and pipeline for one mesh.
    ///
    /// The GLSL is this project's own; `build.rs` compiles it with shaderc, the library the
    /// reference's Vulkan backend uses.
    pub fn load_scene(
        &mut self,
        mesh: &Mesh,
        texture_levels: &[DecodedImage],
    ) -> Result<(), VulkanError> {
        self.load_atlas(texture_levels)?;
        self.set_section((0, 0, 0), mesh, [0.0; 3], SectionFade::none())
    }

    /// Upload the atlas and build the pipelines, dropping every section.
    pub fn load_atlas(&mut self, texture_levels: &[DecodedImage]) -> Result<(), VulkanError> {
        // SAFETY: live handles; `destroy_scene` makes the device idle before it frees the old
        // scene, so everything made below replaces something unused.
        unsafe {
            self.destroy_scene();
            self.depth = Some(self.create_depth_target()?);

            let texture = self.upload_texture(texture_levels)?;
            // These are the reference's own values for the terrain sampler,
            // read from `LevelRenderer`'s chunk layer sampler and from the
            // Vulkan sampler it builds. Both filters are linear, which reads
            // as a surprise for a game of pixel art but is consistent with the
            // atlas reserving padding around every sprite: with nearest
            // sampling that padding would have nothing to do. The blocky look
            // comes from the fragment shader instead, which moves each lookup
            // to its texel's centre before the linear filter sees it.
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::LINEAR)
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        // The reference leaves the third axis at the default,
                        // which is repeat, and never samples it.
                        .address_mode_w(vk::SamplerAddressMode::REPEAT)
                        .mip_lod_bias(0.0)
                        .min_lod(0.0)
                        // An unbounded maximum, which the reference spells as
                        // one thousand.
                        .max_lod(1000.0)
                        .anisotropy_enable(false)
                        .max_anisotropy(1.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let sampler = Sampler::new(&self.device, sampler);

            let descriptors = Descriptors::create(&self.device)?;
            let layouts = [descriptors.layout];
            let descriptor_set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(descriptors.pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.view)
                .sampler(sampler.handle)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);

            let pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Terrain)?;
            let translucent_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::Translucent)?;
            let sky_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::SkyTranslucent)?;
            let sky_overlay_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::SkyOverlay)?;
            let cloud_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Clouds)?;
            let weather_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::Weather)?;
            let entity_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Entity)?;
            let gui_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Gui)?;
            let lines_pipelines = self.create_line_pipelines(descriptors.layout)?;
            let crumbling_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::Crumbling)?;
            let held_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Held)?;
            let text_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Text)?;
            let text_see_through_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::TextSeeThrough)?;
            let text_translucent_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::TextTranslucent)?;
            let shadow_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Shadow)?;
            let swirl_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Swirl)?;
            let glint_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Glint)?;
            let dissolve_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::Dissolve)?;
            let rays_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Rays)?;
            let world_border_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::WorldBorder)?;
            let screen_effect_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::ScreenEffect)?;
            let gui = self.create_gui_pipelines(descriptors.layout)?;

            self.scene = Some(SceneResources {
                texture: Texture {
                    image: texture,
                    sampler,
                },
                descriptors,
                descriptor_set,
                pipeline,
                translucent_pipeline,
                sky_pipeline,
                sky_overlay_pipeline,
                cloud_pipeline,
                weather_pipeline,
                weather_textures: Vec::new(),
                weather_sampler: None,
                weather_sets: Vec::new(),
                entity_pipeline,
                lines_pipelines,
                crumbling_pipeline,
                held_pipeline,
                text_pipeline,
                text_see_through_pipeline,
                text_translucent_pipeline,
                shadow_pipeline,
                swirl_pipeline,
                glint_pipeline,
                dissolve_pipeline,
                rays_pipeline,
                world_border_pipeline,
                world_border_texture: None,
                world_border_set: None,
                screen_effect_pipeline,
                underwater_texture: None,
                underwater_set: None,
                crack_texture: None,
                crack_set: None,
                entity_texture: None,
                entity_set: None,
                gui_pipeline,
                gui_pipelines: gui,
                backdrop: Backdrop::default(),
                gui_texture: None,
                gui_set: None,
                gui_blur: None,
                map_texture: None,
                map_set: None,
            });
            Ok(())
        }
    }

    /// Destroy everything the loaded atlas made: pipelines, textures, sets, sections.
    ///
    /// # Safety
    ///
    /// Only the live handles; it makes the device idle itself.
    pub(super) unsafe fn destroy_scene(&mut self) {
        // SAFETY: `device_wait_idle` runs first, and each object leaves `self` before it is
        // destroyed, so none is used by the GPU or destroyed twice.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.completed.set(self.frames_submitted);
            // Everything the scene made frees itself: the pipelines, textures, samplers and sets. The
            // portal's sets come from the scene's pool, so they go with it.
            self.portal_scene = None;
            self.scene = None;
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
            self.depth = None;
        }
    }
}
