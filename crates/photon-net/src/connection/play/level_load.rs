//! `LevelLoadTracker`'s client side: after a login or a respawn the client waits for the server's
//! "level chunks load start" event, then for the section around the player to be compiled, and
//! only then tells the server it has loaded.

use std::time::{Duration, Instant};

/// How long the client waits for its own chunks before it lets the player in anyway.
const CLIENT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Why a level is loading (`LevelLoadingScreen.Reason`), which picks the screen's background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingReason {
    NetherPortal,
    EndPortal,
    Other,
}

impl LoadingReason {
    /// `ClientPacketListener.determineLevelLoadingReason`: a death shows the plain screen, any
    /// other way into or out of the Nether or the End shows that portal.
    pub fn of_respawn(player_died: bool, dimension: &str, old_dimension: &str) -> Self {
        const NETHER: &str = "minecraft:the_nether";
        const END: &str = "minecraft:the_end";
        if player_died {
            Self::Other
        } else if dimension == NETHER || old_dimension == NETHER {
            Self::NetherPortal
        } else if dimension == END || old_dimension == END {
            Self::EndPortal
        } else {
            Self::Other
        }
    }
}

/// What the tracker needs of the player each tick (`WaitingForPlayerChunk.isReady`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlayerNeed {
    /// The player's section must be compiled before the level counts as ready.
    Section,
    /// The player is outside the level's height, dead or a spectator: nothing to wait for.
    Nothing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClientState {
    WaitingForServer {
        timeout_at: Instant,
    },
    WaitingForPlayerChunk {
        timeout_at: Instant,
        section_ready: bool,
    },
    Ready {
        at: Instant,
    },
}

/// The client-side load of one level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct LevelLoadTracker {
    state: ClientState,
    reason: LoadingReason,
    close_delay: Duration,
}

impl LevelLoadTracker {
    /// `startClientLoad`: waits for the server first.
    pub(super) fn start(now: Instant, reason: LoadingReason) -> Self {
        Self {
            state: ClientState::WaitingForServer {
                timeout_at: now + CLIENT_WAIT_TIMEOUT,
            },
            reason,
            close_delay: Duration::ZERO,
        }
    }

    pub(super) fn reason(&self) -> LoadingReason {
        self.reason
    }

    /// `loadingPacketsReceived`: the server began to send chunks.
    pub(super) fn loading_packets_received(&mut self) {
        if let ClientState::WaitingForServer { timeout_at } = self.state {
            self.state = ClientState::WaitingForPlayerChunk {
                timeout_at,
                section_ready: false,
            };
        }
    }

    /// Whether the renderer's word on the player's section is wanted
    /// (`getPlayerCompiledSectionCallback` is not null).
    pub(super) fn wants_player_section(&self) -> bool {
        matches!(self.state, ClientState::WaitingForPlayerChunk { .. })
    }

    /// The renderer compiled the player's section (the callback ran).
    pub(super) fn player_section_compiled(&mut self) {
        if let ClientState::WaitingForPlayerChunk { section_ready, .. } = &mut self.state {
            *section_ready = true;
        }
    }

    /// `tickClientLoad`.
    pub(super) fn tick(&mut self, now: Instant, need: PlayerNeed) {
        if let ClientState::WaitingForPlayerChunk {
            timeout_at,
            section_ready,
        } = self.state
        {
            let ready = if now > timeout_at {
                log::warn!(
                    "Timed out while waiting for the client to load chunks, letting the player into the world anyway"
                );
                true
            } else {
                need == PlayerNeed::Nothing || section_ready
            };
            if ready {
                self.state = ClientState::Ready { at: now };
            }
        }
    }

    /// `isLevelReady`.
    pub(super) fn is_ready(&self, now: Instant) -> bool {
        matches!(self.state, ClientState::Ready { at } if now >= at + self.close_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(start: Instant, millis: u64) -> Instant {
        start + Duration::from_millis(millis)
    }

    #[test]
    fn waits_for_the_server_however_long_it_takes() {
        let start = Instant::now();
        let mut tracker = LevelLoadTracker::start(start, LoadingReason::Other);
        assert!(!tracker.wants_player_section());
        // Not even the timeout lets the player in before the server has begun.
        tracker.tick(at(start, 60_000), PlayerNeed::Nothing);
        assert!(!tracker.is_ready(at(start, 60_000)));
        // The renderer's word is ignored too.
        tracker.player_section_compiled();
        tracker.loading_packets_received();
        assert!(tracker.wants_player_section());
        tracker.tick(at(start, 61_000), PlayerNeed::Section);
        assert!(tracker.is_ready(at(start, 61_000)));
    }

    #[test]
    fn waits_for_the_players_section() {
        let start = Instant::now();
        let mut tracker = LevelLoadTracker::start(start, LoadingReason::Other);
        tracker.loading_packets_received();
        tracker.tick(at(start, 100), PlayerNeed::Section);
        assert!(!tracker.is_ready(at(start, 100)));
        tracker.player_section_compiled();
        // Ready on the tick after the renderer said so.
        assert!(!tracker.is_ready(at(start, 150)));
        tracker.tick(at(start, 150), PlayerNeed::Section);
        assert!(tracker.is_ready(at(start, 150)));
        assert!(!tracker.wants_player_section());
    }

    #[test]
    fn a_player_with_nothing_to_see_is_ready_at_once() {
        let start = Instant::now();
        let mut tracker = LevelLoadTracker::start(start, LoadingReason::Other);
        tracker.loading_packets_received();
        tracker.tick(at(start, 50), PlayerNeed::Nothing);
        assert!(tracker.is_ready(at(start, 50)));
    }

    #[test]
    fn gives_up_after_thirty_seconds_of_waiting_for_the_player() {
        let start = Instant::now();
        let mut tracker = LevelLoadTracker::start(start, LoadingReason::Other);
        tracker.loading_packets_received();
        tracker.tick(at(start, 30_000), PlayerNeed::Section);
        assert!(!tracker.is_ready(at(start, 30_000)));
        tracker.tick(at(start, 30_001), PlayerNeed::Section);
        assert!(tracker.is_ready(at(start, 30_001)));
    }

    #[test]
    fn the_close_delay_holds_the_ready_level() {
        let start = Instant::now();
        let mut tracker = LevelLoadTracker::start(start, LoadingReason::Other);
        tracker.close_delay = Duration::from_millis(500);
        tracker.loading_packets_received();
        tracker.tick(at(start, 10), PlayerNeed::Nothing);
        assert!(!tracker.is_ready(at(start, 509)));
        assert!(tracker.is_ready(at(start, 510)));
    }

    #[test]
    fn the_reason_follows_the_dimensions() {
        let reason = LoadingReason::of_respawn;
        assert_eq!(
            reason(false, "minecraft:the_nether", "minecraft:overworld"),
            LoadingReason::NetherPortal
        );
        assert_eq!(
            reason(false, "minecraft:overworld", "minecraft:the_nether"),
            LoadingReason::NetherPortal
        );
        assert_eq!(
            reason(false, "minecraft:the_end", "minecraft:overworld"),
            LoadingReason::EndPortal
        );
        // The Nether wins when both are named.
        assert_eq!(
            reason(false, "minecraft:the_end", "minecraft:the_nether"),
            LoadingReason::NetherPortal
        );
        assert_eq!(
            reason(true, "minecraft:the_nether", "minecraft:overworld"),
            LoadingReason::Other
        );
        assert_eq!(
            reason(false, "minecraft:overworld", "minecraft:overworld"),
            LoadingReason::Other
        );
    }
}
