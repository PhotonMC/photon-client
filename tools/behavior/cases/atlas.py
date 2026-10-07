"""The `atlas` family: original synthetic inputs."""

from cases.helpers import *


def atlas_cases():
    """Java `Stitcher`: where each sprite lands on the texture atlas.

    The packer sorts by descending height, then descending width, then sprite
    name, and grows the atlas by powers of two on whichever axis is behind.
    The inputs below drive the parts that decide a layout: equal-size sprites
    where only the name breaks the tie, mixed sizes that force subdivision,
    non-power-of-two sizes rounded up by the mip level, padding that grows with
    mip level and anisotropy, and sets that cannot fit at all.
    """
    def sprite(name, width, height):
        return {"name": name, "width": width, "height": height}

    def row(label, sprites, max_width=256, max_height=256, mip_level=0, anisotropy_bit=0):
        return {
            "label": label,
            "sprites": sprites,
            "max_width": max_width,
            "max_height": max_height,
            "mip_level": mip_level,
            "anisotropy_bit": anisotropy_bit,
        }

    uniform = [sprite(f"minecraft:block/b{index:02d}", 16, 16) for index in range(8)]
    # Same size and same path, different namespace: the comparator falls through
    # to the identifier, which orders on path before namespace.
    namespaces = [
        sprite("zzz:block/same", 16, 16),
        sprite("aaa:block/same", 16, 16),
        sprite("minecraft:block/same", 16, 16),
    ]
    mixed = [
        sprite("minecraft:block/tall", 16, 64),
        sprite("minecraft:block/wide", 64, 16),
        sprite("minecraft:block/small", 8, 8),
        sprite("minecraft:block/big", 32, 32),
        sprite("minecraft:block/tiny", 4, 4),
    ]
    # Path ordering and namespace ordering disagree here: by path `block/a`
    # (namespace zzz) comes first, by namespace `aaa` would. This is the row
    # that pins path-before-namespace rather than just the fallback.
    path_before_namespace = [
        sprite("zzz:block/a", 16, 16),
        sprite("aaa:block/b", 16, 16),
    ]
    ragged = [
        sprite("minecraft:block/odd", 13, 7),
        sprite("minecraft:block/odder", 5, 11),
        sprite("minecraft:block/oddest", 3, 3),
    ]

    rows = [
        row("single", [sprite("minecraft:block/only", 16, 16)]),
        row("uniform-8", uniform),
        row("name-tiebreak", namespaces),
        row("path-before-namespace", path_before_namespace),
        row("mixed-sizes", mixed),
        row("ragged-sizes", ragged),
        row("mip-1", mixed, mip_level=1),
        row("mip-2", mixed, mip_level=2),
        row("mip-4", mixed, mip_level=4),
        row("anisotropy-1", mixed, anisotropy_bit=1),
        row("anisotropy-3", mixed, anisotropy_bit=3),
        row("anisotropy-clamped-9", mixed, anisotropy_bit=9),
        row("mip-2-anisotropy-2", mixed, mip_level=2, anisotropy_bit=2),
        row("narrow-atlas", mixed, max_width=64, max_height=256),
        row("short-atlas", mixed, max_width=256, max_height=64),
        row("exact-fit", [sprite("minecraft:block/full", 64, 64)], max_width=64, max_height=64),
        row("too-large", [sprite("minecraft:block/huge", 128, 128)], max_width=64, max_height=64),
        row("overflow", uniform * 4, max_width=32, max_height=32),
        row("power-of-two-growth", [sprite(f"minecraft:block/g{i:02d}", 16, 16) for i in range(5)]),
    ]
    yield {
        "case_id": "atlas/stitch",
        "family": "atlas",
        "operation": "stitch",
        "rows": rows,
    }
