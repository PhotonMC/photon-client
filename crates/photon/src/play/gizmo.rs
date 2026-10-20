//! The F3 3D crosshair's hand-over to the renderer.

use super::*;

impl Play {
    /// `GameRenderer.renderLevel`: the 3D crosshair, when its entry is shown, in the first person
    /// and with the HUD up.
    pub(super) fn set_debug_gizmo(
        &mut self,
        frame: &FrameCamera,
        first_person: bool,
    ) -> Result<()> {
        let reduced = self.session.reduced_debug_info();
        let shown = first_person
            && !self.controls.hud_hidden
            && self
                .controls
                .debug
                .entries
                .enabled(reduced)
                .contains(&"3d_crosshair");
        let scale = self.surface.scale().gui_scale() as f32;
        let vertices = if shown {
            photon_render::block::gizmo::gizmo_vertices(frame.rotation, scale)
        } else {
            Vec::new()
        };
        self.backend.set_gizmo(&vertices)?;
        Ok(())
    }
}
