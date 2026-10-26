//! One resource pack (`Pack`, `PackResources`, `CompositePackResources`): a source of files, the
//! overlays of its `pack.mcmeta` that apply to this game, and the filter it applies to the packs
//! below it.

use std::collections::BTreeSet;

use photon_core::identifier::Identifier;
use photon_core::json::{self, JsonObject, JsonValue};

use crate::meta::{Compatibility, Filter, PackFormat, compatibility, filter_of, overlays_for};
use crate::source::Source;

/// Why a pack cannot be used.
#[derive(Debug, PartialEq, Eq)]
pub enum PackError {
    /// There is no `pack.mcmeta`, or it is not a JSON object.
    NoMetadata,
    /// The `pack` section is missing or has no description.
    NoPackSection,
}

pub struct Pack {
    id: String,
    source: Box<dyn Source>,
    /// The folders to look in before the pack's own root, the topmost first.
    overlays: Vec<String>,
    filter: Option<Filter>,
    compatibility: Compatibility,
    /// The `description` of the `pack` section, as written (a text component).
    description: JsonValue,
}

impl Pack {
    /// Open a pack for the running game, whose pack format is `current`
    /// (`Pack.readPackMetadata`). A pack whose `pack.mcmeta` has no `pack` section is refused;
    /// one made for another game version is still taken.
    pub fn open(
        id: impl Into<String>,
        source: Box<dyn Source>,
        current: PackFormat,
    ) -> Result<Self, PackError> {
        let meta = read_meta(source.as_ref()).ok_or(PackError::NoMetadata)?;
        let section = meta.object("pack").ok_or(PackError::NoPackSection)?;
        if !section.has("description") {
            return Err(PackError::NoPackSection);
        }
        let description = section
            .get_str("description")
            .cloned()
            .unwrap_or(JsonValue::Null);
        let mut overlays = overlays_for(&meta, current);
        overlays.reverse();
        Ok(Self {
            id: id.into(),
            filter: filter_of(&meta),
            compatibility: compatibility(&meta, current),
            description,
            overlays,
            source,
        })
    }

    /// A pack with no `pack.mcmeta` to read, such as the game's own assets.
    pub fn built_in(id: impl Into<String>, source: Box<dyn Source>) -> Self {
        Self {
            id: id.into(),
            source,
            overlays: Vec::new(),
            filter: None,
            compatibility: Compatibility::Compatible,
            description: JsonValue::Null,
        }
    }

    /// Whether the pack's declared formats include the game's (`PackCompatibility.COMPATIBLE`).
    pub fn is_compatible(&self) -> bool {
        self.compatibility.is_compatible()
    }

    /// `Pack.getCompatibility`.
    pub fn compatibility(&self) -> Compatibility {
        self.compatibility
    }

    /// `Pack.getDescription`, as the JSON the pack wrote.
    pub fn description(&self) -> &JsonValue {
        &self.description
    }

    /// The bytes of the pack's own `pack.png`, its icon.
    pub fn icon(&self) -> Option<Vec<u8>> {
        self.source.read("pack.png")
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn filter(&self) -> Option<&Filter> {
        self.filter.as_ref()
    }

    /// The roots to search, the overlays first and the pack itself last, each with the prefix
    /// its `assets` folder has.
    fn roots(&self) -> impl Iterator<Item = String> + '_ {
        self.overlays
            .iter()
            .map(|overlay| format!("{overlay}/assets"))
            .chain(std::iter::once("assets".to_owned()))
    }

    /// Whether the pack has the file `id` names.
    pub fn contains(&self, id: &Identifier) -> bool {
        self.roots().any(|root| {
            let path = format!("{root}/{}/{}", id.namespace(), id.path());
            self.source.exists(&path)
        })
    }

    /// The bytes of the file `id` names (`getResource`).
    pub fn read(&self, id: &Identifier) -> Option<Vec<u8>> {
        self.roots().find_map(|root| {
            let path = format!("{root}/{}/{}", id.namespace(), id.path());
            self.source.read(&path)
        })
    }

    /// The namespaces the pack has files for (`getNamespaces`).
    pub fn namespaces(&self) -> BTreeSet<String> {
        let all = self.roots().flat_map(|root| self.source.folders_in(&root));
        all.collect()
    }

    /// The ids of the files in the namespace's `directory` and below (`listResources`).
    pub fn list(&self, namespace: &str, directory: &str) -> BTreeSet<Identifier> {
        let mut found = BTreeSet::new();
        for root in self.roots() {
            let base = format!("{root}/{namespace}");
            let below = if directory.is_empty() {
                base.clone()
            } else {
                format!("{base}/{directory}")
            };
            for file in self.source.files_below(&below) {
                let path = file.get(base.len() + 1..).unwrap_or_default();
                match Identifier::from_namespace_and_path(namespace, path) {
                    Ok(id) => {
                        found.insert(id);
                    }
                    Err(_) => log::warn!("invalid path in pack {}: {namespace}:{path}", self.id),
                }
            }
        }
        found
    }
}

fn read_meta(source: &dyn Source) -> Option<JsonObject> {
    let bytes = source.read("pack.mcmeta")?;
    let text = std::str::from_utf8(&bytes).ok()?;
    match json::parse(text).ok()? {
        JsonValue::Object(object) => Some(object),
        _ => None,
    }
}
