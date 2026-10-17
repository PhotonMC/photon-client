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
        if let Some((intensity, uv)) = input.portal {
            // `extractPortalOverlay`: the alpha is lifted to the fourth power below one.
            let mut alpha = intensity;
            if alpha < 1.0 {
                alpha *= alpha;
                alpha *= alpha;
                alpha = alpha * 0.8 + 0.2;
            }
            frame.block_sprite_overlay(uv, (0, 0, width, height), alpha);
        }
        self.crosshair(frame, atlas, width, height, input);
    }
}
