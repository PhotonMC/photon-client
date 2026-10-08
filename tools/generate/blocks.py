"""Generate the block declarations from a block-family capture.

    python3 tools/generate/blocks.py .data/captures/behavior/<id>/outputs.jsonl

Writes three files under crates/photon-data/src/generated/block/:

- `properties.rs`: every state property, named after the field that holds it
  in the reference (`BlockStateProperties.HORIZONTAL_FACING`).
- `classes.rs`: every block class with the class it extends, so a rule can be
  looked up the way the reference dispatches a method.
- `blocks.rs`: every block as the reference's `Blocks` declares it: a name, a
  class, its properties and the `Properties` it sets differently from
  `Properties.of()`.

Nothing a state computes from its shapes is written; see `definition.rs`.
"""
import re

from common import f32, header, load_case, run, rust_str, write

DEFAULTS = {
    "destroyTime": "00000000",
    "explosionResistance": "00000000",
    "friction": "3f19999a",
    "speedFactor": "3f800000",
    "jumpFactor": "3f800000",
    "bounceRestitution": "00000000",
}

# `Properties` flags whose builder sets them away from the default.
FLAGS = [
    ("requiresCorrectToolForDrops", True, "requires_correct_tool_for_drops"),
    ("isRandomlyTicking", True, "random_ticks"),
    ("isAir", True, "air"),
    ("ignitedByLava", True, "ignited_by_lava"),
    ("liquid", True, "liquid"),
    ("forceSolidOff", True, "force_solid_off"),
    ("forceSolidOn", True, "force_solid_on"),
    ("spawnTerrainParticles", False, "no_terrain_particles"),
    ("replaceable", True, "replaceable"),
    ("dynamicShape", True, "dynamic_shape"),
]

FLUIDS = {
    "minecraft:water": "Water",
    "minecraft:flowing_water": "FlowingWater",
    "minecraft:lava": "Lava",
    "minecraft:flowing_lava": "FlowingLava",
}

RULES = [
    ("emissiveRendering", "emissive_rendering"),
    ("isSuffocating", "is_suffocating"),
    ("isViewBlocking", "is_view_blocking"),
    ("isRedstoneConductor", "is_redstone_conductor"),
]


def constant(field):
    """The Rust name of the static a property field becomes."""
    if "." not in field:
        return field
    owner, name = field.split(".")
    return re.sub(r"(?<!^)(?=[A-Z])", "_", owner).upper() + "_" + name


def rule(values):
    if all(values):
        return "Rule::Always"
    if not any(values):
        return "Rule::Never"
    return "Rule::States(&[" + ", ".join("true" if value else "false" for value in values) + "])"


def emission(levels):
    if set(levels) == {0}:
        raise SystemExit("a light function that is never on would read back as the default")
    if len(set(levels)) == 1:
        return f"Emission::Constant({levels[0]})"
    return "Emission::States(&[" + ", ".join(str(level) for level in levels) + "])"


def chain(block):
    """The builder calls a block's declaration makes past `Block::of`."""
    declared = block["declared"]
    calls = []
    if block["properties"]:
        properties = ", ".join("&" + constant(field) for field in block["properties"])
        defaults = ", ".join(rust_str(value) for value in block["default"])
        calls.append(f"properties(&[{properties}], &[{defaults}])")
    if (declared["destroyTime"], declared["explosionResistance"]) != ("00000000", "00000000"):
        calls.append(f"strength({f32(declared['destroyTime'])}, {f32(declared['explosionResistance'])})")
    for field, method in [("friction", "friction"), ("speedFactor", "speed_factor"),
                          ("jumpFactor", "jump_factor"), ("bounceRestitution", "bounce_restitution")]:
        if declared[field] != DEFAULTS[field]:
            calls.append(f"{method}({f32(declared[field])})")
    if not declared["hasCollision"]:
        calls.append("no_collision()")
    elif not declared["canOcclude"]:
        calls.append("no_occlusion()")
    for field, when, method in FLAGS:
        if declared[field] == when:
            calls.append(f"{method}()")
    if declared["offsetType"] != "NONE":
        calls.append(f"offset_type(OffsetType::{declared['offsetType'].capitalize()})")
    if declared["pushReaction"] != "NORMAL":
        reaction = "".join(part.capitalize() for part in declared["pushReaction"].split("_"))
        calls.append(f"push_reaction(PushReaction::{reaction})")
    if block["item"] != block["name"]:
        calls.append(f"item({rust_str(block['item'])})")
    if "fluid" in block:
        calls.append(f"fluid(Fluid::{FLUIDS[block['fluid']]})")
    per_state = block["per_state"]
    if per_state.get("lightEmission") is not None:
        calls.append(f"light_level({emission(per_state['lightEmission'])})")
    for field, method in RULES:
        if per_state.get(field) is not None:
            calls.append(f"{method}({rule(per_state[field])})")
    return calls


def properties_file(properties):
    out = header("Every block state property, named after the field that holds it in the",
                 "blocks.py", ["reference."]) + [
        "use crate::block::definition::{Property, PropertyKind};",
        "",
    ]
    for prop in properties:
        values = ", ".join(rust_str(value) for value in prop["values"])
        out += [
            "#[rustfmt::skip]",
            f"pub static {constant(prop['field'])}: Property = Property {{ name: {rust_str(prop['name'])}, "
            f"kind: PropertyKind::{prop['kind'].capitalize()}, values: &[{values}] }};",
        ]
    out += ["", "/// Every property with the name of the field that holds it, in the order",
            "/// the registry first uses them.", "#[rustfmt::skip]",
            f"pub static FIELDS: [(&str, &Property); {len(properties)}] = ["]
    out += [f"    ({rust_str(prop['field'])}, &{constant(prop['field'])})," for prop in properties]
    out.append("];")
    return out


def classes_file(classes, interfaces):
    out = header("Every block class the registry uses, with the class it extends.", "blocks.py", [
        "",
        "A rule the reference overrides in some classes is looked up by walking",
        "a block's class towards `Block` until one of them has its own; see",
        "[`BlockClass::lineage`].",
    ]) + [
        "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]",
        "pub enum BlockClass {",
    ]
    out += [f"    {entry['name']}," for entry in classes]
    out += ["}", "", "impl BlockClass {",
            "    /// Every class, in the order the registry first reaches them.",
            f"    pub const ALL: [Self; {len(classes)}] = ["]
    out += [f"        Self::{entry['name']}," for entry in classes]
    out += ["    ];", "", "    /// The class this one extends; `None` for `Block`.",
            '    #[expect(clippy::too_many_lines, reason = "one arm per class")]',
            "    pub const fn parent(self) -> Option<Self> {", "        match self {"]
    for entry in classes:
        parent = f"Some(Self::{entry['parent']})" if entry["parent"] else "None"
        out.append(f"            Self::{entry['name']} => {parent},")
    out += ["        }", "    }", "",
            "    /// The interfaces this class itself declares it implements.",
            '    #[expect(clippy::too_many_lines, reason = "one arm per class")]',
            "    pub const fn interfaces(self) -> &'static [BlockInterface] {", "        match self {"]
    for entry in classes:
        if entry["interfaces"]:
            listed = ", ".join(f"BlockInterface::{name}" for name in entry["interfaces"])
            out.append(f"            Self::{entry['name']} => &[{listed}],")
    out += ["            _ => &[],", "        }", "    }", "",
            "    /// The tracked methods this class declares itself (overrides).",
            '    #[expect(clippy::too_many_lines, reason = "one arm per class")]',
            "    pub const fn declares(self) -> &'static [Method] {", "        match self {"]
    for entry in classes:
        if entry["declares"]:
            listed = ", ".join(f"Method::{method[0].upper()}{method[1:]}" for method in entry["declares"])
            out.append(f"            Self::{entry['name']} => &[{listed}],")
    out += ["            _ => &[],", "        }", "    }", "}", "",
            "/// The block methods the client ports class by class.",
            "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]", "pub enum Method {"]
    methods = []
    for entry in classes:
        for method in entry["declares"]:
            if method not in methods:
                methods.append(method)
    out += [f"    {method[0].upper()}{method[1:]}," for method in sorted(methods)]
    out += ["}", "",
            "/// Every interface a block class implements, directly or through another.",
            "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]", "pub enum BlockInterface {"]
    out += [f"    {entry['name']}," for entry in interfaces]
    out += ["}", "", "impl BlockInterface {",
            "    /// Every interface, in the order the registry first reaches them.",
            f"    pub const ALL: [Self; {len(interfaces)}] = ["]
    out += [f"        Self::{entry['name']}," for entry in interfaces]
    out += ["    ];", "", "    /// The interfaces this one extends.",
            "    pub const fn parents(self) -> &'static [Self] {", "        match self {"]
    for entry in interfaces:
        if entry["parents"]:
            listed = ", ".join(f"Self::{name}" for name in entry["parents"])
            out.append(f"            Self::{entry['name']} => &[{listed}],")
    out += ["            _ => &[],", "        }", "    }", "}"]
    return out


def blocks_file(blocks):
    out = header("Every block, in registry order, as the reference's `Blocks` declares it.",
                 "blocks.py") + [
        "use crate::block::classes::BlockClass;",
        "use crate::block::definition::{Block, Emission, Fluid, OffsetType, PushReaction, Rule};",
        "use crate::block::properties::*;",
        "",
        "#[rustfmt::skip]",
        f"pub static BLOCKS: [Block; {len(blocks)}] = [",
    ]
    for block in blocks:
        head = f"    Block::of({rust_str(block['name'])}, BlockClass::{block['class']})"
        out.append(head + "".join(f".{call}" for call in chain(block)) + ",")
    out += ["];", "", "/// Each block's place in the registry, named as `Blocks` names it.",
            "#[rustfmt::skip]", "pub mod id {"]
    for index, block in enumerate(blocks):
        out.append(f"    pub const {block['name'].split(':')[1].upper()}: usize = {index};")
    out.append("}")
    return out


def generate(capture):
    data = load_case(capture, "block/declarations")
    write("block/properties.rs", properties_file(data["properties"]))
    write("block/classes.rs", classes_file(data["classes"], data["interfaces"]))
    write("block/blocks.rs", blocks_file(data["blocks"]))
    print(f"properties={len(data['properties'])} classes={len(data['classes'])} blocks={len(data['blocks'])}")


if __name__ == "__main__":
    run(generate)
