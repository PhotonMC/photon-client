"""Generate crates/voxel-data/src/generated/damage_effects.rs from a sound-family capture.

    python3 tools/generate/damage_effects.py .data/captures/behavior/<id>/outputs.jsonl

What each damage type does to the one it hurts (`DamageType.effects`) and the sound the player
makes for each effect (`DamageEffects.sound`).
"""
from common import header, load_rows, run, rust_str, write


def generate(capture):
    rows = load_rows(capture, "sound/damage-effects")
    effects = [row for row in rows if row["kind"] == "effect"]
    types = [row for row in rows if row["kind"] == "type"]
    out = header("What damage does to the one it hurts.", "damage_effects.py") + [
        "/// Each effect's serialized name and the player's sound for it.",
        "#[rustfmt::skip]",
        f"pub static DAMAGE_EFFECT_SOUNDS: [(&str, &str); {len(effects)}] = [",
    ]
    for row in effects:
        out.append(f"    ({rust_str(row['name'])}, {rust_str(row['sound'])}),")
    out += ["];", "", "/// Each damage type's effect, by the type's registry name.", "#[rustfmt::skip]",
            f"pub static DAMAGE_TYPE_EFFECTS: [(&str, &str); {len(types)}] = ["]
    for row in types:
        out.append(f"    ({rust_str(row['name'])}, {rust_str(row['effect'])}),")
    out.append("];")
    write("damage_effects.rs", out)
    print(f"damage_effects effects={len(effects)} types={len(types)}")


if __name__ == "__main__":
    run(generate)
