//! How the terrain samples the block atlas (`LevelRenderer.chunkLayerSampler`): linearly, with
//! the anisotropy the `textureFiltering` option asks for, in a set of its own so the atlas's
//! other users (items, particles, the sky) keep the plain sampler. The RGSS choice is a switch of
//! the terrain shader instead, pushed with the frame.

use super::*;

/// What the terrain is filtered with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TerrainFiltering {
    /// `TextureFilteringMethod.RGSS`.
    pub(super) rgss: bool,
    /// The sampler's maximum anisotropy, 1 for none.
    pub(super) anisotropy: f32,
}

impl Default for TerrainFiltering {
    fn default() -> Self {
        Self {
            rgss: false,
            anisotropy: 1.0,
        }
    }
}

/// The terrain's set over the atlas, and the sampler in it.
pub(super) struct TerrainSet {
    pub(super) set: vk::DescriptorSet,
    pub(super) sampler: Sampler,
}

impl VulkanBackend {
    /// `textureFiltering` and `maxAnisotropyValue`: RGSS in the terrain shader, or an anisotropic
    /// terrain sampler of `anisotropy` (at most what the device allows; 1 for none). A changed
    /// anisotropy rebuilds the sampler at once, as `LevelExtractor.resetSampler` asks.
    pub fn set_texture_filtering(
        &mut self,
        rgss: bool,
        anisotropy: u32,
    ) -> Result<(), VulkanError> {
        self.filtering.rgss = rgss;
        // SAFETY: the instance and physical device are live.
        let limit = unsafe {
            self.instance
                .get_physical_device_properties(self.physical_device)
                .limits
                .max_sampler_anisotropy
        };
        let anisotropy = (anisotropy.max(1) as f32).min(limit.max(1.0));
        if anisotropy == self.filtering.anisotropy {
            return Ok(());
        }
        self.filtering.anisotropy = anisotropy;
        let Some(view) = self.scene.as_ref().map(|scene| scene.texture.image.view) else {
            return Ok(());
        };
        // SAFETY: the device is idle before the set in use is rewritten and its old sampler
        // freed; the view is the live atlas's.
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|result| VulkanError::Call("vkDeviceWaitIdle", result))?;
            let sampler = self.atlas_sampler(anisotropy)?;
            let scene = self.scene.as_mut().expect("checked above");
            write_atlas_set(&self.device, scene.terrain.set, view, sampler.handle);
            scene.terrain.sampler = sampler;
        }
        Ok(())
    }

    /// The atlas sampler with `anisotropy`.
    ///
    /// These are the reference's own values for the terrain sampler,
    /// read from `LevelRenderer`'s chunk layer sampler and from the
    /// Vulkan sampler it builds. Both filters are linear, which reads
    /// as a surprise for a game of pixel art but is consistent with the
    /// atlas reserving padding around every sprite: with nearest
    /// sampling that padding would have nothing to do. The blocky look
    /// comes from the fragment shader instead, which moves each lookup
    /// to its texel's centre before the linear filter sees it.
    ///
    /// # Safety
    ///
    /// The device must be live.
    pub(super) unsafe fn atlas_sampler(&self, anisotropy: f32) -> Result<Sampler, VulkanError> {
        // SAFETY: the caller's contract.
        unsafe {
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
                        .anisotropy_enable(anisotropy > 1.0)
                        .max_anisotropy(anisotropy),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            Ok(Sampler::new(&self.device, sampler))
        }
    }

    /// A set from the scene's pool holding the atlas `view` with `sampler`.
    ///
    /// # Safety
    ///
    /// The device, the pool and the view must be live.
    pub(super) unsafe fn atlas_set(
        &self,
        descriptors: &Descriptors,
        view: vk::ImageView,
        sampler: vk::Sampler,
    ) -> Result<vk::DescriptorSet, VulkanError> {
        let layouts = [descriptors.layout];
        // SAFETY: the caller's contract.
        unsafe {
            let set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(descriptors.pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            write_atlas_set(&self.device, set, view, sampler);
            Ok(set)
        }
    }
}

/// Point `set` at the atlas `view` sampled by `sampler`.
///
/// # Safety
///
/// Every handle must be live, and no recorded command still in flight may use `set`.
unsafe fn write_atlas_set(
    device: &ash::Device,
    set: vk::DescriptorSet,
    view: vk::ImageView,
    sampler: vk::Sampler,
) {
    let image_info = [vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(view)
        .sampler(sampler)];
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&image_info);
    // SAFETY: the caller's contract.
    unsafe { device.update_descriptor_sets(&[write], &[]) };
}
