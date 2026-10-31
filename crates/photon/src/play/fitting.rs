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

/// A resized or fullscreen window gets a swapchain of its new size; whether it did.
pub(super) fn resize_to_window(
    surface: &WindowSurface,
    backend: &mut VulkanBackend,
    sized_for: &mut (u32, u32),
) -> Result<bool> {
    let size = surface.scale();
    let wanted = (
        size.framebuffer_width() as u32,
        size.framebuffer_height() as u32,
    );
    // A size drawn at is shown at the window's own.
    let shown = surface.drawn_framebuffer().map(|_| {
        let (width, height) = surface.native_framebuffer();
        (width.max(1) as u32, height.max(1) as u32)
    });
    let stale = backend.out_of_date()
        || wanted != *sized_for
        || shown.is_some_and(|shown| backend.present_extent() != shown);
    if stale && wanted.0 > 0 && wanted.1 > 0 {
        *sized_for = wanted;
        backend.resize_scaled(wanted, shown.unwrap_or(wanted))?;
        return Ok(true);
    }
    Ok(false)
}

/// The menu's swapchain fitted to its window.
pub(crate) fn fit_swapchain(client: &mut Client) -> Result<()> {
    resize_to_window(&client.surface, &mut client.backend, &mut client.size).map(drop)
}
