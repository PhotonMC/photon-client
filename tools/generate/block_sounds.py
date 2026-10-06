"""Generate crates/voxel-data/src/generated/block_sounds.rs from a sound-family capture.

    python3 tools/generate/block_sounds.py .data/captures/behavior/<id>/outputs.jsonl

The sound types of the reference (`SoundType`: volume, pitch and the five sounds of a block),
which type each block has, and the few block states that have another.
"""
from common import f32, header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "sound/block-sounds")
    types = [row for row in rows if row["kind"] == "type"]
    blocks = [row for row in rows if row["kind"] == "block"]
    index = {row["name"]: number for number, row in enumerate(types)}
    out = header("The sound type of every block.", "block_sounds.py") + [
        "use crate::block_sound::SoundTypeRow;",
        "",
        "#[rustfmt::skip]",
        f"pub static SOUND_TYPES: [SoundTypeRow; {len(types)}] = [",
    ]
    for row in types:
        sounds = ", ".join(rust_str(row[key]) for key in ("break", "step", "place", "hit", "fall"))
        out.append(f"    SoundTypeRow {{ name: {rust_str(row['name'])}, volume: {f32(row['volume'])}, "
                   f"pitch: {f32(row['pitch'])}, sounds: [{sounds}] }},")
    out += ["];", "", "/// Each block's sound type, by the block's registry order.", "#[rustfmt::skip]",
            f"pub static BLOCK_SOUND_TYPES: [(&str, u8); {len(blocks)}] = ["]
    for row in blocks:
        out.append(f"    ({rust_str(row['name'])}, {index[row['type']]}),")
    out += ["];", ""]
    overrides = [(state, index[name]) for row in blocks for state, name in row["overrides"]]
    out += ["/// The block states whose sound type is not their block's.", "#[rustfmt::skip]",
            f"pub static STATE_SOUND_TYPES: [(u32, u8); {len(overrides)}] = ["]
    for state, number in overrides:
        out.append(f"    ({state}, {number}),")
    out.append("];")
    write("block_sounds.rs", out)
    print(f"block_sounds types={len(types)} blocks={len(blocks)} overrides={len(overrides)}")


if __name__ == "__main__":
    run(generate)
