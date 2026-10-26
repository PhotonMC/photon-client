//! Resource packs (`net.minecraft.server.packs`): where a pack's files come from, what its
//! `pack.mcmeta` says, and the stack of packs the game reads its assets through
//! (`MultiPackResourceManager`).

mod meta;
mod pack;
mod selection;
mod source;
mod stack;

pub use meta::{Compatibility, IdentifierPattern, PackFormat, format_range};
pub use pack::{Pack, PackError};
pub use selection::{
    PackEntry, PackSource, Position, Repository, SelectionConfig, SelectionModel, Side, Title,
    discover_folder, options_lists, pack_directory, vanilla_entry,
};
pub use source::{DirectorySource, Source, ZipSource};
pub use stack::{Resource, ResourceStack};
