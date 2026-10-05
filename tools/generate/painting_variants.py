"""Generate crates/voxel-data/src/generated/painting_variants.rs from an entity-family capture.

    python3 tools/generate/painting_variants.py .data/captures/behavior/<id>/outputs.jsonl

The painting variants of the game's data pack by name: the texture each wears and its size in
blocks, and where the texture is (under `textures/`). The server names the variant a painting wears by its place in the registry; the
registry's names tie that place to this table.
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/painting-variants")
    out = header("The painting variants of the game's data pack.", "painting_variants.py") + [
        "use crate::entity_type::PaintingVariant;",
        "",
        "#[rustfmt::skip]",
        f"pub static PAINTING_VARIANTS: [PaintingVariant; {len(rows)}] = [",
    ]
    for row in rows:
        texture = "painting/" + row["asset"].split(":", 1)[1]
        out.append(
            f"    PaintingVariant {{ name: {rust_str(row['name'])}, asset: {rust_str(row['asset'])}, "
            f"texture: {rust_str(texture)}, width: {row['width']}, height: {row['height']} }},")
    out.append("];")
    write("painting_variants.rs", out)
    print(f"painting_variants={len(rows)}")


if __name__ == "__main__":
    run(generate)
