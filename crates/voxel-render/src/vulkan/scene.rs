//! Loading the atlas and the scene it makes, and destroying them.

use super::*;

impl VulkanBackend {
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
    #[expect(
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
    pub fn load_atlas(&mut self, texture_levels: &[DecodedImage]) -> Result<(), VulkanError> {
        // SAFETY: live handles; `destroy_scene` makes the device idle before it frees the old
        // scene, so everything made below replaces something unused.
        unsafe {
            self.destroy_scene();
            self.depth = Some(self.create_depth_target()?);

            let (texture, texture_memory, texture_view) = self.upload_texture(texture_levels)?;
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

            let bindings = [vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
            let descriptor_layout = self
                .device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorSetLayout", result))?;
            // The atlas's set, the rain's and the snow's, the skin's, the
            // GUI's, the cracks' and the maps'.
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(7)];
            let descriptor_pool = self
                .device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .max_sets(7)
                        .pool_sizes(&sizes),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorPool", result))?;
            let layouts = [descriptor_layout];
            let descriptor_set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);

            let (pipeline_layout, pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Terrain)?;
            let (translucent_pipeline_layout, translucent_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Translucent)?;
            let (sky_pipeline_layout, sky_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::SkyTranslucent)?;
            let (sky_overlay_pipeline_layout, sky_overlay_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::SkyOverlay)?;
            let (cloud_pipeline_layout, cloud_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Clouds)?;
            let (weather_pipeline_layout, weather_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Weather)?;
            let (entity_pipeline_layout, entity_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Entity)?;
            let (gui_pipeline_layout, gui_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Gui)?;
            let (lines_pipeline_layout, lines_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Lines)?;
            let (crumbling_pipeline_layout, crumbling_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Crumbling)?;
            let (held_pipeline_layout, held_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Held)?;
            let (text_pipeline_layout, text_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Text)?;
            let (text_see_through_pipeline_layout, text_see_through_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::TextSeeThrough)?;
            let (text_translucent_pipeline_layout, text_translucent_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::TextTranslucent)?;
            let (shadow_pipeline_layout, shadow_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Shadow)?;
            let (swirl_pipeline_layout, swirl_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Swirl)?;
            let (glint_pipeline_layout, glint_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Glint)?;
            let (dissolve_pipeline_layout, dissolve_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Dissolve)?;
            let (rays_pipeline_layout, rays_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Rays)?;
            let (gui_invert_pipeline_layout, gui_invert_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::GuiInvert)?;
            let (gui_item_pipeline_layout, gui_item_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::GuiItem)?;

            self.scene = Some(SceneResources {
                texture,
                texture_memory,
                texture_view,
                sampler,
                descriptor_pool,
                descriptor_layout,
                descriptor_set,
                pipeline_layout,
                pipeline,
                translucent_pipeline_layout,
                translucent_pipeline,
                sky_pipeline_layout,
                sky_pipeline,
                sky_overlay_pipeline_layout,
                sky_overlay_pipeline,
                cloud_pipeline_layout,
                cloud_pipeline,
                weather_pipeline_layout,
                weather_pipeline,
                weather_textures: Vec::new(),
                weather_sampler: None,
                weather_sets: Vec::new(),
                entity_pipeline_layout,
                entity_pipeline,
                lines_pipeline_layout,
                lines_pipeline,
                crumbling_pipeline_layout,
                crumbling_pipeline,
                held_pipeline_layout,
                held_pipeline,
                text_pipeline_layout,
                text_pipeline,
                text_see_through_pipeline_layout,
                text_see_through_pipeline,
                text_translucent_pipeline_layout,
                text_translucent_pipeline,
                shadow_pipeline_layout,
                shadow_pipeline,
                swirl_pipeline_layout,
                swirl_pipeline,
                glint_pipeline_layout,
                glint_pipeline,
                dissolve_pipeline_layout,
                dissolve_pipeline,
                rays_pipeline_layout,
                rays_pipeline,
                crack_texture: None,
                crack_sampler: None,
                crack_set: None,
                entity_texture: None,
                entity_sampler: None,
                entity_set: None,
                gui_pipeline_layout,
                gui_pipeline,
                gui_invert_pipeline_layout,
                gui_invert_pipeline,
                gui_item_pipeline_layout,
                gui_item_pipeline,
                gui_texture: None,
                gui_sampler: None,
                gui_set: None,
                map_texture: None,
                map_sampler: None,
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
            if let Some(scene) = self.scene.take() {
                self.device.destroy_pipeline(scene.pipeline, None);
                self.device
                    .destroy_pipeline(scene.translucent_pipeline, None);
                for (image, memory, view) in scene.weather_textures {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.weather_sampler {
                    self.device.destroy_sampler(sampler, None);
                }
                self.destroy_texture(scene.crack_texture, scene.crack_sampler);
                self.destroy_texture(scene.entity_texture, scene.entity_sampler);
                for (pipeline, layout) in [
                    (scene.sky_pipeline, scene.sky_pipeline_layout),
                    (
                        scene.sky_overlay_pipeline,
                        scene.sky_overlay_pipeline_layout,
                    ),
                    (scene.cloud_pipeline, scene.cloud_pipeline_layout),
                    (scene.weather_pipeline, scene.weather_pipeline_layout),
                    (scene.entity_pipeline, scene.entity_pipeline_layout),
                    (scene.lines_pipeline, scene.lines_pipeline_layout),
                    (scene.held_pipeline, scene.held_pipeline_layout),
                    (scene.crumbling_pipeline, scene.crumbling_pipeline_layout),
                    (scene.text_pipeline, scene.text_pipeline_layout),
                    (
                        scene.text_see_through_pipeline,
                        scene.text_see_through_pipeline_layout,
                    ),
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    (scene.swirl_pipeline, scene.swirl_pipeline_layout),
                    (scene.glint_pipeline, scene.glint_pipeline_layout),
                    (scene.dissolve_pipeline, scene.dissolve_pipeline_layout),
                    (scene.rays_pipeline, scene.rays_pipeline_layout),
                    (scene.gui_pipeline, scene.gui_pipeline_layout),
                    (scene.gui_invert_pipeline, scene.gui_invert_pipeline_layout),
                    (scene.gui_item_pipeline, scene.gui_item_pipeline_layout),
                ] {
                    self.device.destroy_pipeline(pipeline, None);
                    self.device.destroy_pipeline_layout(layout, None);
                }
                self.destroy_texture(scene.gui_texture, scene.gui_sampler);
                self.destroy_texture(scene.map_texture, scene.map_sampler);
                self.device
                    .destroy_pipeline_layout(scene.translucent_pipeline_layout, None);
                self.device
                    .destroy_pipeline_layout(scene.pipeline_layout, None);
                self.device
                    .destroy_descriptor_pool(scene.descriptor_pool, None);
                self.device
                    .destroy_descriptor_set_layout(scene.descriptor_layout, None);
                self.device.destroy_sampler(scene.sampler, None);
                self.device.destroy_image_view(scene.texture_view, None);
                self.device.destroy_image(scene.texture, None);
                self.device.free_memory(scene.texture_memory, None);
            }
            for (_, section) in std::mem::take(&mut self.sections) {
                self.destroy_section(section);
            }
            for section in std::mem::take(&mut self.moving_sections) {
                self.destroy_section(section);
            }
            for section in std::mem::take(&mut self.retired) {
                self.destroy_section(section);
            }
            // Their copies have nowhere to go now.
            self.staged.clear();
            self.staging_filling.clear();
            if let Some(depth) = self.depth.take() {
                self.device.destroy_image_view(depth.view, None);
                self.device.destroy_image(depth.image, None);
                self.device.free_memory(depth.memory, None);
            }
        }
    }
}
