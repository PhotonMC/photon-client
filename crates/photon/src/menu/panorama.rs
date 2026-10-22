//! The title screen's rotating panorama.

use std::time::Instant;

use super::{Menu, Screen};
use crate::play::Client;

impl Menu {
    /// `Panorama.extractRenderState`: turn the cube by the frame's real time and show it.
    pub(super) fn spin_panorama(&mut self, client: &mut Client) {
        let now = Instant::now();
        let ticks = photon_render::panorama::real_delta_ticks(
            now.duration_since(self.last_spin).as_secs_f32(),
        );
        self.last_spin = now;
        // `panoramaScrollSpeed` is not read yet; its default is a full speed.
        self.spin = photon_render::panorama::spin_after(self.spin, ticks, 1.0);
        let extent = client.backend.description().extent;
        // `Screen.extractBackground` blurs behind every screen but the title's, which draws no
        // background of its own.
        let title = matches!(self.screen, Screen::Title(_));
        let blur = if title { 0 } else { self.blurriness as u32 };
        if let Err(error) = client.backend.set_panorama_blur(blur) {
            log::warn!("panorama blur: {error:?}");
        }
        client
            .backend
            .set_panorama(Some(photon_render::panorama::view_projection(
                -self.spin,
                extent.width,
                extent.height,
            )));
    }
}

/// `panorama_0` to `panorama_5` of the title screen's background, from the asset index, where
/// the real faces are, before the jar, which holds one-pixel stand-ins.
pub(super) fn faces(root: &str) -> super::Result<[photon_render::DecodedImage; 6]> {
    let index = crate::indexed::IndexedAssets::open(std::path::Path::new(root));
    let face = |index_number: usize| -> super::Result<photon_render::DecodedImage> {
        let name = format!("gui/title/background/panorama_{index_number}");
        let indexed = index
            .as_ref()
            .and_then(|index| index.object(&format!("minecraft/textures/{name}.png")));
        match indexed {
            Some(bytes) => Ok(photon_render::texture::png::decode_rgba(&bytes)
                .map_err(|error| format!("panorama: {error:?}"))?),
            None => crate::play::texture(root, &name, "panorama"),
        }
    };
    Ok([face(0)?, face(1)?, face(2)?, face(3)?, face(4)?, face(5)?])
}
