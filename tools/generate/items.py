"""Generate the item declarations from a block-family capture.

    python3 tools/generate/items.py .data/captures/behavior/<id>/outputs.jsonl

Writes two files under crates/photon-data/src/generated/item/:

- `classes.rs`: every item class with the class it extends and the tracked
  methods it declares itself, so a rule is looked up the way the reference
  dispatches a method.
- `items.rs`: every item in registry order as `Items` declares it: a name, a
  class and the `Item.Properties` it sets away from the defaults, with the
  tool components written once each as named statics.

Nothing the game computes from these is written; see `definition.rs`.
"""
import json

from common import f32, header, load_case, run, rust_str, write
from components import convert

METHODS = ["useOn", "use", "overrideStackedOnOther", "overrideOtherStackedOnMe"]


def method_name(method):
    return method[0].upper() + method[1:]


def description_key(item):
    """The translation key an item's name defaults to: its block's for a block item."""
    namespace, path = (item.get("block") or item["name"]).split(":")
    return f"{'block' if item.get('block') else 'item'}.{namespace}.{path}"


def component_calls(item):
    """The `Component` values an item declares, from the capture's JSON."""
    out = []
    for name, value in item["components"].items():
        if name == "minecraft:item_name" and value == {"translate": description_key(item)}:
            continue
        out.append(convert(item["name"], name, value))
    return out


def classes_file(classes):
    out = header("Every item class the registry uses, with the class it extends.", "items.py", [
        "",
        "A rule the reference overrides in some classes is looked up by walking",
        "an item's class towards `Item` until one of them declares it; see",
        "[`ItemClass::owner_of`].",
    ]) + [
        "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]",
        "pub enum ItemClass {",
    ]
    out += [f"    {entry['name']}," for entry in classes]
    out += ["}", "", "impl ItemClass {",
            "    /// Every class, in the order the registry first reaches them.",
            f"    pub const ALL: [Self; {len(classes)}] = ["]
    out += [f"        Self::{entry['name']}," for entry in classes]
    out += ["    ];", "", "    /// The class this one extends; `None` for `Item`.",
            "    pub const fn parent(self) -> Option<Self> {", "        match self {"]
    for entry in classes:
        parent = f"Some(Self::{entry['parent']})" if entry["parent"] else "None"
        out.append(f"            Self::{entry['name']} => {parent},")
    out += ["        }", "    }", "",
            "    /// The tracked methods this class declares itself (overrides).",
            "    pub const fn declares(self) -> &'static [ItemMethod] {", "        match self {"]
    for entry in classes:
        if entry["declares"]:
            listed = ", ".join(f"ItemMethod::{method_name(method)}" for method in entry["declares"])
            out.append(f"            Self::{entry['name']} => &[{listed}],")
    out += ["            _ => &[],", "        }", "    }", "}", "",
            "/// The item methods the client ports class by class.",
            "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]", "pub enum ItemMethod {"]
    out += [f"    {method_name(method)}," for method in METHODS]
    out += ["}"]
    return out


def tool_static(tool):
    def rule(entry):
        blocks = entry["blocks"]
        if isinstance(blocks, str):
            written = f"RuleBlocks::Tag({rust_str(blocks[1:])})"
        else:
            written = "RuleBlocks::Blocks(&[" + ", ".join(rust_str(b) for b in blocks) + "])"
        speed = f"Some({f32(entry['speed'])})" if "speed" in entry else "None"
        correct = entry.get("correct_for_drops")
        correct = "None" if correct is None else f"Some({str(correct).lower()})"
        return f"ToolRule {{ blocks: {written}, speed: {speed}, correct_for_drops: {correct} }}"

    rules = "".join(f"\n        {rule(entry)}," for entry in tool["rules"])
    return (f"Tool {{\n    rules: &[{rules}\n    ],\n"
            f"    default_mining_speed: {f32(tool['default_mining_speed'])},\n"
            f"    damage_per_block: {tool['damage_per_block']},\n"
            f"    can_destroy_blocks_in_creative: {str(tool['can_destroy_blocks_in_creative']).lower()},\n}}")


def items_file(items):
    tools, names = {}, {}
    for item in items:
        tool = item.get("tool")
        if tool is not None:
            key = json.dumps(tool, sort_keys=True)
            if key not in names:
                names[key] = "TOOL_" + item["name"].split(":")[1].upper()
                tools[key] = tool
    out = header("Every item, in registry order, as the reference's `Items` declares it.", "items.py") + [
        "use crate::item::classes::ItemClass;",
        "use crate::item::component::{",
        "    AttributeDisplay, AttributeModifier, Component, Operation, Rarity, SlotGroup,",
        "};",
        "use crate::item::definition::{Equip, Item, RuleBlocks, Tool, ToolRule};",
        "",
    ]
    for key, tool in tools.items():
        out += ["#[rustfmt::skip]", f"static {names[key]}: Tool = {tool_static(tool)};", ""]
    out += ["#[rustfmt::skip]", f"pub static ITEMS: [Item; {len(items)}] = ["]
    for item in items:
        calls = []
        if item["max_stack_size"] != 64:
            calls.append(f"stacks_to({item['max_stack_size']})")
        if item["max_damage"]:
            calls.append(f"durability({item['max_damage']})")
        equip = item.get("equippable")
        if equip:
            calls.append(f"equippable(Equip {{ slot: {rust_str(equip['slot'])}, "
                         f"player: {str(equip['player']).lower()}, "
                         f"swappable: {str(equip['swappable']).lower()}, "
                         f"asset: {rust_opt_str(equip.get('asset'))}, "
                         f"camera_overlay: {rust_opt_str(equip.get('camera_overlay'))} }})")
        remainder = item.get("crafting_remainder")
        if remainder:
            calls.append(f"crafting_remainder({rust_str(remainder['item'])}, {remainder['count']})")
        if item.get("tool") is not None:
            calls.append(f"tool(&{names[json.dumps(item['tool'], sort_keys=True)]})")
        for flag in ("consumable", "blocks_attacks", "kinetic_weapon"):
            if item[flag]:
                calls.append(f"{flag}()")
        if item.get("burn_time"):
            calls.append(f"burns({item['burn_time']})")
        if item.get("brewing_ingredient"):
            calls.append("brews()")
        if item.get("block"):
            calls.append(f"places({rust_str(item['block'])})")
        components = component_calls(item)
        if components:
            calls.append("components(&[" + ", ".join(components) + "])")
        out.append(f"    Item::of({rust_str(item['name'])}, ItemClass::{item['class']})"
                   + "".join(f".{call}" for call in calls) + ",")
    out.append("];")
    return out


def rust_opt_str(text):
    return "None" if text is None else f"Some({rust_str(text)})"


def generate(capture):
    data = load_case(capture, "block/item-declarations")
    write("item/classes.rs", classes_file(data["classes"]))
    write("item/items.rs", items_file(data["items"]))
    print(f"classes={len(data['classes'])} items={len(data['items'])}")


if __name__ == "__main__":
    run(generate)
