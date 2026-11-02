//! What the debug overlay's charts and the game test highlight take from the connection: the
//! ping, the server's tick times, and the blocks the game tests marked.

use super::*;
use std::collections::{HashMap, VecDeque};

/// `LocalSampleLogger.CAPACITY`.
const SAMPLE_CAPACITY: usize = 240;
/// `TpsDebugDimensions.values().length`: the dimensions of a tick time sample.
const TICK_TIME_DIMENSIONS: usize = 4;
/// `DebugSubscriptions.DEDICATED_SERVER_TICK_TIME`: its number in the debug subscription registry.
const TICK_TIME_SUBSCRIPTION: i32 = 0;
/// `GameTestBlockHighlightRenderer.SHOW_POS_DURATION_MS`.
const HIGHLIGHT_DURATION_MS: i64 = 10_000;

/// `LocalSampleLogger`: the last 240 samples of some dimensions each.
#[derive(Debug, Clone)]
pub struct SampleLog {
    dimensions: usize,
    rows: VecDeque<Vec<i64>>,
}

impl SampleLog {
    fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            rows: VecDeque::new(),
        }
    }

    /// `logFullSample`: dimensions the sample does not reach stay zero. A longer sample is the
    /// reference's out-of-bounds copy, so it is refused.
    fn log_full(&mut self, sample: &[i64]) -> bool {
        if sample.len() > self.dimensions {
            return false;
        }
        let mut row = vec![0; self.dimensions];
        row[..sample.len()].copy_from_slice(sample);
        if self.rows.len() == SAMPLE_CAPACITY {
            self.rows.pop_front();
        }
        self.rows.push_back(row);
        true
    }

    /// `logSample`: the first dimension.
    fn log(&mut self, value: i64) {
        self.log_full(&[value]);
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// `get(index, dimension)`, oldest first.
    pub fn get(&self, index: usize, dimension: usize) -> Option<i64> {
        self.rows.get(index)?.get(dimension).copied()
    }

    fn reset(&mut self) {
        self.rows.clear();
    }
}

/// A block the game tests marked: the text over it and when it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestHighlight {
    pub text: String,
    pub remove_at: i64,
}

/// What the charts ask of a tick.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChartRequests {
    /// `showNetworkCharts`: the ping is measured.
    pub ping: bool,
    /// `showFpsCharts`: the server is asked for its tick times.
    pub tick_time: bool,
}

#[derive(Debug)]
pub(super) struct DebugCharts {
    /// `ClientDebugSubscriber.remoteSubscriptions` holds the tick time subscription.
    subscribed_to_tick_time: bool,
    ping: SampleLog,
    tick_time: SampleLog,
    highlights: HashMap<(i32, i32, i32), GameTestHighlight>,
}

impl Default for DebugCharts {
    fn default() -> Self {
        Self {
            subscribed_to_tick_time: false,
            ping: SampleLog::new(1),
            tick_time: SampleLog::new(TICK_TIME_DIMENSIONS),
            highlights: HashMap::new(),
        }
    }
}

impl DebugCharts {
    /// `ClientDebugSubscriber.clear`: the server is no longer subscribed to anything.
    pub(super) fn forget_subscriptions(&mut self) {
        self.subscribed_to_tick_time = false;
    }
}

/// `Util.getMillis`: milliseconds on a clock that only goes forward, from an origin of its own.
fn monotonic_ms() -> i64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis() as i64
}

/// `BlockPos.toShortString`.
fn short_string(pos: (i32, i32, i32)) -> String {
    format!("{}, {}, {}", pos.0, pos.1, pos.2)
}

impl PlaySession {
    /// `ClientPacketListener.tick`'s share for the charts: the ping request while the network
    /// charts are up, then the subscription request when the set of wanted subscriptions changed.
    pub fn tick_debug_charts(&mut self, wants: ChartRequests) -> Result<(), ConnectionError> {
        self.tick_debug_charts_at(wants, monotonic_ms())
    }

    fn tick_debug_charts_at(
        &mut self,
        wants: ChartRequests,
        now_ms: i64,
    ) -> Result<(), ConnectionError> {
        if wants.ping {
            self.send(PLAY_SERVERBOUND_PING_REQUEST_PACKET_ID, |output| {
                output.extend_from_slice(&now_ms.to_be_bytes());
                Ok(())
            })?;
        }
        if wants.tick_time == self.social.charts.subscribed_to_tick_time {
            return Ok(());
        }
        self.social.charts.subscribed_to_tick_time = wants.tick_time;
        self.send(
            PLAY_SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID,
            |output| {
                wire::write_varint(i32::from(wants.tick_time), output);
                if wants.tick_time {
                    wire::write_varint(TICK_TIME_SUBSCRIPTION, output);
                }
                Ok(())
            },
        )
    }

    /// `PingDebugMonitor.onPongReceived`: how long the request took.
    pub(super) fn on_pong_response(&mut self, sent: i64) {
        self.on_pong_response_at(sent, monotonic_ms());
    }

    fn on_pong_response_at(&mut self, sent: i64, now_ms: i64) {
        self.social.charts.ping.log(now_ms - sent);
    }

    /// `DebugScreenOverlay.logRemoteSample`.
    pub(super) fn on_debug_sample(&mut self, sample: &[i64]) -> Result<(), ConnectionError> {
        if self.social.charts.tick_time.log_full(sample) {
            Ok(())
        } else {
            Err(ConnectionError::Codec("debug sample too long"))
        }
    }

    /// `GameTestBlockHighlightRenderer.highlightPos`.
    pub(super) fn on_game_test_highlight(&mut self, positions: ((i32, i32, i32), (i32, i32, i32))) {
        self.on_game_test_highlight_at(positions, monotonic_ms());
    }

    fn on_game_test_highlight_at(
        &mut self,
        (absolute, relative): ((i32, i32, i32), (i32, i32, i32)),
        now_ms: i64,
    ) {
        self.social.charts.highlights.insert(
            absolute,
            GameTestHighlight {
                text: short_string(relative),
                remove_at: now_ms + HIGHLIGHT_DURATION_MS,
            },
        );
    }

    /// The ping samples, in milliseconds.
    pub fn ping_samples(&self) -> &SampleLog {
        &self.social.charts.ping
    }

    /// The server's tick time samples, a dimension of `TpsDebugDimensions` each.
    pub fn tick_time_samples(&self) -> &SampleLog {
        &self.social.charts.tick_time
    }

    /// `DebugScreenOverlay.reset`.
    pub fn reset_chart_samples(&mut self) {
        self.social.charts.ping.reset();
        self.social.charts.tick_time.reset();
    }

    /// `emitGizmos`' share of the markers: the ones whose time is over go, the rest are given.
    pub fn game_test_highlights(&mut self) -> Vec<((i32, i32, i32), GameTestHighlight)> {
        self.game_test_highlights_at(monotonic_ms())
    }

    fn game_test_highlights_at(
        &mut self,
        now_ms: i64,
    ) -> Vec<((i32, i32, i32), GameTestHighlight)> {
        let markers = &mut self.social.charts.highlights;
        markers.retain(|_, marker| now_ms <= marker.remove_at);
        markers.iter().map(|(pos, m)| (*pos, m.clone())).collect()
    }

    /// `LevelExtractor.clear`'s `gameTestBlockHighlightRenderer.clear`.
    pub fn clear_game_test_highlights(&mut self) {
        self.social.charts.highlights.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::play::profile_tests::session;
    use crate::connection::transport::Transport;

    /// The next `count` packets the session sent.
    fn sent(server: &std::net::TcpListener, count: usize) -> Vec<(i32, Vec<u8>)> {
        let (stream, _) = server.accept().expect("accept");
        let mut stream = Transport::from(stream);
        (0..count)
            .map(|_| {
                let packet = crate::connection::read_packet(&mut stream, None).expect("packet");
                let (id, body) = split_packet(&packet).expect("split");
                (id, body.to_vec())
            })
            .collect()
    }

    #[test]
    fn the_ping_goes_out_each_tick_the_network_charts_are_up() {
        let (mut session, server) = session();
        let ping = ChartRequests {
            ping: true,
            tick_time: false,
        };
        session.tick_debug_charts_at(ping, 1234).expect("sent");
        session
            .tick_debug_charts_at(ChartRequests::default(), 1300)
            .expect("sent");
        session.tick_debug_charts_at(ping, 1500).expect("sent");
        let packets = sent(&server, 2);
        let pings: Vec<_> = packets
            .iter()
            .filter(|(id, _)| *id == PLAY_SERVERBOUND_PING_REQUEST_PACKET_ID)
            .map(|(_, body)| i64::from_be_bytes(body[..8].try_into().expect("a long")))
            .collect();
        assert_eq!(pings, [1234, 1500]);
    }

    #[test]
    fn the_subscription_is_requested_only_when_the_wanted_set_changes() {
        let (mut session, server) = session();
        let fps = ChartRequests {
            ping: false,
            tick_time: true,
        };
        session
            .tick_debug_charts_at(ChartRequests::default(), 0)
            .expect("sent");
        session.tick_debug_charts_at(fps, 0).expect("sent");
        session.tick_debug_charts_at(fps, 0).expect("sent");
        session
            .tick_debug_charts_at(ChartRequests::default(), 0)
            .expect("sent");
        let requests: Vec<_> = sent(&server, 2)
            .into_iter()
            .filter(|(id, _)| *id == PLAY_SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID)
            .map(|(_, body)| body)
            .collect();
        assert_eq!(requests, [vec![1, 0], vec![0]]);
    }

    #[test]
    fn a_pong_logs_the_round_trip_and_the_log_keeps_240() {
        let (mut session, _server) = session();
        session.on_pong_response_at(100, 142);
        assert_eq!(session.ping_samples().get(0, 0), Some(42));
        for sent in 0..300 {
            session.on_pong_response_at(sent, sent + 7);
        }
        assert_eq!(session.ping_samples().len(), 240);
        session.reset_chart_samples();
        assert!(session.ping_samples().is_empty());
    }

    #[test]
    fn a_tick_time_sample_fills_the_dimensions_it_has_and_a_longer_one_is_refused() {
        let (mut session, _server) = session();
        session.on_debug_sample(&[5, 3]).expect("logged");
        let log = session.tick_time_samples();
        assert_eq!(
            [0, 1, 2, 3].map(|d| log.get(0, d)),
            [Some(5), Some(3), Some(0), Some(0)]
        );
        assert!(session.on_debug_sample(&[1; 5]).is_err());
    }

    #[test]
    fn a_highlight_shows_the_relative_position_for_ten_seconds() {
        let (mut session, _server) = session();
        session.on_game_test_highlight_at(((10, 64, -3), (1, 2, -3)), 1000);
        session.on_game_test_highlight_at(((10, 64, -3), (4, 5, 6)), 2000);
        let shown = session.game_test_highlights_at(12_000);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].1.text, "4, 5, 6");
        assert!(session.game_test_highlights_at(12_001).is_empty());
    }
}
