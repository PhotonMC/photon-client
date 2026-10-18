//! The GUI atlas sampled smoothly, for the textures whose metadata asks for `blur`.

use super::*;

impl VulkanBackend {
    /// A second set over the GUI atlas with a smooth, clamped sampler (`FilterMode.LINEAR`).
    pub(super) fn create_blur_set(
        &self,
        view: vk::ImageView,
    ) -> Result<Option<(vk::DescriptorSet, Sampler)>, VulkanError> {
        let Some(scene) = self.scene.as_ref() else {
            return Ok(None);
        };
        // SAFETY: live handles; this only creates objects, which the caller stores.
        unsafe {
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
            let layouts = [scene.descriptors.layout];
            let set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(scene.descriptors.pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            self.point_set_at(set, view, sampler.handle);
            Ok(Some((set, sampler)))
        }
    }

    /// After the atlas was replaced, make the smooth set sample the new image.
    pub(super) fn repoint_blur_set(&self, view: vk::ImageView) {
        if let Some((set, sampler)) = self
            .scene
            .as_ref()
            .and_then(|scene| scene.gui_blur.as_ref())
        {
            self.point_set_at(*set, view, sampler.handle);
        }
    }

    /// Make `set`'s one binding sample `view` with `sampler`.
    fn point_set_at(&self, set: vk::DescriptorSet, view: vk::ImageView, sampler: vk::Sampler) {
        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(view)
            .sampler(sampler)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);
        // SAFETY: live handles; `replace_image` idled the device, so nothing in flight reads the set.
        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }
}
