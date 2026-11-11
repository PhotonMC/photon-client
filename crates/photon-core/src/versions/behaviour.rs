//! The version-dependent values that are not yes/no rules (those are rows of
//! [`super::rules`]): numbers and names the active version carries, the per-version table slot
//! and the asset version a bake reads. Every one reads the active version, so a process holds
//! one version at a time.

use crate::version::Version;

/// Where the active version keeps a table that is worked out once per version (an index below
/// `Version::ALL.len()`).
pub fn table_slot() -> usize {
    Version::active().index()
}

/// Whether the custom statistic exists in the active version (the straw bed one is new in 26.3).
pub fn custom_stat_exists(name: &str) -> bool {
    !name.ends_with(":sleep_in_straw_bed") || !Version::active_before_26_3()
}

/// The number of command argument parser types the active version registers: 26.3 added five
/// after the template and heightmap parsers, moving the UUID one from 56 to 61.
pub fn command_parser_count() -> i32 {
    if Version::active_before_26_3() {
        57
    } else {
        62
    }
}

/// The id of the UUID command argument parser (the last one registered).
pub fn command_uuid_parser_id() -> i32 {
    command_parser_count() - 1
}

/// The data version the active version's files carry (`world_version` of its `version.json`).
pub fn data_version() -> i32 {
    if Version::active_before_26_3() {
        4903
    } else {
        5023
    }
}

/// The resource pack format `(major, minor)` of the game the session plays (26.1 shares 26.2's
/// until its jar is read).
pub fn resource_pack_format() -> (i32, i32) {
    if Version::active_before_26_3() {
        (88, 0)
    } else {
        (97, 1)
    }
}

thread_local! {
    /// The version of the assets this thread reads, when that is not the version being played
    /// (a bake of another version's blocks ahead of a join).
    static ASSET_VERSION: std::cell::Cell<Option<Version>> = const { std::cell::Cell::new(None) };
}

/// Run `read` with the assets of `version` as the ones this thread reads, whatever version is
/// being played.
pub fn with_asset_version<R>(version: Option<Version>, read: impl FnOnce() -> R) -> R {
    let before = ASSET_VERSION.with(|cell| cell.replace(version));
    let result = read();
    ASSET_VERSION.with(|cell| cell.set(before));
    result
}

/// Whether block model elements carry the boolean `shade` (before 26.3, which replaced it with
/// `shade_direction_override`). Follows the assets being read, else the version played.
pub fn model_shade_is_boolean() -> bool {
    ASSET_VERSION
        .with(std::cell::Cell::get)
        .map_or_else(Version::active_before_26_3, |version| {
            version < Version::V26_3
        })
}

/// The release name the session plays, as the servers and the debug screen show it.
pub fn game_version_name() -> &'static str {
    Version::active().name()
}
