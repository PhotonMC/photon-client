//! The video of an open game page: played on its own thread, its pictures handed to the surface
//! texture as they come, and stopped when the page closes or shows another game.

use photon_render::DecodedImage;
use photon_video::Player;

use super::Screen;

#[derive(Default)]
pub(super) struct Slot {
    current: Option<(String, Player)>,
    serial: u64,
}

impl Slot {
    /// Before a frame is drawn: follows the open page's video, uploads the newest picture and
    /// tells the page whether there is one to show.
    pub(super) fn update(
        &mut self,
        screen: &mut Screen,
        backend: &mut photon_render::VulkanBackend,
    ) {
        let Screen::Home(home) = screen else {
            self.current = None;
            return;
        };
        let wanted = home.wanted_video().map(str::to_owned);
        if self.current.as_ref().map(|(url, _)| url) != wanted.as_ref() {
            self.serial = 0;
            self.current = wanted.map(|url| {
                let player = Player::start(url.clone());
                (url, player)
            });
        }
        let Some((_, player)) = &self.current else {
            home.set_video_live(false);
            return;
        };
        if let Some(frame) = player.take_frame(self.serial) {
            self.serial = frame.serial;
            let image = DecodedImage {
                width: frame.width,
                height: frame.height,
                rgba: frame.rgba,
            };
            if let Err(error) = backend.set_surface_texture(&image) {
                log::warn!("video frame not uploaded: {error:?}");
            }
        }
        home.set_video_live(self.serial > 0 && !player.failed());
    }
}
