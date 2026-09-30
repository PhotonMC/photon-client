"""Generate crates/voxel-data/src/model/uv_lock.rs from a model-family capture.

    python3 tools/generate/uv_lock.py .data/captures/behavior/<id>/outputs.jsonl

The locked face transforms for every whole-model rotation, taken from the
`uvlock-` rows of `model/model-state`: deduplicated by the model's transform
(the cube symmetry group has 24 elements), with every float stored once.
"""
from common import f32, header, load_rows, run, write

FACES = ["down", "up", "north", "south", "west", "east"]


def generate(capture):
    rows = [row for row in load_rows(capture, "model/model-state") if row["label"].startswith("uvlock-")]
    entries, seen = [], set()
    for row in rows:
        key = tuple(row["transformation"])
        if key in seen:
            continue
        seen.add(key)
        faces = {face["face"]: face for face in row["faces"]}
        entries.append((row["transformation"],
                        [faces[name]["forward"] for name in FACES],
                        [faces[name]["inverse"] for name in FACES]))
    values = sorted({bits for entry in entries for matrix in [entry[0], *entry[1], *entry[2]]
                     for bits in matrix}, key=lambda bits: int(bits, 16))
    index = {bits: number for number, bits in enumerate(values)}

    def indices(matrix):
        return "[" + ", ".join(str(index[bits]) for bits in matrix) + "]"

    out = header("The UV lock tables, read from the reference.", "uv_lock.py", [
        "",
        "With UV lock on, a face's texture keeps its world orientation while the",
        "model turns. The reference works this out once at startup by composing",
        "quaternions and inverting the result, and caches one table per element of",
        "the cube symmetry group.",
        "",
        "Those tables are **not exact**: they hold entries like `0.99999994` and",
        "`-1.0000001`, float error left over from the quaternion path. Recomputing",
        "them would mean reproducing the matrix library's `rotate`, `mul` and",
        "`invertAffine` to the letter, so they are taken from the reference as data",
        "and the parity run compares every entry on every pass.",
    ]) + [
        "/// Every distinct value the tables hold.",
        f"pub const VALUES: [f32; {len(values)}] = [" + ", ".join(f32(bits) for bits in values) + "];",
        "",
        "/// One rotation's tables.",
        "pub struct Entry {",
        "    /// The whole-model transform this belongs to, as indices into [`VALUES`],",
        "    /// flattened column by column.",
        "    pub transform: [u8; 16],",
        "    /// The face transform for each of the six faces, in the order",
        "    /// `Direction` lists them, likewise as indices.",
        "    pub faces: [[u8; 16]; 6],",
        "    /// The inverse of each, which is the one the baking path uses.",
        "    pub inverse_faces: [[u8; 16]; 6],",
        "}",
        "",
        "/// The cube symmetry group has twenty four elements, so the table does too.",
        "#[rustfmt::skip]",
        f"pub const TABLE: [Entry; {len(entries)}] = [",
    ]
    for transform, faces, inverses in entries:
        out.append(f"    Entry {{ transform: {indices(transform)},")
        out.append("        faces: [" + ", ".join(indices(m) for m in faces) + "],")
        out.append("        inverse_faces: [" + ", ".join(indices(m) for m in inverses) + "] },")
    out.append("];")
    write("model/uv_lock.rs", out)
    print(f"values={len(values)} entries={len(entries)}")


if __name__ == "__main__":
    run(generate)
