"""The `math` family: the `shape/queries` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    def shape_clip_query(source, target, position=(0, 0, 0)):
        return {"from": point(*source), "to": point(*target), "pos": block_position(*position)}

    yield {"case_id": "shape/clip/empty", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "queries": [shape_clip_query((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5))]}

    yield {"case_id": "shape/clip/short-segment", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "queries": [shape_clip_query((0.5, 0.5, 0.5), (0.5001, 0.5, 0.5))]}

    yield {"case_id": "shape/clip/inside-directions", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "queries": [
               shape_clip_query((0.5, 0.5, 0.5), (2.0, 0.5, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (-1.0, 0.5, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, 2.0, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, -1.0, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, 0.5, 2.0)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, 0.5, -1.0)),
               shape_clip_query((0.5, 0.5, 0.5), (2.0, 2.0, 2.0)),
           ]}

    yield {"case_id": "shape/clip/outside-offset", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "queries": [shape_clip_query((5.0, -1.5, 5.5), (2.0, -1.5, 5.5), (3, -2, 5))]}

    yield {"case_id": "shape/clip/partial-box", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
           "queries": [shape_clip_query((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5))]}

    yield {"case_id": "shape/clip/joined-fallback", "family": "math", "operation": "shape", "kind": "clip_join",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(2.0, 0.0, 0.0, 3.0, 1.0, 1.0),
           "queries": [shape_clip_query((-1.0, 0.5, 0.5), (4.0, 0.5, 0.5))]}

    yield {"case_id": "shape/closest/empty", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "point": point(0.5, 0.5, 0.5)}

    yield {"case_id": "shape/closest/inside", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "point": point(0.5, 0.5, 0.5)}

    yield {"case_id": "shape/closest/axis-clamp", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
           "point": point(-1.0, 0.5, 0.5)}

    yield {"case_id": "shape/closest/diagonal-clamp", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "point": point(2.0, -1.0, 3.0)}

    yield {"case_id": "shape/closest/join-tie", "family": "math", "operation": "shape", "kind": "closest_join",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(2.0, 0.0, 0.0, 3.0, 1.0, 1.0),
           "point": point(1.5, 0.5, 0.5)}

    yield {"case_id": "shape/closest/translated-box", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(-2.0, -3.0, 4.0, -1.0, -1.0, 6.0),
           "point": point(-4.0, -5.0, 8.0)}

    yield {"case_id": "shape/face/empty", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/face/block", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "north"}

    yield {"case_id": "shape/face/axis-full", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.25, 0.0, 1.0, 0.75, 1.0), "direction": "east"}

    yield {"case_id": "shape/face/west-slice", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.25, 0.0, 0.75, 0.75, 1.0), "direction": "west"}

    yield {"case_id": "shape/face/east-slice", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.25, 0.25, 0.0, 1.0, 0.75, 1.0), "direction": "east"}

    yield {"case_id": "shape/face/missing-boundary", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.25, 0.25, 0.0, 0.75, 0.75, 1.0), "direction": "east"}

    yield {"case_id": "shape/face/up-slice", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.25, 0.25, 0.25, 0.75, 1.0, 0.75), "direction": "up"}

    yield {"case_id": "shape/face/joined", "family": "math", "operation": "shape", "kind": "face_join",
           "first": box_bits(0.25, 0.0, 0.0, 1.0, 0.5, 1.0),
           "second": box_bits(0.25, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/block-block-east", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/gap", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/partial-hole", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/partial-covered", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/negative-direction", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "west"}

    yield {"case_id": "shape/occlusion/empty-occluder", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0), "direction": "up"}

    yield {"case_id": "shape/occlusion/vertical-contact", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
           "occluder": box_bits(0.0, 0.0, 0.5, 1.0, 1.0, 1.0), "direction": "south"}

    yield {"case_id": "shape/occlusion/joined-shape", "family": "math", "operation": "shape",
           "kind": "block_occludes_join", "first": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
           "second": box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/face-empty", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.00000005, 1.0)}

    yield {"case_id": "shape/occlusion/face-block-first", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}

    yield {"case_id": "shape/occlusion/face-block-second", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)}

    yield {"case_id": "shape/occlusion/face-complementary", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0)}

    yield {"case_id": "shape/occlusion/face-gap", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.4, 1.0),
           "occluder": box_bits(0.0, 0.6, 0.0, 1.0, 1.0, 1.0)}

    yield {"case_id": "shape/occlusion/face-depth-complementary", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
           "occluder": box_bits(0.0, 0.0, 0.5, 1.0, 1.0, 1.0)}

    yield {"case_id": "shape/occlusion/merged-complementary", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/merged-block-first", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/merged-first-gap", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/merged-second-gap", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.25, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}

    yield {"case_id": "shape/occlusion/merged-west", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "west"}
