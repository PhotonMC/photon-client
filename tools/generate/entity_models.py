"""Generate crates/photon-data/src/generated/entity_models.rs from an entity-family capture.

    python3 tools/generate/entity_models.py .data/captures/behavior/<id>/outputs.jsonl

Every model layer the reference builds (`LayerDefinitions.createRoots`), by
the name of its `ModelLayerLocation`: the size of its texture, and the tree
of its parts, each with its pose, its cubes and its children by name. Written
as the builders read: `PartPose.offset(...)`, `CubeListBuilder`-like cubes with
only what they set apart from the defaults.
"""
from common import f32, header, load_rows, run, rust_str, write

FACES = {"down": "Down", "east": "East", "north": "North", "south": "South", "up": "Up", "west": "West"}
ALL = ["down", "east", "north", "south", "up", "west"]


def vector(bits):
    return "[" + ", ".join(f32(b) for b in bits) + "]"


def pose(bits):
    position, rotation, scale = bits[0:3], bits[3:6], bits[6:9]
    zero = f32("00000000")
    one = f32("3f800000")
    values = [f32(b) for b in bits]
    if values[6:9] != [one, one, one]:
        return f"PartPose::new({vector(position)}, {vector(rotation)}, {vector(scale)})"
    if values[3:6] == [zero, zero, zero]:
        if values[0:3] == [zero, zero, zero]:
            return "PartPose::ZERO"
        return f"PartPose::offset({vector(position)})"
    if values[0:3] == [zero, zero, zero]:
        return f"PartPose::rotation({vector(rotation)})"
    return f"PartPose::offset_and_rotation({vector(position)}, {vector(rotation)})"


def cube(data):
    text = f"ModelCube::of({vector(data['uv'])}, {vector(data['origin'])}, {vector(data['dimensions'])})"
    zero, one = f32("00000000"), f32("3f800000")
    if [f32(b) for b in data["grow"]] != [zero, zero, zero]:
        text += f".grow({vector(data['grow'])})"
    if data["mirror"]:
        text += ".mirror()"
    if [f32(b) for b in data["uv_scale"]] != [one, one]:
        text += f".uv_scale({vector(data['uv_scale'])})"
    if data["faces"] != ALL:
        text += ".faces(&[" + ", ".join("Face::" + FACES[face] for face in data["faces"]) + "])"
    return text


def part(data, depth):
    indent = "    " * depth
    cubes = ", ".join(cube(c) for c in data["cubes"])
    head = f"ModelPart::of({rust_str(data['name'])}, {pose(data['pose'])}, &[{cubes}], &["
    if not data["children"]:
        return head + "])"
    lines = [head]
    for child in data["children"]:
        lines.append(f"{indent}    {part(child, depth + 1)},")
    lines.append(f"{indent}])")
    return "\n".join(lines)


def generate(capture):
    rows = load_rows(capture, "entity/model-layers")
    ids = [row["label"] for row in rows]
    if ids != sorted(ids):
        raise SystemExit("the layers must come in order of id, for the lookup to search them")
    out = header("Every model layer the reference builds, in order of id.", "entity_models.py",
                 ["The tree of each is as `LayerDefinitions` assembles it: poses, cubes and children",
                  "by name."]) + [
        '#![allow(clippy::approx_constant, reason = "the reference writes 1.5708 and 3.1416 for its angles")]',
        "",
        "use crate::entity_model::{Face, ModelCube, ModelLayer, ModelPart, PartPose};",
        "",
        "#[rustfmt::skip]",
        f"pub static LAYERS: [ModelLayer; {len(rows)}] = [",
    ]
    for row in rows:
        width, height = row["texture_width"], row["texture_height"]
        out.append(f"    ModelLayer::of({rust_str(row['label'])}, {width}, {height}, {part(row['root'], 1)}),")
    out.append("];")
    write("entity_models.rs", out)
    print(f"layers={len(rows)}")


if __name__ == "__main__":
    run(generate)
