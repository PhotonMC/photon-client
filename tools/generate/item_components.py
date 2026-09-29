"""Generate crates/voxel-data/src/item/components.rs from a block-family capture.

    python3 tools/generate/item_components.py .data/captures/behavior/<id>/outputs.jsonl

Every item's default components as a component patch, as a server sends it,
in registry order, from the reference. The port that reads patches is run
over each one on every parity pass, and its writing of the delimited form
is compared with the reference's. The capture is local; the generated table
is committed.
"""

from common import byte_slice, header, load_case, run, write


def generate(capture):
    data = load_case(capture, "block/item-components")
    out = header(
        "Every item's default components as a component patch, in registry order,",
        "item_components.py",
        ["from the reference: the bytes a server sends from the positive component",
         "count on."],
    ) + [
        "#[rustfmt::skip]",
        f"pub static DEFAULT_PATCHES: [&[u8]; {len(data['items'])}] = [",
    ]
    for item in data["items"]:
        out.append(f'    {byte_slice(bytes.fromhex(item["wire"]))}, // {item["name"]}')
    out.append("];")
    write("item/components.rs", out)


if __name__ == "__main__":
    run(generate)
