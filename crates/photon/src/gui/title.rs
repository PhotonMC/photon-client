use super::*;

impl Gui {
    /// A frame of a menu, with nothing behind it but the clear colour.
    pub fn draw_menu_screen(
        &self,
        backend: &mut VulkanBackend,
        surface: &WindowSurface,
        draw: impl FnOnce(&mut photon_render::gui::GuiFrame, &GuiAtlas, &Font),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame = photon_render::gui::GuiFrame {
            millis: self.started_at.elapsed().as_millis() as u64,
            ..photon_render::gui::GuiFrame::default()
        };
        draw(&mut frame, &self.atlas, &self.font);
        let size = surface.scale();
        let scale = size.gui_scale() as f32;
        backend.set_gui(
            &frame,
            [
                size.framebuffer_width() as f32 / scale,
                size.framebuffer_height() as f32 / scale,
            ],
        )?;
        Ok(())
    }
}
