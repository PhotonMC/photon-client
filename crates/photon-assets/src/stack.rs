//! The packs the game reads its assets through, bottom to top (`MultiPackResourceManager` and
//! `FallbackResourceManager`): the topmost pack that has a file wins, except that a pack's
//! filter hides what the packs below it hold.

use std::collections::{BTreeMap, BTreeSet};

use photon_core::identifier::Identifier;
use photon_core::json::{self, JsonObject, JsonValue};

use crate::meta::Filter;
use crate::pack::Pack;

/// A file found in the stack.
pub struct Resource<'a> {
    pack: &'a Pack,
    id: Identifier,
    /// The pack whose `.mcmeta` file belongs to this one.
    meta: Option<&'a Pack>,
}

impl Resource<'_> {
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    /// The id of the pack the file comes from.
    pub fn pack_id(&self) -> &str {
        self.pack.id()
    }

    pub fn bytes(&self) -> Option<Vec<u8>> {
        self.pack.read(&self.id)
    }

    /// The JSON object of the file's `.mcmeta` neighbour, if it has a valid one.
    pub fn metadata(&self) -> Option<JsonObject> {
        let id = meta_id(&self.id)?;
        let bytes = self.meta?.read(&id)?;
        match json::parse(std::str::from_utf8(&bytes).ok()?).ok()? {
            JsonValue::Object(object) => Some(object),
            _ => None,
        }
    }
}

fn meta_id(id: &Identifier) -> Option<Identifier> {
    Identifier::from_namespace_and_path(id.namespace(), format!("{}.mcmeta", id.path())).ok()
}

fn base_of_meta(id: &Identifier) -> Option<Identifier> {
    let base = id.path().strip_suffix(".mcmeta")?;
    Identifier::from_namespace_and_path(id.namespace(), base).ok()
}

/// A pack's part in one namespace (`FallbackResourceManager.PackEntry`).
struct Entry<'a> {
    index: usize,
    has_files: bool,
    filter: Option<&'a Filter>,
}

impl Entry<'_> {
    fn hides(&self, id: &Identifier) -> bool {
        self.filter.is_some_and(|f| f.is_path_filtered(id.path()))
    }
}

pub struct ResourceStack {
    packs: Vec<Pack>,
    namespaces: Vec<BTreeSet<String>>,
}

impl ResourceStack {
    /// A stack of `packs`, the bottom one first.
    pub fn new(packs: Vec<Pack>) -> Self {
        let namespaces = packs.iter().map(Pack::namespaces).collect();
        Self { packs, namespaces }
    }

    pub fn packs(&self) -> &[Pack] {
        &self.packs
    }

    /// Every namespace some pack has files for.
    pub fn namespaces(&self) -> BTreeSet<String> {
        self.namespaces.iter().flatten().cloned().collect()
    }

    /// The packs that matter in `namespace`: those with files for it and those whose filter
    /// reaches it. Empty when no pack has files for it.
    fn entries(&self, namespace: &str) -> Vec<Entry<'_>> {
        if !self.namespaces.iter().any(|set| set.contains(namespace)) {
            return Vec::new();
        }
        let mut entries = Vec::new();
        for (index, pack) in self.packs.iter().enumerate() {
            let has_files = self.namespaces[index].contains(namespace);
            let filter = pack.filter().filter(|f| f.is_namespace_filtered(namespace));
            if has_files || filter.is_some() {
                entries.push(Entry {
                    index,
                    has_files,
                    filter,
                });
            }
        }
        entries
    }

    fn resource(&self, index: usize, id: &Identifier, meta: Option<usize>) -> Resource<'_> {
        Resource {
            pack: &self.packs[index],
            id: id.clone(),
            meta: meta.map(|meta| &self.packs[meta]),
        }
    }

    /// The pack the `.mcmeta` file of `id` is read from, looking from the top down to `from`.
    fn meta_pack(&self, entries: &[Entry<'_>], from: usize, id: &Identifier) -> Option<usize> {
        let meta = meta_id(id)?;
        for entry in entries[from..].iter().rev() {
            if entry.has_files && self.packs[entry.index].contains(&meta) {
                return Some(entry.index);
            }
            if entry.hides(&meta) {
                break;
            }
        }
        None
    }

    /// The topmost version of the file (`getResource`).
    pub fn open(&self, id: &Identifier) -> Option<Resource<'_>> {
        let entries = self.entries(id.namespace());
        for (position, entry) in entries.iter().enumerate().rev() {
            if entry.has_files && self.packs[entry.index].contains(id) {
                let meta = self.meta_pack(&entries, position, id);
                return Some(self.resource(entry.index, id, meta));
            }
            if entry.hides(id) {
                log::warn!(
                    "resource {} was filtered by pack {}",
                    id.as_string(),
                    entry.index
                );
                return None;
            }
        }
        None
    }

    /// Every version of the file, the lowest pack first (`getResourceStack`).
    pub fn stack(&self, id: &Identifier) -> Vec<Resource<'_>> {
        let entries = self.entries(id.namespace());
        let meta = meta_id(id);
        let mut found = Vec::new();
        let mut meta_hidden = false;
        for entry in entries.iter().rev() {
            let pack = &self.packs[entry.index];
            if entry.has_files && pack.contains(id) {
                let has_meta = !meta_hidden && meta.as_ref().is_some_and(|m| pack.contains(m));
                found.push(self.resource(entry.index, id, has_meta.then_some(entry.index)));
            }
            if entry.hides(id) {
                break;
            }
            meta_hidden |= meta.as_ref().is_some_and(|m| entry.hides(m));
        }
        found.reverse();
        found
    }

    /// The files in `directory` (no trailing slash) of every namespace that `keep` accepts,
    /// topmost versions only (`listResources`).
    pub fn list(
        &self,
        directory: &str,
        keep: impl Fn(&Identifier) -> bool,
    ) -> BTreeMap<Identifier, Resource<'_>> {
        let mut result = BTreeMap::new();
        for namespace in self.namespaces() {
            result.extend(self.list_namespace(&namespace, directory, &keep));
        }
        result
    }

    fn list_namespace(
        &self,
        namespace: &str,
        directory: &str,
        keep: &impl Fn(&Identifier) -> bool,
    ) -> BTreeMap<Identifier, Resource<'_>> {
        let entries = self.entries(namespace);
        let mut files = BTreeMap::new();
        let mut metas = BTreeMap::new();
        for (position, entry) in entries.iter().enumerate() {
            files.retain(|id, _| !entry.hides(id));
            metas.retain(|id, _| !entry.hides(id));
            if !entry.has_files {
                continue;
            }
            for id in self.packs[entry.index].list(namespace, directory) {
                match base_of_meta(&id) {
                    Some(base) if keep(&base) => {
                        metas.insert(id, (position, entry.index));
                    }
                    Some(_) => {}
                    None if keep(&id) => {
                        files.insert(id, (position, entry.index));
                    }
                    None => {}
                }
            }
        }
        let mut found = BTreeMap::new();
        for (id, (position, index)) in files {
            let meta = meta_id(&id)
                .and_then(|meta| metas.get(&meta).copied())
                .filter(|(meta_position, _)| *meta_position >= position)
                .map(|(_, meta_index)| meta_index);
            found.insert(id.clone(), self.resource(index, &id, meta));
        }
        found
    }
}
