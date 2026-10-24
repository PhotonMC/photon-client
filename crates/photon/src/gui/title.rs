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

    /// The frame handed to the GPU, with the unifont glyphs its texts asked for put into the atlas
    /// it samples.
    pub(super) fn present(
        backend: &mut VulkanBackend,
        (frame, size): (
            &photon_render::gui::GuiFrame,
            &photon_render::window::WindowScale,
        ),
        atlas: (&GuiAtlas, &mut photon_render::texture::png::DecodedImage),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scale = size.gui_scale() as f32;
        backend.set_gui(
            frame,
            [
                size.framebuffer_width() as f32 / scale,
                size.framebuffer_height() as f32 / scale,
            ],
        )?;
        Self::sync_atlas(atlas, backend)
    }

    /// The unifont glyphs the frame's texts asked for, put into the atlas the GPU samples.
    fn sync_atlas(
        (atlas, image): (&GuiAtlas, &mut photon_render::texture::png::DecodedImage),
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if atlas.apply_glyphs(image) {
            backend.replace_gui_texture(image)?;
        }
        Ok(())
    }

    /// The same, for a screen drawn on its own.
    pub(crate) fn sync_glyphs(
        &mut self,
        backend: &mut VulkanBackend,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::sync_atlas((&self.atlas, &mut self.atlas_image), backend)
    }
}
