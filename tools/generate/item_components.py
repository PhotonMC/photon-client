"""Generate tests/behavior/fixtures/item-default-patches.txt and
crates/photon-data/src/generated/item/component_sets.rs from a block-family capture.

    python3 tools/generate/item_components.py .data/captures/behavior/<id>/outputs.jsonl

Every item's whole default component patch as a server sends it, from the
reference: a line per item, in registry order, of its name and the patch's
bytes in hex. The game does not read these (it declares the few defaults it
uses, see `items.py`); the parity run reads each one with the port that
reads component patches and writes it back, to check that port against a
patch of every component type.

`component_sets.rs` is each item's own component types, transient ones too,
as a bit set: what `ItemStack`'s component count (the advanced tooltip's last
line) starts from.
"""
from common import header, load_case, run, write

FIXTURE = "tests/behavior/fixtures/item-default-patches.txt"


def generate(capture):
    data = load_case(capture, "block/item-components")
    with open(FIXTURE, "w", encoding="utf-8") as handle:
        for item in data["items"]:
            handle.write(f"{item['name']} {item['wire']}\n")
    out = header("Each item's own component types, by type id, as a bit set.", "item_components.py") + [
        "/// Per item, in registry order: bit `n` is set when the item's own components have the",
        "/// type of id `n`.",
        "#[rustfmt::skip]",
        f"pub static DEFAULT_COMPONENT_SETS: [u128; {len(data['items'])}] = [",
    ]
    for item in data["items"]:
        bits = sum(1 << kind for kind in item["types"])
        out.append(f"    0x{bits:x}, // {item['name']}")
    out.append("];")
    write("item/component_sets.rs", out)
    print(f"patches={len(data['items'])}")


if __name__ == "__main__":
    run(generate)
