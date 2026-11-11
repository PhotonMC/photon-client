//! The model layers of the version being played: the newest version's table, or for an older
//! version that table without the layers the version lacks and with its own definitions of the
//! ones that differ (`tables/entity_models_older.rs`).

use std::sync::OnceLock;

use photon_core::version::Version;

use crate::entity_model::ModelLayer;
use crate::tables::entity_models::LAYERS;
use crate::tables::entity_models_older::OLDER;

/// Every layer of the active version, in order of id. Worked out once per version.
pub fn layers() -> &'static [ModelLayer] {
    static LISTS: [OnceLock<&'static [ModelLayer]>; Version::ALL.len()] =
        [const { OnceLock::new() }; Version::ALL.len()];
    let version = Version::active();
    LISTS[version.index()].get_or_init(|| match build(version) {
        Some(list) => Vec::leak(list),
        None => &LAYERS,
    })
}

/// The layers of an older version, or `None` for a version that has the newest table's.
fn build(version: Version) -> Option<Vec<ModelLayer>> {
    let (_, replaced, absent) = OLDER.iter().find(|(older, ..)| *older == version)?;
    let mut list: Vec<ModelLayer> = LAYERS
        .iter()
        .filter(|layer| !absent.contains(&layer.id))
        .copied()
        .collect();
    for layer in *replaced {
        match list.binary_search_by(|have| have.id.cmp(layer.id)) {
            Ok(index) => list[index] = *layer,
            Err(index) => list.insert(index, *layer),
        }
    }
    Some(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_version_loses_what_it_lacks_and_takes_its_own_definitions() {
        for (version, replaced, absent) in OLDER {
            let list = build(*version).expect("a listed version has a table");
            assert!(list.windows(2).all(|pair| pair[0].id < pair[1].id));
            for id in *absent {
                assert!(list.iter().all(|layer| layer.id != *id), "{id}");
            }
            for layer in *replaced {
                let found = list.iter().find(|have| have.id == layer.id);
                assert_eq!(found, Some(layer));
            }
            assert_eq!(
                list.len(),
                LAYERS.len() - absent.len()
                    + replaced
                        .iter()
                        .filter(|layer| LAYERS.iter().all(|have| have.id != layer.id))
                        .count()
            );
        }
    }

    #[test]
    fn the_newest_version_has_no_older_table() {
        let newest = *Version::ALL.last().expect("a version");
        assert!(build(newest).is_none());
    }
}
