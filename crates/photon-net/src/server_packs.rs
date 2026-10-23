//! The resource packs a server asks the client to load (`net.minecraft.client.resources.server`):
//! the state machine that decides what the server is told, the downloads and the cache they
//! are kept in.

mod manager;
mod source;

pub use manager::{
    DownloadRequest, Effect, FinalResult, PromptStatus, ServerPackManager, Update, parse_sha1,
};
pub use source::{PackFetcher, PackPrompt, PackSink, PackSource, PromptOutcome};
