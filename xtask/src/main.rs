//! Repository automation, run as `cargo xtask <command>`.
//!
//! Every quality gate lives here so the git hooks, CI and contributors run
//! the same deterministic checks with nothing installed beyond the Rust
//! toolchain.

mod affected;
mod gate;
mod git;
mod hooks;
mod lint;
mod parity;
mod process;
mod workspace;

use std::process::ExitCode;

const USAGE: &str = "\
usage: cargo xtask <command>

  lint [--quiet]            run the repository lint rules (tightens the baseline)
  check [--no-parity]       full gate: fmt, lint, clippy, tests, parity, deny
  pre-commit                fast gate for staged changes (git hook)
  pre-push                  tests and parity for the commits being pushed (git hook)
  commit-msg <file>         validate a commit message (git hook)
  install-hooks             point git at the tracked hooks in .githooks/
  affected [--base <ref>]   print the crates touched since <ref> and their dependents
  parity [--recapture] [--compare-generators] [--family <name>]...
                            compare the Rust adapter with the Java reference";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let result = match command.as_str() {
        "lint" => lint::run(rest),
        "check" => gate::check(rest),
        "pre-commit" => gate::pre_commit(),
        "pre-push" => gate::pre_push(),
        "commit-msg" => hooks::commit_msg(rest),
        "install-hooks" => hooks::install(),
        "affected" => affected::print(rest),
        "parity" => parity::run(rest),
        _ => Err(format!("unknown command `{command}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
