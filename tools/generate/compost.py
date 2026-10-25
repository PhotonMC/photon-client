"""Generate crates/photon-data/src/generated/compost.rs from a movement-family capture.

    python3 tools/generate/compost.py .data/captures/behavior/<id>/outputs.jsonl

The items a composter takes and the chance each fills a layer (`ComposterBlock.COMPOSTABLES`).
"""
from common import f32, header, load_rows, run, rust_str, write


def generate(capture):
    items = load_rows(capture, "movement/compostables")[0]["items"]
    out = header("The items a composter takes, with the chance each fills a layer.", "compost.py") + [
        "#[rustfmt::skip]",
        f"pub static COMPOSTABLES: [(&str, f32); {len(items)}] = [",
    ]
    for item in items:
        out.append(f"    ({rust_str(item['name'])}, {f32(item['chance'])}),")
    out.append("];")
    write("compost.rs", out)
    print(f"compostables items={len(items)}")


if __name__ == "__main__":
    run(generate)
