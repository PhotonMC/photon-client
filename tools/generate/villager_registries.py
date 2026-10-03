"""Generate crates/voxel-data/src/generated/villager_registries.rs from an entity-family capture.

    python3 tools/generate/villager_registries.py .data/captures/behavior/<id>/outputs.jsonl

The villager types and professions in the order the wire numbers them (the
built-in registries), each with the hat its texture's metadata asks for. The
profession layer reads the hats from the texture files; the table holds what
that read gives, and the parity run compares it with the reference's each time.
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/villager-registries")
    out = header("The villager types and professions, with the hat each one's texture asks for.",
                 "villager_registries.py", ["Names are the registry paths, in wire order."]) + [
        "use crate::villager::Hat;",
        "",
    ]
    for kind, constant in (("type", "VILLAGER_TYPES"), ("profession", "VILLAGER_PROFESSIONS")):
        entries = [row for row in rows if row["kind"] == kind]
        if [row["id"] for row in entries] != list(range(len(entries))):
            raise SystemExit(f"the {kind}s must come in id order, with no gaps")
        out += ["#[rustfmt::skip]", f"pub static {constant}: [(&str, Hat); {len(entries)}] = ["]
        for row in entries:
            out.append(f"    ({rust_str(row['name'])}, Hat::{row['hat'].capitalize()}),")
        out += ["];", ""]
    write("villager_registries.rs", out)
    print(f"villager_registries={len(rows)}")


if __name__ == "__main__":
    run(generate)
