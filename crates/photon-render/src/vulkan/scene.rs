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
                logo: self.create_pipeline(layout, PipelineKind::GuiLogo)?,
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

    /// The held items' pipelines: opaque, then blended for the quads of translucent sprites.
    ///
    /// # Safety
    ///
    /// The device and the layout must be live.
    unsafe fn create_held_pipelines(
        &self,
        layout: vk::DescriptorSetLayout,
    ) -> Result<[Pipeline; 2], VulkanError> {
        // SAFETY: the caller keeps the device and the layout live.
        unsafe {
            Ok([
                self.create_pipeline(layout, PipelineKind::Held)?,
                self.create_pipeline(layout, PipelineKind::HeldTranslucent)?,
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
            let sampler = self.atlas_sampler(1.0)?;
            let descriptors = Descriptors::create(&self.device)?;
            let descriptor_set = self.atlas_set(&descriptors, texture.view, sampler.handle)?;
            let terrain = self.terrain_set(&descriptors, texture.view)?;

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
            let held_pipelines = self.create_held_pipelines(descriptors.layout)?;
            let text_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Text)?;
            let text_see_through_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::TextSeeThrough)?;
            let text_translucent_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::TextTranslucent)?;
            let shadow_pipeline = self.create_pipeline(descriptors.layout, PipelineKind::Shadow)?;
            let mask_pipeline =
                self.create_pipeline(descriptors.layout, PipelineKind::EntityMask)?;
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
                terrain,
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
                held_pipelines,
                text_pipeline,
                text_see_through_pipeline,
                text_translucent_pipeline,
                shadow_pipeline,
                mask_pipeline,
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
