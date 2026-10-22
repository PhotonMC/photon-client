"""Generate crates/photon-data/src/generated/entity_picks.rs from a movement-family capture.

    python3 tools/generate/entity_picks.py .data/captures/behavior/<id>/outputs.jsonl

What the crosshair needs of each entity type, in id order, as a new entity of
the type answers it: whether it can be aimed at, how much its box grows for
the aim, whether the aim can start inside it, whether it is a `Monster` (which
updates its swing time on its own), its size in the standing pose,
and a baby's size for the types that have babies, and how a leash is tied to it
(whether it can be leashed, by four lines, and held by four lines), where each passenger sits, and where the name tag hangs. The player
makes no entity there, so it has no row of answers.
"""
from common import f32, f64, header, load_case, run, rust_bool, write


def size(row, prefix=""):
    return "(" + ", ".join(f32(row[prefix + key]) for key in ("width", "height", "eye_height")) + ")"


def seats(row):
    points = ["(" + ", ".join(f64(bits) for bits in point) + ")" for point in row["passengers"]]
    return "&[" + ", ".join(points) + "]"


def point(row, key):
    if key not in row:
        return "None"
    return "Some((" + ", ".join(f64(bits) for bits in row[key]) + "))"


def generate(capture):
    types = load_case(capture, "movement/entity-pick-table")["types"]
    if [row["id"] for row in types] != list(range(len(types))):
        raise SystemExit("the registry must come in id order, with no gaps")
    out = header("What the crosshair needs of each entity type, in id order.", "entity_picks.py") + [
        "use crate::entity_type::{EntityPick, Leash};",
        "",
        "#[rustfmt::skip]",
        f"pub static ENTITY_PICKS: [Option<EntityPick>; {len(types)}] = [",
    ]
    for row in types:
        if not row["created"]:
            out.append("    None,")
            continue
        baby = f"Some({size(row, 'baby_')})" if "baby_width" in row else "None"
        leash = f"Some(Leash {{ quad: {rust_bool(row['quad'])} }})" if row.get("leashable") else "None"
        out.append(
            "    Some(EntityPick { "
            f"pickable: {rust_bool(row['pickable'])}, pick_radius: {f32(row['pick_radius'])}, "
            f"from_inside: {rust_bool(row['from_inside'])}, monster: {rust_bool(row['monster'])}, "
            f"size: {size(row)}, baby: {baby}, leash: {leash}, "
            f"quad_holder: {rust_bool(row['quad_holder'])}, passengers: {seats(row)}, "
            f"name_tag: {point(row, 'name_tag')}, baby_name_tag: {point(row, 'baby_name_tag')} }}),")
    out.append("];")
    write("entity_picks.rs", out)
    print(f"entity_picks={len(types)}")


if __name__ == "__main__":
    run(generate)
