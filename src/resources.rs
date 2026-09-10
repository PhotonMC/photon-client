//! Read-only asset-index path mapping used by the client resource layer.

use std::collections::BTreeMap;

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
