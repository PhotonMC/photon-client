"""Generate crates/photon-data/src/generated/entity_shadows.rs from a movement-family capture.

    python3 tools/generate/entity_shadows.py .data/captures/behavior/<id>/outputs.jsonl

The shadow radius and strength each entity type's renderer sets in its constructor
(`EntityRenderer.shadowRadius`, `shadowStrength`), and the types whose renderer the oracle could
not build.
"""
from common import f32, header, load_rows, run, rust_str, write


def generate(capture):
    row = load_rows(capture, "movement/renderer-shadows")[0]
    types = row["types"]
    out = header("The shadow each entity type's renderer sets up, as (name, radius, strength).",
                 "entity_shadows.py") + [
        "#[rustfmt::skip]",
        f"pub static SHADOWS: [(&str, f32, f32); {len(types)}] = [",
    ]
    for entry in types:
        out.append(f"    ({rust_str(entry['name'])}, {f32(entry['radius'])}, {f32(entry['strength'])}),")
    out += ["];", "", "/// Types whose renderer the oracle could not build, so no shadow is listed for them.",
            "#[rustfmt::skip]", f"pub static UNBUILT: [&str; {len(row['failed'])}] = ["]
    out += [f"    {rust_str(name)}," for name in row["failed"]]
    out.append("];")
    write("entity_shadows.rs", out)
    print(f"entity shadows={len(types)} unbuilt={len(row['failed'])}")


if __name__ == "__main__":
    run(generate)
