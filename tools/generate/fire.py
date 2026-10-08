"""Generate crates/photon-data/src/generated/fire.rs from a movement-family capture.

    python3 tools/generate/fire.py .data/captures/behavior/<id>/outputs.jsonl

The blocks fire catches on (`FireBlock.bootStrap`): each block's ignite and burn odds.
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "movement/fire-flammability")
    blocks = rows[0]["blocks"]
    out = header("The blocks fire catches on, with their ignite and burn odds.", "fire.py") + [
        "#[rustfmt::skip]",
        f"pub static FLAMMABLE: [(&str, u8, u8); {len(blocks)}] = [",
    ]
    for block in blocks:
        out.append(f"    ({rust_str(block['name'])}, {block['ignite']}, {block['burn']}),")
    out.append("];")
    write("fire.rs", out)
    print(f"fire blocks={len(blocks)}")


if __name__ == "__main__":
    run(generate)
