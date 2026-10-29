//! The hosts the game's content comes from, and the one rule that decides which URL may be
//! fetched: it must sit under one of them. Metadata names URLs of its own; they are untrusted
//! until they pass [`Endpoints::allow`].

use std::time::Duration;

use crate::error::ContentError;
use crate::hash::Sha1Hex;

const META: &str = "https://piston-meta.mojang.com";
const DATA: &str = "https://piston-data.mojang.com";
const RESOURCES: &str = "https://resources.download.minecraft.net";
const MANIFEST_PATH: &str = "/mc/game/version_manifest_v2.json";

/// Where requests go. [`Endpoints::official`] is the only one the shipped game builds.
#[derive(Debug, Clone)]
pub struct Endpoints {
    meta: String,
    data: String,
    resources: String,
    /// Plain HTTP is allowed (a stand-in server on this machine in tests).
    insecure: bool,
    pub(crate) retry_delay: Duration,
}

impl Endpoints {
    /// Mojang's metadata, download and asset servers.
    pub fn official() -> Self {
        Self {
            meta: META.to_owned(),
            data: DATA.to_owned(),
            resources: RESOURCES.to_owned(),
            insecure: false,
            retry_delay: Duration::from_secs(1),
        }
    }

    /// One server on this machine standing in for all three, for tests.
    #[cfg(test)]
    pub(crate) fn stand_in(base: &str) -> Self {
        Self {
            meta: format!("{base}/meta"),
            data: format!("{base}/data"),
            resources: format!("{base}/resources"),
            insecure: true,
            retry_delay: Duration::ZERO,
        }
    }

    pub(crate) fn manifest_url(&self) -> String {
        format!("{}{MANIFEST_PATH}", self.meta)
    }

    /// Where an asset object is served: built from its digest alone.
    pub(crate) fn object_url(&self, digest: &Sha1Hex) -> String {
        format!(
            "{}/{}/{}",
            self.resources,
            digest.fan_out(),
            digest.as_str()
        )
    }

    pub(crate) fn is_secure(&self) -> bool {
        !self.insecure
    }

    /// `url` as it is requested through these endpoints, if it is under the metadata or the
    /// download host (the whole host: `https://piston-meta.mojang.com.example/` is not).
    pub(crate) fn allow(&self, url: &str) -> Result<String, ContentError> {
        for (official, mine) in [(META, &self.meta), (DATA, &self.data)] {
            if let Some(rest) = url
                .strip_prefix(official)
                .filter(|rest| rest.starts_with('/'))
            {
                return Ok(format!("{mine}{rest}"));
            }
        }
        Err(ContentError::ForeignHost(url.chars().take(96).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_two_metadata_hosts_are_followed() {
        let official = Endpoints::official();
        let ok = "https://piston-data.mojang.com/v1/objects/ab/client.jar";
        assert_eq!(official.allow(ok).unwrap(), ok);
        for bad in [
            "http://piston-data.mojang.com/v1/x",
            "https://piston-data.mojang.com.evil.example/v1/x",
            "https://piston-data.mojang.com@evil.example/v1/x",
            "https://evil.example/https://piston-meta.mojang.com/x",
            "https://resources.download.minecraft.net/ab/abcd",
            "file:///etc/passwd",
            "",
        ] {
            assert!(
                matches!(official.allow(bad), Err(ContentError::ForeignHost(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_stand_in_takes_the_place_of_the_official_hosts() {
        let local = Endpoints::stand_in("http://127.0.0.1:9");
        assert_eq!(
            local
                .allow("https://piston-meta.mojang.com/v1/a.json")
                .unwrap(),
            "http://127.0.0.1:9/meta/v1/a.json"
        );
        let digest = Sha1Hex::of_bytes(b"x");
        assert!(
            local
                .object_url(&digest)
                .starts_with("http://127.0.0.1:9/resources/")
        );
    }
}
