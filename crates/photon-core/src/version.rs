//! The Minecraft versions the client can join ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)):
//! one binary, the version chosen at connect time from the protocol number.

/// Declares the versions, oldest first. Adding a version is one line here: the enum, the list,
/// the position, the protocol number and the name all follow from it, and the newest version is
/// the last line.
macro_rules! versions {
    ($($(#[$doc:meta])* $variant:ident = ($name:literal, $protocol:literal);)+) => {
        /// A supported game version. Behaviour defaults to the newest; older versions differ by
        /// code in `versions/` modules.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Version {
            $($(#[$doc])* $variant,)+
        }

        impl Version {
            /// Every supported version, oldest first.
            pub const ALL: &'static [Version] = &[$(Version::$variant),+];

            /// The protocol number the handshake carries and the status answer reports.
            pub const fn protocol(self) -> i32 {
                match self {
                    $(Version::$variant => $protocol,)+
                }
            }

            /// The release name (`26.2`).
            pub const fn name(self) -> &'static str {
                match self {
                    $(Version::$variant => $name,)+
                }
            }
        }
    };
}

versions! {
    /// Java Edition 26.1, protocol 775.
    V26_1 = ("26.1", 775);
    /// Java Edition 26.2, protocol 776.
    V26_2 = ("26.2", 776);
}

impl Version {
    /// The newest supported version: the one the client is held to full parity with.
    pub const NEWEST: Version = Self::ALL[Self::ALL.len() - 1];

    /// The supported version a protocol number belongs to, if there is one.
    pub fn from_protocol(protocol: i32) -> Option<Version> {
        Self::ALL
            .iter()
            .copied()
            .find(|version| version.protocol() == protocol)
    }

    /// The supported version a release name (`26.2`) belongs to, if there is one.
    pub fn from_name(name: &str) -> Option<Version> {
        Self::ALL
            .iter()
            .copied()
            .find(|version| version.name() == name)
    }

    /// The position in [`Version::ALL`], oldest first; usable in `const` code.
    pub const fn index(self) -> usize {
        self as usize
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
        covers(position(self.since), position(self.until), version.index())
    }
}

/// The position of an optional bound.
const fn position(version: Option<Version>) -> Option<usize> {
    match version {
        Some(version) => Some(version.index()),
        None => None,
    }
}

/// Whether position `at` lies between the bounds, both inclusive and either optional.
const fn covers(since: Option<usize>, until: Option<usize>, at: usize) -> bool {
    if let Some(since) = since
        && at < since
    {
        return false;
    }
    if let Some(until) = until
        && at > until
    {
        return false;
    }
    true
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
        for version in Version::ALL.iter().copied() {
            assert_eq!(Version::from_protocol(version.protocol()), Some(version));
        }
        assert_eq!(Version::from_protocol(775), Some(Version::V26_1));
        assert_eq!(Version::from_protocol(774), None);
    }

    #[test]
    fn the_list_runs_oldest_first_and_ends_at_the_newest() {
        assert!(Version::ALL.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(Version::ALL.last(), Some(&Version::NEWEST));
    }

    #[test]
    fn every_version_sits_at_its_position_in_the_list() {
        for (position, version) in Version::ALL.iter().enumerate() {
            assert_eq!(version.index(), position);
            assert_eq!(Version::ALL[version.index()], *version);
            assert_eq!(Version::from_name(version.name()), Some(*version));
        }
        assert_eq!(Version::from_name("0.0"), None);
    }

    #[test]
    fn protocol_numbers_and_names_are_unique() {
        for (i, a) in Version::ALL.iter().enumerate() {
            for b in &Version::ALL[i + 1..] {
                assert_ne!(a.protocol(), b.protocol());
                assert_ne!(a.name(), b.name());
            }
        }
    }

    #[test]
    fn the_bounds_of_a_span_hold_at_every_position_around_them() {
        // Three positions stand for an older, a middle and a newer version.
        let spans = [
            (None, None, [true, true, true]),
            (Some(1), None, [false, true, true]),
            (None, Some(1), [true, true, false]),
            (Some(1), Some(1), [false, true, false]),
            (Some(2), None, [false, false, true]),
            (None, Some(0), [true, false, false]),
        ];
        for (since, until, expected) in spans {
            for (at, want) in expected.into_iter().enumerate() {
                assert_eq!(covers(since, until, at), want, "{since:?} {until:?} {at}");
            }
        }
    }

    #[test]
    fn a_span_without_bounds_covers_every_version() {
        for version in Version::ALL.iter().copied() {
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
