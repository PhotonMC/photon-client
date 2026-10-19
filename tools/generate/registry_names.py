"""Generate crates/photon-data/src/generated/registry_names.rs from a block-family capture.

    python3 tools/generate/registry_names.py .data/parity/26.2/block.jsonl.gz

The block entity type, menu type, stat type and custom stat registries in id order (and how each custom stat is formatted): the name of each
entry, which is also its place in the list (the id the wire uses).
"""
from common import header, load_case, run, rust_str, write


def generate(capture):
    data = load_case(capture, "block/registry-names")
    out = header("The names of the registries whose wire ids the client reads, in id order.", "registry_names.py")
    for constant, key in (
        ("BLOCK_ENTITY_TYPES", "block_entity_types"),
        ("MENU_TYPES", "menu_types"),
        ("STAT_TYPES", "stat_types"),
        ("CUSTOM_STATS", "custom_stats"),
        ("CUSTOM_STAT_FORMATTERS", "custom_stat_formatters"),
    ):
        names = data[key]
        out += ["#[rustfmt::skip]", f"pub static {constant}: [&str; {len(names)}] = ["]
        out += [f"    {rust_str(name)}," for name in names]
        out += ["];", ""]
    write("registry_names.rs", out)
    print(f"block_entity_types={len(data['block_entity_types'])} menu_types={len(data['menu_types'])} custom_stats={len(data['custom_stats'])}")


if __name__ == "__main__":
    run(generate)
