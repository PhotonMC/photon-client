"""The `image` family: original synthetic inputs."""

import struct
from cases.helpers import *


def _png_chunk(kind, payload):
    import struct, zlib
    return (struct.pack(">I", len(payload)) + kind + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF))


def _encode_png(width, height, colour_type, bit_depth, rows, palette=None,
                transparency=None, filters=None):
    """Build a synthetic PNG. Original content, so the corpus stays traceable.

    `rows` holds the raw sample bytes per scanline, unfiltered; `filters` picks
    the filter byte for each row so every filter type can be exercised.
    """
    import struct, zlib
    header = struct.pack(">IIBBBBB", width, height, bit_depth, colour_type, 0, 0, 0)
    chunks = [_png_chunk(b"IHDR", header)]
    if palette is not None:
        chunks.append(_png_chunk(b"PLTE", bytes(palette)))
    if transparency is not None:
        chunks.append(_png_chunk(b"tRNS", bytes(transparency)))

    filters = filters or [0] * height
    stride = len(rows[0])
    raw = bytearray()
    previous = bytes(stride)
    # Sample width in bytes, used by Sub/Average/Paeth; sub-byte depths use 1.
    sample = max(1, (bit_depth * {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[colour_type]) // 8)
    for row, kind in zip(rows, filters):
        raw.append(kind)
        line = bytearray(len(row))
        for index, value in enumerate(row):
            left = row[index - sample] if index >= sample else 0
            up = previous[index]
            up_left = previous[index - sample] if index >= sample else 0
            if kind == 0:
                line[index] = value
            elif kind == 1:
                line[index] = (value - left) & 0xFF
            elif kind == 2:
                line[index] = (value - up) & 0xFF
            elif kind == 3:
                line[index] = (value - (left + up) // 2) & 0xFF
            else:
                estimate = left + up - up_left
                to_left = abs(estimate - left)
                to_up = abs(estimate - up)
                to_up_left = abs(estimate - up_left)
                if to_left <= to_up and to_left <= to_up_left:
                    best = left
                elif to_up <= to_up_left:
                    best = up
                else:
                    best = up_left
                line[index] = (value - best) & 0xFF
        raw.extend(line)
        previous = bytes(row)
    chunks.append(_png_chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
    chunks.append(_png_chunk(b"IEND", b""))
    return b"\x89PNG\r\n\x1a\n" + b"".join(chunks)


def image_trim_cases():
    """The pictures armor trims are drawn with: a synthetic picture recoloured by a palette (a colour
    of the key palette replaced by the permutation's, its alpha scaled by the replacement's; other
    opaque colours kept; clear pixels kept), by the real mapping; and the name of the picture each
    pattern in each material takes on each armor's layer, from the real `ArmorTrim`."""
    def png(pixels, width):
        rows = [bytes(b for pixel in pixels[y * width:(y + 1) * width] for b in pixel)
                for y in range(len(pixels) // width)]
        return _encode_png(width, len(rows), 6, 8, rows).hex()

    keys = [(200, 10, 10, 255), (10, 200, 10, 255), (10, 10, 200, 255), (90, 90, 90, 255),
            (250, 250, 250, 255), (1, 2, 3, 0)]
    values = [(255, 0, 0, 255), (0, 255, 0, 128), (0, 0, 255, 255), (12, 34, 56, 200),
              (255, 255, 255, 0), (9, 9, 9, 255)]
    base = [
        keys[0], keys[1], keys[2], keys[3], keys[4], (1, 2, 3, 255), (77, 77, 77, 255), (0, 0, 0, 0),
        (200, 10, 10, 100), (10, 200, 10, 255), (10, 10, 200, 1), (90, 90, 90, 254),
        (250, 250, 250, 255), (5, 5, 5, 255), (0, 0, 0, 0), (200, 10, 10, 255),
    ]
    other = [(0, 0, 0, 255), (255, 128, 0, 255), (10, 200, 10, 255), (7, 7, 7, 40),
             (200, 10, 10, 255), (90, 90, 90, 90), (1, 1, 1, 255), (2, 2, 2, 255)]
    yield {
        "case_id": "image/palette-permutation",
        "family": "image",
        "operation": "palette_permutation",
        "rows": [
            {"label": "mixed", "png": png(base, 4), "keys": png(keys, 6), "values": png(values, 6)},
            {"label": "other-pixels", "png": png(other, 4), "keys": png(keys, 6), "values": png(values, 6)},
            {"label": "one-colour", "png": png([keys[0]], 1), "keys": png(keys[:1], 1), "values": png(values[:1], 1)},
        ],
    }
    patterns = ["sentry", "dune", "coast", "wild", "ward", "eye", "vex", "tide", "snout", "rib", "spire",
                "wayfinder", "shaper", "silence", "raiser", "host", "flow", "bolt"]
    materials = ["quartz", "iron", "netherite", "redstone", "copper", "gold", "emerald", "diamond", "lapis",
                 "amethyst", "resin"]
    equipment = ["iron", "diamond", "gold", "netherite", "copper", "chainmail", "leather", "turtle_scute"]
    rows = []
    for index, pattern in enumerate(patterns):
        for material in materials:
            layer = ("humanoid", "humanoid_leggings")[index % 2]
            rows.append({"label": f"{pattern}-{material}-{layer}", "pattern": f"minecraft:{pattern}",
                         "material": f"minecraft:{material}", "equipment": f"minecraft:{equipment[index % len(equipment)]}",
                         "layer": layer})
    for asset in equipment:
        for material in ("iron", "gold", "copper", "diamond", "netherite", "quartz"):
            rows.append({"label": f"over-{asset}-{material}", "pattern": "minecraft:sentry",
                         "material": f"minecraft:{material}", "equipment": f"minecraft:{asset}",
                         "layer": "humanoid"})
    yield {"case_id": "image/trim-sprite-id", "family": "image", "operation": "trim_sprite_id", "rows": rows}


def image_cases():
    """Java `NativeImage.read`: every PNG colour type reduced to RGBA8.

    The reference hands the bytes to stb with four components requested, so
    grayscale, palette and RGB all come back as RGBA and 16-bit samples are cut
    to their high byte. The inputs below cover each colour type, both bit
    depths, all five row filters and the transparency chunk.
    """
    def rgba_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 17 + y) % 256, (y * 23 + x) % 256,
                                ((x + y) * 5) % 256, (255 - x * 9) % 256))
                for y in range(height)]

    def rgb_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 31 + y) % 256, (y * 13) % 256, (x * y) % 256))
                for y in range(height)]

    def gray_rows(width, height):
        return [bytes((x * 37 + y * 11) % 256 for x in range(width))
                for y in range(height)]

    def gray_alpha_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 37 + y * 11) % 256, (x * 40) % 256))
                for y in range(height)]

    def indexed_rows(width, height, count):
        return [bytes((x + y) % count for x in range(width)) for y in range(height)]

    def rgba16_rows(width, height):
        return [bytes(b for x in range(width)
                      for channel in ((x * 257 + y) % 65536, (y * 999) % 65536,
                                      (x * y * 13) % 65536, 65535 - (x * 1234) % 65536)
                      for b in (channel >> 8, channel & 0xFF))
                for y in range(height)]

    def packed_rows(width, height, bit_depth, value_at):
        """Pack samples most significant bits first, padding each row."""
        rows = []
        for y in range(height):
            bits = []
            for x in range(width):
                value = value_at(x, y) & ((1 << bit_depth) - 1)
                bits.extend((value >> (bit_depth - 1 - b)) & 1 for b in range(bit_depth))
            while len(bits) % 8:
                bits.append(0)
            rows.append(bytes(
                sum(bit << (7 - offset) for offset, bit in enumerate(bits[base:base + 8]))
                for base in range(0, len(bits), 8)
            ))
        return rows

    palette = [c for index in range(4)
               for c in (index * 60 % 256, 255 - index * 40 % 256, index * 85 % 256)]

    rows = [
        ("rgba8-filter-none", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 0, 0, 0]),
        ("rgba8-filter-sub", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [1, 1, 1, 1]),
        ("rgba8-filter-up", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 2, 2, 2]),
        ("rgba8-filter-average", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 3, 3, 3]),
        ("rgba8-filter-paeth", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 4, 4, 4]),
        ("rgba8-mixed-filters", 6, 5, 6, 8, rgba_rows(6, 5), None, None, [0, 1, 2, 3, 4]),
        ("rgb8", 4, 4, 2, 8, rgb_rows(4, 4), None, None, [0, 1, 2, 4]),
        ("gray8", 4, 4, 0, 8, gray_rows(4, 4), None, None, [0, 1, 2, 4]),
        ("gray-alpha8", 4, 4, 4, 8, gray_alpha_rows(4, 4), None, None, [0, 1, 2, 4]),
        ("indexed", 4, 4, 3, 8, indexed_rows(4, 4, 4), palette, None, [0, 1, 2, 4]),
        ("indexed-trns", 4, 4, 3, 8, indexed_rows(4, 4, 4), palette, [0, 80, 170, 255], [0] * 4),
        ("gray1", 8, 4, 0, 1, packed_rows(8, 4, 1, lambda x, y: (x + y) % 2), None, None, [0, 1, 2, 4]),
        ("gray2", 8, 4, 0, 2, packed_rows(8, 4, 2, lambda x, y: (x + y) % 4), None, None, [0, 1, 2, 4]),
        ("gray4", 8, 4, 0, 4, packed_rows(8, 4, 4, lambda x, y: (x * 3 + y) % 16), None, None, [0, 1, 2, 4]),
        ("indexed1", 8, 4, 3, 1, packed_rows(8, 4, 1, lambda x, y: (x + y) % 2), palette, None, [0, 1, 2, 4]),
        ("indexed2", 8, 4, 3, 2, packed_rows(8, 4, 2, lambda x, y: (x + y) % 4), palette, None, [0, 1, 2, 4]),
        ("indexed4", 8, 4, 3, 4, packed_rows(8, 4, 4, lambda x, y: (x + y) % 4), palette, None, [0, 1, 2, 4]),
        ("indexed4-trns", 8, 4, 3, 4, packed_rows(8, 4, 4, lambda x, y: (x + y) % 4), palette, [0, 80, 170, 255], [0, 1, 2, 4]),
        ("gray1-odd-width", 5, 3, 0, 1, packed_rows(5, 3, 1, lambda x, y: (x * y) % 2), None, None, [0, 1, 4]),
        ("indexed2-odd-width", 3, 3, 3, 2, packed_rows(3, 3, 2, lambda x, y: (x + 2 * y) % 4), palette, None, [0, 1, 4]),
        ("gray8-trns", 4, 4, 0, 8, gray_rows(4, 4), None, [0, 0], [0, 1, 2, 4]),
        ("gray8-trns-hit", 4, 4, 0, 8, [bytes([0, 37, 74, 111]) for _ in range(4)], None, [0, 0], [0] * 4),
        ("gray2-trns", 8, 4, 0, 2, packed_rows(8, 4, 2, lambda x, y: (x + y) % 4), None, [0, 0], [0, 1, 2, 4]),
        ("gray1-trns", 8, 4, 0, 1, packed_rows(8, 4, 1, lambda x, y: (x + y) % 2), None, [0, 1], [0, 1, 2, 4]),
        ("rgb8-trns", 4, 4, 2, 8, rgb_rows(4, 4), None, [0, 0, 0, 0, 0, 0], [0, 1, 2, 4]),
        ("rgba16", 3, 3, 6, 16, rgba16_rows(3, 3), None, None, [0, 1, 4]),
        ("single-pixel", 1, 1, 6, 8, rgba_rows(1, 1), None, None, [0]),
        ("tall", 2, 8, 6, 8, rgba_rows(2, 8), None, None, [0, 1, 2, 3, 4, 1, 2, 3]),
        ("wide", 8, 2, 6, 8, rgba_rows(8, 2), None, None, [4, 3]),
    ]

    yield {
        "case_id": "image/item-side-faces",
        "family": "image",
        "operation": "item_side_faces",
        "rows": [
            {"label": name, "texture": f"assets/minecraft/textures/item/{name}.png"}
            for name in ("diamond_sword", "stick", "apple", "bow", "fishing_rod", "glass_bottle",
                         "leather_helmet", "leather_helmet_overlay", "potion_overlay", "bucket",
                         "iron_ingot", "torch" if False else "coal")
        ],
    }

    yield {
        "case_id": "image/srgb-tables",
        "family": "image",
        "operation": "srgb_tables",
        "rows": [{"label": "tables"}],
    }

    def gradient_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 255 // max(width - 1, 1)),
                                (y * 255 // max(height - 1, 1)),
                                ((x + y) * 255 // max(width + height - 2, 1)),
                                255))
                for y in range(height)]

    def noisy_rows(width, height):
        state = [12345]
        def step():
            state[0] = (state[0] * 1103515245 + 12345) & 0x7FFFFFFF
            return (state[0] >> 16) & 0xFF
        return [bytes(step() for _ in range(width * 4)) for _ in range(height)]

    def cutout_rows(width, height):
        """A shape with hard transparent edges, like a plant or a leaf."""
        rows = []
        for y in range(height):
            row = bytearray()
            for x in range(width):
                inside = abs(x - width // 2) + abs(y - height // 2) < width // 3
                row += bytes([(x * 13) % 256, (y * 29) % 256, 180, 255 if inside else 0])
            rows.append(bytes(row))
        return rows

    def soft_alpha_rows(width, height):
        """Alpha that ramps rather than jumping, so the coverage search has to
        converge on a scale instead of reproducing one exactly."""
        return [bytes(b for x in range(width)
                      for b in (200, 120, 60, (x * 255) // max(width - 1, 1)))
                for _ in range(height)]

    def clear_rows(width, height):
        """No opaque texel at all, which leaves the fill with no colour to
        spread and the dark fill with no darkest colour to find."""
        return [bytes([90, 140, 200, 0] * width) for _ in range(height)]

    # label, width, height, texels, levels, strategy, bias, path, transparency
    mip_rows = [
        ("gradient-16", 16, 16, gradient_rows(16, 16), 4, "MEAN", 0.0, "block/parity", (False, False)),
        ("gradient-8", 8, 8, gradient_rows(8, 8), 3, "MEAN", 0.0, "block/parity", (False, False)),
        ("noisy-16", 16, 16, noisy_rows(16, 16), 4, "MEAN", 0.0, "block/parity", (False, False)),
        ("noisy-32", 32, 32, noisy_rows(32, 32), 4, "MEAN", 0.0, "block/parity", (False, False)),
        ("flat-4", 4, 4, [bytes([200, 100, 50, 255]) * 4 for _ in range(4)], 2, "MEAN", 0.0,
         "block/parity", (False, False)),
        # A single level requested means the source comes back untouched.
        ("no-mips", 16, 16, gradient_rows(16, 16), 0, "MEAN", 0.0, "block/parity", (False, False)),
        # The cutout strategies spread colour into the transparent area before
        # halving, then rescale alpha so the visible area keeps its size.
        ("cutout-16", 16, 16, cutout_rows(16, 16), 4, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("cutout-32", 32, 32, cutout_rows(32, 32), 4, "CUTOUT", 0.0, "block/parity", (False, False)),
        # A stricter reference alpha moves the coverage the search aims at.
        ("strict-cutout-16", 16, 16, cutout_rows(16, 16), 4, "STRICT_CUTOUT", 0.0,
         "block/parity", (False, False)),
        ("cutout-bias", 16, 16, cutout_rows(16, 16), 4, "CUTOUT", 0.1, "block/parity", (False, False)),
        ("cutout-soft", 16, 16, soft_alpha_rows(16, 16), 4, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("dark-cutout-16", 16, 16, cutout_rows(16, 16), 4, "DARK_CUTOUT", 0.0,
         "block/parity", (False, False)),
        # Filling is skipped for item textures, so level zero keeps its holes.
        ("cutout-item", 16, 16, cutout_rows(16, 16), 4, "CUTOUT", 0.0, "item/parity", (False, False)),
        # Asking for no mips still fills, because that runs before the early exit.
        ("cutout-no-mips", 16, 16, cutout_rows(16, 16), 0, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("dark-cutout-no-mips", 16, 16, cutout_rows(16, 16), 0, "DARK_CUTOUT", 0.0,
         "block/parity", (False, False)),
        ("cutout-clear", 8, 8, clear_rows(8, 8), 3, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("dark-cutout-clear", 8, 8, clear_rows(8, 8), 3, "DARK_CUTOUT", 0.0,
         "block/parity", (False, False)),
        # AUTO picks its strategy from the transparency it is told about.
        ("auto-opaque", 16, 16, cutout_rows(16, 16), 4, "AUTO", 0.0, "block/parity", (False, False)),
        ("auto-transparent", 16, 16, cutout_rows(16, 16), 4, "AUTO", 0.0, "block/parity", (True, False)),
        ("auto-translucent", 16, 16, cutout_rows(16, 16), 4, "AUTO", 0.0, "block/parity", (False, True)),
    ]
    yield {
        "case_id": "image/mip-levels",
        "family": "image",
        "operation": "mip_levels",
        "rows": [
            {
                "label": label,
                "name": f"minecraft:{path}",
                "levels": levels,
                "strategy": strategy,
                "bias": float_bits(bias),
                "transparent": transparency[0],
                "translucent": transparency[1],
                "png": _encode_png(w, h, 6, 8, data, None, None, [0] * h).hex(),
            }
            for label, w, h, data, levels, strategy, bias, path, transparency in mip_rows
        ],
    }

    # What the atlas measures from a sprite's pixels to resolve AUTO. The scan
    # separates fully transparent from partly transparent, and a texture can be
    # both at once.
    transparency_rows = [
        ("opaque", 8, 8, [bytes([10, 20, 30, 255] * 8) for _ in range(8)]),
        ("transparent", 8, 8, clear_rows(8, 8)),
        ("cutout", 16, 16, cutout_rows(16, 16)),
        ("translucent", 8, 8, [bytes([10, 20, 30, 128] * 8) for _ in range(8)]),
        ("both", 8, 8, [bytes([10, 20, 30, 0 if y % 2 else 128] * 8) for y in range(8)]),
        # 254 counts as partly transparent; only exactly 255 is opaque.
        ("almost-opaque", 8, 8, [bytes([10, 20, 30, 254] * 8) for _ in range(8)]),
        ("single-hole", 8, 8, [bytes(b for x in range(8)
                                    for b in (10, 20, 30, 0 if (x, y) == (3, 5) else 255))
                               for y in range(8)]),
    ]
    yield {
        "case_id": "image/compute-transparency",
        "family": "image",
        "operation": "compute_transparency",
        "rows": [
            {
                "label": label,
                "png": _encode_png(w, h, 6, 8, data, None, None, [0] * h).hex(),
            }
            for label, w, h, data in transparency_rows
        ],
    }

    # The texture section of a sprite's .mcmeta. Every field is optional, so
    # what an absent field falls back to is the substance here.
    metadata_rows = [
        ("empty", "{}"),
        ("blur", '{"blur": true}'),
        ("clamp", '{"clamp": true}'),
        ("strategy-mean", '{"mipmap_strategy": "mean"}'),
        ("strategy-cutout", '{"mipmap_strategy": "cutout"}'),
        ("strategy-strict", '{"mipmap_strategy": "strict_cutout"}'),
        ("strategy-dark", '{"mipmap_strategy": "dark_cutout"}'),
        ("strategy-auto", '{"mipmap_strategy": "auto"}'),
        ("bias", '{"alpha_cutoff_bias": 0.25}'),
        ("bias-negative", '{"alpha_cutoff_bias": -0.5}'),
        ("bias-integer", '{"alpha_cutoff_bias": 1}'),
        ("all", '{"blur": true, "clamp": true, "mipmap_strategy": "dark_cutout",'
                ' "alpha_cutoff_bias": 0.125}'),
        # Names the enum does not carry, and shapes the codec does not take.
        ("strategy-unknown", '{"mipmap_strategy": "smooth"}'),
        ("strategy-uppercase", '{"mipmap_strategy": "MEAN"}'),
        ("strategy-number", '{"mipmap_strategy": 1}'),
        ("blur-string", '{"blur": "true"}'),
        ("bias-string", '{"alpha_cutoff_bias": "0.25"}'),
        ("unknown-field", '{"unexpected": 1}'),
        ("array", "[]"),
        ("string", '"texture"'),
    ]
    yield {
        "case_id": "image/texture-metadata",
        "family": "image",
        "operation": "texture_metadata",
        "rows": [{"label": label, "json": text} for label, text in metadata_rows],
    }

    # Where a stitched sprite's texels land. The packer hands back the padded
    # slot's origin, not the sprite's own, so the padding has to be added back
    # before anything is written or any UV is taken.
    sprite_uv_rows = [
        ("origin", 16, 16, 256, 256, 0, 0, 0),
        ("padded", 16, 16, 256, 256, 0, 0, 1),
        ("offset", 16, 16, 256, 256, 18, 36, 1),
        ("mip-padding", 16, 16, 256, 256, 32, 64, 4),
        ("anisotropic-padding", 16, 16, 512, 512, 64, 128, 16),
        ("oblong", 8, 32, 128, 512, 10, 40, 2),
        # An atlas whose side is not a power of two still divides exactly.
        ("odd-atlas", 16, 16, 300, 200, 20, 30, 1),
    ]
    offsets = [float_bits(value) for value in (0.0, 0.25, 0.5, 1.0, -0.5, 1.5)]
    yield {
        "case_id": "image/sprite-uv",
        "family": "image",
        "operation": "sprite_uv",
        "rows": [
            {
                "label": label,
                "width": w,
                "height": h,
                "atlas_width": atlas_w,
                "atlas_height": atlas_h,
                "x": x,
                "y": y,
                "padding": padding,
                "offsets": offsets,
            }
            for label, w, h, atlas_w, atlas_h, x, y, padding in sprite_uv_rows
        ],
    }

    # The biome tints index a 256x256 colour map by temperature and rainfall.
    # A synthetic map keeps the case self-contained; what is being pinned is
    # the indexing, not the game's own artwork.
    colour_map_png = _encode_png(
        256, 256, 6, 8,
        [bytes(b for x in range(256) for b in (x, y, (x * y) % 256, 255)) for y in range(256)],
        None, None, [0] * 256,
    ).hex()
    colour_map_rows = [
        ("grass/default", "grass", 0.5, 1.0),
        ("grass/cold-wet", "grass", 0.0, 1.0),
        ("grass/hot-dry", "grass", 1.0, 0.0),
        ("grass/hot-wet", "grass", 1.0, 1.0),
        ("grass/mid", "grass", 0.8, 0.4),
        ("foliage/default", "foliage", 0.5, 1.0),
        ("foliage/plains", "foliage", 0.8, 0.4),
        ("dry-foliage/default", "dry_foliage", 0.5, 1.0),
        # Rain is scaled by temperature first, so the same rain lands elsewhere
        # at a different temperature.
        ("grass/rain-scaling", "grass", 0.25, 1.0),
        # The truncation is toward zero, so a fraction of a step is dropped.
        ("grass/just-under-a-step", "grass", 0.5 + 1.0 / 512.0, 1.0),
        # The lookup guards the upper bound but not the lower one.
        ("grass/above-one", "grass", 1.5, 1.0),
        ("grass/below-zero", "grass", -0.5, 1.0),
        ("grass/negative-rain", "grass", 0.5, -1.0),
    ]
    yield {
        "case_id": "image/colour-map",
        "family": "image",
        "operation": "colour_map",
        "colour_map": colour_map_png,
        "rows": [
            {
                "label": label,
                "kind": kind,
                "temperature": float_bits(temperature, "f64"),
                "rain": float_bits(rain, "f64"),
            }
            for label, kind, temperature, rain in colour_map_rows
        ],
    }

    # How a tint folds into a quad's colour: per channel, with an integer
    # division, and with white on either side short-circuiting.
    multiply_rows = [
        ("white-left", "ffffffff", "ff48b518"),
        ("white-right", "ff48b518", "ffffffff"),
        ("both-white", "ffffffff", "ffffffff"),
        ("foliage-on-grey", "ff48b518", "ff808080"),
        ("halves", "80808080", "80808080"),
        ("black", "ff000000", "ff48b518"),
        ("one-each", "01010101", "01010101"),
        # The top byte is unsigned, so a full alpha survives the divide.
        ("full-alpha", "ff010203", "ff040506"),
        ("max", "fefefefe", "fefefefe"),
    ]
    yield {
        "case_id": "image/argb-multiply",
        "family": "image",
        "operation": "argb_multiply",
        "rows": [
            {"label": label, "lhs": lhs, "rhs": rhs}
            for label, lhs, rhs in multiply_rows
        ],
    }

    yield {
        "case_id": "image/tint-constants",
        "family": "image",
        "operation": "tint_constants",
        "rows": [{"label": "constants"}],
    }

    yield {
        "case_id": "image/png-decode",
        "family": "image",
        "operation": "png_decode",
        "rows": [
            {
                "label": label,
                "png": _encode_png(w, h, ct, bd, data, pal, trns, flt).hex(),
            }
            for label, w, h, ct, bd, data, pal, trns, flt in rows
        ],
    }
