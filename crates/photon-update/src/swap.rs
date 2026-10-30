//! Applying a staged release before the game opens: the archive is extracted next to the install
//! and swapped in by renaming folders, then the new binary is started in place of the old one.

use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;

use crate::{Error, is_newer, staged};

/// A swap that happened.
#[derive(Debug, PartialEq, Eq)]
pub struct Applied {
    pub version: String,
    /// The new binary, in the install folder.
    pub binary: PathBuf,
}

fn io(error: std::io::Error) -> Error {
    Error::Io(error.to_string())
}

/// If `pending` holds a release that passes every check again now (signature, newer than
/// `current`, archive hash, contents) the install folder `install` becomes that release and the
/// old one is kept as `<install>.old` until the next swap. `binary` is the executable's file name.
/// Any failure leaves `install` as it was; the pending release stays for the next start unless it
/// is not trusted any more.
pub fn apply(
    install: &Path,
    pending: &Path,
    key: Option<&[u8; 32]>,
    (current, binary): (&str, &str),
) -> Result<Option<Applied>, Error> {
    if !pending.exists() {
        return Ok(None);
    }
    let release = staged(pending, key)?;
    if !is_newer(current, &release.release.version) {
        let _ = fs::remove_dir_all(pending);
        return Ok(None);
    }
    let sibling = |suffix: &str| {
        let mut name = install.file_name().unwrap_or_default().to_os_string();
        name.push(suffix);
        install.with_file_name(name)
    };
    let (new, old) = (sibling(".new"), sibling(".old"));
    let _ = fs::remove_dir_all(&new);
    let archive = fs::read(&release.archive).map_err(io)?;
    if let Err(error) = extract(&archive, &new, binary) {
        let _ = fs::remove_dir_all(&new);
        return Err(error);
    }
    let _ = fs::remove_dir_all(&old);
    if fs::rename(install, &old).is_ok() {
        if let Err(error) = fs::rename(&new, install) {
            let _ = fs::rename(&old, install);
            let _ = fs::remove_dir_all(&new);
            return Err(io(error));
        }
    } else {
        // A folder with a running program in it cannot always be renamed (Windows); the program's
        // own file can, so each file is moved aside and the new one put in its place.
        swap_files(&new, install).map_err(io)?;
        let _ = fs::remove_dir_all(&new);
    }
    let _ = fs::remove_dir_all(pending);
    Ok(Some(Applied {
        version: release.release.version,
        binary: install.join(binary),
    }))
}

/// Every file of `new` replaces the same path under `install`; the file it replaces is renamed to
/// `<name>.old` first (a running executable can be renamed, not overwritten).
pub(crate) fn swap_files(new: &Path, install: &Path) -> std::io::Result<()> {
    fs::create_dir_all(install)?;
    for entry in fs::read_dir(new)? {
        let entry = entry?;
        let target = install.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            swap_files(&entry.path(), &target)?;
            continue;
        }
        if target.exists() {
            let mut aside = target.clone().into_os_string();
            aside.push(".old");
            let _ = fs::remove_file(&aside);
            fs::rename(&target, &aside)?;
        }
        fs::rename(entry.path(), &target)?;
    }
    Ok(())
}

/// The regular files of a gzip'd ustar `archive` into `dest`, without their first path component
/// (the archive's own folder). Anything but plain files and folders, any path that leaves `dest`,
/// and an archive without `binary` is refused.
fn extract(archive: &[u8], dest: &Path, binary: &str) -> Result<(), Error> {
    let mut tar = Vec::new();
    GzDecoder::new(archive)
        .take(1 << 30)
        .read_to_end(&mut tar)
        .map_err(io)?;
    fs::create_dir_all(dest).map_err(io)?;
    let mut at = 0;
    while at + 512 <= tar.len() && tar[at..at + 512].iter().any(|byte| *byte != 0) {
        let header = &tar[at..at + 512];
        let size = octal(&header[124..136]).ok_or(Error::BadName)? as usize;
        let body = at + 512;
        let end = body.checked_add(size).filter(|end| *end <= tar.len());
        let end = end.ok_or(Error::BadName)?;
        match header[156] {
            b'0' | 0 => {
                let target = target(header, dest)?;
                if let Some(target) = target {
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent).map_err(io)?;
                    }
                    fs::write(&target, &tar[body..end]).map_err(io)?;
                    let executable = octal(&header[100..108]).unwrap_or(0) & 0o111 != 0;
                    set_executable(&target, executable);
                }
            }
            // Folders are created with their files; extended headers carry nothing we use.
            b'5' | b'x' | b'g' => {}
            _ => return Err(Error::BadName),
        }
        at = body + size.div_ceil(512) * 512;
    }
    if dest.join(binary).is_file() {
        Ok(())
    } else {
        Err(Error::BadManifest("the archive has no binary"))
    }
}

/// Where the entry goes under `dest`, or `None` for the archive's own folder entry.
fn target(header: &[u8], dest: &Path) -> Result<Option<PathBuf>, Error> {
    let text = |field: &[u8]| {
        let end = field
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(field.len());
        String::from_utf8_lossy(&field[..end]).into_owned()
    };
    let (name, prefix) = (text(&header[0..100]), text(&header[345..500]));
    let full = if prefix.is_empty() {
        name
    } else {
        format!("{prefix}/{name}")
    };
    let mut parts = Path::new(&full).components();
    // The first component is the archive's folder.
    match parts.next() {
        Some(Component::Normal(_)) => {}
        _ => return Err(Error::BadName),
    }
    let mut target = dest.to_path_buf();
    for part in parts {
        match part {
            Component::Normal(part) => target.push(part),
            Component::CurDir => {}
            _ => return Err(Error::BadName),
        }
    }
    Ok((target != dest).then_some(target))
}

fn octal(field: &[u8]) -> Option<u64> {
    let text: String = field
        .iter()
        .take_while(|byte| **byte != 0)
        .map(|byte| *byte as char)
        .collect();
    u64::from_str_radix(text.trim(), 8).ok()
}

#[cfg(unix)]
fn set_executable(path: &Path, executable: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if executable { 0o755 } else { 0o644 };
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_executable(_path: &Path, _executable: bool) {}
