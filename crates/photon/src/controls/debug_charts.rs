//! `DebugScreenOverlay`'s chart flags and the keys that switch them: F3 with 1 to 4.

use photon_gui::hud::debug::EntryList;
use photon_net::connection::ChartRequests;

/// Which charts are switched on. They only show while the overlay does.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChartFlags {
    pub profiler: bool,
    pub fps: bool,
    pub network: bool,
    pub lightmap: bool,
}

/// `DebugScreenOverlay.frameTimeLogger`: the nanoseconds between the last 240 frames.
#[derive(Debug, Default)]
pub struct FrameLog {
    last: Option<std::time::Instant>,
    samples: std::collections::VecDeque<i64>,
}

impl FrameLog {
    /// `logFrameDuration`, called where `Minecraft.runTick` counts the frame. The first frame has no
    /// frame before it to measure from, so it is not logged.
    pub fn frame(&mut self, now: std::time::Instant) {
        if let Some(last) = self.last.replace(now) {
            if self.samples.len() == photon_gui::hud::debug_chart::CAPACITY as usize {
                self.samples.pop_front();
            }
            self.samples
                .push_back(now.saturating_duration_since(last).as_nanos() as i64);
        }
    }

    /// The samples, oldest first.
    pub fn samples(&self) -> impl Iterator<Item = i64> + '_ {
        self.samples.iter().copied()
    }
}

/// `renderX = !overlayVisible || !renderX`, and a chart switched on shows the overlay.
fn flip(flag: &mut bool, overlay: &mut EntryList) -> bool {
    *flag = !overlay.is_overlay_visible() || !*flag;
    if *flag {
        overlay.set_overlay_visible(true);
    }
    *flag
}

impl ChartFlags {
    /// `toggleProfilerChart`.
    pub fn toggle_profiler(&mut self, overlay: &mut EntryList) {
        flip(&mut self.profiler, overlay);
    }

    /// `toggleFpsCharts`: the other full-width charts go.
    pub fn toggle_fps(&mut self, overlay: &mut EntryList) {
        if flip(&mut self.fps, overlay) {
            self.network = false;
            self.lightmap = false;
        }
    }

    /// `toggleNetworkCharts`.
    pub fn toggle_network(&mut self, overlay: &mut EntryList) {
        if flip(&mut self.network, overlay) {
            self.fps = false;
            self.lightmap = false;
        }
    }

    /// `toggleLightmapTexture`.
    pub fn toggle_lightmap(&mut self, overlay: &mut EntryList) {
        if flip(&mut self.lightmap, overlay) {
            self.fps = false;
            self.network = false;
        }
    }

    /// What the connection is to ask for this tick (`showNetworkCharts`, `showFpsCharts`).
    pub fn requests(&self, overlay: &EntryList) -> ChartRequests {
        let shown = overlay.is_overlay_visible();
        ChartRequests {
            ping: shown && self.network,
            tick_time: shown && self.fps,
        }
    }

    /// The flags as the overlay's footer lists them.
    pub fn footer_state(&self) -> (bool, bool, bool, bool) {
        (self.profiler, self.fps, self.network, self.lightmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_log_keeps_the_last_240_gaps_and_skips_the_first_frame() {
        let start = std::time::Instant::now();
        let mut log = FrameLog::default();
        for index in 0..250_u64 {
            log.frame(start + std::time::Duration::from_millis(index * 10));
        }
        assert_eq!(log.samples().count(), 240);
        assert!(log.samples().all(|gap| gap == 10_000_000));
    }

    #[test]
    fn a_chart_switched_on_shows_the_overlay_and_sends_the_others_away() {
        let mut overlay = EntryList::default();
        let mut charts = ChartFlags::default();
        charts.toggle_fps(&mut overlay);
        assert!(overlay.is_overlay_visible());
        assert_eq!(
            charts.requests(&overlay),
            ChartRequests {
                ping: false,
                tick_time: true
            }
        );
        charts.toggle_network(&mut overlay);
        assert_eq!(
            charts.requests(&overlay),
            ChartRequests {
                ping: true,
                tick_time: false
            }
        );
        assert!(!charts.fps);
    }

    #[test]
    fn with_the_overlay_hidden_the_key_switches_the_chart_on_not_off() {
        let mut overlay = EntryList::default();
        let mut charts = ChartFlags::default();
        charts.toggle_network(&mut overlay);
        overlay.set_overlay_visible(false);
        // Still on, but not shown.
        assert_eq!(charts.requests(&overlay), ChartRequests::default());
        charts.toggle_network(&mut overlay);
        assert!(charts.network && overlay.is_overlay_visible());
        charts.toggle_network(&mut overlay);
        assert!(!charts.network);
    }

    #[test]
    fn the_profiler_chart_leaves_the_others_alone() {
        let mut overlay = EntryList::default();
        let mut charts = ChartFlags::default();
        charts.toggle_lightmap(&mut overlay);
        charts.toggle_profiler(&mut overlay);
        assert_eq!(charts.footer_state(), (true, false, false, true));
    }
}
