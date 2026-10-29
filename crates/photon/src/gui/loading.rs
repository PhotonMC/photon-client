//! The loading overlay of a resource reload, drawn over the frame.

use photon_gui::loading_overlay::LoadingOverlay;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// The overlay while there is one, how far the reload says it is, and the clock it fades by.
pub(crate) struct LoadingLayer {
    overlay: Option<LoadingOverlay>,
    progress: f32,
    started_at: std::time::Instant,
}

impl LoadingLayer {
    pub(super) fn new(started_at: std::time::Instant) -> Self {
        Self {
            overlay: None,
            progress: 0.0,
            started_at,
        }
    }

    /// `Util.getMillis` for the overlay: the time since the interface began.
    pub(super) fn now(&self) -> i64 {
        self.started_at.elapsed().as_millis() as i64
    }

    /// `Gui.extractRenderState`: the overlay in the place of the screen.
    pub(super) fn draw(&mut self, frame: &mut GuiFrame, atlas: &GuiAtlas, size: (i32, i32)) {
        let now = self.now();
        if let Some(overlay) = self.overlay.as_mut() {
            overlay.draw(frame, atlas, size, (now, self.progress), &mut |_| {});
        }
    }
}

impl super::Gui {
    /// Show the loading overlay of a reload (`Gui.setOverlay`).
    pub fn set_loading(&mut self, overlay: Option<LoadingOverlay>) {
        self.loading.overlay = overlay;
        self.loading.progress = 0.0;
    }

    /// Whether a loading overlay is up (`gui.overlay() != null`).
    pub fn is_loading(&self) -> bool {
        self.loading.overlay.is_some()
    }

    /// The loading overlay, while there is one.
    pub fn loading_mut(&mut self) -> Option<&mut LoadingOverlay> {
        self.loading.overlay.as_mut()
    }

    /// `Util.getMillis` for the loading overlay.
    pub fn loading_clock(&self) -> i64 {
        self.loading.now()
    }

    /// What `ReloadInstance.getActualProgress` says.
    pub fn set_reload_progress(&mut self, progress: f32) {
        self.loading.progress = progress;
    }
}
