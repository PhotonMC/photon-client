//! The loading overlay of the first start (`Minecraft`'s `new LoadingOverlay(.., false)`): the
//! logo over the brand colour while the interface loads, then its fade-out over the title.

use std::time::Instant;

use photon_core::resources::FolderResources;
use photon_gui::loading_overlay::LoadingOverlay;
use photon_render::gui::{GuiAtlas, GuiFrame};

use super::{Menu, draw_screen};
use crate::play::Client;

/// The overlay before the interface exists: the little atlas it draws from.
pub(super) struct Splash {
    overlay: LoadingOverlay,
    atlas: GuiAtlas,
    started: Instant,
}

/// The clock of `Util.getMillis` for an overlay begun at `started`.
pub(super) fn millis(started: Instant) -> i64 {
    started.elapsed().as_millis() as i64
}

impl Splash {
    /// Put the logo on the GPU. Without it (no vanilla logo, no pipeline) the start goes on blind.
    pub(super) fn open(client: &mut Client, root: &std::path::Path) -> Option<Self> {
        let Some((atlas, texture)) = GuiAtlas::load_splash(&FolderResources::new(root)) else {
            log::warn!("the logo could not be read from {}", root.display());
            return None;
        };
        if let Err(error) = client.backend.load_gui_texture(&texture) {
            log::warn!("the logo could not be put on the GPU: {error}");
            return None;
        }
        Some(Self {
            overlay: LoadingOverlay::new(false, false),
            atlas,
            started: Instant::now(),
        })
    }

    /// One frame of the overlay alone, at `progress` of the load.
    pub(super) fn frame(&mut self, client: &mut Client, progress: f32) {
        let scale = client.surface.scale();
        let mut frame = GuiFrame::default();
        let size = (scale.gui_scaled_width(), scale.gui_scaled_height());
        let now = millis(self.started);
        let clear = self
            .overlay
            .draw(&mut frame, &self.atlas, size, (now, progress), &mut |_| {});
        let gui_scale = scale.gui_scale() as f32;
        let drawn = client.backend.set_gui(
            &frame,
            [
                scale.framebuffer_width() as f32 / gui_scale,
                scale.framebuffer_height() as f32 / gui_scale,
            ],
        );
        if let Err(error) = drawn {
            log::warn!("the loading overlay could not be drawn: {error}");
            return;
        }
        let colour = clear.map_or([0.0, 0.0, 0.0, 1.0], argb_clear);
        if let Err(error) = client.backend.draw_scene(colour, super::IDENTITY) {
            log::warn!("the loading overlay could not be shown: {error}");
        }
    }

    /// Hand the overlay and its clock on to the title screen's frames.
    pub(super) fn into_overlay(self) -> (LoadingOverlay, Instant) {
        (self.overlay, self.started)
    }
}

/// An ARGB brand colour as the clear colour of a frame.
pub(super) fn argb_clear(colour: u32) -> [f32; 4] {
    let channel = |shift: u32| ((colour >> shift) & 0xFF) as f32 / 255.0;
    [channel(16), channel(8), channel(0), 1.0]
}

/// Put the overlay up with its first frame, as the window opens.
pub(super) fn begin(client: &mut Client) -> Option<Splash> {
    let root = photon_core::content_root::assets().unwrap_or_default();
    let mut splash = Splash::open(client, std::path::Path::new(&root));
    match splash.as_mut() {
        Some(splash) => splash.frame(client, 0.0),
        None => log::warn!("the loading overlay has no logo to show"),
    }
    splash
}

impl Menu {
    /// One frame of the screen, under the loading overlay of the first start while it lasts.
    /// The colour the frame is cleared to.
    pub(super) fn draw_frame(
        &mut self,
        client: &mut Client,
        size: (i32, i32),
    ) -> Result<[f32; 4], Box<dyn std::error::Error>> {
        let mouse = self.mouse;
        let screen = &mut self.screen;
        let lang = self.gui.lang();
        let host = &mut self.host;
        let loading = &mut self.loading;
        let mut clear = [0.0, 0.0, 0.0, 1.0];
        let mut covered = false;
        self.gui.draw_menu_screen(
            &mut client.backend,
            &client.surface,
            |frame, atlas, font| {
                let Some((overlay, started)) = loading else {
                    draw_screen(screen, host, (frame, atlas, font), (lang, mouse, size));
                    return;
                };
                let now = millis(*started);
                // `LoadingOverlay.tick`: a done load answers and the fade-out starts.
                if overlay.ready(true, now) {
                    overlay.start_fade_out(now);
                }
                // The overlay stands in the place of the screen until it fades out.
                let mut under = |frame: &mut GuiFrame| {
                    draw_screen(screen, host, (frame, atlas, font), (lang, mouse, size));
                };
                if let Some(colour) = overlay.draw(frame, atlas, size, (now, 1.0), &mut under) {
                    clear = argb_clear(colour);
                    covered = true;
                }
            },
        )?;
        // The overlay stands in the place of the screen, panorama included.
        if covered {
            client.backend.set_panorama(None);
        }
        if self
            .loading
            .as_ref()
            .is_some_and(|(overlay, started)| overlay.finished(millis(*started)))
        {
            self.loading = None;
        }
        Ok(clear)
    }
}
