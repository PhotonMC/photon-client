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

    /// F3+G: the chunk borders round the player's chunk, when their entry is shown.
    pub(super) fn chunk_borders(
        &self,
        player: &photon_world::local_player::LocalPlayer,
        view: crate::overlays::FrameView,
        camera: [f32; 3],
    ) -> Vec<photon_render::block::outline::LineVertex> {
        let shown = self
            .controls
            .debug
            .entries
            .enabled(self.session.reduced_debug_info());
        if !shown.contains(&"chunk_borders") {
            return Vec::new();
        }
        let at = player.position();
        let block = [at.x, at.y, at.z].map(|value| value.floor() as i32);
        let bounds = self.session.world().dimension();
        photon_render::block::chunk_borders::chunk_border_vertices(
            &photon_render::block::chunk_borders::Borders {
                chunk: [block[0] & !15, block[2] & !15],
                section_min_y: block[1] & !15,
                min_y: bounds.min_y,
                max_y: bounds.min_y + bounds.height - 1,
                origin: view.origin,
                camera,
            },
        )
    }
}
