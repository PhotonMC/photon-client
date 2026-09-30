"""Generate crates/voxel-data/src/item/behaviour.rs from a block-family capture.

    python3 tools/generate/item_behaviour.py .data/captures/behavior/<id>/outputs.jsonl

Each item's class, the classes whose own `useOn` and `use` it runs, whether
the default `Item.use` does nothing for its default stack, and the block a
block item places.
"""

from common import header, load_case, run, rust_str, write


def generate(capture):
    data = load_case(capture, "block/behaviours")
    out = header("Which class's own rule each item runs, from the reference.", "item_behaviour.py") + [
        "pub struct ItemBehaviour {",
        "    pub class: &'static str,",
        "    /// The classes whose own `useOn` and `use` the item runs.",
        "    pub use_on: &'static str,",
        "    pub use_: &'static str,",
        "    /// Whether the default `Item.use` does nothing for the default stack.",
        "    pub default_use_pass: bool,",
        "    /// The block a block item places.",
        "    pub block: Option<&'static str>,",
        "}",
        "",
        "#[rustfmt::skip]",
        f"pub static ITEMS: [ItemBehaviour; {len(data['items'])}] = [",
    ]
    for item in data["items"]:
        block = item.get("block")
        placed = "None" if block is None else f"Some({rust_str(block)})"
        out.append(
            f"    ItemBehaviour {{ class: {rust_str(item['class'])}, use_on: {rust_str(item['use_on'])}, "
            f"use_: {rust_str(item['use'])}, default_use_pass: {str(item['default_use_pass']).lower()}, "
            f"block: {placed} }}, // {item['name']}"
        )
    out.append("];")
    write("item/behaviour.rs", out)
    print(f"items={len(data['items'])}")


if __name__ == "__main__":
    run(generate)
