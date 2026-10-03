"""Generate crates/voxel-data/src/generated/entity_attributes.rs from an entity-family capture.

    python3 tools/generate/entity_attributes.py .data/captures/behavior/<id>/outputs.jsonl

The attributes each living entity type starts with (`DefaultAttributes`: the
base value of every attribute its supplier has), by entity type name, and each
attribute's default and range, in registry order. The client builds an entity
with these and the server's `UpdateAttributes` packets change them.
"""
from common import f64, header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/attributes")
    types = [row for row in rows if row["kind"] == "type"]
    ranges = [row for row in rows if row["kind"] == "range"]
    if [row["id"] for row in ranges] != list(range(len(ranges))):
        raise SystemExit("the attributes must come in id order, with no gaps")
    out = header("The attributes each living entity type starts with, and each attribute's range.",
                 "entity_attributes.py") + [
        "use crate::entity_type::AttributeRange;",
        "",
        "#[rustfmt::skip]",
        f"pub static ATTRIBUTE_RANGES: [AttributeRange; {len(ranges)}] = [",
    ]
    for row in ranges:
        bounds = f"Some(({f64(row['min'])}, {f64(row['max'])}))" if "min" in row else "None"
        out.append(f"    AttributeRange {{ name: {rust_str(row['label'])}, default: {f64(row['default'])}, range: {bounds} }},")
    out += ["];", "", "#[rustfmt::skip]",
            f"pub static ENTITY_ATTRIBUTES: [(&str, &[(&str, f64)]); {len(types)}] = ["]
    for row in types:
        values = ", ".join(f"({rust_str(pair['name'])}, {f64(pair['base'])})" for pair in row["attributes"])
        out.append(f"    ({rust_str(row['label'])}, &[{values}]),")
    out.append("];")
    write("entity_attributes.rs", out)
    print(f"entity_attributes={len(types)}/{len(ranges)}")


if __name__ == "__main__":
    run(generate)
