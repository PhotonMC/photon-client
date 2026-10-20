//! What the frame brings up to date before drawing: the swapchain, vsync, the chunk meshes.

use super::*;

impl Play {
    /// A resized or fullscreen window gets a swapchain of its new size.
    pub(super) fn fit_swapchain(&mut self) -> Result<()> {
        self.backend.set_vsync(self.controls.options.enable_vsync);
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
