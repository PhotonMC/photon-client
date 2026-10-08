"""Generate crates/voxel-data/src/generated/leaves.rs from a movement-family capture.

    python3 tools/generate/leaves.py .data/captures/behavior/<id>/outputs.jsonl

Each leaves block's chance of letting a leaf fall (`LeavesBlock.leafParticleChance`) and the leaf it
lets fall: the tinted one, or a type of its own with its colour.
"""
from common import f32, header, load_rows, run, rust_str, write


def generate(capture):
    blocks = load_rows(capture, "movement/leaf-particles")[0]["blocks"]
    out = header("The leaves blocks' falling leaves.", "leaves.py") + [
        "/// Name, chance, the leaf's particle type (`tinted`: the block's tint), and its colour.",
        "#[rustfmt::skip]",
        f"pub static LEAVES: [(&str, f32, &str, Option<i32>); {len(blocks)}] = [",
    ]
    for block in blocks:
        color = f"Some({block['color']})" if "color" in block else "None"
        out.append(f"    ({rust_str(block['name'])}, {f32(block['chance'])}, {rust_str(block['particle'])}, {color}),")
    out.append("];")
    write("leaves.rs", out)
    print(f"leaves blocks={len(blocks)}")


if __name__ == "__main__":
    run(generate)
