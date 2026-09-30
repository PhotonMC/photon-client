"""Generate crates/voxel-data/src/generated/item/attributes.rs from a block-family capture.

    python3 tools/generate/attributes.py .data/captures/behavior/<id>/outputs.jsonl

Every attribute of the built-in registry in registry order, written as
`Attributes` declares it: a name and a sentiment where it is not the
default. The description key and the tooltip colours follow from those and
are checked here against what the reference reports.
"""
from common import header, load_case, run, rust_str, write

SENTIMENTS = {("BLUE", "RED"): "Positive", ("RED", "BLUE"): "Negative", ("GRAY", "GRAY"): "Neutral"}


def generate(capture):
    data = load_case(capture, "block/tooltip-registries")["attributes"]
    out = header("Every attribute, in registry order, as the reference's `Attributes`", "attributes.py",
                 ["declares it."]) + [
        "use crate::item::attribute::{Attribute, Sentiment};",
        "",
        "#[rustfmt::skip]",
        f"pub static ATTRIBUTES: [Attribute; {len(data)}] = [",
    ]
    for entry in data:
        path = entry["name"].split(":")[1]
        if entry["description_id"] != f"attribute.name.{path}":
            raise SystemExit(f"{entry['name']}: a description key that is not derived from the name")
        sentiment = SENTIMENTS[(entry["style_up"], entry["style_down"])]
        call = "" if sentiment == "Positive" else f".sentiment(Sentiment::{sentiment})"
        out.append(f"    Attribute::of({rust_str(entry['name'])}){call},")
    out.append("];")
    write("item/attributes.rs", out)
    print(f"attributes={len(data)}")


if __name__ == "__main__":
    run(generate)
