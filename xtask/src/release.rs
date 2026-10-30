//! `cargo xtask release-key` and `cargo xtask release-sign`: the owner's one-time key and the
//! release workflow's signing step. A private key is never printed, never put in an argument
//! and never written anywhere but the file the owner names; the workflow reads it from an
//! environment variable and keeps it in memory.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};

/// The environment variable the signing step reads the private key from (hex of a PKCS#8
/// document). It is the secret of the `release` environment.
const SIGNING_KEY_VARIABLE: &str = "RELEASE_SIGNING_KEY";

/// The key being retired during a rotation: when set, every manifest is signed with it as well.
const PREVIOUS_KEY_VARIABLE: &str = "RELEASE_SIGNING_KEY_PREVIOUS";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).ok())
        .collect()
}

/// `cargo xtask release-key <private-key-file>`
pub fn key(args: &[String]) -> Result<(), String> {
    let [path] = args else {
        return Err("usage: cargo xtask release-key <private-key-file>".to_owned());
    };
    let document = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .map_err(|_| "the system has no random source".to_owned())?;
    let pair = Ed25519KeyPair::from_pkcs8(document.as_ref())
        .map_err(|_| "the generated key is not usable".to_owned())?;
    write_private(Path::new(path), &hex(document.as_ref()))?;
    println!(
        "private key written to {path} (not shown; keep it secret and make an offline backup)"
    );
    println!("public key: {}", hex(pair.public_key().as_ref()));
    println!("add the public key to RELEASE_KEYS in crates/photon-update/src/lib.rs");
    Ok(())
}

/// The key file: new, readable by its owner only.
fn write_private(path: &Path, text: &str) -> Result<(), String> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| {
        format!(
            "{}: {error} (an existing file is never overwritten)",
            path.display()
        )
    })?;
    file.write_all(text.as_bytes())
        .map_err(|error| error.to_string())
}

/// `cargo xtask release-sign --archives <dir> --tag <vX.Y.Z> [--notes <line>]`
pub fn sign(args: &[String]) -> Result<(), String> {
    let (dir, tag, notes) = sign_arguments(args)?;
    let secret = std::env::var(SIGNING_KEY_VARIABLE).unwrap_or_default();
    let previous = std::env::var(PREVIOUS_KEY_VARIABLE).unwrap_or_default();
    let secrets: Vec<&str> = [secret.as_str(), previous.as_str()]
        .into_iter()
        .filter(|secret| !secret.trim().is_empty())
        .collect();
    sign_in(photon_update::RELEASE_KEYS, &secrets, (&dir, &tag, &notes))
}

/// Signs the manifests of the archives in `dir` with `secret`, if `keys` (what clients trust)
/// holds its public key and `tag` is this source's version.
fn sign_in(
    keys: &[[u8; 32]],
    secrets: &[&str],
    (dir, tag, notes): (&Path, &str, &str),
) -> Result<(), String> {
    preflight(keys, tag)?;
    let version = env!("CARGO_PKG_VERSION");
    let pairs = if secrets.is_empty() {
        vec![signing_pair(keys, "")?]
    } else {
        secrets
            .iter()
            .map(|secret| signing_pair(keys, secret))
            .collect::<Result<Vec<_>, _>>()?
    };
    for (triple, path) in archives_in(dir, version)? {
        let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("an archive name is not text")?;
        let manifest = manifest_text(version, name, &bytes, notes);
        let signature: String = pairs
            .iter()
            .map(|pair| format!("{}\n", hex(pair.sign(manifest.as_bytes()).as_ref())))
            .collect();
        // What a client would do with it, before anything is published.
        photon_update::verify_manifest(keys, manifest.as_bytes(), &signature)
            .map_err(|error| format!("the signed manifest does not verify: {error:?}"))?;
        fs::write(dir.join(format!("photon-{triple}.manifest")), &manifest)
            .map_err(|error| error.to_string())?;
        fs::write(
            dir.join(format!("photon-{triple}.manifest.sig")),
            &signature,
        )
        .map_err(|error| error.to_string())?;
        println!("signed photon-{triple}.manifest ({} bytes)", bytes.len());
    }
    Ok(())
}

/// `cargo xtask release-check <vX.Y.Z>`: what can be known without the secret, so a release that
/// cannot be signed fails before anyone is asked to approve it.
pub fn check(args: &[String]) -> Result<(), String> {
    let [tag] = args else {
        return Err("usage: cargo xtask release-check <vX.Y.Z>".to_owned());
    };
    preflight(photon_update::RELEASE_KEYS, tag)
}

fn preflight(keys: &[[u8; 32]], tag: &str) -> Result<(), String> {
    let version = env!("CARGO_PKG_VERSION");
    if tag != format!("v{version}") {
        return Err(format!(
            "the tag {tag} is not v{version}, the version of this source"
        ));
    }
    if keys.is_empty() {
        return Err("RELEASE_KEYS in crates/photon-update/src/lib.rs is empty: no client would trust a release".to_owned());
    }
    Ok(())
}

fn sign_arguments(args: &[String]) -> Result<(PathBuf, String, String), String> {
    let usage = "usage: cargo xtask release-sign --archives <dir> --tag <vX.Y.Z> [--notes <line>]";
    let (mut dir, mut tag, mut notes) = (None, None, String::new());
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let value = rest.next().ok_or(usage)?;
        match arg.as_str() {
            "--archives" => dir = Some(PathBuf::from(value)),
            "--tag" => tag = Some(value.clone()),
            "--notes" => notes = value.replace(['\n', '\r'], " "),
            _ => return Err(usage.to_owned()),
        }
    }
    Ok((dir.ok_or(usage)?, tag.ok_or(usage)?, notes))
}

/// The key pair of the secret, which must be one of the keys the clients trust.
fn signing_pair(keys: &[[u8; 32]], secret: &str) -> Result<Ed25519KeyPair, String> {
    if secret.trim().is_empty() {
        return Err(format!(
            "{SIGNING_KEY_VARIABLE} is not set: the release environment has no signing secret"
        ));
    }
    let document = unhex(secret.trim()).ok_or(format!("{SIGNING_KEY_VARIABLE} is not hex"))?;
    let pair = Ed25519KeyPair::from_pkcs8(&document)
        .map_err(|_| format!("{SIGNING_KEY_VARIABLE} is not a signing key"))?;
    let public: [u8; 32] = pair
        .public_key()
        .as_ref()
        .try_into()
        .map_err(|_| "the public key has an unexpected length".to_owned())?;
    if !keys.contains(&public) {
        return Err(
            "the secret's public key is not in RELEASE_KEYS: no client would trust the signature"
                .to_owned(),
        );
    }
    Ok(pair)
}

/// The archives `xtask package` made for this version, with their target triples.
fn archives_in(dir: &Path, version: &str) -> Result<Vec<(String, PathBuf)>, String> {
    let prefix = format!("photon-{version}-");
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let triple = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix(&prefix))
            .and_then(|name| name.strip_suffix(".tar.gz"));
        if let Some(triple) = triple {
            found.push((triple.to_owned(), path.clone()));
        }
    }
    found.sort();
    if found.is_empty() {
        return Err(format!(
            "no photon-{version}-<target>.tar.gz in {}",
            dir.display()
        ));
    }
    Ok(found)
}

/// The manifest of one archive (the format `photon-update` reads).
fn manifest_text(version: &str, archive: &str, bytes: &[u8], notes: &str) -> String {
    let protocols: Vec<String> = photon_core::version::Version::ALL
        .iter()
        .map(|version| version.protocol().to_string())
        .collect();
    format!(
        "version={version}\narchive={archive}\nsha256={}\nsize={}\nnotes={notes}\nprotocols={}\n",
        hex(&Sha256::digest(bytes)),
        bytes.len(),
        protocols.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (String, [u8; 32]) {
        let document = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(document.as_ref()).unwrap();
        (
            hex(document.as_ref()),
            pair.public_key().as_ref().try_into().unwrap(),
        )
    }

    fn archives(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("photon-release-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let version = env!("CARGO_PKG_VERSION");
        fs::write(
            dir.join(format!("photon-{version}-x86_64-unknown-linux-gnu.tar.gz")),
            b"archive",
        )
        .unwrap();
        dir
    }

    fn tag() -> String {
        format!("v{}", env!("CARGO_PKG_VERSION"))
    }

    #[test]
    fn a_signed_release_verifies_under_the_listed_key() {
        let (secret, public) = pair();
        let dir = archives("ok");
        sign_in(&[public], &[&secret], (&dir, &tag(), "notes")).unwrap();
        let manifest = fs::read(dir.join("photon-x86_64-unknown-linux-gnu.manifest")).unwrap();
        let signature =
            fs::read_to_string(dir.join("photon-x86_64-unknown-linux-gnu.manifest.sig")).unwrap();
        let release = photon_update::verify_manifest(&[public], &manifest, &signature).unwrap();
        assert_eq!((release.size, release.notes.as_str()), (7, "notes"));
        assert_eq!(release.protocols, [775, 776]);
    }

    #[test]
    fn a_rotation_release_carries_both_signatures() {
        let ((new_secret, new), (old_secret, old)) = (pair(), pair());
        let dir = archives("rotate");
        sign_in(&[old, new], &[&new_secret, &old_secret], (&dir, &tag(), "")).unwrap();
        let manifest = fs::read(dir.join("photon-x86_64-unknown-linux-gnu.manifest")).unwrap();
        let signatures =
            fs::read_to_string(dir.join("photon-x86_64-unknown-linux-gnu.manifest.sig")).unwrap();
        assert_eq!(signatures.lines().count(), 2);
        // A client that still trusts only the old key, or only the new one, accepts it.
        assert!(photon_update::verify_manifest(&[old], &manifest, &signatures).is_ok());
        assert!(photon_update::verify_manifest(&[new], &manifest, &signatures).is_ok());
    }

    #[test]
    fn signing_fails_loudly_without_a_trusted_key_a_secret_or_the_right_tag() {
        let (secret, public) = pair();
        let (_, other) = pair();
        let dir = archives("refuse");
        let refused = |keys: &[[u8; 32]], secret: &str, tag: &str| {
            sign_in(keys, &[secret], (&dir, tag, "")).unwrap_err()
        };
        assert!(refused(&[], &secret, &tag()).contains("RELEASE_KEYS"));
        assert!(refused(&[public], "", &tag()).contains("not set"));
        assert!(refused(&[other], &secret, &tag()).contains("not in RELEASE_KEYS"));
        assert!(refused(&[public], &secret, "v0.0.0").contains("not v"));
        assert!(
            !dir.join("photon-x86_64-unknown-linux-gnu.manifest")
                .exists()
        );
    }

    #[test]
    fn a_key_file_is_new_and_never_overwritten() {
        let path = std::env::temp_dir().join(format!("photon-key-{}", std::process::id()));
        let _ = fs::remove_file(&path);
        write_private(&path, "secret").unwrap();
        assert!(write_private(&path, "other").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "secret");
        let _ = fs::remove_file(&path);
    }
}
