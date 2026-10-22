//! What the frame brings up to date before drawing: the swapchain, vsync, the chunk meshes.

use super::*;

impl Play {
    /// A resized or fullscreen window gets a swapchain of its new size.
    pub(super) fn fit_swapchain(&mut self) -> Result<()> {
        self.backend.set_vsync(self.controls.options.enable_vsync);
        let quality = &self.controls.options.quality;
        // `maxAnisotropyValue` applies only to the anisotropic choice.
        let anisotropy = match quality.texture_filtering {
            photon_game::options::TextureFiltering::Anisotropic => {
                1 << quality.max_anisotropy_bit.clamp(1, 3)
            }
            _ => 1,
        };
        self.backend.set_texture_filtering(
            quality.texture_filtering == photon_game::options::TextureFiltering::Rgss,
            anisotropy,
        )?;
        if std::mem::take(&mut self.controls.reload_chunks) {
            self.sections.rebuild_all(&mut self.backend)?;
        }
        self.sections
            .set_ambient_occlusion(self.controls.options.ambient_occlusion, &mut self.backend)?;
        if resize_to_window(&self.surface, &mut self.backend, &mut self.sized_for)? {
            let (width, height) = self.backend.extent();
            self.aspect = width as f32 / height as f32;
        }
        Ok(())
    }
}
