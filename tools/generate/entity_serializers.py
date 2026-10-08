"""Generate crates/photon-data/src/generated/entity_serializers.rs from an entity-family capture.

    python3 tools/generate/entity_serializers.py .data/captures/behavior/<id>/outputs.jsonl

The entity data serializers in the order the wire numbers them: the name of
the `EntityDataSerializers` field of each, which is also its place in the list.
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/data-serializers")
    if [row["id"] for row in rows] != list(range(len(rows))):
        raise SystemExit("the serializers must come in id order, with no gaps")
    out = header("Every entity data serializer, in id order.", "entity_serializers.py") + [
        "#[rustfmt::skip]",
        f"pub static ENTITY_DATA_SERIALIZERS: [&str; {len(rows)}] = [",
    ]
    for row in rows:
        out.append(f"    {rust_str(row['label'])},")
    out.append("];")
    write("entity_serializers.rs", out)
    print(f"entity_serializers={len(rows)}")


if __name__ == "__main__":
    run(generate)
