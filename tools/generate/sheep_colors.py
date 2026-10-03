"""Generate crates/voxel-data/src/generated/sheep_colors.rs from an entity-family capture.

    python3 tools/generate/sheep_colors.py .data/captures/behavior/<id>/outputs.jsonl

What a sheep's wool is multiplied by for each of the sixteen dyes
(`ColorLerper.Type.SHEEP`), as the reference's ARGB, in dye id order. A
`jeb_` sheep's colour is worked out from these in Rust and compared with the
reference's at several points of its cycle.
"""
from common import header, load_rows, run, write


def generate(capture):
    rows = [row for row in load_rows(capture, "entity/sheep-colors") if row["label"].startswith("dye-")]
    if [row["label"] for row in rows] != [f"dye-{index}" for index in range(len(rows))]:
        raise SystemExit("the dyes must come in id order, with no gaps")
    out = header("What a sheep's wool is multiplied by, for each dye.", "sheep_colors.py") + [
        "#[rustfmt::skip]",
        f"pub static SHEEP_COLORS: [u32; {len(rows)}] = [",
    ]
    for row in rows:
        out.append(f"    0x{row['color'] & 0xFFFFFFFF:08X},")
    out.append("];")
    textures = [row for row in load_rows(capture, "entity/sheep-colors") if row["label"].startswith("texture-dye-")]
    if [row["label"] for row in textures] != [f"texture-dye-{index}" for index in range(len(rows))]:
        raise SystemExit("the texture colours must come in dye id order, one per dye")
    out += ["", "/// `DyeColor.getTextureDiffuseColor` (a wolf's collar), by dye id.", "#[rustfmt::skip]",
            f"pub static DYE_TEXTURE_COLORS: [u32; {len(textures)}] = ["]
    for row in textures:
        out.append(f"    0x{row['color'] & 0xFFFFFFFF:08X},")
    out.append("];")
    write("sheep_colors.rs", out)
    print(f"sheep_colors={len(rows)} dye_texture_colors={len(textures)}")


if __name__ == "__main__":
    run(generate)
