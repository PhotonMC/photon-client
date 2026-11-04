//! `photon register-links`: tells the desktop that `photon://` links open this program, so the
//! website's Play and sign-in buttons reach it. Linux writes a desktop entry for the current user;
//! Windows names the registry values it needs. Nothing here needs a session or a backend.

use std::path::{Path, PathBuf};

/// The desktop entry that makes `exe` the handler of `photon://` links.
pub fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Photon\nNoDisplay=true\nTerminal=false\n\
         MimeType=x-scheme-handler/photon;\nExec={} open-link %u\n",
        exec_word(&exe.to_string_lossy())
    )
}

/// One word of an `Exec` line. The format has two layers: the quoting of the command line, where
/// `"`, `` ` ``, `$` and `\` need a backslash and `%` is doubled, then the string escapes of the
/// file, where every backslash is doubled again.
fn exec_word(word: &str) -> String {
    let mut quoted = String::from("\"");
    for c in word.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                quoted.push('\\');
                quoted.push(c);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted.replace('\\', "\\\\")
}

/// The command a Windows registry key `photon\shell\open\command` holds for `exe`.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn windows_command(exe: &Path) -> String {
    format!("\"{}\" open-link \"%1\"", exe.display())
}

/// Where the current user's desktop entries live.
fn applications_dir() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|home| Path::new(&home).join(".local").join("share"))
        })?;
    Some(data.join("applications"))
}

/// Registers the running program as the handler of `photon://` links.
pub fn run_register() -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    if cfg!(windows) {
        println!("Windows needs, under HKCU\\Software\\Classes\\photon:");
        println!("  (Default) = \"URL:Photon\", URL Protocol = \"\"");
        println!("  shell\\open\\command = {}", windows_command(&exe));
        return Err("registering on Windows is not done by this command yet".into());
    }
    if !cfg!(target_os = "linux") {
        return Err("registering links is only done on Linux so far".into());
    }
    let dir = applications_dir().ok_or("no home directory for the desktop entry")?;
    std::fs::create_dir_all(&dir)?;
    let file = dir.join("photon.desktop");
    std::fs::write(&file, desktop_entry(&exe))?;
    // Browsers list the apps that handle a scheme from this cache (`gio mime` reads it); without
    // it Chrome says "No apps installed". A missing tool only leaves the cache as it was.
    let _ = std::process::Command::new("update-desktop-database")
        .arg(&dir)
        .status();
    let status = std::process::Command::new("xdg-mime")
        .args(["default", "photon.desktop", "x-scheme-handler/photon"])
        .status()?;
    if !status.success() {
        return Err(format!("xdg-mime failed: {status}").into());
    }
    println!("registered {} for photon:// links", file.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entry_names_the_scheme_and_the_program() {
        let text = desktop_entry(Path::new("/opt/photon/photon"));
        assert!(text.contains("MimeType=x-scheme-handler/photon;\n"));
        assert!(text.contains("Exec=\"/opt/photon/photon\" open-link %u\n"));
        assert!(text.contains("NoDisplay=true"));
    }

    #[test]
    fn a_path_with_a_space_stays_one_word() {
        assert_eq!(exec_word("/home/a b/photon"), "\"/home/a b/photon\"");
    }

    #[test]
    fn reserved_characters_are_escaped_in_both_layers() {
        assert_eq!(exec_word("/a$b"), "\"/a\\\\$b\"");
        assert_eq!(exec_word("/a\\b"), "\"/a\\\\\\\\b\"");
        assert_eq!(exec_word("/100%"), "\"/100%%\"");
        assert_eq!(exec_word("/a\"b"), "\"/a\\\\\"b\"");
    }

    #[test]
    fn the_windows_command_quotes_the_program_and_the_link() {
        let command = windows_command(Path::new("C:\\Program Files\\Photon\\photon.exe"));
        assert_eq!(
            command,
            "\"C:\\Program Files\\Photon\\photon.exe\" open-link \"%1\""
        );
    }
}
