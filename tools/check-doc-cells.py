#!/usr/bin/env python3
"""Fail when a roadmap or plan table cell starts accumulating scenario names.

Slice status belongs in that slice's own record, linked from `docs/index.md`.
The general tables carry blockers, dependencies and evidence class only, so a
cell that keeps growing is the signal that the list moved to the wrong file.
"""

import sys
from pathlib import Path

LIMIT = 700
TARGETS = ("docs/roadmap.md", "docs/remaining-test-plan.md")

def main() -> int:
    root = Path(__file__).resolve().parents[1]
    failures = []
    for name in TARGETS:
        path = root / name
        for number, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
            if not line.startswith("| ") or set(line) <= set("| -"):
                continue
            if len(line) > LIMIT:
                label = line.split("|")[1].strip()[:40]
                failures.append(f"{name}:{number} {label!r} {len(line)} > {LIMIT}")
    for failure in failures:
        print(failure, file=sys.stderr)
    if failures:
        print(
            f"\n{len(failures)} table row(s) over {LIMIT} characters. Move the "
            "scenario list into the slice record and link it instead.",
            file=sys.stderr,
        )
        return 1
    print(f"doc cells ok (limit {LIMIT})")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
