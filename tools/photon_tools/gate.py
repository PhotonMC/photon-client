"""The commit gate and the Java/Rust parity run."""

import json
import re
import sys

from .common import run_captured, say


# Every behavior family except `math` (which only runs on x86-64 Linux).
PARITY_FAMILIES = ["atlas", "binary", "block", "entity", "gui", "image", "json", "model",
                   "movement", "nbt", "resources", "sound", "timer", "utf8", "window", "wire"]
NOISE = ("block v0.1.6", "future-incompat")


def diagnostics(lines, limit=60):
    """Warning and error blocks of a cargo run, without the known noise."""
    shown, context = [], 0
    for line in lines:
        if line.startswith(("warning", "error")):
            context = 10
            keep = not any(noise in line for noise in NOISE)
        elif context:
            context -= 1
        else:
            keep = False
            continue
        if keep:
            shown.append(line)
    return shown[:limit]


def parity_summary(capture, families):
    """Run the Java/Rust comparison and return (ok, one summary line)."""
    families = families or PARITY_FAMILIES
    command = [sys.executable, "tools/behavior/run.py", "capture" if capture else "compare"]
    for family in families:
        command += ["--family", family]
    adapter = "target/release/photon-oracle"
    if capture:
        _, lines = run_captured(command)
        last = parse_report(lines)
        if last.get("status") != "reference_ready":
            return False, f"capture: {summary_of(last, lines)}"
    _, lines = run_captured([sys.executable, "tools/behavior/run.py", "compare",
                             *command[3:], "--adapter", adapter])
    report = parse_report(lines)
    return report.get("status") == "parity_passed", f"parity: {summary_of(report, lines)}"


def parse_report(lines):
    for line in reversed(lines):
        try:
            report = json.loads(line)
        except ValueError:
            continue
        if isinstance(report, dict):
            return report
    return {}


def summary_of(report, lines):
    if not report:
        return " | ".join(lines[-3:])
    difference = str(report.get("difference"))[:600]
    return f"{report.get('status')} {report.get('case_count')} {difference} {report.get('reason') or ''}".strip()


def check(args):
    ok = True

    def step(label, command, fatal_only_on_output=False):
        nonlocal ok
        code, lines = run_captured(command)
        problems = diagnostics(lines)
        failed = code != 0 or (fatal_only_on_output and problems)
        if failed or problems:
            ok = False
            say(f"{label}: problems")
            for line in (problems or lines[-40:]):
                print(line)
        return lines

    run_captured(["cargo", "fmt", "--all"])
    step("build", ["cargo", "build", "--release", "--workspace", "--all-targets"], True)
    code, lines = run_captured(["cargo", "test", "--release", "--workspace", "--all-targets"])
    passed = failed = 0
    for line in lines:
        found = re.match(r"test result: \w+\. (\d+) passed; (\d+) failed", line)
        if found:
            passed += int(found.group(1))
            failed += int(found.group(2))
        elif "FAILED" in line or "panicked" in line:
            print(line)
    print(f"tests passed {passed} failed {failed}")
    if code != 0 or failed:
        ok = False
        if not failed:
            print("\n".join(lines[-30:]))
    step("clippy", ["cargo", "clippy", "--release", "--workspace", "--all-targets"], True)
    good, line = parity_summary(args.capture, args.family)
    print(line)
    ok = ok and good
    for script in ("generate-doc-index.py", "check-doc-cells.py", "check-doc-links.py",
                   "task-progress.py --check"):
        code, lines = run_captured([sys.executable, *f"tools/{script}".split()])
        if code != 0:
            ok = False
            print("\n".join(lines[-15:]))
    print("CHECK OK" if ok else "CHECK FAILED")
    raise SystemExit(0 if ok else 1)


def parity(args):
    good, line = parity_summary(args.capture, args.family)
    print(line)
    raise SystemExit(0 if good else 1)
