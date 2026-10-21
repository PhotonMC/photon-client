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

    /// Every line of the frame in one buffer: the aimed block's outline, the bobbers' lines and
    /// the debug boxes.
    pub(super) fn world_lines(
        &self,
        (player, view, camera): (
            &photon_world::local_player::LocalPlayer,
            crate::overlays::FrameView,
            [f32; 3],
        ),
        (line_width, hook_lines): (f32, Vec<photon_render::block::outline::LineVertex>),
    ) -> Vec<photon_render::block::outline::LineVertex> {
        let high_contrast = self.controls.options.high_contrast_block_outline;
        let mut lines = crate::overlays::outline(
            &self.session,
            player,
            view,
            camera,
            (line_width, high_contrast),
        );
        lines.extend(hook_lines);
        lines.extend(self.chunk_borders(player, view, camera));
        lines.extend(self.entity_hitboxes(view, camera));
        lines
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

    /// F3+B: the boxes of the entities the frame draws, when their entry is shown: the box in
    /// white, the eye-height slab of a living one in red, and its view vector as a blue arrow.
    pub(super) fn entity_hitboxes(
        &self,
        view: crate::overlays::FrameView,
        camera: [f32; 3],
    ) -> Vec<photon_render::block::outline::LineVertex> {
        use photon_render::block::hitboxes::{arrow_strips, cuboid_strips};
        let shown = self
            .controls
            .debug
            .entries
            .enabled(self.session.reduced_debug_info());
        if !shown.contains(&"entity_hitboxes") {
            return Vec::new();
        }
        let partial = view.partial as f32;
        let place = |x: f64, y: f64, z: f64| -> [f32; 3] {
            let at = [x, y, z];
            std::array::from_fn(|axis| (at[axis] - f64::from(view.origin[axis])) as f32)
        };
        let mut lines = Vec::new();
        for entity in self.session.world().entities().drawn() {
            if entity.is_invisible() {
                continue;
            }
            let at = entity.interpolated_position(partial);
            let latest = entity.position();
            let bb =
                entity
                    .bounding_box()
                    .move_by(at.0 - latest.0, at.1 - latest.1, at.2 - latest.2);
            let corners =
                |y0: f64, y1: f64| (place(bb.min_x, y0, bb.min_z), place(bb.max_x, y1, bb.max_z));
            lines.extend(cuboid_strips(corners(bb.min_y, bb.max_y), [1.0; 4], camera));
            let eye = f64::from(entity.eye_height());
            if entity.is_living_entity() {
                let low = bb.min_y + eye - f64::from(0.01_f32);
                let high = bb.min_y + eye + f64::from(0.01_f32);
                lines.extend(cuboid_strips(
                    corners(low, high),
                    [1.0, 0.0, 0.0, 1.0],
                    camera,
                ));
            }
            let (yaw, pitch, head) = entity.interpolated_rotation(partial);
            let yaw = if entity.is_living_entity() { head } else { yaw };
            let direction = view_vector(pitch, yaw);
            let from: [f32; 3] = place(at.0, at.1 + eye, at.2);
            let to = std::array::from_fn(|axis| from[axis] + direction[axis] * 2.0);
            lines.extend(arrow_strips((from, to), [0.0, 0.0, 1.0, 1.0], camera));
        }
        lines
    }
}

/// `Entity.calculateViewVector`: the way a pitch and yaw in degrees look.
fn view_vector(pitch: f32, yaw: f32) -> [f32; 3] {
    use photon_core::math::{mth_cos_f64, mth_sin_f64};
    let to_radians = std::f32::consts::PI / 180.0;
    let (x_rot, y_rot) = (pitch * to_radians, -yaw * to_radians);
    let (y_sin, y_cos) = (mth_sin_f64(f64::from(y_rot)), mth_cos_f64(f64::from(y_rot)));
    let (x_sin, x_cos) = (mth_sin_f64(f64::from(x_rot)), mth_cos_f64(f64::from(x_rot)));
    [y_sin * x_cos, -x_sin, y_cos * x_cos]
}
