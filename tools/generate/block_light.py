"""Generate crates/voxel-data/src/block/light.rs from a block-family capture.

    python3 tools/generate/block_light.py .data/captures/behavior/<id>/outputs.jsonl

What the block renderer's lighting reads of each state, from the reference.
"""
from common import f32, header, index_list, intern, load_case, run, write


def generate(capture):
    data = load_case(capture, "block/light-properties")
    traits, rows = intern(block["states"] for block in data["blocks"])
    out = header("What the block renderer's lighting reads of each state, from the reference.",
                 "block_light.py") + [
        "/// One set of lighting traits.",
        "pub struct LightTraits {",
        "    pub emission: u8,",
        "    /// Drawn at full brightness whatever the light around it.",
        "    pub emissive: bool,",
        "    /// The ambient occlusion shade it casts.",
        "    pub shade: f32,",
        "    pub view_blocking: bool,",
        "    pub dampening: u8,",
        "    pub solid_render: bool,",
        "    pub collision_full_block: bool,",
        "    /// Light is stopped by the shape rather than the whole block.",
        "    pub use_shape_for_light_occlusion: bool,",
        "    pub can_occlude: bool,",
        "}",
        "",
        "#[rustfmt::skip]",
        f"pub static TRAITS: [LightTraits; {len(traits)}] = [",
    ]
    for key in traits:
        emission, emissive, shade, view, damp, solid, full, use_shape, occlude = key.split("|")
        out.append(
            f"    LightTraits {{ emission: {emission}, emissive: {emissive}, shade: {f32(shade)}, "
            f"view_blocking: {view}, dampening: {damp}, solid_render: {solid}, "
            f"collision_full_block: {full}, use_shape_for_light_occlusion: {use_shape}, "
            f"can_occlude: {occlude} }},"
        )
    out += ["];", "", "/// Each block's traits: one for every state, or one per state.",
            "#[rustfmt::skip]", f"pub static BLOCKS: [&[u16]; {len(rows)}] = ["]
    out += [f"    {index_list(ids)}," for ids in rows]
    out += ["];"]
    write("block/light.rs", out)
    print(f"traits={len(traits)} blocks={len(rows)}")


if __name__ == "__main__":
    run(generate)
