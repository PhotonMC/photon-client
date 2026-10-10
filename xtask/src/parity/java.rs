//! The Java side: finding the JDK, compiling the oracle once per source
//! set, and running it on a batch of cases.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::digest;
use crate::process;

/// HotSpot answers `Math.sin/cos/log/pow/exp` with its own stubs on x86-64,
/// which round differently from fdlibm; with them off, `Math` is `StrictMath`.
pub const JVM_FLAGS: &[&str] = &["-XX:+UnlockDiagnosticVMOptions", "-XX:-UseLibmIntrinsic"];
/// Registry bootstrapping and block walks take minutes on a loaded machine.
const TIMEOUT: Duration = Duration::from_secs(900);

pub struct Jdk {
    pub java: PathBuf,
    pub javac: PathBuf,
    /// The `-version` output of both tools: part of every case key.
    pub versions: String,
}

impl Jdk {
    /// `JAVA_HOME` if set, else `PATH`; both tools must be the pinned major.
    pub fn find(major: u32) -> Result<Self, String> {
        let tool = |name: &str| -> PathBuf {
            match std::env::var_os("JAVA_HOME") {
                Some(home) => Path::new(&home).join("bin").join(name),
                None => PathBuf::from(name),
            }
        };
        let (java, javac) = (tool("java"), tool("javac"));
        let mut versions = String::new();
        for (path, marker) in [(&java, "version \""), (&javac, "javac ")] {
            let output = Command::new(path)
                .arg("-version")
                .output()
                .map_err(|_| format!("Java {major} tool missing: {}", path.display()))?;
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let found = text
                .split(marker)
                .nth(1)
                .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
                .and_then(|digits| digits.parse::<u32>().ok());
            if found != Some(major) {
                return Err(format!(
                    "expected Java {major}; {} says {}",
                    path.display(),
                    text.trim()
                ));
            }
            versions.push_str(text.trim());
            versions.push('\n');
        }
        Ok(Self {
            java,
            javac,
            versions,
        })
    }

    /// Compiles the sources into a class directory named by their digest;
    /// an existing directory is reused.
    pub fn compile(&self, sources: &[PathBuf], classpath: &str) -> Result<PathBuf, String> {
        let mut parts = Vec::new();
        for source in sources {
            let bytes =
                std::fs::read(source).map_err(|error| format!("{}: {error}", source.display()))?;
            parts.push(format!("{}={}", source.display(), digest(&bytes)));
        }
        parts.push(classpath.to_owned());
        let dir = process::root()
            .join(".data/parity/classes")
            .join(&digest(parts.join("\n").as_bytes())[..16]);
        if dir.join(".complete").is_file() {
            return Ok(dir);
        }
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        let output = Command::new(&self.javac)
            .args(["-encoding", "UTF-8", "-nowarn", "-cp", classpath, "-d"])
            .arg(&dir)
            .args(sources)
            .env_remove("CLASSPATH")
            .output()
            .map_err(|error| format!("javac: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "javac failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        std::fs::write(dir.join(".complete"), b"").map_err(|error| error.to_string())?;
        Ok(dir)
    }

    /// Runs the oracle on JSONL input and returns its JSONL output.
    pub fn run(
        &self,
        classes: &Path,
        classpath: &str,
        input: &str,
        assets: &Path,
    ) -> Result<String, String> {
        let separator = if cfg!(windows) { ";" } else { ":" };
        let mut command = Command::new(&self.java);
        command
            .args([
                "-Djava.awt.headless=true",
                "-Duser.language=en",
                "-Duser.country=US",
            ])
            .arg("-Duser.timezone=UTC")
            .args(JVM_FLAGS)
            .arg("-cp")
            .arg(format!("{}{separator}{classpath}", classes.display()))
            .arg("Oracle")
            .env("PHOTON_ASSETS", assets);
        for name in [
            "JAVA_TOOL_OPTIONS",
            "_JAVA_OPTIONS",
            "JDK_JAVA_OPTIONS",
            "CLASSPATH",
        ] {
            command.env_remove(name);
        }
        run_with_timeout(command, input, TIMEOUT)
    }
}

/// Feeds `input`, collects stdout, and kills the process after `timeout`.
pub fn run_with_timeout(
    mut command: Command,
    input: &str,
    timeout: Duration,
) -> Result<String, String> {
    let mut child = command
        .current_dir(process::root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start {command:?}: {error}"))?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let input = input.to_owned();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let mut stdout = child.stdout.take().ok_or("no stdout")?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        stdout.read_to_string(&mut text).map(|_| text)
    });
    let mut stderr = child.stderr.take().ok_or("no stderr")?;
    let errors = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            break status;
        }
        if started.elapsed() > timeout {
            let _ = child.kill();
            return Err(format!("timed out after {}s", timeout.as_secs()));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let _ = writer.join();
    let output = reader
        .join()
        .map_err(|_| "reader panicked")?
        .map_err(|error| error.to_string())?;
    let errors = errors.join().unwrap_or_default();
    if status.success() {
        Ok(output)
    } else {
        let tail: Vec<&str> = errors.lines().rev().take(25).collect();
        Err(format!(
            "exited {status}:\n{}",
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        ))
    }
}
