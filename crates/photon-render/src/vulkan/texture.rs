//! Textures: uploading, sampling and destroying.

use super::*;

impl VulkanBackend {
    /// Upload mip levels of one image through a staging buffer, and make a view of it.
    ///
    /// # Safety
    ///
    /// No frame may be recording: it submits setup work and waits for the queue.
    pub(super) unsafe fn upload_texture(
        &self,
        levels: &[DecodedImage],
    ) -> Result<Image, VulkanError> {
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
            let staging = self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
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
            let texture = Image::new(&self.device, (image, memory, view), format);
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
            let copied = self.run_once(|command| {
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
                    staging.buffer,
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
            });
            copied?;
            Ok(texture)
        }
    }

    /// Load the rain and snow textures, sampled at the nearest texel and
    /// repeating, as the reference's plain resource textures are.
    pub fn load_weather_textures(
        &mut self,
        rain: &DecodedImage,
        snow: &DecodedImage,
    ) -> Result<(), VulkanError> {
        if self.replace_weather_textures(rain, snow)? {
            return Ok(());
        }
        // SAFETY: live handles; this only creates objects and stores them in the scene.
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptors.layout) else {
                return Ok(());
            };
            let pool = self.scene.as_ref().expect("checked above").descriptors.pool;
            let sampler = self.repeating_sampler()?;
            let mut textures = Vec::new();
            let mut sets = Vec::new();
            for image in [rain, snow] {
                let texture = self.upload_texture(std::slice::from_ref(image))?;
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
                    .image_view(texture.view)
                    .sampler(sampler.handle)];
                let write = vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info);
                self.device.update_descriptor_sets(&[write], &[]);
                textures.push(texture);
                sets.push(set);
            }
            let scene = self.scene.as_mut().expect("checked above");
            scene.weather_textures = textures;
            scene.weather_sampler = Some(sampler);
            scene.weather_sets = sets;
            Ok(())
        }
    }

    /// A sampler at the nearest texel that repeats, as the reference's plain resource textures use.
    ///
    /// # Safety
    ///
    /// The device must be live.
    unsafe fn repeating_sampler(&self) -> Result<Sampler, VulkanError> {
        // SAFETY: the caller's contract; the create-info points at nothing that outlives the call.
        let handle = unsafe {
            self.device.create_sampler(
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
        }
        .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
        Ok(Sampler::new(&self.device, handle))
    }

    /// The rain and snow again (a join, or the packs reloaded): the two sets are kept and made to
    /// point at the new images, since the pool never gives a set back. Whether they were there.
    fn replace_weather_textures(
        &mut self,
        rain: &DecodedImage,
        snow: &DecodedImage,
    ) -> Result<bool, VulkanError> {
        let Some(scene) = self.scene.as_ref() else {
            return Ok(false);
        };
        let Some(sampler) = scene.weather_sampler.as_ref().map(|sampler| sampler.handle) else {
            return Ok(false);
        };
        if scene.weather_sets.len() != 2 || scene.weather_textures.len() != 2 {
            return Ok(false);
        }
        let sets = scene.weather_sets.clone();
        for (index, image) in [rain, snow].into_iter().enumerate() {
            let replaced = self.replace_image(image, sets[index], sampler)?;
            if let Some(scene) = self.scene.as_mut() {
                scene.weather_textures[index] = replaced;
            }
        }
        Ok(true)
    }

    /// Load the forcefield texture the world border overlay samples,
    /// sampled at the nearest texel and repeating, as the reference's
    /// plain resource textures are.
    pub fn load_world_border_texture(
        &mut self,
        forcefield: &DecodedImage,
    ) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.world_border_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, set)) = self.upload_repeating_texture(forcefield)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.world_border_texture = Some(texture);
        scene.world_border_set = Some(set);
        Ok(())
    }

    /// Load the texture the water lays over the view under water, sampled like the forcefield.
    pub fn load_underwater_texture(&mut self, image: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.underwater_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, set)) = self.upload_repeating_texture(image)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.underwater_texture = Some(texture);
        scene.underwater_set = Some(set);
        Ok(())
    }

    /// A plain resource texture in a set of its own, sampled at the nearest texel and repeating.
    pub(super) fn upload_repeating_texture(
        &mut self,
        image: &DecodedImage,
    ) -> Result<Option<(Texture, vk::DescriptorSet)>, VulkanError> {
        // SAFETY: live handles; this only creates objects and hands them back.
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptors.layout) else {
                return Ok(None);
            };
            let pool = self.scene.as_ref().expect("checked above").descriptors.pool;
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
            let sampler = Sampler::new(&self.device, sampler);
            let image = self.upload_texture(std::slice::from_ref(image))?;
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
                .image_view(image.view)
                .sampler(sampler.handle)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);
            Ok(Some((Texture { image, sampler }, set)))
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
        let Some((texture, set)) = self.upload_sampled_texture(stages)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.crack_texture = Some(texture);
        scene.crack_set = Some(set);
        Ok(())
    }

    /// Hand over the entity textures' image. One already in use is replaced: the descriptor set
    /// is kept and made to point at the new image (a skin arrived, or the packs reloaded).
    pub fn load_entity_texture(&mut self, skin: &DecodedImage) -> Result<(), VulkanError> {
        let Some(scene) = self.scene.as_ref() else {
            return Ok(());
        };
        if let Some(set) = scene.entity_set {
            let Some(sampler) = scene
                .entity_texture
                .as_ref()
                .map(|texture| texture.sampler.handle)
            else {
                return Ok(());
            };
            let image = self.replace_image(skin, set, sampler)?;
            if let Some(texture) = self
                .scene
                .as_mut()
                .and_then(|scene| scene.entity_texture.as_mut())
            {
                texture.image = image;
            }
            return Ok(());
        }
        let Some((texture, set)) = self.upload_sampled_texture(skin)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.entity_texture = Some(texture);
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
            let Some((texture, set)) = self.upload_sampled_texture(image)? else {
                return Ok(());
            };
            let scene = self.scene.as_mut().expect("checked above");
            scene.map_texture = Some(texture);
            scene.map_set = Some(set);
            return Ok(());
        }
        let (set, sampler) = {
            let scene = self.scene.as_ref().expect("checked above");
            let texture = scene.map_texture.as_ref().expect("made with the set");
            (
                scene.map_set.expect("checked above"),
                texture.sampler.handle,
            )
        };
        let image = self.replace_image(image, set, sampler)?;
        if let Some(texture) = self
            .scene
            .as_mut()
            .and_then(|scene| scene.map_texture.as_mut())
        {
            texture.image = image;
        }
        Ok(())
    }

    /// Upload `image` and make the descriptor `set` point at it with `sampler`; the image it
    /// pointed at is left to the caller to drop. The device is made idle first, so nothing in
    /// flight still reads the old one.
    fn replace_image(
        &mut self,
        image: &DecodedImage,
        set: vk::DescriptorSet,
        sampler: vk::Sampler,
    ) -> Result<Image, VulkanError> {
        // SAFETY: live handles; `device_wait_idle` runs before an old texture is replaced.
        unsafe {
            let _ = self.device.device_wait_idle();
            let image = self.upload_texture(std::slice::from_ref(image))?;
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image.view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);
            Ok(image)
        }
    }

    /// Hand over the GUI atlas again after icons were put into it: the descriptor set is kept and
    /// made to point at the new image.
    pub fn replace_gui_texture(&mut self, atlas: &DecodedImage) -> Result<(), VulkanError> {
        let Some((set, sampler)) = self
            .scene
            .as_ref()
            .and_then(|scene| Some((scene.gui_set?, scene.gui_texture.as_ref()?.sampler.handle)))
        else {
            return Ok(());
        };
        let image = self.replace_image(atlas, set, sampler)?;
        self.repoint_blur_set(image.view);
        if let Some(texture) = self
            .scene
            .as_mut()
            .and_then(|scene| scene.gui_texture.as_mut())
        {
            texture.image = image;
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
        let Some((texture, set)) = self.upload_sampled_texture(atlas)? else {
            return Ok(());
        };
        let blur = self.create_blur_set(texture.image.view)?;
        let scene = self.scene.as_mut().expect("checked above");
        scene.gui_texture = Some(texture);
        scene.gui_set = Some(set);
        scene.gui_blur = blur;
        Ok(())
    }

    /// Upload one texture with a nearest, clamped sampler and a descriptor
    /// set for it.
    pub(super) fn upload_sampled_texture(
        &mut self,
        skin: &DecodedImage,
    ) -> Result<Option<(Texture, vk::DescriptorSet)>, VulkanError> {
        // SAFETY: live handles; this only creates objects, which the caller stores.
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptors.layout) else {
                return Ok(None);
            };
            let pool = self.scene.as_ref().expect("checked above").descriptors.pool;
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
            let sampler = Sampler::new(&self.device, sampler);
            let image = self.upload_texture(std::slice::from_ref(skin))?;
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
                .image_view(image.view)
                .sampler(sampler.handle)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);
            Ok(Some((Texture { image, sampler }, set)))
        }
    }
}
