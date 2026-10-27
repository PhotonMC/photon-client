"""Generate crates/photon-data/src/generated/entity_types.rs from an entity-family capture.

    python3 tools/generate/entity_types.py .data/captures/behavior/<id>/outputs.jsonl

The entity type registry in id order: the name of each type, which is also
its place in the list (the id the wire uses).
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/types")
    if [row["id"] for row in rows] != list(range(len(rows))):
        raise SystemExit("the registry must come in id order, with no gaps")
    out = header("Every entity type of the built-in registry, in id order.", "entity_types.py") + [
        "#[rustfmt::skip]",
        f"pub static ENTITY_TYPES: [&str; {len(rows)}] = [",
    ]
    for row in rows:
        out.append(f"    {rust_str(row['label'])},")
    out.append("];")
    out += ["", "/// The types built with `notInPeaceful` (`EntityType.isAllowedInPeaceful`).", "#[rustfmt::skip]"]
    refused = [row["label"] for row in rows if not row["allowed_in_peaceful"]]
    out.append(f"pub static NOT_IN_PEACEFUL: [&str; {len(refused)}] = [")
    out += [f"    {rust_str(name)}," for name in refused]
    out.append("];")
    write("entity_types.rs", out)
    print(f"entity_types={len(rows)}")


if __name__ == "__main__":
    run(generate)
