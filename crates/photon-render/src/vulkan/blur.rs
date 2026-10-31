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

pub(super) const COLOUR_RANGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
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
                (false, false),
            )?;
            let blur = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &BLUR_FRAGMENT_SHADER),
                (vk::ShaderStageFlags::FRAGMENT, 24),
                (false, false),
            )?;
            let copy = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &COPY_FRAGMENT_SHADER),
                (vk::ShaderStageFlags::FRAGMENT, 0),
                (true, false),
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
        if radius == 0 {
            return Ok(());
        }
        self.ensure_blur_targets()
    }

    /// Make the blur's two images at the frame's size, when there are none or it has changed.
    pub(super) fn ensure_blur_targets(&mut self) -> Result<(), VulkanError> {
        let extent = self.description.extent;
        let current = self
            .scene
            .as_ref()
            .and_then(|scene| scene.backdrop.blur.as_ref())
            .and_then(|blur| blur.targets.as_ref())
            .map(|targets| targets.extent);
        if current == Some(extent) || self.scene.is_none() {
            return Ok(());
        }
        self.ensure_blur_scene()?;
        // SAFETY: the wait leaves the GPU done with the old images and sets before they are
        // replaced; the handles are live.
        unsafe {
            self.wait_in_flight()?;
            let (a, b) = (self.colour_target(extent)?, self.colour_target(extent)?);
            let blur = self
                .scene
                .as_mut()
                .and_then(|scene| scene.backdrop.blur.as_mut())
                .expect("made above");
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

    /// Make the blur's pipelines, sampler and sets, once.
    pub(super) fn ensure_blur_scene(&mut self) -> Result<(), VulkanError> {
        let Some(scene) = self
            .scene
            .as_ref()
            .filter(|scene| scene.backdrop.blur.is_none())
        else {
            return Ok(());
        };
        let (layout, pool) = (scene.descriptors.layout, scene.descriptors.pool);
        // SAFETY: live handles; `layout` and `pool` belong to the scene's descriptors.
        let blur = unsafe { self.create_blur_scene(layout, pool)? };
        if let Some(scene) = self.scene.as_mut() {
            scene.backdrop.blur = Some(blur);
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
                .and_then(|scene| scene.backdrop.blur.as_ref())
                .is_some_and(|blur| blur.targets.is_some())
    }

    /// Whether the blur's images exist at the frame's size, so the frame can be blurred.
    pub(super) fn blur_images_ready(&self, scene: &SceneResources) -> bool {
        scene
            .backdrop
            .blur
            .as_ref()
            .and_then(|blur| blur.targets.as_ref())
            .is_some_and(|targets| targets.extent == self.description.extent)
    }

    /// An image of the frame's format that can be drawn to and sampled.
    ///
    /// # Safety
    ///
    /// Live handles.
    pub(super) unsafe fn colour_target(&self, extent: vk::Extent2D) -> Result<Image, VulkanError> {
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
        let (Some(panorama), Some(matrix)) = (
            scene.backdrop.panorama.as_ref(),
            self.panorama_view.as_ref(),
        ) else {
            return;
        };
        let Some(blur) = scene.backdrop.blur.as_ref() else {
            return;
        };
        let Some(targets) = blur.targets.as_ref() else {
            return;
        };
        // SAFETY: recording outside a scope (see `# Safety`); the images, sets and pipelines are
        // live.
        unsafe {
            self.offscreen_pass(&targets.a, targets.extent, |this| {
                this.bind_state(
                    &blur.cube,
                    Some(panorama.set),
                    &[(vk::ShaderStageFlags::VERTEX, 0, &as_bytes(matrix)[..64])],
                );
                this.device.cmd_draw(this.command_buffer, 36, 1, 0, 0);
            });
            self.blur_rounds(blur, targets, self.panorama_blur as f32);
        }
    }

    /// Blur the first image by `radius` texels, along x and y over the second, three times.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; the images, sets and pipelines are live, and the first
    /// image holds what is to be blurred, readable by its set. Each pass writes a whole image the
    /// previous pass has finished with.
    unsafe fn blur_rounds(&self, blur: &BlurScene, targets: &Targets, radius: f32) {
        let extent = targets.extent;
        let size = [extent.width as f32, extent.height as f32];
        // SAFETY: the caller's contract.
        unsafe {
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

    /// `GameRenderer.processBlurEffect` over the frame so far: end the scene's rendering, copy the
    /// swapchain image into the first blur image, blur it, and begin the rendering again with the
    /// blurred image drawn over the whole frame and the depth cleared.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope over swapchain image `index`, which is in the
    /// colour attachment layout; `scene` is live and the blur images exist at the frame's size.
    pub(super) unsafe fn record_world_blur(
        &self,
        scene: &SceneResources,
        index: usize,
        radius: u32,
    ) {
        let Some(blur) = scene.backdrop.blur.as_ref() else {
            return;
        };
        let (Some(targets), Some(depth)) = (blur.targets.as_ref(), self.targets.depth.as_ref())
        else {
            return;
        };
        // SAFETY: the caller's contract; the copy is between two images of the same format and
        // size, both outside a scope.
        unsafe {
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
            self.copy_frame(self.frame_target(index).0, &targets.a, targets.extent);
            self.blur_rounds(blur, targets, radius as f32);
            self.begin_rendering(self.frame_target(index).1, depth.view);
            self.draw_blurred_panorama(scene);
        }
    }

    /// Copy the swapchain image (in the colour attachment layout, and left in it) into `target`,
    /// which is left readable by its set.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; both images are live, of the frame's format, at
    /// `extent`.
    unsafe fn copy_frame(&self, frame: vk::Image, target: &Image, extent: vk::Extent2D) {
        let layouts = |from, to, image: vk::Image| {
            // SAFETY: the caller's contract.
            unsafe {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    COLOUR_RANGE,
                    from,
                    to,
                );
            }
        };
        let layer = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1);
        let region = vk::ImageCopy::default()
            .src_subresource(layer)
            .dst_subresource(layer)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            });
        use vk::ImageLayout as Layout;
        layouts(
            Layout::COLOR_ATTACHMENT_OPTIMAL,
            Layout::TRANSFER_SRC_OPTIMAL,
            frame,
        );
        layouts(
            Layout::UNDEFINED,
            Layout::TRANSFER_DST_OPTIMAL,
            target.image,
        );
        // SAFETY: the caller's contract, and the layouts above.
        unsafe {
            self.device.cmd_copy_image(
                self.command_buffer,
                frame,
                Layout::TRANSFER_SRC_OPTIMAL,
                target.image,
                Layout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }
        layouts(
            Layout::TRANSFER_SRC_OPTIMAL,
            Layout::COLOR_ATTACHMENT_OPTIMAL,
            frame,
        );
        layouts(
            Layout::TRANSFER_DST_OPTIMAL,
            Layout::SHADER_READ_ONLY_OPTIMAL,
            target.image,
        );
    }

    /// Put the blurred image on the frame.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `blur_active` holds.
    pub(super) unsafe fn draw_blurred_panorama(&self, scene: &SceneResources) {
        let Some(blur) = scene.backdrop.blur.as_ref() else {
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
    pub(super) unsafe fn offscreen_pass(
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
