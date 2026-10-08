use super::*;

impl Gui {
    /// A frame of the title screen, with nothing behind it but the clear colour.
    pub fn draw_title(
        &self,
        backend: &mut VulkanBackend,
        surface: &WindowSurface,
        screen: &voxel_render::gui::screen::title::TitleScreen,
        mouse: (i32, i32),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame = voxel_render::gui::GuiFrame {
            millis: self.started_at.elapsed().as_millis() as u64,
            ..voxel_render::gui::GuiFrame::default()
        };
        screen.draw(&mut frame, &self.atlas, &self.font, mouse);
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
