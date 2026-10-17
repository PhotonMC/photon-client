//! The end portal and end gateway surfaces' own pipeline and textures.

use super::*;

/// What the portal surfaces are drawn with: their pipeline, the end sky and the portal's picture,
/// and the set each is sampled from, the sky's first.
pub(super) struct PortalScene {
    pub(super) pipeline: Pipeline,
    pub(super) sets: [vk::DescriptorSet; 2],
    _textures: [Texture; 2],
}

impl VulkanBackend {
    /// Load the end sky and the picture of the end portal, which its surfaces layer: each repeats
    /// and is sampled at the nearest texel, in a set of its own.
    pub fn load_end_portal_textures(
        &mut self,
        (sky, portal): (&DecodedImage, &DecodedImage),
    ) -> Result<(), VulkanError> {
        let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptors.layout) else {
            return Ok(());
        };
        if self.portal_scene.is_some() {
            return Ok(());
        }
        let (Some((first, first_set)), Some((second, second_set))) = (
            self.upload_repeating_texture(sky)?,
            self.upload_repeating_texture(portal)?,
        ) else {
            return Ok(());
        };
        // SAFETY: `layout` is the live layout of the scene's sets.
        let pipeline = unsafe { self.create_pipeline(layout, PipelineKind::Portal)? };
        self.portal_scene = Some(PortalScene {
            pipeline,
            sets: [first_set, second_set],
            _textures: [first, second],
        });
        Ok(())
    }
}
