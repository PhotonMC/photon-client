"""Generate tests/behavior/fixtures/item-default-patches.txt from a block-family capture.

    python3 tools/generate/item_components.py .data/captures/behavior/<id>/outputs.jsonl

Every item's whole default component patch as a server sends it, from the
reference: a line per item, in registry order, of its name and the patch's
bytes in hex. The game does not read these (it declares the few defaults it
uses, see `items.py`); the parity run reads each one with the port that
reads component patches and writes it back, to check that port against a
patch of every component type.
"""
from common import load_case, run

FIXTURE = "tests/behavior/fixtures/item-default-patches.txt"


def generate(capture):
    data = load_case(capture, "block/item-components")
    with open(FIXTURE, "w", encoding="utf-8") as handle:
        for item in data["items"]:
            handle.write(f"{item['name']} {item['wire']}\n")
    print(f"patches={len(data['items'])}")


if __name__ == "__main__":
    run(generate)
