//! A registry entry known by name, with the versions that have it. The entity type, sound
//! event, menu type and block entity type registries are lists of these; like blocks and
//! items, an entry holds its place in the union list and the wire id of a version skips the
//! entries that version lacks.

use photon_core::version::{Version, VersionSpan};

/// A registry entry: its name and the versions it exists in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Named {
    /// The name with its namespace (`minecraft:pig`).
    pub name: &'static str,
    /// The versions that have the entry.
    pub versions: VersionSpan,
}

impl Named {
    /// An entry every version has.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            versions: VersionSpan::ALL,
        }
    }

    /// The first version that has the entry.
    pub const fn since(mut self, version: Version) -> Self {
        self.versions = self.versions.since(version);
        self
    }

    /// The last version that has the entry.
    pub const fn until(mut self, version: Version) -> Self {
        self.versions = self.versions.until(version);
        self
    }

    /// Whether the entry exists in `version`.
    pub const fn exists_in(&self, version: Version) -> bool {
        self.versions.contains(version)
    }
}

/// The names of the entries, in id order.
pub fn names(entries: &'static [Named]) -> impl Iterator<Item = &'static str> {
    entries.iter().map(|entry| entry.name)
}

/// The numbering of `version`: a gap for each entry it lacks.
pub fn presence(entries: &[Named], version: Version) -> impl Iterator<Item = bool> + '_ {
    entries.iter().map(move |entry| entry.exists_in(version))
}
