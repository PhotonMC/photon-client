//! Rewriting rectangles of the atlases the GPU samples.

use super::*;

impl VulkanBackend {
    /// Rewrite rectangles of the loaded atlas: each a level, a corner, a
    /// size and its tightly packed RGBA8 texels. Animated sprites use it
    /// every tick their frame changes.
    pub fn update_atlas(&mut self, regions: &[AtlasRegion]) -> Result<(), VulkanError> {
        let Some(scene) = self.scene.as_ref() else {
            return Ok(());
        };
        let image = scene.texture.image.image;
        self.write_regions(image, regions)
    }

    /// The same for the GUI atlas: the unifont glyphs put into its cells.
    pub fn update_gui_atlas(&mut self, regions: &[AtlasRegion]) -> Result<(), VulkanError> {
        let Some(texture) = self
            .scene
            .as_ref()
            .and_then(|scene| scene.gui_texture.as_ref())
        else {
            return Ok(());
        };
        let image = texture.image.image;
        self.write_regions(image, regions)
    }

    fn write_regions(&self, image: vk::Image, regions: &[AtlasRegion]) -> Result<(), VulkanError> {
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
            for region in regions {
                copies.push(copy_of(region, staged.len() as u64));
                staged.extend_from_slice(&region.rgba);
            }
            let deepest = regions.iter().map(|region| region.level).max().unwrap_or(0);
            let staging = self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(deepest + 1)
                .layer_count(1);

            self.run_once(|command| {
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
                    staging.buffer,
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
            })
        }
    }
}

/// The copy of one region from a staging buffer, whose texels start at `offset`.
fn copy_of(region: &AtlasRegion, offset: u64) -> vk::BufferImageCopy {
    vk::BufferImageCopy::default()
        .buffer_offset(offset)
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
        })
}
