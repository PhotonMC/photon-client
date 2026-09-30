"""Generate crates/voxel-data/src/block/states.rs from a block-family capture.

    python3 tools/generate/block_states.py .data/captures/behavior/<id>/outputs.jsonl

The block registry: every block in registry order with its properties, its
default state and where its states start, and the tint each block's layers
take outside a world. The lookups on it are hand-written in `registry.rs`.
"""
from common import header, index_list, load_case, run, rust_str, write


def generate(capture):
    data = load_case(capture, "block/state-definitions")
    tinted = load_case(capture, "block/tints")["tinted"]
    properties = sorted({(p["name"], p["kind"], tuple(p["values"]))
                         for block in data["blocks"] for p in block["properties"]})
    index = {key: number for number, key in enumerate(properties)}
    out = header("The block registry, read from the reference.", "block_states.py", [
        "",
        "A block state arrives from the server as one number. Block state numbers",
        "run contiguously in registry order, so each block only needs the number",
        "it starts at; a block's states are the cartesian product of its",
        "properties in order, with the last varying fastest. The lookups that",
        "rely on both are in `registry.rs`, with tests that check them.",
    ]) + [
        "/// What kind of values a property takes.",
        "#[derive(Clone, Copy, PartialEq, Eq, Debug)]",
        "pub enum PropertyKind {",
        "    Boolean,",
        "    Integer,",
        "    Enum,",
        "}",
        "",
        "/// One property, with its values in the order the registry lists them.",
        "pub struct Property {",
        "    pub name: &'static str,",
        "    pub kind: PropertyKind,",
        "    pub values: &'static [&'static str],",
        "}",
        "",
        "/// One block of the registry.",
        "pub struct BlockDefinition {",
        "    pub name: &'static str,",
        "    /// Indices into [`PROPERTIES`], in the block's own order.",
        "    pub properties: &'static [u8],",
        "    /// The default state's value of each property, as an index into its values.",
        "    pub default: &'static [u8],",
        "    /// The number of the block's first state.",
        "    pub first_state_id: u32,",
        "    /// How many states the block has.",
        "    pub state_count: u16,",
        "}",
        "",
        "/// Every distinct property the registry uses.",
        "#[rustfmt::skip]",
        f"pub static PROPERTIES: [Property; {len(properties)}] = [",
    ]
    for name, kind, values in properties:
        written = ", ".join(rust_str(value) for value in values)
        out.append(f"    Property {{ name: {rust_str(name)}, kind: PropertyKind::{kind.capitalize()}, values: &[{written}] }},")
    out += ["];", "", "/// Every block, in registry order.", "#[rustfmt::skip]",
            f"pub static BLOCKS: [BlockDefinition; {len(data['blocks'])}] = ["]
    for block in data["blocks"]:
        keys = [(p["name"], p["kind"], tuple(p["values"])) for p in block["properties"]]
        defaults = [key[2].index(value) for key, value in zip(keys, block["default"])]
        out.append(
            f"    BlockDefinition {{ name: {rust_str(block['name'])}, "
            f"properties: {index_list(index[key] for key in keys)}, default: {index_list(defaults)}, "
            f"first_state_id: {block['first_state_id']}, state_count: {block['state_count']} }},"
        )
    out += ["];", "", "/// The tint each block's layers take outside a world, as packed colours.",
            "///", "/// Only the blocks the reference registers a tint source for appear here.",
            "#[rustfmt::skip]", f"pub static TINTS: [(&str, &[u32]); {len(tinted)}] = ["]
    for entry in tinted:
        layers = ", ".join(f"0x{layer.upper()}" for layer in entry["layers"])
        out.append(f"    ({rust_str(entry['name'])}, &[{layers}]),")
    out.append("];")
    write("block/states.rs", out)
    print(f"properties={len(properties)} blocks={len(data['blocks'])} tinted={len(tinted)}")


if __name__ == "__main__":
    run(generate)
