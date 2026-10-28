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
}
