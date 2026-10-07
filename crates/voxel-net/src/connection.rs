//! Java-protocol connection facade.
//!
//! Wire codecs and observations are in [`protocol`], phase transitions are in
//! [`session`], and local real-server scenarios are isolated in the private
//! [`probe`] module. The facade re-exports the stable observation and probe
//! entry points used by the existing tools.

mod entities;
mod play;
mod probe;
mod protocol;
mod session;

pub use play::PlaySession;
pub use probe::*;
pub use protocol::*;
pub use session::query_status_and_ping;
