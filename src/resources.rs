//! Read-only asset-index path mapping used by the client resource layer.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::utf8;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    namespace: String,
    id: String,
    version: String,
}

const KNOWN_PACK_STRING_MAX_LENGTH: usize = 32_767;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownPackCodecError {
    Read(utf8::ReadError),
    Write(utf8::WriteError),
    InvalidUtf16,
}

impl KnownPack {
    pub const VANILLA_NAMESPACE: &'static str = "minecraft";
    pub const VANILLA_VERSION: &'static str = "26.2";

    pub fn new(namespace: String, id: String, version: String) -> Self {
        Self {
            namespace,
            id,
            version,
        }
    }

    pub fn vanilla(id: String) -> Self {
        Self::new(
            Self::VANILLA_NAMESPACE.to_owned(),
            id,
            Self::VANILLA_VERSION.to_owned(),
        )
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn is_vanilla(&self) -> bool {
        self.namespace == Self::VANILLA_NAMESPACE
    }

    /// Decode the three sequential STRING_UTF8 fields used by Java's stream codec.
    pub fn decode_stream(input: &mut &[u8]) -> Result<Self, KnownPackCodecError> {
        let namespace = decode_known_pack_string(input)?;
        let id = decode_known_pack_string(input)?;
        let version = decode_known_pack_string(input)?;
        Ok(Self::new(namespace, id, version))
    }

    /// Encode the three sequential STRING_UTF8 fields used by Java's stream codec.
    pub fn encode_stream(&self, output: &mut Vec<u8>) -> Result<(), KnownPackCodecError> {
        encode_known_pack_string(output, &self.namespace)?;
        encode_known_pack_string(output, &self.id)?;
        encode_known_pack_string(output, &self.version)?;
        Ok(())
    }
}

fn decode_known_pack_string(input: &mut &[u8]) -> Result<String, KnownPackCodecError> {
    let units =
        utf8::read_utf8(input, KNOWN_PACK_STRING_MAX_LENGTH).map_err(KnownPackCodecError::Read)?;
    String::from_utf16(&units).map_err(|_| KnownPackCodecError::InvalidUtf16)
}

fn encode_known_pack_string(output: &mut Vec<u8>, value: &str) -> Result<(), KnownPackCodecError> {
    let units: Vec<u16> = value.encode_utf16().collect();
    utf8::write_utf8(output, &units, KNOWN_PACK_STRING_MAX_LENGTH)
        .map_err(KnownPackCodecError::Write)
}

impl fmt::Display for KnownPack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}:{}", self.namespace, self.id, self.version)
    }
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

/// Ordered roots used by Java's vanilla pack resource builder.
///
/// Roots are pushed in low-to-high order and searched in reverse order. The
/// namespace set is supplied by the caller instead of being inferred from
/// files on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaPackResources {
    roots: ResourcePackStack,
    namespaces: BTreeSet<String>,
}

impl VanillaPackResources {
    pub fn new<I>(namespaces: I, roots: Vec<ResourcePack>) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self {
            roots: ResourcePackStack::new(roots),
            namespaces: namespaces.into_iter().collect(),
        }
    }

    pub fn resource(&self, location: &str) -> Option<(&str, &[u8])> {
        let namespace = location.split_once(':')?.0;
        if !self.namespaces.contains(namespace) {
            return None;
        }
        self.roots.resource(location)
    }

    pub fn namespaces(&self) -> Vec<&str> {
        self.namespaces.iter().map(String::as_str).collect()
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
