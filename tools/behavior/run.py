#!/usr/bin/env python3
"""Local Java reference capture and strict adapter comparison; no downloads."""

import sys

sys.dont_write_bytecode = True

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import tempfile

from cases import build_cases

ROOT = Path(__file__).resolve().parents[2]
SPEC = ROOT / "tests/behavior"
DATA = ROOT / ".data"


class Blocked(Exception):
    pass


class Failed(Exception):
    pass


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def save(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False, allow_nan=False) + "\n", encoding="utf-8")


def check_file(path, sha1):
    if not path.is_file():
        raise Blocked(f"Missing reference file: {path}")
    if hashlib.sha1(path.read_bytes()).hexdigest() != sha1:
        raise Blocked(f"Reference hash mismatch: {path}")


def execute(command, cwd, label, stdin="", timeout=60):
    env = os.environ.copy()
    for name in ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"):
        env.pop(name, None)
    try:
        result = subprocess.run(command, input=stdin, capture_output=True, text=True,
                                encoding="utf-8", cwd=cwd, env=env, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        raise Failed(f"{label} timed out after {timeout}s") from error
    (cwd / f"{label}.stdout").write_text(result.stdout, encoding="utf-8")
    (cwd / f"{label}.stderr").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise Failed(f"{label} exited {result.returncode}; see {cwd / (label + '.stderr')}")
    return result.stdout, result.stderr


def native_platform():
    """LWJGL's name for this machine's natives, as the version file lists them."""
    system, machine = platform.system(), platform.machine().lower()
    arm = machine in ("arm64", "aarch64")
    if system == "Darwin":
        return "macos-arm64" if arm else "macos"
    if system == "Windows":
        return "windows-arm64" if arm else ("windows" if machine.endswith("64") else "windows-x86")
    return "linux"


def native_key(key):
    """Families name the Linux natives; another machine loads its own."""
    suffix = "-natives-linux"
    return key[:-len(suffix)] + "-natives-" + native_platform() if key.endswith(suffix) else key


def preflight(families, runtime, jdk_home, run_dir):
    pin = json.loads((SPEC / "reference.json").read_text(encoding="utf-8"))
    unknown = set(families) - pin["families"].keys()
    if unknown:
        raise Blocked(f"Unknown families: {sorted(unknown)}")
    client = runtime / pin["client_file"]
    check_file(client, pin["client_sha1"])
    keys = sorted({native_key(key) for family in families
                   for key in pin["families"][family]["libraries"]})
    classpath = [str(client)]
    for key in keys:
        entry = pin["libraries"][key]
        path = runtime / "libs" / entry["file"]
        check_file(path, entry["sha1"])
        classpath.append(str(path))
    commands = {}
    versions = {}
    for name in ("java", "javac"):
        found = str(jdk_home / "bin" / name) if jdk_home else shutil.which(name)
        if not found or not Path(found).is_file():
            raise Blocked(f"Java {pin['java_major']} tool missing: {name}")
        commands[name] = str(Path(found).resolve())
        out, err = execute([commands[name], "-version"], run_dir, f"{name}-version")
        version = (out + err).strip()
        pattern = r'version "(\d+)' if name == "java" else r'javac (\d+)'
        match = re.search(pattern, version)
        if not match or int(match.group(1)) != pin["java_major"]:
            raise Blocked(f"Expected {name} {pin['java_major']}; got {version}")
        versions[name] = version
    sources = [SPEC / "java/Oracle.java"] + [SPEC / "java" / pin["families"][f]["source"] for f in sorted(families)]
    # A family that builds on another's helpers (the screens use the movement
    # family's menus) names those sources too.
    for family in sorted(families):
        for extra in pin["families"][family].get("extra_sources", []):
            if SPEC / "java" / extra not in sources:
                sources.append(SPEC / "java" / extra)
    # The math family calls one LWJGL helper and would otherwise need natives,
    # so it compiles a stub. Compiled classes come before the jars, so that stub
    # would also shadow the real class for any family that pulls LWJGL in for
    # real; in that case the real one is already available and wins.
    if "math" in families and "org.lwjgl:lwjgl" not in keys:
        sources.append(SPEC / "java/org/lwjgl/system/MemoryUtil.java")
    identity = {
        "schema_version": 1, "minecraft_version": pin["version"], "protocol": pin["protocol"],
        "client_sha1": pin["client_sha1"], "libraries": {k: pin["libraries"][k] for k in keys},
        "jdk": versions, "os": platform.system(), "arch": platform.machine(),
        "families": sorted(families), "jvm_flags": JVM_FLAGS,
        "oracle_sources": {p.name: digest(p.read_bytes()) for p in sources},
        "runner_sha256": digest(Path(__file__).read_bytes()),
    }
    return identity, commands, os.pathsep.join(classpath), sources


# HotSpot answers Math.sin/cos/log/pow/exp with its own stubs on x86-64, which round differently
# from fdlibm in some arguments (and from other platforms' JVMs). The reference runs with them off,
# so `Math.*` is `StrictMath.*` and the same bits on every machine.
JVM_FLAGS = ["-XX:+UnlockDiagnosticVMOptions", "-XX:-UseLibmIntrinsic"]


def parse_records(raw, cases):
    def invalid_constant(value):
        raise ValueError(f"Non-JSON number: {value}")

    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"Duplicate JSON field: {key}")
            result[key] = value
        return result

    try:
        records = [json.loads(line, parse_constant=invalid_constant, object_pairs_hook=unique_object)
                   for line in raw.splitlines()]
    except (ValueError, TypeError) as error:
        raise Failed(f"Invalid JSONL adapter output: {error}") from error
    if len(records) != len(cases):
        raise Failed(f"Expected {len(cases)} records, received {len(records)}")
    for request, response in zip(cases, records):
        if (not isinstance(response, dict) or type(response.get("schema_version")) is not int
                or response["schema_version"] != 1 or response.get("case_id") != request["case_id"]
                or not isinstance(response.get("result"), dict)
                or ("diagnostics" in response and not isinstance(response["diagnostics"], dict))
                or set(response) - {"schema_version", "case_id", "result", "diagnostics"}):
            raise Failed(f"Malformed, reordered or missing record for {request['case_id']}")
    return records


def first_difference(expected, actual, path="result"):
    if type(expected) is not type(actual):
        return {"path": path, "expected": expected, "actual": actual}
    if isinstance(expected, dict):
        if expected.keys() != actual.keys():
            return {"path": path + ".keys", "expected": sorted(expected), "actual": sorted(actual)}
        for key in sorted(expected):
            difference = first_difference(expected[key], actual[key], path + "." + key)
            if difference:
                return difference
    elif isinstance(expected, list):
        if len(expected) != len(actual):
            return {"path": path + ".length", "expected": len(expected), "actual": len(actual)}
        for index, (left, right) in enumerate(zip(expected, actual)):
            difference = first_difference(left, right, f"{path}[{index}]")
            if difference:
                return difference
    elif expected != actual:
        return {"path": path, "expected": expected, "actual": actual}
    return None


def compare_records(expected, actual):
    # parse_records has already validated record count, IDs, schema and order.
    for left, right in zip(expected, actual, strict=True):
        difference = first_difference(left["result"], right["result"])
        if difference:
            return {"case_id": left["case_id"], **difference}
    return None


def load_capture(path, identity, cases):
    if not path.is_dir():
        raise Blocked("No capture for these sources, inputs and environment; run capture explicitly")
    try:
        manifest = json.loads((path / "manifest.json").read_text(encoding="utf-8"))
        raw = (path / "outputs.jsonl").read_text(encoding="utf-8")
        inputs = (path / "inputs.jsonl").read_bytes()
    except (OSError, ValueError) as error:
        raise Blocked(f"Incomplete capture: {path}") from error
    if (manifest.get("identity") != identity or manifest.get("outputs_sha256") != digest(raw.encode())
            or digest(inputs) != identity["inputs_sha256"]):
        raise Blocked(f"Capture identity/content mismatch: {path}; existing captures are never overwritten")
    return parse_records(raw, cases)


def run_command(args, run_dir, report):
    pin = json.loads((SPEC / "reference.json").read_text(encoding="utf-8"))
    families = sorted(set(args.family or pin["families"]))
    report["families"] = families
    identity, commands, classpath, sources = preflight(families, args.runtime.resolve(), args.jdk_home, run_dir)
    cases = build_cases(families)
    stdin = "".join(encode(case) + "\n" for case in cases)
    identity["inputs_sha256"] = digest(stdin.encode())
    key = digest(encode(identity).encode())
    capture = DATA / "captures/behavior" / key
    report.update(reference_id=key, case_count=len(cases), capture=str(capture))
    save(run_dir / "identity.json", identity)
    (run_dir / "inputs.jsonl").write_text(stdin, encoding="utf-8")
    if args.command == "preflight":
        report["status"] = "preflight_ready"
        return 0
    if args.command == "compare":
        if args.adapter is None or not args.adapter.is_file() or not os.access(args.adapter, os.X_OK):
            raise Blocked("Rust adapter missing/not executable; no parity result (use --adapter PATH)")
        expected = load_capture(capture, identity, cases)
        adapter = args.adapter.resolve()
        report["adapter_sha256"] = digest(adapter.read_bytes())
        # The adapter runs every record in one process; a loaded machine needs more than a minute.
        raw, _ = execute([str(adapter)], run_dir, "adapter", stdin, timeout=600)
        actual = parse_records(raw, cases)
        report["difference"] = compare_records(expected, actual)
        report["status"] = "failed" if report["difference"] else "parity_passed"
        return 1 if report["difference"] else 0
    expected = load_capture(capture, identity, cases) if args.command == "verify" else None
    classes = run_dir / "classes"
    classes.mkdir()
    execute([commands["javac"], "-encoding", "UTF-8", "-cp", classpath, "-d", str(classes),
             *map(str, sources)], run_dir, "javac")
    java = [commands["java"], "-Djava.awt.headless=true", "-Duser.language=en", "-Duser.country=US",
            "-Duser.timezone=UTC", *JVM_FLAGS, "-cp", str(classes) + os.pathsep + classpath, "Oracle"]
    # Bootstrapping the game's registries and walking every block state takes
    # longer than the other families put together, so that family gets room.
    java_timeout = 900 if {"block", "movement"} & set(families) else 60
    raw, _ = execute(java, run_dir, "java-first", stdin, timeout=java_timeout)
    first = parse_records(raw, cases)
    second_raw, _ = execute(java, run_dir, "java-second", stdin, timeout=java_timeout)
    second = parse_records(second_raw, cases)
    report["difference"] = compare_records(first, second)
    if report["difference"]:
        raise Failed("Java reference is not repeatable; capture was not saved")
    if expected is not None:
        report["difference"] = compare_records(expected, first)
        if report["difference"]:
            raise Failed("Java differs from the saved reference; capture was not modified")
    elif capture.exists():
        report["difference"] = compare_records(load_capture(capture, identity, cases), first)
        if report["difference"]:
            raise Failed("Existing capture differs; capture was not overwritten")
    else:
        staging = run_dir / "capture"
        staging.mkdir()
        (staging / "inputs.jsonl").write_text(stdin, encoding="utf-8")
        (staging / "outputs.jsonl").write_text(raw, encoding="utf-8")
        save(staging / "manifest.json", {"identity": identity, "outputs_sha256": digest(raw.encode())})
        capture.parent.mkdir(parents=True, exist_ok=True)
        staging.rename(capture)
        prune_captures(capture, identity["families"])
    report.update(status="reference_ready", java_repetitions=2, rust_status="not_compared")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["preflight", "capture", "verify", "compare"])
    parser.add_argument("--family", action="append", help="Repeat to select families; default: all")
    parser.add_argument("--runtime", type=Path, default=DATA / "minecraft/runtime")
    parser.add_argument("--jdk-home", type=Path, help="JDK containing bin/java and bin/javac")
    parser.add_argument("--adapter", type=Path, help="Executable JSONL adapter; only used by compare")
    args = parser.parse_args()
    if args.jdk_home:
        args.jdk_home = args.jdk_home.resolve()
    base = DATA / "runs/behavior"
    base.mkdir(parents=True, exist_ok=True)
    run_dir = Path(tempfile.mkdtemp(prefix=args.command + "-", dir=base))
    report = {"command": args.command, "reproduce": shlex.join([sys.executable, *sys.argv]),
              "invocation_cwd": str(Path.cwd()), "report_dir": str(run_dir)}
    try:
        code = run_command(args, run_dir, report)
    except Blocked as error:
        code = 2
        report.update(status="blocked", reason=str(error))
    except (Failed, OSError, ValueError) as error:
        code = 1
        report.update(status="failed", reason=str(error))
    save(run_dir / "report.json", report)
    print(encode(report))
    # A passing run leaves nothing worth inspecting; only failures keep their files.
    if code == 0:
        shutil.rmtree(run_dir, ignore_errors=True)
    return code


KEPT_CAPTURES = 2


def prune_captures(newest, families):
    """Keep only the newest captures of one family selection."""
    same = []
    for directory in newest.parent.iterdir():
        try:
            manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if manifest["identity"].get("families") == families:
            same.append(directory)
    same.sort(key=lambda directory: directory.stat().st_mtime, reverse=True)
    for stale in same[KEPT_CAPTURES:]:
        if stale != newest:
            shutil.rmtree(stale, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
