//! The menu background's blur (`GameRenderer.processBlurEffect`, the `blur` post chain): the cube
//! is drawn into an offscreen image, blurred along x and y three times over a second image, and
//! put on the frame under the GUI.

use super::shaders::{
    BLUR_FRAGMENT_SHADER, COPY_FRAGMENT_SHADER, PANORAMA_FRAGMENT_SHADER, PANORAMA_VERTEX_SHADER,
    SCREENQUAD_VERTEX_SHADER,
};
use super::*;

/// How many times the chain blurs along x and then y (`post_effect/blur.json`).
const ROUNDS: usize = 3;

/// The two images the chain ping-pongs over, at the size of the frame.
struct Targets {
    extent: vk::Extent2D,
    a: Image,
    b: Image,
}

/// The blur's pipelines, sampler and sets, and its images once they exist.
pub(super) struct BlurScene {
    cube: Pipeline,
    blur: Pipeline,
    copy: Pipeline,
    _sampler: Sampler,
    set_a: vk::DescriptorSet,
    set_b: vk::DescriptorSet,
    targets: Option<Targets>,
}

const COLOUR_RANGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: 1,
    base_array_layer: 0,
    layer_count: 1,
};

impl VulkanBackend {
    /// The blur's pipelines, sampler and two sets, made with the panorama.
    ///
    /// # Safety
    ///
    /// Live handles; `layout` and `pool` belong to the scene's descriptors.
    pub(super) unsafe fn create_blur_scene(
        &self,
        layout: vk::DescriptorSetLayout,
        pool: vk::DescriptorPool,
    ) -> Result<BlurScene, VulkanError> {
        // SAFETY: the caller's contract; every create-info points at locals.
        unsafe {
            let cube = self.create_plain_pipeline(
                layout,
                (&PANORAMA_VERTEX_SHADER, &PANORAMA_FRAGMENT_SHADER),
                (vk::ShaderStageFlags::VERTEX, 64),
                false,
            )?;
            let blur = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &BLUR_FRAGMENT_SHADER),
                (vk::ShaderStageFlags::FRAGMENT, 24),
                false,
            )?;
            let copy = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &COPY_FRAGMENT_SHADER),
                (vk::ShaderStageFlags::FRAGMENT, 0),
                true,
            )?;
            // The chain's inputs are filtered linearly (`"bilinear": true`).
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::LINEAR)
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .max_lod(0.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let sampler = Sampler::new(&self.device, sampler);
            let layouts = [layout, layout];
            let sets = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?;
            Ok(BlurScene {
                cube,
                blur,
                copy,
                _sampler: sampler,
                set_a: sets[0],
                set_b: sets[1],
                targets: None,
            })
        }
    }

    /// Blur the cube behind the menu by `radius` texels (`menuBackgroundBlurriness`), or not at
    /// all for 0. Makes the images at the frame's size when there are none or it has changed.
    pub fn set_panorama_blur(&mut self, radius: u32) -> Result<(), VulkanError> {
        self.panorama_blur = radius;
        let extent = self.description.extent;
        let current = self
            .scene
            .as_ref()
            .and_then(|scene| scene.panorama.as_ref())
            .and_then(|panorama| panorama.blur.targets.as_ref())
            .map(|targets| targets.extent);
        if radius == 0
            || current == Some(extent)
            || self.scene.as_ref().is_none_or(|s| s.panorama.is_none())
        {
            return Ok(());
        }
        // SAFETY: the wait leaves the GPU done with the old images and sets before they are
        // replaced; the handles are live.
        unsafe {
            self.wait_in_flight()?;
            let (a, b) = (self.colour_target(extent)?, self.colour_target(extent)?);
            let blur = &mut self
                .scene
                .as_mut()
                .and_then(|scene| scene.panorama.as_mut())
                .expect("checked above")
                .blur;
            for (set, image) in [(blur.set_a, &a), (blur.set_b, &b)] {
                let info = [vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(image.view)
                    .sampler(blur._sampler.handle)];
                self.device.update_descriptor_sets(
                    &[vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(&info)],
                    &[],
                );
            }
            blur.targets = Some(Targets { extent, a, b });
        }
        Ok(())
    }

    /// Whether the cube is blurred this frame.
    pub(super) fn blur_active(&self) -> bool {
        self.panorama_blur > 0
            && self.panorama_view.is_some()
            && self
                .scene
                .as_ref()
                .and_then(|scene| scene.panorama.as_ref())
                .is_some_and(|panorama| panorama.blur.targets.is_some())
    }

    /// An image of the frame's format that can be drawn to and sampled.
    ///
    /// # Safety
    ///
    /// Live handles.
    unsafe fn colour_target(&self, extent: vk::Extent2D) -> Result<Image, VulkanError> {
        // SAFETY: the image's memory is bound before the view is made, and the result owns all
        // three.
        unsafe {
            let format = self.description.surface_format;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: extent.width,
                            height: extent.height,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImage", result))?;
            let requirements = self.device.get_image_memory_requirements(image);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindImageMemory", result))?;
            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(COLOUR_RANGE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok(Image::new(&self.device, (image, memory, view), format))
        }
    }

    /// Draw the cube into the first image and blur it there, before the frame's own rendering
    /// scope begins.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; `scene` is live and `blur_active` holds.
    pub(super) unsafe fn record_blurred_panorama(&self, scene: &SceneResources) {
        let (Some(panorama), Some(matrix)) = (scene.panorama.as_ref(), self.panorama_view.as_ref())
        else {
            return;
        };
        let blur = &panorama.blur;
        let Some(targets) = blur.targets.as_ref() else {
            return;
        };
        let extent = targets.extent;
        let radius = self.panorama_blur as f32;
        // SAFETY: recording outside a scope (see `# Safety`); the images, sets and pipelines are
        // live, and each pass writes a whole image the previous pass has finished with.
        unsafe {
            self.offscreen_pass(&targets.a, extent, |this| {
                this.bind_state(
                    &blur.cube,
                    Some(panorama.set),
                    &[(vk::ShaderStageFlags::VERTEX, 0, &as_bytes(matrix)[..64])],
                );
                this.device.cmd_draw(this.command_buffer, 36, 1, 0, 0);
            });
            let size = [extent.width as f32, extent.height as f32];
            for _ in 0..ROUNDS {
                for (direction, from, to) in [
                    ([1.0, 0.0], blur.set_a, &targets.b),
                    ([0.0, 1.0], blur.set_b, &targets.a),
                ] {
                    let push = [direction[0], direction[1], radius, 0.0, size[0], size[1]];
                    self.offscreen_pass(to, extent, |this| {
                        this.bind_state(
                            &blur.blur,
                            Some(from),
                            &[(vk::ShaderStageFlags::FRAGMENT, 0, &as_bytes(&push)[..24])],
                        );
                        this.device.cmd_draw(this.command_buffer, 3, 1, 0, 0);
                    });
                }
            }
        }
    }

    /// Put the blurred image on the frame.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `blur_active` holds.
    pub(super) unsafe fn draw_blurred_panorama(&self, scene: &SceneResources) {
        let Some(blur) = scene.panorama.as_ref().map(|panorama| &panorama.blur) else {
            return;
        };
        // SAFETY: recording (see `# Safety`); the pipeline and set are live.
        unsafe {
            self.bind_state(&blur.copy, Some(blur.set_a), &[]);
            self.device.cmd_draw(self.command_buffer, 3, 1, 0, 0);
        }
    }

    /// One rendering scope over `target`, which `draw` fills; the image is readable afterwards.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; `target` is live and `draw` records only draws.
    unsafe fn offscreen_pass(
        &self,
        target: &Image,
        extent: vk::Extent2D,
        draw: impl FnOnce(&Self),
    ) {
        // SAFETY: recording outside a scope (see `# Safety`).
        unsafe {
            transition(
                &self.device,
                self.command_buffer,
                target.image,
                COLOUR_RANGE,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            );
            let attachment = [vk::RenderingAttachmentInfo::default()
                .image_view(target.view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::DONT_CARE)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let rendering = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(extent))
                .layer_count(1)
                .color_attachments(&attachment);
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
            draw(self);
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
            transition(
                &self.device,
                self.command_buffer,
                target.image,
                COLOUR_RANGE,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            );
        }
    }
}
