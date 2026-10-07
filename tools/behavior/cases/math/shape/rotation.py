"""The `math` family: the `shape/rotation` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)

    yield {"case_id": "shape/rotate/identity", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "IDENTITY"}

    yield {"case_id": "shape/rotate/x-90", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_X_NEG"}

    yield {"case_id": "shape/rotate/y-90", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Y_NEG"}

    yield {"case_id": "shape/rotate/z-90", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Z_NEG"}

    yield {"case_id": "shape/rotate/x-180", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_180_FACE_YZ"}

    yield {"case_id": "shape/rotate/y-180", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_180_FACE_XZ"}

    yield {"case_id": "shape/rotate/z-180", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_180_FACE_XY"}

    yield {"case_id": "shape/rotate/x-90-positive", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_X_POS"}

    yield {"case_id": "shape/rotate/y-90-positive", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Y_POS"}

    yield {"case_id": "shape/rotate/z-90-positive", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Z_POS"}

    yield {"case_id": "shape/rotate/cube-grid", "family": "math", "operation": "shape", "kind": "rotate",
           "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), "rotation": "ROT_90_Y_NEG"}

    yield {"case_id": "shape/rotate/joined-cells", "family": "math", "operation": "shape", "kind": "rotate_join",
           "first": box_bits(0.0, 0.0, 0.0, 0.5, 0.5, 1.0),
           "second": box_bits(0.5, 0.5, 0.0, 1.0, 1.0, 1.0), "rotation": "ROT_90_Z_NEG"}

    def center_bits(x, y, z):
        return {"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}

    yield {"case_id": "shape/rotate-center/y-asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": rotate_box, "rotation": "ROT_90_Y_NEG",
           "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-center/x-cube", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "rotation": "ROT_90_X_NEG", "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-center/z-grid", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "rotation": "ROT_90_Z_POS", "center": center_bits(0.75, 0.25, 0.5)}

    yield {"case_id": "shape/rotate-center/identity", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": rotate_box, "rotation": "IDENTITY",
           "center": center_bits(2.0, -1.0, 3.0)}

    yield {"case_id": "shape/rotate-all-center/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all_center", "box": rotate_box,
           "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-all-center/grid", "family": "math", "operation": "shape",
           "kind": "rotate_all_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "center": center_bits(0.75, 0.25, 0.5)}

    yield {"case_id": "shape/rotate-all-center/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all_center", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "center": center_bits(2.0, -1.0, 3.0)}

    yield {"case_id": "shape/rotate-horizontal-axis/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis", "box": rotate_box}

    yield {"case_id": "shape/rotate-horizontal-axis/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}

    yield {"case_id": "shape/rotate-horizontal-axis/empty", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}

    yield {"case_id": "shape/rotate-horizontal-axis-center/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis_center", "box": rotate_box,
           "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-horizontal-axis-center/grid", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "center": center_bits(0.75, 0.25, 0.5)}

    yield {"case_id": "shape/rotate-horizontal-axis-center/empty", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis_center", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "center": center_bits(2.0, -1.0, 3.0)}

    yield {"case_id": "shape/rotate-all-axis/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis", "box": rotate_box}

    yield {"case_id": "shape/rotate-all-axis/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}

    yield {"case_id": "shape/rotate-all-axis/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}

    yield {"case_id": "shape/rotate-all-axis-center/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis_center", "box": rotate_box,
           "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-all-axis-center/grid", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "center": center_bits(0.75, 0.25, 0.5)}

    yield {"case_id": "shape/rotate-all-axis-center/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis_center", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "center": center_bits(2.0, -1.0, 3.0)}

    yield {"case_id": "shape/rotate-horizontal/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal", "box": rotate_box}

    yield {"case_id": "shape/rotate-horizontal/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}

    yield {"case_id": "shape/rotate-horizontal/empty", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}

    yield {"case_id": "shape/rotate-horizontal-initial/identity", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box, "initial": "IDENTITY"}

    yield {"case_id": "shape/rotate-horizontal-initial/x-90", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box, "initial": "ROT_90_X_NEG"}

    yield {"case_id": "shape/rotate-horizontal-initial/y-180", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "initial": "ROT_180_FACE_XZ"}

    yield {"case_id": "shape/rotate-horizontal-initial/y-90", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box,
           "initial": "ROT_90_Y_NEG"}

    yield {"case_id": "shape/rotate-horizontal-initial/z-90", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box,
           "initial": "ROT_90_Z_NEG"}

    yield {"case_id": "shape/rotate-horizontal-initial/xy-180", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box,
           "initial": "ROT_180_FACE_XY"}

    yield {"case_id": "shape/rotate-all-initial/identity", "family": "math", "operation": "shape",
           "kind": "rotate_all_initial", "box": rotate_box, "initial": "IDENTITY",
           "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-all-initial/x-90", "family": "math", "operation": "shape",
           "kind": "rotate_all_initial", "box": rotate_box, "initial": "ROT_90_X_NEG",
           "center": center_bits(0.25, 0.5, 0.75)}

    yield {"case_id": "shape/rotate-all-initial/y-180", "family": "math", "operation": "shape",
           "kind": "rotate_all_initial", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "initial": "ROT_180_FACE_XZ", "center": center_bits(0.75, 0.25, 0.5)}

    yield {"case_id": "shape/rotate-all/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all", "box": rotate_box}

    yield {"case_id": "shape/rotate-all/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_all", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}

    yield {"case_id": "shape/rotate-all/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}

    yield {"case_id": "shape/rotate-attach-face/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face", "box": rotate_box}

    yield {"case_id": "shape/rotate-attach-face/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}

    yield {"case_id": "shape/rotate-attach-face/empty", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}

    yield {"case_id": "shape/rotate-attach-face-initial/identity", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face_initial", "box": rotate_box, "initial": "IDENTITY"}

    yield {"case_id": "shape/rotate-attach-face-initial/x-90", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face_initial", "box": rotate_box, "initial": "ROT_90_X_NEG"}

    yield {"case_id": "shape/rotate-attach-face-initial/y-180", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face_initial", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "initial": "ROT_180_FACE_XZ"}
