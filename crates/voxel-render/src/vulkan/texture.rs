//! Textures: uploading, sampling and destroying.

use super::*;

impl VulkanBackend {
    /// Rewrite rectangles of the loaded atlas: each a level, a corner, a
    /// size and its tightly packed RGBA8 texels. Animated sprites use it
    /// every tick their frame changes.
    pub fn update_atlas(&mut self, regions: &[AtlasRegion]) -> Result<(), VulkanError> {
        let Some(scene) = self.scene.as_ref() else {
            return Ok(());
        };
        if regions.is_empty() {
            return Ok(());
        }
        // SAFETY: live handles; the fence wait leaves the previous frame done sampling the atlas
        // before it is overwritten.
        unsafe {
            // The previous frame may still be sampling the atlas.
            self.wait_in_flight()?;
            let mut staged = Vec::new();
            let mut copies = Vec::with_capacity(regions.len());
            let mut deepest = 0;
            for region in regions {
                copies.push(
                    vk::BufferImageCopy::default()
                        .buffer_offset(staged.len() as u64)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(region.level)
                                .layer_count(1),
                        )
                        .image_offset(vk::Offset3D {
                            x: region.x as i32,
                            y: region.y as i32,
                            z: 0,
                        })
                        .image_extent(vk::Extent3D {
                            width: region.width,
                            height: region.height,
                            depth: 1,
                        }),
                );
                staged.extend_from_slice(&region.rgba);
                deepest = deepest.max(region.level);
            }
            let (staging, staging_memory) =
                self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let image = scene.texture;
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(deepest + 1)
                .layer_count(1);
            let result = self.run_once(|command| {
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
                self.device.cmd_copy_buffer_to_image(
                    command,
                    staging,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &copies,
                );
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
            });
            self.device.destroy_buffer(staging, None);
            self.device.free_memory(staging_memory, None);
            result
        }
    }

    /// Upload mip levels of one image through a staging buffer, and make a view of it.
    ///
    /// # Safety
    ///
    /// No frame may be recording: it submits setup work and waits for the queue.
    #[expect(
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
    pub(super) unsafe fn upload_texture(
        &self,
        levels: &[DecodedImage],
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), VulkanError> {
        // SAFETY: live handles; the staging buffer lives only until `run_once` has finished the
        // copies, and the image's memory is bound before it is viewed.
        unsafe {
            let base = levels.first().ok_or(VulkanError::NoAtlasLevels)?;
            let mut staged = Vec::new();
            let mut offsets = Vec::with_capacity(levels.len());
            for level in levels {
                offsets.push(staged.len() as u64);
                staged.extend_from_slice(&level.rgba);
            }
            let (staging, staging_memory) =
                self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let format = vk::Format::R8G8B8A8_UNORM;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: base.width,
                            height: base.height,
                            depth: 1,
                        })
                        .mip_levels(levels.len() as u32)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
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

            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(levels.len() as u32)
                .layer_count(1);
            // Every level is already built on the processor, so nothing is
            // blitted here: each one is its own copy out of the staging buffer.
            let regions: Vec<vk::BufferImageCopy> = levels
                .iter()
                .zip(&offsets)
                .enumerate()
                .map(|(level, (image, offset))| {
                    vk::BufferImageCopy::default()
                        .buffer_offset(*offset)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(level as u32)
                                .layer_count(1),
                        )
                        .image_extent(vk::Extent3D {
                            width: image.width,
                            height: image.height,
                            depth: 1,
                        })
                })
                .collect();
            self.run_once(|command| {
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
                self.device.cmd_copy_buffer_to_image(
                    command,
                    staging,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                );
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
            })?;
            self.device.destroy_buffer(staging, None);
            self.device.free_memory(staging_memory, None);

            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(range),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok((image, memory, view))
        }
    }

    /// Destroy an image with its view and memory, and the sampler it was read with, if they were made.
    ///
    /// # Safety
    ///
    /// The GPU must be done with the objects.
    pub(super) unsafe fn destroy_texture(
        &self,
        texture: Option<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
        sampler: Option<vk::Sampler>,
    ) {
        // SAFETY: each object is passed by value, so it is destroyed once; the caller guarantees
        // the GPU is done with them.
        unsafe {
            if let Some((image, memory, view)) = texture {
                self.device.destroy_image_view(view, None);
                self.device.destroy_image(image, None);
                self.device.free_memory(memory, None);
            }
            if let Some(sampler) = sampler {
                self.device.destroy_sampler(sampler, None);
            }
        }
    }

    /// Load the rain and snow textures, sampled at the nearest texel and
    /// repeating, as the reference's plain resource textures are.
    pub fn load_weather_textures(
        &mut self,
        rain: &DecodedImage,
        snow: &DecodedImage,
    ) -> Result<(), VulkanError> {
        // SAFETY: live handles; this only creates objects and stores them in the scene.
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptor_layout) else {
                return Ok(());
            };
            let pool = self.scene.as_ref().expect("checked above").descriptor_pool;
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::NEAREST)
                        .min_filter(vk::Filter::NEAREST)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::REPEAT)
                        .address_mode_v(vk::SamplerAddressMode::REPEAT)
                        .address_mode_w(vk::SamplerAddressMode::REPEAT)
                        .max_lod(0.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let mut textures = Vec::new();
            let mut sets = Vec::new();
            for image in [rain, snow] {
                let (texture, memory, view) = self.upload_texture(std::slice::from_ref(image))?;
                let layouts = [layout];
                let set = self
                    .device
                    .allocate_descriptor_sets(
                        &vk::DescriptorSetAllocateInfo::default()
                            .descriptor_pool(pool)
                            .set_layouts(&layouts),
                    )
                    .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
                let image_info = [vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(view)
                    .sampler(sampler)];
                let write = vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info);
                self.device.update_descriptor_sets(&[write], &[]);
                textures.push((texture, memory, view));
                sets.push(set);
            }
            let scene = self.scene.as_mut().expect("checked above");
            scene.weather_textures = textures;
            scene.weather_sampler = Some(sampler);
            scene.weather_sets = sets;
            Ok(())
        }
    }

    /// Load the skin entities are drawn with, sampled texel by texel.
    /// Hand over the ten crack stages, stacked top to bottom in one image.
    pub fn load_crack_texture(&mut self, stages: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.crack_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, sampler, set)) = self.upload_sampled_texture(stages)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.crack_texture = Some(texture);
        scene.crack_sampler = Some(sampler);
        scene.crack_set = Some(set);
        Ok(())
    }

    pub fn load_entity_texture(&mut self, skin: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.entity_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, sampler, set)) = self.upload_sampled_texture(skin)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.entity_texture = Some(texture);
        scene.entity_sampler = Some(sampler);
        scene.entity_set = Some(set);
        Ok(())
    }

    /// Hand over the maps' image (the maps' colours and the decoration sprites), which replaces the
    /// last: the descriptor set is kept and made to point at the new image.
    pub fn set_map_texture(&mut self, image: &DecodedImage) -> Result<(), VulkanError> {
        if self.scene.is_none() {
            return Ok(());
        }
        if self
            .scene
            .as_ref()
            .is_some_and(|scene| scene.map_set.is_none())
        {
            let Some((texture, sampler, set)) = self.upload_sampled_texture(image)? else {
                return Ok(());
            };
            let scene = self.scene.as_mut().expect("checked above");
            scene.map_texture = Some(texture);
            scene.map_sampler = Some(sampler);
            scene.map_set = Some(set);
            return Ok(());
        }
        // SAFETY: live handles; `device_wait_idle` runs before an old texture is replaced.
        unsafe {
            let _ = self.device.device_wait_idle();
            let (texture, memory, view) = self.upload_texture(std::slice::from_ref(image))?;
            let scene = self.scene.as_mut().expect("checked above");
            let (set, sampler) = (
                scene.map_set.expect("checked above"),
                scene.map_sampler.expect("made with the set"),
            );
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);
            if let Some((old_image, old_memory, old_view)) =
                scene.map_texture.replace((texture, memory, view))
            {
                self.device.destroy_image_view(old_view, None);
                self.device.destroy_image(old_image, None);
                self.device.free_memory(old_memory, None);
            }
        }
        Ok(())
    }

    /// Load the GUI sprites' atlas, sampled texel by texel.
    pub fn load_gui_texture(&mut self, atlas: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.gui_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, sampler, set)) = self.upload_sampled_texture(atlas)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.gui_texture = Some(texture);
        scene.gui_sampler = Some(sampler);
        scene.gui_set = Some(set);
        Ok(())
    }

    /// Upload one texture with a nearest, clamped sampler and a descriptor
    /// set for it.
    #[expect(
        clippy::type_complexity,
        reason = "the handles one upload creates, returned together"
    )]
    pub(super) fn upload_sampled_texture(
        &mut self,
        skin: &DecodedImage,
    ) -> Result<
        Option<(
            (vk::Image, vk::DeviceMemory, vk::ImageView),
            vk::Sampler,
            vk::DescriptorSet,
        )>,
        VulkanError,
    > {
        // SAFETY: live handles; this only creates objects, which the caller stores.
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptor_layout) else {
                return Ok(None);
            };
            let pool = self.scene.as_ref().expect("checked above").descriptor_pool;
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::NEAREST)
                        .min_filter(vk::Filter::NEAREST)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .max_lod(0.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let (texture, memory, view) = self.upload_texture(std::slice::from_ref(skin))?;
            let layouts = [layout];
            let set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);
            Ok(Some(((texture, memory, view), sampler, set)))
        }
    }
}
