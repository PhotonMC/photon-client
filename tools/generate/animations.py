"""Generate crates/voxel-data/src/generated/animations.rs from an entity-family capture.

    python3 tools/generate/animations.py .data/captures/behavior/<id>/outputs.jsonl

Every keyframe animation definition of the reference's `definitions` package, by
`Class.FIELD`: its length, whether it loops, and for each bone (in the order the
reference keeps them) each channel's target and keyframes. A model is posed from
them by the keyframe player; the parity run compares the table with the
reference's on every pass.
"""
from common import f32, header, load_rows, run, rust_str, write

TARGETS = {"POSITION": "Position", "ROTATION": "Rotation", "SCALE": "Scale"}
INTERPOLATIONS = {"LINEAR": "Linear", "CATMULLROM": "CatmullRom"}


def vector(bits):
    return "[" + ", ".join(f32(value) for value in bits) + "]"


def keyframe(frame):
    if frame["interpolation"] not in INTERPOLATIONS:
        raise SystemExit(f"unknown interpolation {frame['interpolation']}")
    interpolation = INTERPOLATIONS[frame["interpolation"]]
    if frame["pre"] == frame["post"]:
        return f"Keyframe::new({f32(frame['time'])}, {vector(frame['post'])}, Interpolation::{interpolation})"
    return (f"Keyframe::split({f32(frame['time'])}, {vector(frame['pre'])}, "
            f"{vector(frame['post'])}, Interpolation::{interpolation})")


def generate(capture):
    rows = load_rows(capture, "entity/animation-definitions")
    out = header("Every keyframe animation definition of the reference.", "animations.py") + [
        '#![expect(clippy::approx_constant, reason = "the reference\'s own values: some are close to pi")]',
        "",
        "use crate::animation::{Bone, Channel, Definition, Interpolation, Keyframe, Target};",
        "",
        "#[rustfmt::skip]",
        f"pub static ANIMATIONS: [Definition; {len(rows)}] = [",
    ]
    for row in rows:
        out.append("    Definition {")
        out.append(f"        name: {rust_str(row['label'])}, length: {f32(row['length'])}, looping: {'true' if row['looping'] else 'false'},")
        out.append("        bones: &[")
        for bone in row["bones"]:
            out.append(f"            Bone {{ name: {rust_str(bone['name'])}, channels: &[")
            for channel in bone["channels"]:
                if channel["target"] not in TARGETS:
                    raise SystemExit(f"unknown target {channel['target']}")
                out.append(f"                Channel {{ target: Target::{TARGETS[channel['target']]}, keyframes: &[")
                for frame in channel["keyframes"]:
                    out.append(f"                    {keyframe(frame)},")
                out.append("                ] },")
            out.append("            ] },")
        out.append("        ],")
        out.append("    },")
    out.append("];")
    write("animations.rs", out)
    print(f"animations={len(rows)}")


if __name__ == "__main__":
    run(generate)
