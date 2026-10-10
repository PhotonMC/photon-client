//! The Java/Rust behavior comparison.
//!
//! The runner itself is still `tools/behavior/run.py`; this command is the
//! stable entry point the gates use until the runner moves into Rust.

use crate::process;

/// Every behavior family except `math`, whose reference only holds on
/// x86-64 Linux.
pub const FAMILIES: &[&str] = &[
    "atlas",
    "binary",
    "block",
    "entity",
    "gui",
    "image",
    "json",
    "model",
    "movement",
    "nbt",
    "resources",
    "sound",
    "timer",
    "utf8",
    "window",
    "wire",
];

/// `cargo xtask parity [--capture] [--family <name>]...`
pub fn run(args: &[String]) -> Result<(), String> {
    let mut capture = false;
    let mut families = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--capture" => capture = true,
            "--family" => families.push(rest.next().ok_or("--family needs a name")?.clone()),
            other => return Err(format!("unknown parity option `{other}`")),
        }
    }
    if families.is_empty() {
        families = FAMILIES.iter().map(|family| (*family).to_owned()).collect();
    }
    compare(capture, &families)
}

/// Whether the local Java reference needed for parity is present.
pub fn reference_present() -> bool {
    process::root().join(".data/minecraft/runtime").is_dir()
}

pub fn compare(capture: bool, families: &[String]) -> Result<(), String> {
    process::run(
        "cargo",
        &["build", "--release", "--package", "photon-oracle"],
    )?;
    let mut selection = Vec::new();
    for family in families {
        selection.push("--family");
        selection.push(family.as_str());
    }
    if capture {
        let mut args = vec!["tools/behavior/run.py", "capture"];
        args.extend(&selection);
        process::run_with("python3", &args, &env())?;
    }
    let mut args = vec!["tools/behavior/run.py", "compare"];
    args.extend(&selection);
    args.extend(["--adapter", "target/release/photon-oracle"]);
    process::run_with("python3", &args, &env())
}

/// The oracle and the adapter read game assets from the local reference copy.
fn env() -> [(&'static str, String); 1] {
    let assets = process::root().join(".data/minecraft/assets/jar/assets");
    [("PHOTON_ASSETS", assets.display().to_string())]
}
