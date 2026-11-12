//! Mods a server delivers (ADR 0024): the server names them on the `photon:mods` channel by id,
//! hash, size and address; the client fetches what its cache lacks, checks every byte against the
//! hash, and keeps the file by that hash, so a mod that did not change is never fetched again.

mod cache;
mod consent;
mod fetch;
mod manifest;
mod sync;

pub use cache::{CacheError, ModCache};
pub use consent::{CONSENT_FILE, Consent, Record, decide, needs_consent};
pub use fetch::{FetchError, ensure};
pub use manifest::{HELLO_CHANNEL, MODS_CHANNEL, Manifest, ManifestError, ModEntry, hello};
pub use sync::{Admission, ModSync, Plan, Question, Reason, Refused};

/// The most bytes one mod may have.
pub const MAX_MOD_BYTES: u64 = 8 << 20;
