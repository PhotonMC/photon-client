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
