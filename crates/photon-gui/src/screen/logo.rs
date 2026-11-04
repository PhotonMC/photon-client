//! The game logo of the credits roll (`LogoRenderer`): the wordmark with the edition under it.

use photon_render::gui::{GuiAtlas, GuiFrame};

/// `LogoRenderer.extractRenderState`: the logo `top` units down, the edition overlapping its foot
/// by seven.
pub fn draw_logo(frame: &mut GuiFrame, atlas: &GuiAtlas, width: i32, top: i32) {
    frame.blit_texture(
        atlas,
        "title/minecraft",
        (256, 64),
        (0, 0),
        (width / 2 - 128, top, 256, 44),
    );
    frame.blit_texture(
        atlas,
        "title/edition",
        (128, 16),
        (0, 0),
        (width / 2 - 64, top + 44 - 7, 128, 14),
    );
}
