"""Generate crates/voxel-data/src/generated/entity_variants.rs from an entity-family capture.

    python3 tools/generate/entity_variants.py .data/captures/behavior/<id>/outputs.jsonl

The pig, cow and chicken variants of the game's data pack, by kind and name: the
texture an adult and a baby wear and the model family each asks for. The
server names the variant an entity wears by its place in its registry; the
registry's names are what ties that place to this table.
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/variants")
    out = header("The pig, cow and chicken variants of the game's data pack.", "entity_variants.py") + [
        "use crate::entity_type::EntityVariant;",
        "",
        "#[rustfmt::skip]",
        f"pub static ENTITY_VARIANTS: [EntityVariant; {len(rows)}] = [",
    ]
    for row in rows:
        out.append(
            f"    EntityVariant {{ kind: {rust_str(row['kind'])}, name: {rust_str(row['name'])}, "
            f"asset: {rust_str(row['asset'])}, baby_asset: {rust_str(row['baby_asset'])}, "
            f"model: {rust_str(row['model'])} }},")
    out.append("];")
    write("entity_variants.rs", out)
    print(f"entity_variants={len(rows)}")


if __name__ == "__main__":
    run(generate)
