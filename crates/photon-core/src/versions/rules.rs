//! The behaviour rules of a version ([ADR 0013](../../../../docs/adr/0013-multi-version-content-model.md)):
//! one row per difference, grouped by the part of the game it belongs to. A row names the rule,
//! says what it decides and gives the version the answer turns at; nothing here holds logic.
//!
//! A rule is a `bool` field of its domain's struct. `Rules::of` resolves every rule for a
//! version once; code reads the field (`rules.world.boat_clamp_is_uniform`) and never compares
//! versions itself.

use crate::version::Version;

/// Declares a domain's rules. `since V` holds from version `V` on, `before V` holds in every
/// version older than `V`.
macro_rules! rules {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$doc:meta])* $rule:ident: $kind:ident $turn:ident,)+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            $($(#[$doc])* pub $rule: bool,)+
        }

        impl $name {
            /// Every rule of the domain with the version its answer turns at.
            pub const TURNS: &'static [(&'static str, Version)] =
                &[$((stringify!($rule), Version::$turn),)+];

            /// The domain's rules as `version` has them.
            pub const fn of(version: Version) -> Self {
                Self {
                    $($rule: rules!(@holds $kind $turn, version),)+
                }
            }
        }
    };
    (@holds since $turn:ident, $version:expr) => {
        $version.index() >= Version::$turn.index()
    };
    (@holds before $turn:ident, $version:expr) => {
        $version.index() < Version::$turn.index()
    };
}

mod game;
mod hud;
mod protocol;
mod render;
mod world;

pub use game::GameRules;
pub use hud::HudRules;
pub use protocol::ProtocolRules;
pub use render::RenderRules;
pub use world::WorldRules;

/// Every behaviour rule of one version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    /// The version the rules are of.
    pub version: Version,
    /// The level: blocks, entities, movement, particles and their sounds.
    pub world: WorldRules,
    /// What the player's actions do: breaking, using, attacking, menus' contents.
    pub game: GameRules,
    /// The HUD and the screens the game opens while playing.
    pub hud: HudRules,
    /// How things are drawn.
    pub render: RenderRules,
    /// What is read from and written to the connection and to files.
    pub protocol: ProtocolRules,
}

impl Rules {
    const fn resolve(version: Version) -> Self {
        Self {
            version,
            world: WorldRules::of(version),
            game: GameRules::of(version),
            hud: HudRules::of(version),
            render: RenderRules::of(version),
            protocol: ProtocolRules::of(version),
        }
    }

    /// The rules of `version`.
    pub fn of(version: Version) -> &'static Rules {
        const fn all<const N: usize>() -> [Rules; N] {
            let mut rules = [Rules::resolve(Version::ALL[0]); N];
            let mut index = 0;
            while index < N {
                rules[index] = Rules::resolve(Version::ALL[index]);
                index += 1;
            }
            rules
        }
        static ALL: [Rules; Version::ALL.len()] = all();
        &ALL[version.index()]
    }

    /// Every rule with its domain and the version its answer turns at: what a dropped version
    /// leaves to delete, and what a check of unused or duplicate rules walks.
    pub fn turns() -> impl Iterator<Item = (&'static str, &'static str, Version)> {
        let domains: [(&str, &[(&str, Version)]); 5] = [
            ("world", WorldRules::TURNS),
            ("game", GameRules::TURNS),
            ("hud", HudRules::TURNS),
            ("render", RenderRules::TURNS),
            ("protocol", ProtocolRules::TURNS),
        ];
        domains.into_iter().flat_map(|(domain, turns)| {
            turns
                .iter()
                .map(move |(rule, version)| (domain, *rule, *version))
        })
    }
}

/// The rules of the version this process plays. The interim seam: a process holds one version
/// at a time, and code that has a session's rules should take them from there instead.
pub fn rules() -> &'static Rules {
    Rules::of(Version::active())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_holds_on_its_side_of_the_turn_only() {
        rules! {
            Sample {
                /// Holds from 26.3.
                newer: since V26_3,
                /// Holds before 26.3.
                older: before V26_3,
            }
        }
        assert_eq!(
            Sample::of(Version::V26_2),
            Sample {
                newer: false,
                older: true
            }
        );
        assert_eq!(
            Sample::of(Version::V26_3),
            Sample {
                newer: true,
                older: false
            }
        );
        assert_eq!(Sample::TURNS.len(), 2);
    }

    #[test]
    fn rule_names_are_unique_within_a_domain() {
        let mut seen = std::collections::HashSet::new();
        for (domain, rule, _) in Rules::turns() {
            assert!(
                seen.insert((domain, rule)),
                "{domain}.{rule} is declared twice"
            );
        }
    }

    #[test]
    fn the_rules_of_a_version_name_it() {
        for version in Version::ALL {
            assert_eq!(Rules::of(*version).version, *version);
        }
    }
}
