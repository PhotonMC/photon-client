//! `SkinTextureDownloader.downloadSkin`: a texture's file from the local copy, or from the
//! url, which is then kept as the local copy.

use std::io;
use std::path::Path;
use std::time::Duration;

/// The longest a texture's download may take.
const TIMEOUT: Duration = Duration::from_secs(20);

/// The texture's file: the bytes of `local_copy` where it is a regular file, otherwise the
/// body of `url` (any status but 2xx fails), written to `local_copy` as far as that works.
pub fn fetch_texture(url: &str, local_copy: &Path) -> io::Result<Vec<u8>> {
    if local_copy.is_file() {
        return std::fs::read(local_copy);
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build()
        .into();
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| io::Error::other(format!("Failed to open {url}: {error}")))?;
    let bytes = response
        .body_mut()
        .read_to_vec()
        .map_err(|error| io::Error::other(format!("Failed to read {url}: {error}")))?;
    // A failed write only costs the next download.
    if let Some(parent) = local_copy.parent()
        && std::fs::create_dir_all(parent).is_ok()
    {
        let _ = std::fs::write(local_copy, &bytes);
    }
    Ok(bytes)
}

/// `SkinManager.TextureCache.registerTexture`'s file: the first two characters of the SHA-1 of
/// the texture's hash name the folder, the whole of it the file.
pub fn local_copy(root: &Path, texture_hash: &str) -> std::path::PathBuf {
    let name = photon_core::sha1::hash_unencoded_chars(texture_hash);
    let folder = if name.len() > 2 { &name[..2] } else { "xx" };
    root.join(folder).join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_local_copy_is_in_a_folder_of_the_first_two_digits() {
        let path = local_copy(Path::new("skins"), "abc");
        let name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(name.len(), 40);
        assert_eq!(
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            &name[..2]
        );
    }

    #[test]
    fn a_kept_copy_is_read_without_a_request() {
        let dir = std::env::temp_dir().join(format!("photon-skin-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("copy");
        std::fs::write(&file, [1, 2, 3]).unwrap();
        assert_eq!(
            fetch_texture("http://invalid.invalid/x", &file).unwrap(),
            [1, 2, 3]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
