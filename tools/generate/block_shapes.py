"""Generate the collision and outline shape tables from a block-family capture.

    python3 tools/generate/block_shapes.py .data/captures/behavior/<id>/outputs.jsonl

Writes crates/voxel-data/src/block/collision.rs and outline.rs. Both are the
same kind of table: every distinct shape as its boxes, and for every block the
shape of each state, whether it moves with the block's random offset and
whether the reference recomputes it from context.
"""
from common import f64, header, index_list, load_case, run, rust_bool, write

TABLES = [
    (
        "block/collision-shapes",
        "block/collision.rs",
        "Every block state's collision shape, taken from the reference.",
        [
            "",
            "The shapes are the ones the world's collision walk sees with no entity in",
            "the question. For a few blocks that is only the starting point: a random",
            "offset moves some with their position, and a handful are dynamic and read",
            "the entity or the block entity. Those blocks are flagged.",
        ],
    ),
    (
        "block/outline-shapes",
        "block/outline.rs",
        "Every block state's outline shape, taken from the reference.",
        [
            "",
            "The outline is the shape picking aims at: what a click on a block hits.",
            "It is often not the collision shape; plants have an outline and no",
            "collision. A block whose outline moves with its random offset is listed",
            "with the shape it is built from, before the offset.",
        ],
    ),
]


def table(capture, case_id, path, summary, notes):
    data = load_case(capture, case_id)
    shapes, blocks = data["shapes"], data["blocks"]
    out = header(summary, "block_shapes.py", notes) + [
        "use super::shape::{BlockShapes, ShapeBox};",
        "",
        f"/// Every distinct shape, as its boxes.\n#[rustfmt::skip]\npub static SHAPES: [&[ShapeBox]; {len(shapes)}] = [",
    ]
    for shape in shapes:
        boxes = ", ".join("[" + ", ".join(f64(bits) for bits in box) + "]" for box in shape)
        out.append(f"    &[{boxes}],")
    out += ["];", "", f"/// Every block, in registry order.\n#[rustfmt::skip]\npub static BLOCKS: [BlockShapes; {len(blocks)}] = ["]
    for block in blocks:
        if not block.get("offset_explained", True):
            raise SystemExit(f"{block['name']}: shape not explained by its offset")
        out.append(
            f"    BlockShapes {{ shapes: {index_list(block['shapes'])}, "
            f"offset: {rust_bool(block['position_dependent'])}, dynamic: {rust_bool(block['dynamic'])} }},"
        )
    out.append("];")
    write(path, out)
    print(f"{path}: shapes={len(shapes)} blocks={len(blocks)}")


def generate(capture):
    for case_id, path, summary, notes in TABLES:
        table(capture, case_id, path, summary, notes)


if __name__ == "__main__":
    run(generate)
