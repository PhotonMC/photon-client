"""Generate the mob effect and potion declarations from a block-family capture.

    python3 tools/generate/potions.py .data/captures/behavior/<id>/outputs.jsonl

Writes `crates/voxel-data/src/item/mob_effects.rs` and `potions.rs`: every mob
effect and potion in registry order, as `MobEffects` and `Potions` declare
them. A reference to an effect or an attribute is by name; an effect's
description key and tooltip colour are checked against what the reference
reports and follow from its name and category.
"""
import struct

from common import f32, header, load_case, run, rust_str, write

CATEGORIES = {"BLUE": "beneficial", "RED": "harmful", "YELLOW": "neutral"}


def amount(value):
    """A modifier amount: the float it was written as, when it is one."""
    as_float = struct.unpack(">f", struct.pack(">f", value))[0]
    if as_float == value:
        bits = f"{struct.unpack('>I', struct.pack('>f', value))[0]:08x}"
        return f"{f32(bits)}_f32 as f64"
    literal = repr(value)
    return literal if "." in literal or "e" in literal else literal + ".0"


def generate(capture):
    data = load_case(capture, "block/potions")
    attributes = [entry["name"] for entry in load_case(capture, "block/tooltip-registries")["attributes"]]
    effects = data["effects"]
    out = header("Every mob effect, in registry order, as the reference's `MobEffects`", "potions.py",
                 ["declares it."]) + [
        "use super::effect::{EffectModifier, MobEffect};",
        "",
        "#[rustfmt::skip]",
        f"pub static MOB_EFFECTS: [MobEffect; {len(effects)}] = [",
    ]
    for effect in effects:
        path = effect["name"].split(":")[1]
        if effect["description_id"] != f"effect.minecraft.{path}":
            raise SystemExit(f"{effect['name']}: a description key that is not derived from the name")
        call = f"MobEffect::{CATEGORIES[effect['style']]}({rust_str(effect['name'])}, 0x{effect['color']:06X})"
        if effect["modifiers"]:
            modifiers = ", ".join(
                f"EffectModifier {{ attribute: {rust_str(attributes[m['attribute']])}, "
                f"amount: {amount(m['amount'])}, operation: {m['operation']} }}"
                for m in effect["modifiers"])
            call += f".modifying(&[{modifiers}])"
        out.append(f"    {call},")
    out.append("];")
    write("item/mob_effects.rs", out)

    names = [effect["name"] for effect in effects]
    potions = data["potions"]
    out = header("Every potion, in registry order, as the reference's `Potions` declares it.",
                 "potions.py") + [
        "use super::effect::{Potion, PotionEffect};",
        "",
        "#[rustfmt::skip]",
        f"pub static POTIONS: [Potion; {len(potions)}] = [",
    ]
    for potion in potions:
        effects_text = ", ".join(
            f"PotionEffect {{ effect: {rust_str(names[e['effect']])}, amplifier: {e['amplifier']}, "
            f"duration: {e['duration']}, visible: {str(e['visible']).lower()} }}" for e in potion["effects"])
        call = f"Potion::of({rust_str(potion['name'])}, &[{effects_text}])"
        if potion["potion_name"] != potion["name"].split(":")[1]:
            call += f".variant_of({rust_str(potion['potion_name'])})"
        out.append(f"    {call},")
    out.append("];")
    write("item/potions.rs", out)
    print(f"effects={len(effects)} potions={len(potions)}")


if __name__ == "__main__":
    run(generate)
