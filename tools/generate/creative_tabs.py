"""Generate crates/voxel-data/src/generated/item/creative_tabs.rs from a block-family capture.

    python3 tools/generate/creative_tabs.py .data/captures/behavior/<id>/outputs.jsonl

The creative inventory's tabs and what each shows, in the reference's order.
A stack is written as its item and the components it carries beyond the
item's defaults, in the order the reference's patch lists them, from the
JSON each component's own codec writes.
"""

from common import header, load_case, run, rust_str, write
from components import SUPPORT, convert


def generate(capture):
    data = load_case(capture, "block/creative-tabs")
    names = load_case(capture, "block/item-declarations")["component_types"]

    def stack(entry):
        if entry["negative"]:
            raise SystemExit(f"{entry['item']}: a stack that removes components")
        order = [names[component] for component in entry["positive"]]
        if sorted(order) != sorted(entry["components"]):
            raise SystemExit(f"{entry['item']}: the patch and its components disagree")
        expr = f"CreativeStack::of({rust_str(entry['item'])})"
        if order:
            parts = [convert(entry["item"], name, entry["components"][name]) for name in order]
            expr += ".with(&[" + ", ".join(parts) + "])"
        return expr

    body = []
    for tab in data["tabs"]:
        items = [stack(item) for item in tab["items"]]
        search = [stack(item) for item in tab["search_items"]]
        body.append("    CreativeTab {")
        body.append(f'        name: {rust_str(tab["name"])}, kind: TabKind::{tab["type"].capitalize()}, '
                    f'top: {str(tab["row"] == "TOP").lower()}, column: {tab["column"]},')
        body.append(
            f'        aligned_right: {str(tab["aligned_right"]).lower()}, '
            f'show_title: {str(tab["show_title"]).lower()}, '
            f'can_scroll: {str(tab["can_scroll"]).lower()}, should_display: {str(tab["should_display"]).lower()},')
        body.append(f'        background: {rust_str(tab["background"])}, title: {rust_str(tab["title"])},')
        body.append(f"        icon: {stack(tab['icon'])},")
        body.append("        items: &[")
        body += [f"            {item}," for item in items]
        body.append("        ],")
        body.append("        search_items: &[")
        if search != items:
            body += [f"            {item}," for item in search]
        body.append("        ],")
        body.append("    },")

    text = "\n".join(body)
    used = [name for name in SUPPORT if f"{name}" in text]
    imports = ", ".join(["Component"] + used)
    out = header("The creative inventory's tabs and what each shows, in the reference's",
                 "creative_tabs.py", [
        "order, for the default features and a player without operator",
        "permissions."]) + [
        "pub use crate::item::creative::{CreativeStack, CreativeTab, TabKind};",
        f"use crate::item::component::{{{imports}}};",
        "",
        f"pub const DEFAULT_TAB: &str = {rust_str(data['default_tab'])};",
        "",
        "#[rustfmt::skip]",
        f"pub static CREATIVE_TABS: [CreativeTab; {len(data['tabs'])}] = [",
    ] + body + ["];"]
    write("item/creative_tabs.rs", out)
    print(f"tabs={len(data['tabs'])} stacks={sum(len(t['items']) + len(t['search_items']) for t in data['tabs'])}")


if __name__ == "__main__":
    run(generate)
