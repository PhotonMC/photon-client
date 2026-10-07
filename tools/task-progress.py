#!/usr/bin/env python3
"""Recount the task board's progress block from its checkboxes.

Each item on docs/tasks/README.md is one line:

    - [x] [ID](file.md) Title — SIZE · bağımlı: A, B · bitti (hash)
    - [ ] **K-M9** Scope not yet split into tasks — M×8

A `×N` after the size is an estimate of how many tasks of that size a scope
item not yet split into tasks holds; it weighs N times.

A checked box is finished work (`bitti`, `bölündü`: the rest lives in its
children, `kapandı`); an empty one is work left. `### ` headings group the
items by milestone. Sizes weigh the share: S 1, M 2, L 3.

Run plainly to rewrite the block between the progress markers; `--check`
only fails when the block is out of date (the gate runs it that way).
"""

import re
import sys
from pathlib import Path

BOARD = Path(__file__).resolve().parents[1] / "docs" / "tasks" / "README.md"
START, END = "<!-- progress:start -->", "<!-- progress:end -->"
WEIGHTS = {"S": 1, "M": 2, "L": 3}
ITEM = re.compile(
    r"^- \[(?P<done>[ x])\] (?:\[(?P<id>[\w.-]+)\]\([^)]*\)|\*\*(?P<id2>[\w.-]+)\*\*) "
    r"(?P<title>.*?) — (?P<size>S|M|L)(?:×(?P<count>\d+))?(?: · (?P<rest>.*))?$"
)


def items(text):
    """The board's items as (group, id, done, weight, dependencies)."""
    group = None
    for line in text.split("\n"):
        if line.startswith("### "):
            group = line[4:].strip()
            continue
        match = ITEM.match(line)
        if not match:
            continue
        rest = match["rest"] or ""
        needs = re.search(r"bağımlı: ([^·]+)", rest)
        needs = [name.strip() for name in needs[1].split(",")] if needs else []
        weight = WEIGHTS[match["size"]] * int(match["count"] or 1)
        yield group, match["id"] or match["id2"], match["done"] == "x", weight, needs


def bar(share, width=20):
    filled = round(share * width)
    return "█" * filled + "░" * (width - filled)


def block(text):
    board = list(items(text))
    done_ids = {name for _, name, done, _, _ in board if done}
    groups = {}
    for group, _, done, weight, _ in board:
        counts = groups.setdefault(group, [0, 0, 0, 0])
        counts[0] += 1
        counts[1] += done
        counts[2] += weight
        counts[3] += weight * done
    total = [sum(counts[index] for counts in groups.values()) for index in range(4)]
    # Scope items are planned into tasks first, so only tasks are offered as next.
    ready = [name for _, name, done, _, needs in board
             if not done and not name.startswith("K-") and all(need in done_ids for need in needs)]
    lines = [
        START,
        f"**Toplam: %{100 * total[3] / total[2]:.0f} bitti** (ağırlıklı) · "
        f"{total[1]}/{total[0]} madde işaretli, **{total[0] - total[1]} madde kaldı** "
        f"(tahmini {(total[2] - total[3]) // 2} M-görev eşdeğeri iş).",
        "",
        "| Grup | İlerleme | % | Madde |",
        "| --- | --- | ---: | ---: |",
    ]
    for group, (count, done, weight, weight_done) in groups.items():
        share = weight_done / weight
        lines.append(f"| {group} | `{bar(share)}` | {100 * share:.0f} | {done}/{count} |")
    lines += [
        "",
        "Bağımlılıkları bitmiş açık görevler (sıradaki adaylar): "
        + (", ".join(ready) if ready else "yok") + ".",
        END,
    ]
    return "\n".join(lines)


def main():
    text = BOARD.read_text(encoding="utf-8")
    head, rest = text.split(START, 1)
    _, tail = rest.split(END, 1)
    updated = head + block(text) + tail
    if "--check" in sys.argv[1:]:
        if updated != text:
            print("docs/tasks/README.md progress block is stale: run `python3 tools/task-progress.py`",
                  file=sys.stderr)
            return 1
        print("task progress ok")
        return 0
    BOARD.write_text(updated, encoding="utf-8")
    print(block(text).split("\n")[1])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
