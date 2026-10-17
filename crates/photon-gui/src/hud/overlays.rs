//! What the HUD draws over the view before the crosshair (`Hud.extractCameraOverlays`).

use super::*;

/// What the camera overlays are made of this frame.
#[derive(Debug, Clone, Default)]
pub struct CameraOverlays {
    /// The vignette's border warning, from nothing to one, while the vignette option is on.
    pub vignette: Option<f32>,
    /// The powder snow frame's alpha while the player has ticks frozen (`getPercentFrozen`).
    pub powder_snow: Option<f32>,
    /// The portal effect's intensity and the portal sprite's place in the block atlas, while it
    /// is above zero.
    pub portal: Option<(f32, [f32; 4])>,
}

/// `ARGB.as8BitChannel`.
fn channel(value: f32) -> u32 {
    (value * 255.0).floor() as u32
}

/// `Hud.extractVignette`'s tint: the brightness of the light about the camera, turned red as the
/// border warns.
pub fn vignette_colour(vignette_brightness: f32, border_warning: f32) -> u32 {
    let brightness = vignette_brightness.clamp(0.0, 1.0);
    let (red, green_blue) = if border_warning > 0.0 {
        let strength = border_warning.clamp(0.0, 1.0);
        (
            brightness * (1.0 - strength),
            brightness + (1.0 - brightness) * strength,
        )
    } else {
        (brightness, brightness)
    };
    0xFF00_0000 | channel(red) << 16 | channel(green_blue) << 8 | channel(green_blue)
}

impl Hud {
    /// The vignette's brightness as a tick left it.
    pub fn set_vignette_brightness(&mut self, brightness: f32) {
        self.vignette_brightness = brightness;
    }

    /// `Hud.updateVignetteBrightness`, once a tick: the brightness of the light at the camera's
    /// eyes, inverted, is eased towards a hundredth of the way a tick.
    pub fn update_vignette(&mut self, level_brightness: f32) {
        let target = (1.0 - level_brightness).clamp(0.0, 1.0);
        self.vignette_brightness += (target - self.vignette_brightness) * 0.01;
    }

    /// `extractCameraOverlays` (the vignette, the powder snow frame and the portal) and then the
    /// crosshair.
    pub(super) fn camera_overlays_and_crosshair(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (width, height): (i32, i32),
        input: &HudInput,
    ) {
        let overlays = &input.overlays;
        if let Some(warning) = overlays.vignette {
            frame.vignette_tinted(
                atlas,
                "textures/misc/vignette",
                (0, 0, width, height),
                vignette_colour(self.vignette_brightness, warning),
            );
        }
        if let Some(alpha) = overlays.powder_snow {
            frame.texture_overlay(
                atlas,
                "textures/misc/powder_snow_outline",
                (0, 0, width, height),
                alpha,
            );
        }
        if let Some((intensity, uv)) = overlays.portal {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The vignette starts at one and eases a hundredth of the way a tick to the dark of the
    /// light about the camera: in full light (brightness one) it fades towards nothing.
    #[test]
    fn the_vignette_eases_to_the_dark_of_the_light() {
        let mut hud = Hud::default();
        hud.update_vignette(1.0);
        assert_eq!(hud.vignette_brightness, 0.99);
        hud.update_vignette(0.0);
        assert!(hud.vignette_brightness > 0.99);
        // Brighter than one, or darker than none, still aims within the range.
        hud.update_vignette(3.0);
        assert!(hud.vignette_brightness < 1.0);
    }
}
