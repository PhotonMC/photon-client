//! The video surface's texture: one image the size of the frame, replaced as new frames arrive.

use super::*;

impl VulkanBackend {
    /// Hand over the video's newest frame (RGBA), which replaces the last: the descriptor set is
    /// kept and made to point at the new image. The first call makes the texture. Every update
    /// waits for the device to be idle (`replace_image`), which is fine for an occasional frame but
    /// not for 30 frames a second: streaming needs a persistent image with a staging upload.
    pub fn set_surface_texture(&mut self, image: &DecodedImage) -> Result<(), VulkanError> {
        if self.scene.is_none() {
            return Ok(());
        }
        let existing = self.scene.as_ref().and_then(|scene| {
            Some((
                scene.surface_set?,
                scene.surface_texture.as_ref()?.sampler.handle,
            ))
        });
        let Some((set, sampler)) = existing else {
            let Some((texture, set)) = self.upload_sampled_texture(image)? else {
                return Ok(());
            };
            let scene = self.scene.as_mut().expect("checked above");
            scene.surface_texture = Some(texture);
            scene.surface_set = Some(set);
            return Ok(());
        };
        let image = self.replace_image(image, set, sampler)?;
        if let Some(texture) = self
            .scene
            .as_mut()
            .and_then(|scene| scene.surface_texture.as_mut())
        {
            texture.image = image;
        }
        Ok(())
    }
}
