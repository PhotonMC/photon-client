//! What the HUD draws over the view before the crosshair.

use super::*;

impl Hud {
    /// `extractCameraOverlays` (the powder snow frame) and then the crosshair.
    pub(super) fn camera_overlays_and_crosshair(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (width, height): (i32, i32),
        input: &HudInput,
    ) {
        if let Some(alpha) = input.powder_snow {
            frame.texture_overlay(
                atlas,
                "textures/misc/powder_snow_outline",
                (0, 0, width, height),
                alpha,
            );
        }
        self.crosshair(frame, atlas, width, height, input);
    }
}
