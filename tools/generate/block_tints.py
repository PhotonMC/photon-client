"""Generate crates/photon-data/src/generated/block/tints.rs from a block-family capture.

    python3 tools/generate/block_tints.py .data/captures/behavior/<id>/outputs.jsonl

The tint each block's layers take outside a world, for the blocks the
reference's `BlockColors` registers a tint source for.
"""
from common import header, load_case, run, rust_str, write


def generate(capture):
    tinted = load_case(capture, "block/tints")["tinted"]
    out = header("The tint each block's layers take outside a world, as packed colours.",
                 "block_tints.py", ["", "Only the blocks the reference registers a tint source for appear here."]) + [
        "#[rustfmt::skip]",
        f"pub static TINTS: [(&str, &[u32]); {len(tinted)}] = [",
    ]
    for entry in tinted:
        layers = ", ".join(f"0x{layer.upper()}" for layer in entry["layers"])
        out.append(f"    ({rust_str(entry['name'])}, &[{layers}]),")
    out.append("];")
    write("block/tints.rs", out)
    print(f"tinted={len(tinted)}")


if __name__ == "__main__":
    run(generate)
