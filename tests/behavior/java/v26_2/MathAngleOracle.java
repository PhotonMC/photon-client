import com.google.gson.JsonObject;
import net.minecraft.util.Mth;
import org.joml.Quaternionf;
import org.joml.Vector3f;

class MathAngleOracle extends MathAlgorithmOracle {
    static JsonObject mthAngleWrapDifferenceCase(JsonObject request) {
        var result = new JsonObject();
        if (!request.get("type").getAsString().equals("f32")) {
            throw new IllegalArgumentException("Unknown Mth angle wrap/difference type");
        }
        String function = request.get("fn").getAsString();
        float value = switch (function) {
            case "wrap_degrees_90" -> Mth.wrapDegrees90(Oracle.f32(request, "value"));
            case "degrees_difference" -> Mth.degreesDifference(
                Oracle.f32(request, "from"), Oracle.f32(request, "to"));
            case "degrees_difference_abs" -> Mth.degreesDifferenceAbs(
                Oracle.f32(request, "from"), Oracle.f32(request, "to"));
            default -> throw new IllegalArgumentException("Unknown Mth angle wrap/difference function");
        };
        result.addProperty("bits", Oracle.bits(value));
        return result;
    }

    static JsonObject mthAngleApproachCase(JsonObject request) {
        var result = new JsonObject();
        if (!request.get("type").getAsString().equals("f32")) {
            throw new IllegalArgumentException("Unknown Mth angle approach type");
        }
        String function = request.get("fn").getAsString();
        float value = switch (function) {
            case "rotate_if_necessary" -> Mth.rotateIfNecessary(
                Oracle.f32(request, "base"), Oracle.f32(request, "target"),
                Oracle.f32(request, "max"));
            case "approach" -> Mth.approach(
                Oracle.f32(request, "current"), Oracle.f32(request, "target"),
                Oracle.f32(request, "increment"));
            case "approach_degrees" -> Mth.approachDegrees(
                Oracle.f32(request, "current"), Oracle.f32(request, "target"),
                Oracle.f32(request, "increment"));
            default -> throw new IllegalArgumentException("Unknown Mth angle approach function");
        };
        result.addProperty("bits", Oracle.bits(value));
        return result;
    }

    static JsonObject mthAtan2Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.atan2(
            f64(request, "y"), f64(request, "x"))));
        return result;
    }

    static JsonObject mthAngleHelpersCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        String function = request.get("fn").getAsString();
        if (function.equals("pack_degrees")) {
            if (!width.equals("f32")) {
                throw new IllegalArgumentException("Unknown Mth packDegrees type");
            }
            byte value = Mth.packDegrees(Oracle.f32(request, "input"));
            result.addProperty("value", Byte.toString(value));
        } else if (function.equals("unpack_degrees")) {
            if (!width.equals("i8")) {
                throw new IllegalArgumentException("Unknown Mth unpackDegrees type");
            }
            byte input = Byte.parseByte(request.get("input").getAsString());
            result.addProperty("bits", Oracle.bits(Mth.unpackDegrees(input)));
        } else if (function.equals("wrap_degrees")) {
            if (!width.equals("i64")) {
                throw new IllegalArgumentException("Unknown Mth long wrapDegrees type");
            }
            long input = Long.parseLong(request.get("input").getAsString());
            result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(input)));
        } else {
            throw new IllegalArgumentException("Unknown Mth angle function");
        }
        return result;
    }

    static JsonObject mthRotationAroundAxisCase(JsonObject request) {
        var result = new JsonObject();
        Vector3f axis = new Vector3f(
            Oracle.f32(request, "axis_x"),
            Oracle.f32(request, "axis_y"),
            Oracle.f32(request, "axis_z"));
        Quaternionf rotation = new Quaternionf(
            Oracle.f32(request, "rotation_x"),
            Oracle.f32(request, "rotation_y"),
            Oracle.f32(request, "rotation_z"),
            Oracle.f32(request, "rotation_w"));
        result.add("value", snapshot(Mth.rotationAroundAxis(
            axis, rotation, new Quaternionf())));
        return result;
    }
}
