//! The `wire` family: protocol codecs and the handshake, status, login and
//! configuration packets.

mod advancements;
mod codec;
mod configuration;
mod crypto;
mod finished;
mod framing;
mod login;
mod registry;
mod stats;
mod status;
mod tags;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    let mut cases = configuration::cases();
    cases.extend(framing::cases());
    cases.extend(login::cases());
    cases.extend(crypto::cases());
    cases.extend(advancements::cases());
    cases.extend(stats::cases());
    cases.extend(status::cases());
    cases
}
