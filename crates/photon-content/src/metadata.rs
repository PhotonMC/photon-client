//! The three documents the download is read from: the version manifest, a version's metadata
//! and an asset index. All of them are untrusted text: every digest is checked before it is
//! used, every URL before it is requested, every name before it is a file name.

use photon_core::json::{self, JsonObject, JsonValue};

use crate::cache::Id;
use crate::error::ContentError;
use crate::hash::Sha1Hex;

/// A file the metadata promises: where it is, its digest and its size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub url: String,
    pub sha1: Sha1Hex,
    pub size: u64,
}

/// What a version's metadata gives the download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub client: Artifact,
    pub asset_index_id: Id,
    pub asset_index: Artifact,
}

/// One asset object of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetObject {
    pub name: String,
    pub sha1: Sha1Hex,
    pub size: u64,
}

fn bad(what: &str) -> ContentError {
    ContentError::Metadata(what.to_owned())
}

fn object_of<'a>(parent: &'a JsonObject, key: &str) -> Result<&'a JsonObject, ContentError> {
    parent.object(key).ok_or_else(|| bad(key))
}

fn size_of(object: &JsonObject) -> Result<u64, ContentError> {
    object
        .i32("size")
        .and_then(|size| u64::try_from(size).ok())
        .ok_or_else(|| bad("size"))
}

fn artifact(object: &JsonObject) -> Result<Artifact, ContentError> {
    Ok(Artifact {
        url: object.text("url").ok_or_else(|| bad("url"))?,
        sha1: Sha1Hex::parse(&object.text("sha1").ok_or_else(|| bad("sha1"))?)?,
        size: size_of(object)?,
    })
}

/// The URL and digest the version manifest gives for `id`.
pub fn manifest_entry(text: &str, id: &Id) -> Result<(String, Sha1Hex), ContentError> {
    let manifest = json::parse_object(text).ok_or_else(|| bad("version manifest"))?;
    let versions = manifest.array("versions").ok_or_else(|| bad("versions"))?;
    for entry in versions {
        let JsonValue::Object(entry) = entry else {
            continue;
        };
        if entry.text("id").as_deref() == Some(id.as_str()) {
            let url = entry.text("url").ok_or_else(|| bad("url"))?;
            let sha1 = Sha1Hex::parse(&entry.text("sha1").ok_or_else(|| bad("sha1"))?)?;
            return Ok((url, sha1));
        }
    }
    Err(bad("the pinned version is not in the manifest"))
}

/// The client jar and the asset index a version's metadata names.
pub fn version_info(text: &str, id: &Id) -> Result<VersionInfo, ContentError> {
    let version = json::parse_object(text).ok_or_else(|| bad("version metadata"))?;
    if version.text("id").as_deref() != Some(id.as_str()) {
        return Err(bad("version id"));
    }
    let downloads = object_of(&version, "downloads")?;
    let index = object_of(&version, "assetIndex")?;
    Ok(VersionInfo {
        client: artifact(object_of(downloads, "client")?)?,
        asset_index_id: Id::parse(&index.text("id").ok_or_else(|| bad("assetIndex id"))?)?,
        asset_index: artifact(index)?,
    })
}

/// The objects of an asset index, in the order of the index.
pub fn asset_objects(text: &str) -> Result<Vec<AssetObject>, ContentError> {
    let index = json::parse_object(text).ok_or_else(|| bad("asset index"))?;
    let objects = object_of(&index, "objects")?;
    let mut found = Vec::with_capacity(objects.len());
    for (name, entry) in objects.entries() {
        let JsonValue::Object(entry) = entry else {
            return Err(bad("asset entry"));
        };
        found.push(AssetObject {
            name: String::from_utf16(name).map_err(|_| bad("asset name"))?,
            sha1: Sha1Hex::parse(&entry.text("hash").ok_or_else(|| bad("hash"))?)?,
            size: size_of(entry)?,
        });
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    const H1: &str = "1111111111111111111111111111111111111111";
    const H2: &str = "2222222222222222222222222222222222222222";

    fn id(text: &str) -> Id {
        Id::parse(text).unwrap()
    }

    #[test]
    fn the_manifest_gives_the_url_and_digest_of_the_pinned_id() {
        let text = format!(
            r#"{{"versions":[{{"id":"a","url":"u/a","sha1":"{H2}"}},{{"id":"b","url":"u/b","sha1":"{H1}"}}]}}"#
        );
        let (url, sha1) = manifest_entry(&text, &id("b")).unwrap();
        assert_eq!((url.as_str(), sha1.as_str()), ("u/b", H1));
        assert!(manifest_entry(&text, &id("c")).is_err());
    }

    #[test]
    fn a_version_names_its_jar_and_index() {
        let text = format!(
            r#"{{"id":"26.2","downloads":{{"client":{{"url":"u/jar","sha1":"{H1}","size":5}}}},"assetIndex":{{"id":"32","url":"u/i","sha1":"{H2}","size":7}}}}"#
        );
        let info = version_info(&text, &id("26.2")).unwrap();
        assert_eq!(info.client.size, 5);
        assert_eq!(info.asset_index_id.as_str(), "32");
        assert_eq!(info.asset_index.sha1.as_str(), H2);
        assert!(
            version_info(&text, &id("26.1")).is_err(),
            "another version's metadata"
        );
    }

    #[test]
    fn an_index_with_a_bad_digest_or_size_is_refused() {
        let ok = format!(r#"{{"objects":{{"a/b.ogg":{{"hash":"{H1}","size":3}}}}}}"#);
        let objects = asset_objects(&ok).unwrap();
        assert_eq!((objects[0].name.as_str(), objects[0].size), ("a/b.ogg", 3));
        for text in [
            r#"{"objects":{"a":{"hash":"../..","size":3}}}"#.to_owned(),
            format!(r#"{{"objects":{{"a":{{"hash":"{H1}","size":-1}}}}}}"#),
            format!(r#"{{"objects":{{"a":{{"hash":"{H1}"}}}}}}"#),
            "[]".to_owned(),
        ] {
            assert!(asset_objects(&text).is_err(), "{text}");
        }
    }
}
