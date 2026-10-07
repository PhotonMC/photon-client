"""The `model` family: original synthetic inputs."""

import struct
from cases.helpers import *


def model_cases():
    """Java `TextureSlots`: how a model's texture variables resolve.

    A model's texture map may name a sprite directly or point at another slot
    with a leading `#`. The chain is walked child first, but applied parent
    first, so a child overrides its parent; a reference clears an inherited
    value and a value clears an inherited reference. References then resolve to
    a fixed point, and anything still dangling is dropped rather than failing.
    """
    def ref(slot, target):
        return {"slot": slot, "raw": "#" + target}

    def raw(slot, value):
        return {"slot": slot, "raw": value}

    def value(slot, sprite, force_translucent=False):
        return {"slot": slot, "sprite": sprite, "force_translucent": force_translucent}

    stone = "minecraft:block/stone"
    dirt = "minecraft:block/dirt"
    oak = "minecraft:block/oak_planks"

    rows = [
        ("empty", []),
        ("single-value", [[raw("all", stone)]]),
        ("object-form", [[value("all", stone, True)]]),
        # A child naming a sprite wins over the parent's sprite.
        ("child-overrides-parent", [[raw("all", dirt)], [raw("all", stone)]]),
        # cube_all shape: parent points every face at #all, child fills it in.
        ("cube-all", [
            [raw("all", stone)],
            [ref("top", "all"), ref("bottom", "all"), ref("side", "all"),
             ref("particle", "all")],
        ]),
        # The reference target itself arrives from a grandparent.
        ("three-level", [
            [raw("all", oak)],
            [ref("side", "all")],
            [ref("north", "side"), ref("particle", "side")],
        ]),
        # A chain of references collapsing to one sprite.
        ("reference-chain", [[raw("all", stone)],
                             [ref("top", "all")],
                             [ref("side", "top")],
                             [ref("north", "side")]]),
        # A child reference replaces an inherited concrete value.
        ("reference-clears-value", [[ref("all", "side")],
                                    [raw("all", stone), raw("side", dirt)]]),
        # A child value replaces an inherited reference.
        ("value-clears-reference", [[raw("all", stone)],
                                    [ref("all", "side"), raw("side", dirt)]]),
        # Nothing to point at: the slot is dropped, not an error.
        ("dangling-reference", [[ref("top", "nothing")]]),
        ("self-reference", [[ref("top", "top")]]),
        ("cycle", [[ref("top", "side"), ref("side", "top")]]),
        # A cycle next to a resolvable reference: only the good one survives.
        ("cycle-plus-resolvable", [[raw("all", stone)],
                                   [ref("top", "side"), ref("side", "top"),
                                    ref("bottom", "all")]]),
        ("force-translucent-inherited", [[value("all", stone, True)],
                                         [ref("side", "all")]]),
        ("mixed-levels", [
            [raw("all", stone), value("particle", dirt, True)],
            [ref("top", "all"), raw("bottom", oak)],
            [ref("side", "bottom"), ref("north", "missing")],
        ]),
        # Same slot named at every level: the deepest child still wins.
        ("deep-override", [[raw("all", oak)], [raw("all", dirt)], [raw("all", stone)]]),
    ]

    def bits(value):
        import struct
        return f"{struct.unpack('>I', struct.pack('>f', value))[0]:08x}"

    def box(from_xyz, to_xyz):
        return ([bits(v) for v in from_xyz], [bits(v) for v in to_xyz])

    facings = ["down", "up", "north", "south", "west", "east"]
    boxes = [
        ("full", (0.0, 0.0, 0.0), (16.0, 16.0, 16.0)),
        ("slab", (0.0, 0.0, 0.0), (16.0, 8.0, 16.0)),
        ("offset", (2.0, 3.0, 4.0), (14.0, 13.0, 12.0)),
        ("thin", (7.5, 0.0, 7.5), (8.5, 16.0, 8.5)),
        ("degenerate", (4.0, 4.0, 4.0), (4.0, 12.0, 12.0)),
        # The element grammar allows reaching outside the block.
        ("oversized", (-16.0, -16.0, -16.0), (32.0, 32.0, 32.0)),
        ("fractional", (0.5, 1.25, 2.125), (15.5, 14.75, 13.875)),
        # Inverted bounds are not rejected by the corner layout itself.
        ("inverted", (12.0, 12.0, 12.0), (4.0, 4.0, 4.0)),
    ]
    geometry_rows = []
    for name, from_xyz, to_xyz in boxes:
        from_bits, to_bits = box(from_xyz, to_xyz)
        for facing in facings:
            geometry_rows.append({
                "label": f"{name}-{facing}",
                "from": from_bits,
                "to": to_bits,
                "facing": facing,
            })

    rotation_rows = []
    # The angles real models use, plus values the grammar allows but no
    # vanilla model writes.
    angles = [0.0, 22.5, -22.5, 45.0, -45.0, 90.0, 180.0, 270.0, 360.0, 30.0, -7.5]
    for axis in ("x", "y", "z"):
        for angle in angles:
            for rescale in (False, True):
                rotation_rows.append({
                    "label": f"{axis}-{angle}-{'rescale' if rescale else 'plain'}",
                    "from": [bits(v) for v in (0.0, 0.0, 0.0)],
                    "to": [bits(v) for v in (16.0, 16.0, 16.0)],
                    "facing": "up",
                    "origin": [bits(v) for v in (8.0, 8.0, 8.0)],
                    "axis": axis,
                    "angle": bits(angle),
                    "rescale": rescale,
                })
    # A non-centred origin and a partial box, which is what stairs and torches
    # actually look like.
    for facing in ("down", "north", "east"):
        rotation_rows.append({
            "label": f"offset-origin-{facing}",
            "from": [bits(v) for v in (4.0, 0.0, 4.0)],
            "to": [bits(v) for v in (12.0, 10.0, 12.0)],
            "facing": facing,
            "origin": [bits(v) for v in (0.0, 8.0, 16.0)],
            "axis": "y",
            "angle": bits(22.5),
            "rescale": True,
        })

    def corner(x, y, z):
        return [bits(x), bits(y), bits(z)]

    winding_rows = [
        # A full-block face in the reference's own corner order for each side.
        ("up-ordered", [corner(0, 1, 0), corner(0, 1, 1), corner(1, 1, 1), corner(1, 1, 0)]),
        ("down-ordered", [corner(0, 0, 1), corner(0, 0, 0), corner(1, 0, 0), corner(1, 0, 1)]),
        ("north-ordered", [corner(1, 1, 0), corner(1, 0, 0), corner(0, 0, 0), corner(0, 1, 0)]),
        ("south-ordered", [corner(0, 1, 1), corner(0, 0, 1), corner(1, 0, 1), corner(1, 1, 1)]),
        ("west-ordered", [corner(0, 1, 0), corner(0, 0, 0), corner(0, 0, 1), corner(0, 1, 1)]),
        ("east-ordered", [corner(1, 1, 1), corner(1, 0, 1), corner(1, 0, 0), corner(1, 1, 0)]),
        # The same face wound backwards: the normal flips and so does the
        # direction the bakery settles on.
        ("up-reversed", [corner(1, 1, 0), corner(1, 1, 1), corner(0, 1, 1), corner(0, 1, 0)]),
        # Shuffled corners of a real face, which is what a rotation leaves
        # behind and what the rewind exists to repair.
        ("up-shuffled", [corner(1, 1, 1), corner(0, 1, 0), corner(1, 1, 0), corner(0, 1, 1)]),
        # A partial box rather than the whole block.
        ("slab-top", [corner(0, 0.5, 0), corner(0, 0.5, 1), corner(1, 0.5, 1), corner(1, 0.5, 0)]),
        # Degenerate input: all four corners equal, so the normal is not finite.
        ("degenerate", [corner(0.5, 0.5, 0.5)] * 4),
        # Collinear corners, another way to lose the normal.
        ("collinear", [corner(0, 0, 0), corner(1, 0, 0), corner(2, 0, 0), corner(3, 0, 0)]),
        # A diagonal quad, whose nearest direction is not exact.
        ("diagonal", [corner(0, 0, 0), corner(1, 0, 0), corner(1, 1, 1), corner(0, 1, 1)]),
    ]

    euler_rows = []
    triples = [
        (0.0, 0.0, 0.0),
        (22.5, 0.0, 0.0), (0.0, 22.5, 0.0), (0.0, 0.0, 22.5),
        (90.0, 0.0, 0.0), (0.0, 90.0, 0.0), (0.0, 0.0, 90.0),
        (22.5, 45.0, 0.0), (0.0, 22.5, 45.0), (45.0, 0.0, 22.5),
        (22.5, 22.5, 22.5), (45.0, 45.0, 45.0), (90.0, 90.0, 90.0),
        (-22.5, 30.0, -7.5), (180.0, 90.0, -45.0), (360.0, 360.0, 360.0),
    ]
    for x, y, z in triples:
        for rescale in (False, True):
            euler_rows.append({
                "label": f"{x}-{y}-{z}-{'rescale' if rescale else 'plain'}",
                "x": bits(x),
                "y": bits(y),
                "z": bits(z),
                "rescale": rescale,
            })

    uv_rows = []
    uv_sets = [
        ("full", (0.0, 0.0, 16.0, 16.0)),
        ("partial", (2.0, 4.0, 14.0, 12.0)),
        # A flipped rectangle, which models use to mirror a texture.
        ("flipped-u", (16.0, 0.0, 0.0, 16.0)),
        ("flipped-both", (16.0, 16.0, 0.0, 0.0)),
        ("fractional", (0.5, 1.25, 15.5, 14.75)),
    ]
    # Accepted rotations, values that wrap onto them, and values that do not.
    degrees_list = [0, 90, 180, 270, 360, 450, -90, -180, -270, -360, 720, 45, 30, 1, -45, 100]
    for name, uvs in uv_sets:
        for degrees in degrees_list:
            uv_rows.append({
                "label": f"{name}-{degrees}",
                "uvs": [bits(v) for v in uvs],
                "degrees": degrees,
            })

    cube_all = (
        '{"parent":"minecraft:block/cube_all",'
        '"textures":{"all":"minecraft:block/stone"}}'
    )
    full_element = (
        '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
        '"faces":{"down":{"texture":"#all","cullface":"down"},'
        '"up":{"texture":"#all","cullface":"up"},'
        '"north":{"texture":"#all","cullface":"north"},'
        '"south":{"texture":"#all","cullface":"south"},'
        '"west":{"texture":"#all","cullface":"west"},'
        '"east":{"texture":"#all","cullface":"east"}}}]}'
    )
    display_right = ('{"display":{"thirdperson_righthand":{"rotation":[75,45,0],'
                     '"translation":[0,2.5,0],"scale":[0.375,0.375,0.375]},'
                     '"firstperson_righthand":{"rotation":[0,45,0],"scale":[0.4,0.4,0.4]},'
                     '"gui":{"rotation":[30,225,0],"scale":[0.625,0.625,0.625]},'
                     '"ground":{"translation":[0,3,0],"scale":[0.25,0.25,0.25]}}}')
    display_clamped = ('{"display":{"fixed":{"translation":[100,-100,8],"scale":[9,-9,0.5]},'
                       '"head":{"rotation":[0,180,0],"translation":[0,13,7]},'
                       '"on_shelf":{"translation":[0,0,-1]},'
                       '"firstperson_lefthand":{"rotation":[0,-90,25]},'
                       '"thirdperson_lefthand":{"scale":[1,1,1]}}}')
    model_texts = [
        ("display-right-hands", display_right),
        ("display-clamped-and-left", display_clamped),
        ("empty-object", "{}"),
        ("parent-only", cube_all),
        ("full-cube", full_element),
        ("ambient-occlusion-false", '{"ambientocclusion":false,' + full_element[1:]),
        ("gui-light-front", '{"gui_light":"front",' + full_element[1:]),
        # A face may carry its own uv rectangle, a tint and a rotation.
        ("face-detail", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                        '"faces":{"up":{"texture":"#t","uv":[1,2,15,14],'
                        '"tintindex":3,"rotation":270}}}]}'),
        # Rotation blocks, both spellings.
        ("rotation-axis", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                          '"rotation":{"origin":[8,8,8],"axis":"y","angle":22.5,"rescale":true},'
                          '"faces":{"up":{"texture":"#t"}}}]}'),
        ("rotation-euler", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                           '"rotation":{"origin":[0,8,16],"x":22.5,"y":45,"z":0},'
                           '"faces":{"up":{"texture":"#t"}}}]}'),
        ("shade-false", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"shade":false,'
                        '"faces":{"up":{"texture":"#t"}}}]}'),
        ("light-emission", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"light_emission":15,'
                           '"faces":{"up":{"texture":"#t"}}}]}'),
        # The element grammar permits reaching outside the block, up to a limit.
        ("bounds-edge", '{"elements":[{"from":[-16,-16,-16],"to":[32,32,32],'
                        '"faces":{"up":{"texture":"#t"}}}]}'),
        ("textures-mixed", '{"textures":{"all":"minecraft:block/stone","side":"#all",'
                           '"particle":{"sprite":"minecraft:block/dirt","force_translucent":true}}}'),
        ("multiple-elements", '{"elements":[' +
            '{"from":[0,0,0],"to":[16,8,16],"faces":{"up":{"texture":"#a"}}},' +
            '{"from":[0,8,0],"to":[16,16,16],"faces":{"down":{"texture":"#b"}}}]}'),
    ]
    rejected_models = [
        ("bounds-too-low", '{"elements":[{"from":[-17,0,0],"to":[16,16,16],'
                           '"faces":{"up":{"texture":"#t"}}}]}'),
        ("bounds-too-high", '{"elements":[{"from":[0,0,0],"to":[33,16,16],'
                            '"faces":{"up":{"texture":"#t"}}}]}'),
        ("short-vector", '{"elements":[{"from":[0,0],"to":[16,16,16],'
                         '"faces":{"up":{"texture":"#t"}}}]}'),
        ("no-faces", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{}}]}'),
        ("unknown-facing", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                           '"faces":{"sideways":{"texture":"#t"}}}]}'),
        ("shade-not-boolean", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"shade":1,'
                              '"faces":{"up":{"texture":"#t"}}}]}'),
        ("light-emission-high", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                                '"light_emission":16,"faces":{"up":{"texture":"#t"}}}]}'),
        ("light-emission-negative", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                                    '"light_emission":-1,"faces":{"up":{"texture":"#t"}}}]}'),
        ("rotation-empty", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                           '"rotation":{"origin":[8,8,8]},"faces":{"up":{"texture":"#t"}}}]}'),
        ("rotation-bad-axis", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                              '"rotation":{"origin":[8,8,8],"axis":"w","angle":22.5},'
                              '"faces":{"up":{"texture":"#t"}}}]}'),
        ("uv-short", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                     '"faces":{"up":{"texture":"#t","uv":[1,2,3]}}}]}'),
        ("face-rotation-invalid", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                                  '"faces":{"up":{"texture":"#t","rotation":45}}}]}'),
        ("missing-texture", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                            '"faces":{"up":{}}}]}'),
        ("not-an-object", "[]"),
        ("trailing-comma", '{"elements":[],}'),
    ]

    state_rows = []
    # UV lock is left out; the reason is recorded in the slice document.
    for x in (0, 90, 180, 270, 360, -90, -180, 450):
        for y in (0, 90, 180, 270, 360, -90, -180, 450):
            state_rows.append({
                "label": f"x{x}-y{y}",
                "x": x,
                "y": y,
                "z": 0,
                "uvlock": False,
            })
    # With UV lock on, the face transformations stop being identity and come
    # from a table the reference builds once from quaternions. Every quadrant
    # combination is driven, because the table is keyed by the composed group
    # and several combinations land on the same element.
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for z in (0, 90, 180, 270):
                state_rows.append({
                    "label": f"uvlock-x{x}-y{y}-z{z}",
                    "x": x,
                    "y": y,
                    "z": z,
                    "uvlock": True,
                })
    # The third axis on its own, and combined with the other two, because the
    # group composition is not commutative.
    for z in (90, 180, 270):
        state_rows.append({"label": f"z{z}", "x": 0, "y": 0, "z": z, "uvlock": False})
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for z in (90, 180, 270):
                state_rows.append({
                    "label": f"x{x}-y{y}-z{z}",
                    "x": x,
                    "y": y,
                    "z": z,
                    "uvlock": False,
                })

    blockstate_texts = [
        # The plain shape: one key, one model.
        ("single", '{"variants": {"": {"model": "minecraft:block/stone"}}}'),
        ("property-key", '{"variants": {"facing=north": {"model": "minecraft:block/furnace"}}}'),
        ("rotation", '{"variants": {"": {"model": "minecraft:block/stairs",'
                     ' "x": 90, "y": 180, "z": 270, "uvlock": true}}}'),
        # Angles are taken modulo a full turn before the quadrant is chosen.
        ("rotation-wrapped", '{"variants": {"": {"model": "minecraft:block/a",'
                             ' "x": 450, "y": -90}}}'),
        ("rotation-bad", '{"variants": {"": {"model": "minecraft:block/a", "x": 45}}}'),
        # A list is a weighted choice; the weight defaults to one.
        ("weighted", '{"variants": {"": [{"model": "minecraft:block/a"},'
                     ' {"model": "minecraft:block/b", "weight": 3}]}}'),
        ("weighted-zero", '{"variants": {"": [{"model": "minecraft:block/a", "weight": 0}]}}'),
        ("weighted-negative",
         '{"variants": {"": [{"model": "minecraft:block/a", "weight": -1}]}}'),
        ("weighted-empty", '{"variants": {"": []}}'),
        ("several-keys", '{"variants": {"half=bottom": {"model": "minecraft:block/a"},'
                         ' "half=top": {"model": "minecraft:block/b"}}}'),
        # A model name is required and must be a valid identifier.
        ("no-model", '{"variants": {"": {}}}'),
        ("bad-model", '{"variants": {"": {"model": "Not An Id"}}}'),
        ("empty-variants", '{"variants": {}}'),
        ("neither", "{}"),
        ("both", '{"variants": {"": {"model": "minecraft:block/a"}},'
                 ' "multipart": [{"apply": {"model": "minecraft:block/b"}}]}'),
        # Multipart: a condition is optional, and its two forms nest.
        ("multipart-plain", '{"multipart": [{"apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-when", '{"multipart": [{"when": {"north": "true"},'
                           ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-multi-term", '{"multipart": [{"when": {"facing": "north|south"},'
                                 ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-negated", '{"multipart": [{"when": {"facing": "!north"},'
                              ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-or", '{"multipart": [{"when": {"OR": [{"north": "true"},'
                         ' {"south": "true"}]}, "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-and", '{"multipart": [{"when": {"AND": [{"north": "true"},'
                          ' {"south": "true"}]}, "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-nested", '{"multipart": [{"when": {"OR": [{"north": "true"},'
                             ' {"AND": [{"south": "true"}, {"east": "false"}]}]},'
                             ' "apply": {"model": "minecraft:block/a"}}]}'),
        # A number or a boolean in a term is taken as its text.
        ("multipart-boolean", '{"multipart": [{"when": {"north": true},'
                              ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-number", '{"multipart": [{"when": {"level": 3},'
                             ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-float", '{"multipart": [{"when": {"level": 3.5},'
                            ' "apply": {"model": "minecraft:block/a"}}]}'),
        # Empty terms and empty maps are refused.
        ("multipart-empty-term", '{"multipart": [{"when": {"facing": ""},'
                                 ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty-piece", '{"multipart": [{"when": {"facing": "north|"},'
                                  ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty-when", '{"multipart": [{"when": {},'
                                 ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty", '{"multipart": []}'),
        ("multipart-no-apply", '{"multipart": [{"when": {"north": "true"}}]}'),
        # A combiner holds exactly one operation.
        ("multipart-two-operations", '{"multipart": [{"when": {"OR": [{"north": "true"}],'
                                     ' "AND": [{"south": "true"}]},'
                                     ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-lowercase-operation", '{"multipart": [{"when": {"or": [{"north": "true"}]},'
                                          ' "apply": {"model": "minecraft:block/a"}}]}'),
        # An operation alongside a property: does the combiner take the one
        # key it knows and drop the rest, or refuse the whole condition?
        ("multipart-operation-and-property",
         '{"multipart": [{"when": {"OR": [{"north": "true"}], "north": "true"},'
         ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty-operation-list",
         '{"multipart": [{"when": {"AND": []}, "apply": {"model": "minecraft:block/a"}}]}'),
        # The selector string is not validated while parsing, only later when
        # it meets a state definition.
        ("nonsense-selector", '{"variants": {"=,,=nope": {"model": "minecraft:block/a"}}}'),
        ("multipart-weighted", '{"multipart": [{"apply": [{"model": "minecraft:block/a"},'
                               ' {"model": "minecraft:block/b", "weight": 2}]}]}'),
        ("not-an-object", "[]"),
    ]
    # What a UV-locked face does to a texture coordinate. Every rotation and
    # every face is driven, because the tables differ per pair, and the sample
    # coordinates include the corners, the centre and points off the face.
    uv_lock_rows = []
    faces = ("down", "up", "north", "south", "west", "east")
    samples = (0.0, 1.0, 0.5, 0.25, 0.125, -0.5, 1.5)
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for facing in faces:
                for u in samples:
                    uv_lock_rows.append({
                        "label": f"x{x}-y{y}-{facing}-u{u}",
                        "x": x, "y": y, "z": 0, "uvlock": True, "facing": facing,
                        "u": float_bits(u), "v": float_bits(0.375),
                    })
    # The third axis, and the unlocked case where the coordinate passes
    # through untouched.
    for z in (90, 180, 270):
        for facing in faces:
            uv_lock_rows.append({
                "label": f"z{z}-{facing}",
                "x": 0, "y": 0, "z": z, "uvlock": True, "facing": facing,
                "u": float_bits(0.25), "v": float_bits(0.375),
            })
    for facing in faces:
        uv_lock_rows.append({
            "label": f"unlocked-{facing}",
            "x": 90, "y": 180, "z": 0, "uvlock": False, "facing": facing,
            "u": float_bits(0.25), "v": float_bits(0.375),
        })
    # Which face a vector points at. The search starts at north with the
    # smallest positive float as the bar, so a vector with no positive
    # component at all comes back north rather than as nothing.
    nearest_rows = [
        ("down", 0.0, -1.0, 0.0), ("up", 0.0, 1.0, 0.0),
        ("north", 0.0, 0.0, -1.0), ("south", 0.0, 0.0, 1.0),
        ("west", -1.0, 0.0, 0.0), ("east", 1.0, 0.0, 0.0),
        ("zero", 0.0, 0.0, 0.0),
        ("all-negative", -1.0, -1.0, -1.0),
        ("diagonal", 1.0, 1.0, 0.0),
        ("tie-up-east", 1.0, 1.0, 1.0),
        ("tiny-positive", 1e-44, 0.0, 0.0),
        ("below-the-bar", 1e-46, 0.0, 0.0),
        ("negative-zero", -0.0, -0.0, -0.0),
        ("nan", float("nan"), 0.0, 0.0),
        ("infinity", float("inf"), 0.0, 0.0),
        ("negative-infinity", float("-inf"), 0.0, 0.0),
        ("mixed", 0.5, -0.9, 0.2),
    ]
    yield {
        "case_id": "model/nearest-direction",
        "family": "model",
        "operation": "nearest_direction",
        "rows": [
            {"label": label, "x": float_bits(x), "y": float_bits(y), "z": float_bits(z)}
            for label, x, y, z in nearest_rows
        ],
    }

    # Where a face's cullface direction lands once the variant is turned.
    cull_rows = []
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for z in (0, 90, 180, 270):
                for facing in ("down", "up", "north", "south", "west", "east"):
                    cull_rows.append({
                        "label": f"x{x}-y{y}-z{z}-{facing}",
                        "x": x, "y": y, "z": z, "facing": facing,
                    })
    yield {
        "case_id": "model/cull-rotation",
        "family": "model",
        "operation": "cull_rotation",
        "rows": cull_rows,
    }

    yield {
        "case_id": "model/uv-lock",
        "family": "model",
        "operation": "uv_lock",
        "rows": uv_lock_rows,
    }

    yield {
        "case_id": "model/blockstate-parse",
        "family": "model",
        "operation": "blockstate_parse",
        "rows": [{"label": label, "json": text} for label, text in blockstate_texts],
    }

    yield {
        "case_id": "model/model-state",
        "family": "model",
        "operation": "model_state",
        "rows": state_rows,
    }

    yield {
        "case_id": "model/parse-model",
        "family": "model",
        "operation": "parse_model",
        "rows": [
            {"label": label, "text": text.encode("utf-8").hex()}
            for label, text in model_texts + rejected_models
        ],
    }

    yield {
        "case_id": "model/face-uv-rotation",
        "family": "model",
        "operation": "face_uv_rotation",
        "rows": uv_rows,
    }

    yield {
        "case_id": "model/euler-rotation",
        "family": "model",
        "operation": "euler_rotation",
        "rows": euler_rows,
    }

    yield {
        "case_id": "model/quad-winding",
        "family": "model",
        "operation": "quad_winding",
        "rows": [{"label": label, "corners": corners} for label, corners in winding_rows],
    }

    yield {
        "case_id": "model/element-rotation",
        "family": "model",
        "operation": "element_rotation",
        "rows": rotation_rows,
    }

    yield {
        "case_id": "model/face-geometry",
        "family": "model",
        "operation": "face_geometry",
        "rows": geometry_rows,
    }

    yield {
        "case_id": "model/texture-slots",
        "family": "model",
        "operation": "texture_slots",
        "rows": [{"label": label, "chain": chain} for label, chain in rows],
    }
