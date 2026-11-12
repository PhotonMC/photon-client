//! How many map decoration types a version registers.

use photon_core::version::Version;

/// How many of the map decoration types (`tables/map_data.rs`, in the newest version's order)
/// the version registers: 26.3 added five structures after the trial chambers.
pub fn count(version: Version) -> usize {
    match version {
        Version::V26_1 | Version::V26_2 => 35,
        _ => crate::tables::map_data::MAP_DECORATION_TYPES.len(),
    }
}

/// [`count`] of the version being played.
pub fn played_count() -> usize {
    count(Version::active())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v26_2_numbers_five_fewer_decorations_than_26_3() {
        let all = crate::tables::map_data::MAP_DECORATION_TYPES.len();
        assert_eq!(count(Version::V26_3), all);
        assert_eq!(count(Version::V26_2), all - 5);
        let last = crate::tables::map_data::MAP_DECORATION_TYPES[all - 6];
        assert_eq!(last.name, "minecraft:trial_chambers");
    }
}
