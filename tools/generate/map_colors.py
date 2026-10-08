"""Generate crates/voxel-data/src/generated/map_colors.rs from a movement-family capture.

    python3 tools/generate/map_colors.py .data/captures/behavior/<id>/outputs.jsonl

Each block's map colour (`BlockState.getMapColor`), the states whose colour is not their block's,
and the falling blocks' dust colours (`FallingBlock.getDustColor`).
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    blocks = load_rows(capture, "movement/block-map-colors")[0]["blocks"]
    states = sorted((state, color) for block in blocks for state, color in block["states"])
    dust = [(block["name"], block["dust"]) for block in blocks if "dust" in block]
    out = header("Each block's map colour, the states with another, and the falling blocks' dust.",
                 "map_colors.py") + [
        "/// Each block's map colour id, by name, in registry order.",
        "#[rustfmt::skip]",
        f"pub static BLOCK_MAP_COLORS: [(&str, u8); {len(blocks)}] = [",
    ]
    for block in blocks:
        out.append(f"    ({rust_str(block['name'])}, {block['color']}),")
    out += ["];", "", "/// The states whose map colour id is not their block's, by state id.", "#[rustfmt::skip]",
            f"pub static STATE_MAP_COLORS: [(u32, u8); {len(states)}] = ["]
    for state, color in states:
        out.append(f"    ({state}, {color}),")
    out += ["];", "", "/// The falling blocks' dust colours (ARGB, as `int`).", "#[rustfmt::skip]",
            f"pub static DUST_COLORS: [(&str, i32); {len(dust)}] = ["]
    for name, color in dust:
        out.append(f"    ({rust_str(name)}, {color}),")
    out.append("];")
    write("map_colors.rs", out)
    print(f"map_colors blocks={len(blocks)} states={len(states)} dust={len(dust)}")


if __name__ == "__main__":
    run(generate)
