"""Generate crates/voxel-data/src/block/destroy.rs from a block-family capture.

    python3 tools/generate/block_destroy.py .data/captures/behavior/<id>/outputs.jsonl

What breaking reads of each state, from the reference: the destroy time, whether
drops need the right tool, whether it is air, and the state the client predicts
in its place. Game master blocks are listed by registry index.
"""
from common import header, index_list, intern, load_case, run, f32, write


def generate(capture):
    data = load_case(capture, "block/destroy-properties")
    traits, rows = intern(block["states"] for block in data["blocks"])
    game_master = [number for number, block in enumerate(data["blocks"]) if block["game_master"]]
    out = header("What breaking a block reads of each state, from the reference.", "block_destroy.py") + [
        "/// One set of destroy traits.",
        "pub struct DestroyTraits {",
        "    /// -1 cannot be broken.",
        "    pub destroy_time: f32,",
        "    pub requires_correct_tool: bool,",
        "    pub air: bool,",
        "    /// The state left in its place: the block of the fluid it holds.",
        "    pub fluid_block: u32,",
        "}",
        "",
        "#[rustfmt::skip]",
        f"pub static TRAITS: [DestroyTraits; {len(traits)}] = [",
    ]
    for key in traits:
        speed, requires, air, fluid = key.split("|")
        out.append(
            f"    DestroyTraits {{ destroy_time: {f32(speed)}, requires_correct_tool: {requires}, "
            f"air: {air}, fluid_block: {fluid} }},"
        )
    out += ["];", "", "/// Each block's traits: one for every state, or one per state.",
            "#[rustfmt::skip]", f"pub static BLOCKS: [&[u16]; {len(rows)}] = ["]
    out += [f"    {index_list(ids)}," for ids in rows]
    out += ["];", "", "/// The registry indices of game master blocks.",
            f"pub static GAME_MASTER: [u16; {len(game_master)}] = "
            + index_list(game_master)[1:] + ";"]
    write("block/destroy.rs", out)
    print(f"traits={len(traits)} blocks={len(rows)} game_master={len(game_master)}")


if __name__ == "__main__":
    run(generate)
