#!/usr/bin/env python3
"""Move slice/parity records from docs/ to docs/records/ and fix every link.

Records are `docs/*-conformance.md` and `docs/*-record.md`. Relative Markdown
links are re-resolved against the old location and rewritten for the new one,
and plain `docs/<name>.md` mentions (code comments, instructions) are updated.
Idempotent: with nothing left to move it only reports that.
"""

import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
RECORDS = DOCS / "records"
LINK = re.compile(r"(\]\()([^)\s]+)(\))")
TEXTUAL = {".md", ".rs", ".py", ".toml", ".java", ".txt"}


def is_record(path: Path) -> bool:
    return path.parent == DOCS and path.name.endswith(("-conformance.md", "-record.md"))


def tracked() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return [ROOT / name for name in out.splitlines() if (ROOT / name).is_file()]


def main() -> None:
    files = tracked()
    moved = {path: RECORDS / path.name for path in files if is_record(path)}
    if not moved:
        print("nothing to move")
        return
    names = {path.name for path in moved}
    RECORDS.mkdir(exist_ok=True)

    def fix_target(owner_old: Path, owner_new: Path, target: str) -> str:
        file_part, sep, fragment = target.partition("#")
        if not file_part or re.match(r"^[a-z][a-z0-9+.-]*:", file_part):
            return target
        resolved = (owner_old.parent / file_part).resolve()
        if not resolved.exists():
            return target
        resolved = moved.get(resolved, resolved)
        relative = os.path.relpath(resolved, owner_new.parent)
        return relative + sep + fragment

    rewritten = {}
    for path in files:
        if path.suffix not in TEXTUAL or path.is_relative_to(ROOT / ".data"):
            continue
        text = path.read_text(encoding="utf-8")
        new_path = moved.get(path, path)
        if path.suffix == ".md":
            text = LINK.sub(
                lambda m: m.group(1) + fix_target(path, new_path, m.group(2)) + m.group(3), text
            )
        text = re.sub(
            r"docs/([a-z0-9-]+\.md)",
            lambda m: ("docs/records/" if m.group(1) in names else "docs/") + m.group(1),
            text,
        )
        rewritten[path] = text
    for old, new in moved.items():
        subprocess.run(["git", "mv", str(old), str(new)], cwd=ROOT, check=True)
    for path, text in rewritten.items():
        target = moved.get(path, path)
        if target.read_text(encoding="utf-8") != text:
            target.write_text(text, encoding="utf-8")
    print(f"moved {len(moved)} records to docs/records/")


if __name__ == "__main__":
    main()
