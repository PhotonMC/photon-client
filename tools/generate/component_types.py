"""Generate crates/photon-data/src/generated/item/component_types.rs from a block-family capture.

    python3 tools/generate/component_types.py .data/captures/behavior/<id>/outputs.jsonl

The data component types in registry order, each a named id (`RARITY`),
as the reference's `DataComponents` registers them.
"""
from common import header, load_case, run, rust_str, write


def constant(name):
    return name.split(":")[1].replace("/", "_").upper()


def generate(capture):
    names = load_case(capture, "block/item-declarations")["component_types"]
    out = header("The data component types in registry order, as `DataComponents` registers",
                 "component_types.py", ["them: each a named id."])
    for number, name in enumerate(names):
        out.append(f"pub const {constant(name)}: i32 = {number};")
    out += ["", "/// Every component type's name, by id.", "#[rustfmt::skip]",
            f"pub static NAMES: [&str; {len(names)}] = ["]
    out += [f"    {rust_str(name)}," for name in names]
    out += ["];", "",
            "/// A component type's id by name.",
            "pub fn id(name: &str) -> Option<i32> {",
            "    NAMES.iter().position(|known| *known == name).map(|id| id as i32)",
            "}"]
    write("item/component_types.rs", out)
    print(f"component types={len(names)}")


if __name__ == "__main__":
    run(generate)
