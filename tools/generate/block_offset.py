"""Generate crates/voxel-data/src/block/offset.rs from a block-family capture.

    python3 tools/generate/block_offset.py .data/captures/behavior/<id>/outputs.jsonl

The blocks the renderer nudges off the grid, whether the nudge is vertical
too, and the block's own limits, from the reference.
"""
from common import f32, header, load_case, run, rust_bool, rust_str, write


def generate(capture):
    blocks = load_case(capture, "block/offset-properties")["blocks"]
    out = header("The blocks the renderer nudges off the grid, from the reference.", "block_offset.py") + [
        "/// A block with a random offset.",
        "pub struct BlockOffset {",
        "    pub name: &'static str,",
        "    /// The nudge is vertical as well as horizontal.",
        "    pub vertical: bool,",
        "    pub max_horizontal: f32,",
        "    pub max_vertical: f32,",
        "}",
        "",
        "#[rustfmt::skip]",
        f"pub static OFFSETS: [BlockOffset; {len(blocks)}] = [",
    ]
    for block in blocks:
        out.append(
            f"    BlockOffset {{ name: {rust_str(block['name'])}, vertical: {rust_bool(block['kind'] == 'xyz')}, "
            f"max_horizontal: {f32(block['max_horizontal'])}, max_vertical: {f32(block['max_vertical'])} }},"
        )
    out.append("];")
    write("block/offset.rs", out)
    print(f"blocks={len(blocks)}")


if __name__ == "__main__":
    run(generate)
