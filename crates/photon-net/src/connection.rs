//! Java-protocol connection facade.
//!
//! Wire codecs are in `photon-protocol`, the observations in [`trace`], phase transitions are in
//! [`session`], and local real-server scenarios are isolated in the private
//! [`probe`] module. The facade re-exports the stable observation and probe
//! entry points used by the existing tools.

mod entities;
mod play;
mod probe;
mod session;
mod trace;

pub use photon_protocol::play::*;
pub use play::PlaySession;
pub use probe::*;
pub use session::query_status_and_ping;
pub use trace::*;
