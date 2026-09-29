"""Generate crates/voxel-data/src/item/stacking.rs from a block-family capture.

    python3 tools/generate/item_stacking.py .data/captures/behavior/<id>/outputs.jsonl

What container clicks read from every item's defaults, in registry order, from
the reference. The capture is local; the generated table is committed, and the
parity run compares it with the reference on every pass.
"""

from common import header, load_case, run, rust_bool, rust_str, write


def generate(capture):
    data = load_case(capture, "block/item-stacking")
    out = header("What container clicks read from every item's defaults, in registry",
                 "item_stacking.py", ["order, from the reference."]) + [
        "/// An `equippable` component.",
        "pub struct Equip {",
        "    /// The equipment slot's name (`head`, `chest`...).",
        "    pub slot: &'static str,",
        "    /// A player may wear it.",
        "    pub player: bool,",
        "}",
        "",
        "/// The item crafting leaves behind, and how many.",
        "pub struct Remainder {",
        "    pub item: &'static str,",
        "    pub count: i32,",
        "}",
        "",
        "pub struct ItemStacking {",
        "    pub max_stack_size: i32,",
        "    /// Zero when the item has no durability.",
        "    pub max_damage: i32,",
        "    pub equip: Option<Equip>,",
        "    pub crafting_remainder: Option<Remainder>,",
        "    /// The classes whose `overrideStackedOnOther` and",
        "    /// `overrideOtherStackedOnMe` the item runs.",
        "    pub stacked_on_other: &'static str,",
        "    pub other_stacked_on_me: &'static str,",
        "}",
        "",
        "#[rustfmt::skip]",
        f"pub static ITEM_STACKING: [ItemStacking; {len(data['items'])}] = [",
    ]
    for item in data["items"]:
        equip = "None"
        if "equip_slot" in item:
            equip = f'Some(Equip {{ slot: {rust_str(item["equip_slot"])}, player: {rust_bool(item["player_can_equip"])} }})'
        remainder = "None"
        if "crafting_remainder" in item:
            remainder = (f'Some(Remainder {{ item: {rust_str(item["crafting_remainder"])}, '
                         f'count: {item["crafting_remainder_count"]} }})')
        out.append(
            f'    ItemStacking {{ max_stack_size: {item["max_stack_size"]}, max_damage: {item["max_damage"]}, '
            f'equip: {equip}, crafting_remainder: {remainder}, '
            f'stacked_on_other: "{item["stacked_on_other"]}", other_stacked_on_me: "{item["other_stacked_on_me"]}" }}, '
            f'// {item["name"]}'
        )
    out.append("];")
    write("item/stacking.rs", out)


if __name__ == "__main__":
    run(generate)
