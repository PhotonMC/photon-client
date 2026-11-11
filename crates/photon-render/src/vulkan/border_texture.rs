//! The forcefield texture the world border is drawn with, in the two ways a version samples it.

use super::*;
use crate::border::forcefield_mip_levels;
use crate::texture::mipmap::{MipmapStrategy, compute_transparency, generate_mip_levels_with};

impl SceneResources {
    /// The set the world border samples: the picture with its mip levels where the version has
    /// them, and the plain one until that is made.
    pub(super) fn border_set(&self) -> Option<vk::DescriptorSet> {
        let mipped = photon_core::versions::rules().render.forcefield_has_mipmaps;
        self.world_border_mipped
            .as_ref()
            .filter(|_| mipped)
            .or(self.world_border.as_ref())
            .map(|(_, set)| *set)
    }
}

impl VulkanBackend {
    /// Load the forcefield texture the world border overlay samples. 26.2 samples the one level
    /// at the nearest texel; 26.3 gives it mip levels and a linear minification filter. Both are
    /// made, so a version switch picks one per frame without loading again.
    pub fn load_world_border_texture(
        &mut self,
        forcefield: &DecodedImage,
    ) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.world_border.is_some())
        {
            return Ok(());
        }
        let Some((texture, set)) = self.upload_repeating_texture(forcefield)? else {
            return Ok(());
        };
        let levels = forcefield_mip_levels(forcefield.width, forcefield.height);
        let mips = generate_mip_levels_with(
            "misc/forcefield",
            forcefield,
            levels,
            MipmapStrategy::Auto,
            0.0,
            compute_transparency(forcefield),
        );
        let mipped = self.upload_mipped_texture(&mips)?;
        let scene = self.scene.as_mut().expect("checked above");
        scene.world_border = Some((texture, set));
        scene.world_border_mipped = mipped;
        Ok(())
    }

    /// An image with its mip levels in a set of its own, repeating, magnified at the nearest
    /// texel and minified linearly between levels (`MipmappedTexture`).
    fn upload_mipped_texture(
        &mut self,
        levels: &[DecodedImage],
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
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                        .address_mode_u(vk::SamplerAddressMode::REPEAT)
                        .address_mode_v(vk::SamplerAddressMode::REPEAT)
                        .address_mode_w(vk::SamplerAddressMode::REPEAT)
                        .max_lod(vk::LOD_CLAMP_NONE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let sampler = Sampler::new(&self.device, sampler);
            let image = self.upload_texture(levels)?;
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
