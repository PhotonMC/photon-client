//! What differs between the charts: how a sample reads, how tall and what colour its bar is, and
//! the lines and labels a chart adds (`PingDebugChart`, `FpsDebugChart`, `TpsDebugChart` and
//! `BandwidthDebugChart`).

use super::{DIMENSIONS, HEIGHT, fixed, horizontal, java_round, ramp, shaded};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

const PING_TOP: f64 = 500.0;
const FPS_TOP_MS: f64 = 33.333_333_333_333_336;
const BANDWIDTH_TOP: f64 = 1_048_576.0;
const TICK_METHOD: u32 = -6_745_839_i32 as u32;
const TASKS: u32 = -4_548_257_i32 as u32;
const OTHER: u32 = -10_547_572_i32 as u32;
const GREEN: u32 = 0xFF00_FF00;
const YELLOW: u32 = 0xFFFF_FF00;
const RED: u32 = 0xFFFF_0000;
const CYAN: u32 = 0xFF00_FFFF;
const LAVENDER: u32 = -6_250_241_i32 as u32;
const WHITE: u32 = 0xFFFF_FFFF;

/// The kinds of chart this draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// Round trip times in milliseconds.
    Ping,
    /// Frame times in nanoseconds, with the frame rate limit option.
    Fps { limit: i32 },
    /// The server's tick times in nanoseconds, with the milliseconds a tick should take.
    Tps { mspt: f32 },
    /// Bytes received in a tick.
    Bandwidth,
}

impl Kind {
    /// `toDisplayString`.
    pub(super) fn display(self, sample: f64) -> String {
        match self {
            Self::Ping => format!("{} ms", java_round(sample)),
            Self::Fps { .. } | Self::Tps { .. } => format!("{} ms", java_round(sample / 1.0e6)),
            Self::Bandwidth => bytes_per_second(sample * 20.0),
        }
    }

    /// `getSampleHeight`.
    pub(super) fn bar_height(self, sample: f64) -> i32 {
        match self {
            Self::Ping => java_round(sample * 60.0 / PING_TOP) as i32,
            Self::Fps { .. } => java_round(sample / 1.0e6 * 60.0 / FPS_TOP_MS) as i32,
            Self::Tps { mspt } => java_round(sample / 1.0e6 * 60.0 / f64::from(mspt)) as i32,
            Self::Bandwidth => bandwidth_height(sample * 20.0),
        }
    }

    /// `getSampleColor`.
    pub(super) fn colour(self, sample: i64) -> u32 {
        let value = sample as f64;
        match self {
            Self::Ping => ramp(value, (0.0, 250.0, PING_TOP), (GREEN, YELLOW, RED)),
            Self::Fps { .. } => ramp(value / 1.0e6, (0.0, 28.0, 56.0), (GREEN, YELLOW, RED)),
            Self::Tps { mspt } => {
                let mspt = f64::from(mspt);
                ramp(
                    value / 1.0e6,
                    (mspt, mspt * 1.125, mspt * 1.25),
                    (GREEN, YELLOW, RED),
                )
            }
            Self::Bandwidth => ramp(
                value * 20.0,
                (0.0, 8192.0, 1.048_576e7),
                (CYAN, LAVENDER, RED),
            ),
        }
    }

    /// `getValueForAggregation`: a tick's time less the time it was idle.
    pub(super) fn aggregate(self, row: &[i64; DIMENSIONS]) -> i64 {
        match self {
            Self::Tps { .. } => row[0] - row[3],
            _ => row[0],
        }
    }

    /// `extractSampleBars`: the sample's bar, and the parts a tick is made of over it.
    pub(super) fn bars(
        self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        row: &[i64; DIMENSIONS],
        (x, bottom): (i32, i32),
    ) {
        let height = self.bar_height(row[0] as f64);
        frame.fill(
            atlas,
            (x, bottom - height, x + 1, bottom),
            self.colour(row[0]),
        );
        if let Self::Tps { .. } = self {
            let method = self.bar_height(row[1] as f64);
            frame.fill(atlas, (x, bottom - method, x + 1, bottom), TICK_METHOD);
            let tasks = self.bar_height(row[2] as f64);
            let top = bottom - method - tasks;
            frame.fill(atlas, (x, top, x + 1, bottom - method), TASKS);
            let other = self.bar_height((row[0] - row[3] - row[1] - row[2]) as f64);
            frame.fill(atlas, (x, top - other, x + 1, top), OTHER);
        }
    }

    /// `extractAdditionalLinesAndLabels`.
    pub(super) fn extras(
        self,
        frame: &mut GuiFrame,
        fonts: (&GuiAtlas, &Font),
        (left, width, bottom): (i32, i32, i32),
    ) {
        let (atlas, _) = fonts;
        let top = bottom - HEIGHT;
        let across = |y| (left, left + width - 1, y);
        match self {
            Self::Ping => shaded(frame, fonts, "500 ms", (left + 1, top + 1)),
            Self::Fps { limit } => {
                let fps = if photon_core::versions::behaviour::debug_footer_fps_lower_case() {
                    "fps"
                } else {
                    "FPS"
                };
                shaded(frame, fonts, &format!("30 {fps}"), (left + 1, top + 1));
                shaded(
                    frame,
                    fonts,
                    &format!("60 {fps}"),
                    (left + 1, bottom - 30 + 1),
                );
                horizontal(frame, atlas, across(bottom - 30), WHITE);
                if limit > 0 && limit <= 250 {
                    let y = bottom - self.bar_height(1.0e9 / f64::from(limit)) - 1;
                    horizontal(frame, atlas, across(y), CYAN);
                }
            }
            Self::Tps { mspt } => {
                let tps = 1000.0_f32 / mspt;
                let unit = if photon_core::versions::behaviour::debug_footer_fps_lower_case() {
                    "tps"
                } else {
                    "TPS"
                };
                let text = format!("{} {unit}", fixed(f64::from(tps), 1));
                shaded(frame, fonts, &text, (left + 1, top + 1));
            }
            Self::Bandwidth => {
                for rate in [64.0, 1024.0, 16384.0] {
                    let y = bottom - bandwidth_height(rate);
                    shaded(frame, fonts, &bytes_per_second(rate), (left + 1, y + 1));
                    horizontal(frame, atlas, across(y), WHITE);
                }
                let y = bottom - bandwidth_height(BANDWIDTH_TOP);
                shaded(
                    frame,
                    fonts,
                    &bytes_per_second(BANDWIDTH_TOP),
                    (left + 1, y + 1),
                );
            }
        }
    }
}

/// `BandwidthDebugChart.toDisplayStringInternal`.
fn bytes_per_second(rate: f64) -> String {
    if rate >= BANDWIDTH_TOP {
        format!("{} MiB/s", fixed(rate / BANDWIDTH_TOP, 1))
    } else if rate >= 1024.0 {
        format!("{} KiB/s", fixed(rate / 1024.0, 1))
    } else {
        format!("{} B/s", rate.floor() as i64)
    }
}

/// `BandwidthDebugChart.getSampleHeightInternal`: the rate on a logarithmic scale.
fn bandwidth_height(rate: f64) -> i32 {
    java_round((rate + 1.0).ln() * 60.0 / BANDWIDTH_TOP.ln()) as i32
}
