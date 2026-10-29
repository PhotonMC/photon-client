//! Repository automation, run as `cargo xtask <command>`.
//!
//! Every quality gate lives here so the git hooks, CI and contributors run
//! the same deterministic checks with nothing installed beyond the Rust
//! toolchain.

mod affected;
mod dev;
mod entities;
mod gate;
mod git;
mod hooks;
mod lint;
mod parity;
mod process;
mod progress;
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
  parity [--recapture] [--family <name>]...
                            compare the Rust adapter with the Java reference
  parity --print-cases [--family <name>]...
                            print the case lines (the oracle and adapter input)
  setup [--skip-objects] [--named] [--decompile] [--java <path>]
                            the pinned game from Mojang's servers into .data/minecraft
  server [--accept-eula] [--background] [--stop] [--dir <path>] [--memory <size>] [--java <path>]
                            set up and start the test server (port 25600, RCON 25601)
  play [address] [--user <name>] [--size <w> <h>] [--fullscreen] [--camera back|front]
       [--profile] [--no-build]
                            build and start the client
  rcon <command>...         send commands to the test server
  bench [address] [--name <n>] [--repeat <n>] [--warmup <s>] [--size <w> <h>]
        [--route still|run|dive|void] [--vsync on|off] [--fps-limit <n>]
        [--render-distance <n>] [--validation] [--no-build]
        [--save | --compare [--threshold <percent>]]
                            fixed scripted run, frame timing report in .data/runs/bench;
                            --save keeps it as the route's baseline, --compare fails on a p99 rise
  progress [--check]        rewrite (or verify) the README's computed progress block
  entities                  the reference's renderer types and what the client sources name
  shot <name> <script> [--user <name>] [--address <a>] [--rcon <cmd>]... [--wait <s>]
                            scripted client run, first frame in .data/runs/shots";

type Command = fn(&[String]) -> Result<(), String>;

/// Every command and the function that runs it.
const COMMANDS: &[(&str, Command)] = &[
    ("lint", lint::run),
    ("check", gate::check),
    ("pre-commit", |_| gate::pre_commit()),
    ("pre-push", |_| gate::pre_push()),
    ("commit-msg", hooks::commit_msg),
    ("install-hooks", |_| hooks::install()),
    ("affected", affected::print),
    ("parity", parity::run),
    ("setup", dev::setup),
    ("server", dev::server),
    ("play", dev::play),
    ("rcon", dev::rcon),
    ("bench", dev::bench),
    ("shot", dev::shot),
    ("entities", entities::run),
    ("progress", progress::run),
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((name, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let result = match COMMANDS.iter().find(|(command, _)| command == name) {
        Some((_, run)) => run(rest),
        None => Err(format!("unknown command `{name}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
