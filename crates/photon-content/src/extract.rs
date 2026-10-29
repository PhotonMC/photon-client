//! The client jar's resources: everything but the classes, extracted into a folder that exists
//! only once it is complete.

use std::fs::{self, File};
use std::io;
use std::path::Path;

use crate::error::{ContentError, io_error};

/// More than this extracted is not a game jar.
const MAX_EXTRACTED: u64 = 2 << 30;

/// Extract the non-class members of `jar` into `dest`. It is built in `dest.tmp` and renamed,
/// so a folder at `dest` is complete; a leftover `dest.tmp` of an interrupted run is replaced.
pub(crate) fn extract_resources(jar: &Path, dest: &Path) -> Result<(), ContentError> {
    let mut staging = dest.as_os_str().to_owned();
    staging.push(".tmp");
    let staging = Path::new(&staging);
    if staging.exists() {
        fs::remove_dir_all(staging).map_err(|error| io_error(staging, &error))?;
    }
    fs::create_dir_all(staging).map_err(|error| io_error(staging, &error))?;
    let file = File::open(jar).map_err(|error| io_error(jar, &error))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| io_error(jar, &error))?;
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut member = archive
            .by_index(index)
            .map_err(|error| io_error(jar, &error))?;
        let name = member.name().map_or_else(
            |_| "(unreadable name)".to_owned(),
            std::borrow::Cow::into_owned,
        );
        let relative = member
            .enclosed_name()
            .ok_or_else(|| ContentError::UnsafePath(name.chars().take(96).collect()))?;
        if member.is_dir() || relative.extension().is_some_and(|ext| ext == "class") {
            continue;
        }
        total = total.saturating_add(member.size());
        if total > MAX_EXTRACTED {
            return Err(ContentError::Mismatch("the jar expands too far".to_owned()));
        }
        let target = staging.join(&relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, &error))?;
        }
        let mut out = File::create(&target).map_err(|error| io_error(&target, &error))?;
        io::copy(&mut member, &mut out).map_err(|error| io_error(&target, &error))?;
    }
    fs::rename(staging, dest).map_err(|error| io_error(dest, &error))
}
