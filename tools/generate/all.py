"""Regenerate every table of the data crate from one capture.

    python3 tools/generate/all.py .data/captures/behavior/<id>/outputs.jsonl

The capture must hold the block and model families (a capture of all the
families does). Afterwards run `cargo fmt --all` and the parity comparison;
a new reference version shows up there as a difference to review.
"""
import importlib
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))

GENERATORS = [
    "block_shapes", "block_tints", "blocks", "creative_tabs",
    "items", "item_components",
    "potions", "tooltip_registries",
]


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    for name in GENERATORS:
        importlib.import_module(name).generate(sys.argv[1])


if __name__ == "__main__":
    main()
