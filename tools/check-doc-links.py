#!/usr/bin/env python3
"""Fail when a relative Markdown link points at a file that does not exist.

Covers every tracked Markdown file. External links and pure anchors are not
checked; a link's `#fragment` is ignored, only the file must exist.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"\]\(([^)\s]+)\)")
FENCE = re.compile(r"^\s*(```|~~~)")


def tracked_markdown() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return [ROOT / name for name in out.splitlines() if (ROOT / name).exists()]


def main() -> int:
    failures = []
    for path in tracked_markdown():
        in_fence = False
        for number, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
            if FENCE.match(line):
                in_fence = not in_fence
            if in_fence:
                continue
            for target in LINK.findall(line):
                if re.match(r"^[a-z][a-z0-9+.-]*:", target) or target.startswith("#"):
                    continue
                file_part = target.split("#", 1)[0]
                if file_part and not (path.parent / file_part).exists():
                    failures.append(f"{path.relative_to(ROOT)}:{number} -> {target}")
    for failure in failures:
        print(failure, file=sys.stderr)
    if failures:
        print(f"\n{len(failures)} broken relative link(s).", file=sys.stderr)
        return 1
    print("doc links ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
