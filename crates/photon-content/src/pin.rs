//! The game version this build downloads. The version id and the client jar's digest are
//! pinned; the version metadata and the asset index are read through the manifest and may be
//! republished by Mojang (a changed asset index under the same id), the jar may not.

/// A pinned game version.
#[derive(Debug, Clone, Copy)]
pub struct Pin {
    pub id: &'static str,
    pub client_sha1: &'static str,
}

/// The version `tests/behavior/reference.json` pins; a test keeps the two equal.
pub const PINNED: Pin = Pin {
    id: "26.2",
    client_sha1: "2dc72797acbc1b63fc16a11c4ac393605f453754",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pin_is_the_one_the_reference_names() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/behavior/reference.json");
        let Some(reference) = photon_core::json::read_object(&path) else {
            println!("missing reference: {}", path.display());
            return;
        };
        assert_eq!(reference.text("version").as_deref(), Some(PINNED.id));
        assert_eq!(
            reference.text("client_sha1").as_deref(),
            Some(PINNED.client_sha1)
        );
    }
}
