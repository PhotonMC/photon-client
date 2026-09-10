//! Read-only asset-index path mapping used by the client resource layer.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    file_hash: Option<String>,
    children: BTreeMap<String, Node>,
}

impl Node {
    fn new() -> Self {
        Self {
            file_hash: None,
            children: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetIndex {
    root: Node,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetResolution<'a> {
    Missing,
    Directory,
    File { hash: &'a str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetIndexError {
    EmptyPath,
    EmptyPathSegment,
    HashTooShort,
}

impl AssetIndex {
    pub fn from_entries<I>(entries: I) -> Result<Self, AssetIndexError>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let mut index = Self { root: Node::new() };
        for (path, hash) in entries {
            index.insert(path, hash)?;
        }
        Ok(index)
    }

    fn insert(&mut self, path: String, hash: String) -> Result<(), AssetIndexError> {
        if path.is_empty() {
            return Err(AssetIndexError::EmptyPath);
        }
        if hash.len() < 2 {
            return Err(AssetIndexError::HashTooShort);
        }

        let segments: Vec<&str> = path.split('/').collect();
        if segments.iter().any(|segment| segment.is_empty()) {
            return Err(AssetIndexError::EmptyPathSegment);
        }

        let mut node = &mut self.root;
        for segment in &segments {
            node = node
                .children
                .entry((*segment).to_owned())
                .or_insert_with(Node::new);
        }
        node.file_hash = Some(hash);
        Ok(())
    }

    pub fn resolve(&self, path: &str) -> AssetResolution<'_> {
        if path.is_empty() {
            return AssetResolution::Directory;
        }

        let mut node = &self.root;
        for segment in path.split('/') {
            if segment.is_empty() {
                return AssetResolution::Missing;
            }
            let Some(next) = node.children.get(segment) else {
                return AssetResolution::Missing;
            };
            node = next;
        }

        if !node.children.is_empty() {
            AssetResolution::Directory
        } else if let Some(hash) = node.file_hash.as_deref() {
            AssetResolution::File { hash }
        } else {
            AssetResolution::Missing
        }
    }
}

pub fn object_path(hash: &str) -> String {
    format!("{}/{}", &hash[..2], hash)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackFormat {
    major: i32,
    minor: i32,
}

impl PackFormat {
    pub const fn new(major: i32, minor: i32) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> i32 {
        self.major
    }

    pub const fn minor(self) -> i32 {
        self.minor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackCompatibility {
    TooOld,
    TooNew,
    Unknown,
    Compatible,
}

impl PackCompatibility {
    pub fn for_version(
        declared_min: PackFormat,
        declared_max: PackFormat,
        game: PackFormat,
    ) -> Self {
        if declared_min.major() == i32::MAX {
            Self::Unknown
        } else if declared_max < game {
            Self::TooOld
        } else if game < declared_min {
            Self::TooNew
        } else {
            Self::Compatible
        }
    }

    pub const fn is_compatible(self) -> bool {
        matches!(self, Self::Compatible)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::TooOld => "TOO_OLD",
            Self::TooNew => "TOO_NEW",
            Self::Unknown => "UNKNOWN",
            Self::Compatible => "COMPATIBLE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePack {
    id: String,
    resources: BTreeMap<String, Vec<u8>>,
}

impl ResourcePack {
    pub fn new(id: String) -> Self {
        Self {
            id,
            resources: BTreeMap::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn insert(&mut self, location: String, bytes: Vec<u8>) {
        self.resources.insert(location, bytes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePackStack {
    packs: Vec<ResourcePack>,
}

impl ResourcePackStack {
    pub fn new(packs: Vec<ResourcePack>) -> Self {
        Self { packs }
    }

    pub fn resource(&self, location: &str) -> Option<(&str, &[u8])> {
        self.packs.iter().rev().find_map(|pack| {
            pack.resources
                .get(location)
                .map(|bytes| (pack.id.as_str(), bytes.as_slice()))
        })
    }

    pub fn resource_stack(&self, location: &str) -> Vec<(&str, &[u8])> {
        self.packs
            .iter()
            .filter_map(|pack| {
                pack.resources
                    .get(location)
                    .map(|bytes| (pack.id.as_str(), bytes.as_slice()))
            })
            .collect()
    }

    pub fn pack_ids(&self) -> Vec<&str> {
        self.packs.iter().map(ResourcePack::id).collect()
    }

    pub fn namespaces(&self) -> Vec<&str> {
        let mut namespaces = BTreeSet::new();
        for pack in &self.packs {
            for location in pack.resources.keys() {
                if let Some((namespace, _)) = location.split_once(':') {
                    namespaces.insert(namespace);
                }
            }
        }
        namespaces.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableResourceManager {
    resources: ResourcePackStack,
}

impl Default for ReloadableResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReloadableResourceManager {
    pub fn new() -> Self {
        Self {
            resources: ResourcePackStack::new(Vec::new()),
        }
    }

    pub fn reload(&mut self, packs: Vec<ResourcePack>) {
        self.resources = ResourcePackStack::new(packs);
    }

    pub fn resources(&self) -> &ResourcePackStack {
        &self.resources
    }
}
