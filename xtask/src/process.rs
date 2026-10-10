//! Running external tools with uniform error reporting.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root: the parent of this crate's manifest directory.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
        .to_path_buf()
}

/// Runs a command in the repository root with inherited output.
pub fn run(program: &str, args: &[&str]) -> Result<(), String> {
    run_with(program, args, &[])
}

/// Like [`run`], with extra environment variables.
pub fn run_with(program: &str, args: &[&str], env: &[(&str, String)]) -> Result<(), String> {
    println!("$ {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .envs(env.iter().map(|(key, value)| (*key, value)))
        .current_dir(root())
        .status()
        .map_err(|error| format!("cannot start `{program}`: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program} {}` failed ({status})", args.join(" ")))
    }
}

/// Runs a command in the repository root and returns its standard output.
pub fn output(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root())
        .output()
        .map_err(|error| format!("cannot start `{program}`: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "`{program} {}` failed: {}",
            args.join(" "),
            stderr.trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("`{program}` printed non-UTF-8 output"))
}

/// Whether an optional tool is on `PATH`.
pub fn available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
