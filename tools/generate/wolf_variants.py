"""Generate crates/photon-data/src/generated/wolf_variants.rs from an entity-family capture.

    python3 tools/generate/wolf_variants.py .data/captures/behavior/<id>/outputs.jsonl

The wolf variants of the game's data pack by name: the textures a grown wolf
and a baby wear, wild, tame and angry. The server names the variant a wolf
wears by its place in the registry; the registry's names tie that place to
this table.
"""
from common import header, load_rows, run, rust_str, write

KEYS = ["wild", "tame", "angry", "baby_wild", "baby_tame", "baby_angry"]


def generate(capture):
    rows = load_rows(capture, "entity/wolf-variants")
    out = header("The wolf variants of the game's data pack.", "wolf_variants.py") + [
        "use crate::entity_type::WolfVariant;",
        "",
        "#[rustfmt::skip]",
        f"pub static WOLF_VARIANTS: [WolfVariant; {len(rows)}] = [",
    ]
    for row in rows:
        assets = ", ".join(rust_str(row[key]) for key in KEYS)
        out.append(f"    WolfVariant {{ name: {rust_str(row['name'])}, assets: [{assets}] }},")
    out.append("];")
    write("wolf_variants.rs", out)
    print(f"wolf_variants={len(rows)}")


if __name__ == "__main__":
    run(generate)
