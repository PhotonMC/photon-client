//! The Minecraft versions the client can join ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)):
//! one binary, the version chosen at connect time from the protocol number.

/// A supported game version. Behaviour defaults to the newest; older versions differ by code in
/// `versions/` modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    /// Java Edition 26.2, protocol 776.
    V26_2,
}

impl Version {
    /// The newest supported version: the one the client is held to full parity with.
    pub const NEWEST: Version = Version::V26_2;

    /// Every supported version, oldest first.
    pub const ALL: [Version; 1] = [Version::V26_2];

    /// The protocol number the handshake carries and the status answer reports.
    pub const fn protocol(self) -> i32 {
        match self {
            Version::V26_2 => 776,
        }
    }

    /// The supported version a protocol number belongs to, if there is one.
    pub fn from_protocol(protocol: i32) -> Option<Version> {
        Self::ALL
            .into_iter()
            .find(|version| version.protocol() == protocol)
    }
}

impl Version {
    /// The position in [`Version::ALL`], oldest first; usable in `const` code.
    pub const fn index(self) -> usize {
        match self {
            Version::V26_2 => 0,
        }
    }
}

/// The versions a definition exists in: from `since` and up to and including `until`; with
/// neither it exists in every version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionSpan {
    since: Option<Version>,
    until: Option<Version>,
}

impl VersionSpan {
    /// Every version.
    pub const ALL: VersionSpan = VersionSpan {
        since: None,
        until: None,
    };

    /// The first version that has the definition.
    pub const fn since(mut self, version: Version) -> Self {
        self.since = Some(version);
        self
    }

    /// The last version that has the definition; a newer version removed it.
    pub const fn until(mut self, version: Version) -> Self {
        self.until = Some(version);
        self
    }

    /// Whether the definition exists in `version`.
    pub const fn contains(self, version: Version) -> bool {
        let at = version.index();
        if let Some(since) = self.since
            && at < since.index()
        {
            return false;
        }
        if let Some(until) = self.until
            && at > until.index()
        {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_version_is_protocol_776() {
        assert_eq!(Version::NEWEST.protocol(), 776);
    }

    #[test]
    fn every_version_is_found_again_by_its_protocol() {
        for version in Version::ALL {
            assert_eq!(Version::from_protocol(version.protocol()), Some(version));
        }
        assert_eq!(Version::from_protocol(775), None);
    }

    #[test]
    fn the_list_runs_oldest_first_and_ends_at_the_newest() {
        assert!(Version::ALL.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(Version::ALL.last(), Some(&Version::NEWEST));
    }

    #[test]
    fn a_span_without_bounds_covers_every_version() {
        for version in Version::ALL {
            assert!(VersionSpan::ALL.contains(version));
        }
    }

    #[test]
    fn since_and_until_bound_the_span_inclusively() {
        let v = Version::V26_2;
        assert!(VersionSpan::ALL.since(v).contains(v));
        assert!(VersionSpan::ALL.until(v).contains(v));
        assert!(VersionSpan::ALL.since(v).until(v).contains(v));
    }
}
