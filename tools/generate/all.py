"""Regenerate every table of the data crate from one capture.

    python3 tools/generate/all.py .data/captures/behavior/<id>/outputs.jsonl

The capture must hold the block, model and entity families (a capture of all
the families does). Afterwards run `cargo fmt --all` and the parity comparison;
a new reference version shows up there as a difference to review.
"""
import importlib
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))

GENERATORS = [
    "block_tints", "blocks", "creative_tabs",
    "component_types", "items", "item_components",
    "attributes", "potions", "entity_models",
]


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    for name in GENERATORS:
        importlib.import_module(name).generate(sys.argv[1])


if __name__ == "__main__":
    main()
