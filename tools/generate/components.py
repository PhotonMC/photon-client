"""Component values from the reference's JSON as `Component` Rust expressions.

Shared by the item and creative tab generators: both declare the components
the client reads, as typed values, from the JSON each component's own codec
writes.
"""
import struct

from common import amount, f32, rust_str

RARITIES = {"common": "Common", "uncommon": "Uncommon", "rare": "Rare", "epic": "Epic"}
OPERATIONS = {"add_value": "AddValue", "add_multiplied_base": "AddMultipliedBase",
              "add_multiplied_total": "AddMultipliedTotal"}
SLOTS = {"any": "Any", "mainhand": "MainHand", "offhand": "OffHand", "hand": "Hand", "feet": "Feet",
         "legs": "Legs", "chest": "Chest", "head": "Head", "armor": "Armor", "body": "Body",
         "saddle": "Saddle"}

#: The names a generated file may need imported beside `Component`.
SUPPORT = ["AttributeDisplay", "AttributeModifier", "Operation", "Rarity", "SlotGroup"]


def float_literal(value):
    bits = f"{struct.unpack('>I', struct.pack('>f', value))[0]:08x}"
    return f32(bits)


def convert(owner, name, value):
    """The `Component` expression for one component; `owner` names the item or
    stack for messages."""
    kind = name.split(":")[1]
    if kind == "item_name":
        return f"Component::ItemName({rust_str(value['translate'])})"
    if kind == "rarity":
        return f"Component::Rarity(Rarity::{RARITIES[value]})"
    if kind == "attribute_modifiers":
        entries = []
        for entry in value:
            display = entry.get("display")
            if display not in (None, {"type": "hidden"}):
                raise SystemExit(f"{owner}: attribute display {display}")
            entries.append(
                f"AttributeModifier {{ attribute: {rust_str(entry['type'])}, id: {rust_str(entry['id'])}, "
                f"amount: {amount(entry['amount'])}, operation: Operation::{OPERATIONS[entry['operation']]}, "
                f"slot: SlotGroup::{SLOTS[entry['slot']]}, display: AttributeDisplay::"
                f"{'Default' if display is None else 'Hidden'} }}")
        return "Component::AttributeModifiers(&[" + ", ".join(entries) + "])"
    if kind == "enchantment_glint_override":
        return f"Component::EnchantmentGlintOverride({str(value).lower()})"
    if kind == "map_color":
        return f"Component::MapColor(0x{value:06X})"
    if kind == "potion_contents":
        if set(value) - {"potion"}:
            raise SystemExit(f"{owner}: potion contents {value}")
        potion = value.get("potion")
        return f"Component::PotionContents({'None' if potion is None else f'Some({rust_str(potion)})'})"
    if kind == "potion_duration_scale":
        return f"Component::PotionDurationScale({float_literal(value)})"
    if kind == "use_cooldown":
        if set(value) != {"seconds"}:
            raise SystemExit(f"{owner}: use cooldown {value}")
        return f"Component::UseCooldown({float_literal(value['seconds'])})"
    if kind == "fireworks":
        if set(value) != {"flight_duration"}:
            raise SystemExit(f"{owner}: fireworks {value}")
        return f"Component::Fireworks {{ flight_duration: {value['flight_duration']} }}"
    if kind == "instrument":
        return f"Component::Instrument({rust_str(value)})"
    if kind == "jukebox_playable":
        return f"Component::JukeboxPlayable({rust_str(value)})"
    if kind == "block_state":
        pairs = ", ".join(f"({rust_str(k)}, {rust_str(v)})" for k, v in value.items())
        return f"Component::BlockState(&[{pairs}])"
    if kind == "ominous_bottle_amplifier":
        return f"Component::OminousBottleAmplifier({value})"
    if kind == "stored_enchantments":
        pairs = ", ".join(f"({rust_str(k)}, {v})" for k, v in value.items())
        return f"Component::StoredEnchantments(&[{pairs}])"
    if kind == "painting/variant":
        return f"Component::PaintingVariant({rust_str(value)})"
    if kind == "suspicious_stew_effects":
        pairs = ", ".join(f"({rust_str(e['id'])}, {e['duration']})" for e in value)
        return f"Component::SuspiciousStewEffects(&[{pairs}])"
    if kind == "banner_patterns":
        pairs = ", ".join(f"({rust_str(e['pattern'])}, {rust_str(e['color'])})" for e in value)
        return f"Component::BannerPatterns(&[{pairs}])"
    if kind == "tooltip_display":
        if set(value) != {"hidden_components"}:
            raise SystemExit(f"{owner}: tooltip display {value}")
        names = ", ".join(rust_str(n) for n in value["hidden_components"])
        return f"Component::TooltipDisplay(&[{names}])"
    raise SystemExit(f"{owner}: component {name} is not modelled")
