"""Generate crates/photon-data/src/generated/sound_events.rs from a sound-family capture.

    python3 tools/generate/sound_events.py .data/captures/behavior/<id>/outputs.jsonl

The sound event registry in id order: the name of each event, which is also its place in the
list (the id the sound packets carry).
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "sound/events")
    if [row["id"] for row in rows] != list(range(len(rows))):
        raise SystemExit("the registry must come in id order, with no gaps")
    out = header("Every sound event of the built-in registry, in id order.", "sound_events.py") + [
        "#[rustfmt::skip]",
        f"pub static SOUND_EVENTS: [&str; {len(rows)}] = [",
    ]
    for row in rows:
        out.append(f"    {rust_str(row['label'])},")
    out.append("];")
    write("sound_events.rs", out)
    print(f"sound_events={len(rows)}")


if __name__ == "__main__":
    run(generate)
