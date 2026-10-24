import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.util.Mth;
import net.minecraft.util.SmoothDouble;

class MathInterpolationOracle extends MathDirectionOracle {
    static JsonObject smoothDoubleCase(JsonObject request) {
        var result = new JsonObject();
        SmoothDouble smooth = new SmoothDouble();
        var events = new JsonArray();
        for (JsonElement entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            String op = step.get("op").getAsString();
            if (op.equals("reset")) {
                smooth.reset();
                var state = new JsonObject();
                state.addProperty("op", "reset");
                events.add(state);
            } else if (op.equals("delta")) {
                double value = smooth.getNewDeltaValue(f64(step, "target_delta"), f64(step, "time"));
                var state = new JsonObject();
                state.addProperty("op", "delta");
                state.addProperty("value", Oracle.bits(value));
                events.add(state);
            } else {
                throw new IllegalArgumentException("Unknown SmoothDouble operation");
            }
        }
        result.add("events", events);
        return result;
    }

    static JsonObject mthClampedLerpCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.clampedLerp(
                Oracle.f32(request, "factor"), Oracle.f32(request, "min"),
                Oracle.f32(request, "max"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.clampedLerp(
                f64(request, "factor"), f64(request, "min"), f64(request, "max"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth clampedLerp type");
        }
        return result;
    }

    static JsonObject mthQuantizeCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("value", Integer.toString(Mth.quantize(
            f64(request, "value"), Integer.parseInt(request.get("resolution").getAsString()))));
        return result;
    }

    static JsonObject mthInverseLerpCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.inverseLerp(
                Oracle.f32(request, "value"), Oracle.f32(request, "min"),
                Oracle.f32(request, "max"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.inverseLerp(
                f64(request, "value"), f64(request, "min"), f64(request, "max"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth inverseLerp type");
        }
        return result;
    }

    static JsonObject mthLerpCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.lerp(
                Oracle.f32(request, "alpha"), Oracle.f32(request, "p0"),
                Oracle.f32(request, "p1"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.lerp(
                f64(request, "alpha"), f64(request, "p0"), f64(request, "p1"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth lerp type");
        }
        return result;
    }

    static JsonObject mthLerpIntCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("value", Integer.toString(Mth.lerpInt(
            Oracle.f32(request, "alpha"),
            Integer.parseInt(request.get("p0").getAsString()),
            Integer.parseInt(request.get("p1").getAsString()))));
        return result;
    }

    static JsonObject mthLerpDiscreteCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("value", Integer.toString(Mth.lerpDiscrete(
            Oracle.f32(request, "alpha"),
            Integer.parseInt(request.get("p0").getAsString()),
            Integer.parseInt(request.get("p1").getAsString()))));
        return result;
    }

    static JsonObject mthSmoothstepCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.smoothstep(
            f64(request, "x"))));
        return result;
    }

    static JsonObject mthSmoothstepDerivativeCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.smoothstepDerivative(
            f64(request, "x"))));
        return result;
    }

    static JsonObject mthRotLerpCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.rotLerp(
                Oracle.f32(request, "alpha"), Oracle.f32(request, "from"),
                Oracle.f32(request, "to"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.rotLerp(
                f64(request, "alpha"), f64(request, "from"), f64(request, "to"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth rotLerp type");
        }
        return result;
    }

    static JsonObject mthRotLerpRadCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.rotLerpRad(
            Oracle.f32(request, "alpha"), Oracle.f32(request, "from"),
            Oracle.f32(request, "to"))));
        return result;
    }

    static JsonObject mthTriangleWaveCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.triangleWave(
            Oracle.f32(request, "index"), Oracle.f32(request, "period"))));
        return result;
    }

    static JsonObject mthClampedMapF64Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.clampedMap(
            f64(request, "value"), f64(request, "from_min"), f64(request, "from_max"),
            f64(request, "to_min"), f64(request, "to_max"))));
        return result;
    }

    static JsonObject mthClampedMapF32Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.clampedMap(
            Oracle.f32(request, "value"), Oracle.f32(request, "from_min"),
            Oracle.f32(request, "from_max"), Oracle.f32(request, "to_min"),
            Oracle.f32(request, "to_max"))));
        return result;
    }

    static JsonObject mthMapF64Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.map(
            f64(request, "value"), f64(request, "from_min"), f64(request, "from_max"),
            f64(request, "to_min"), f64(request, "to_max"))));
        return result;
    }

    static JsonObject mthMapF32Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.map(
            Oracle.f32(request, "value"), Oracle.f32(request, "from_min"),
            Oracle.f32(request, "from_max"), Oracle.f32(request, "to_min"),
            Oracle.f32(request, "to_max"))));
        return result;
    }

    static JsonObject mthWobbleF64Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.wobble(
            f64(request, "coord"))));
        return result;
    }

    static JsonObject mthInterpolationCase(JsonObject request) {
        var result = new JsonObject();
        String function = request.get("fn").getAsString();
        switch (function) {
            case "lerp2" -> result.addProperty("bits", Oracle.bits(Mth.lerp2(
                f64(request, "alpha1"), f64(request, "alpha2"),
                f64(request, "x00"), f64(request, "x10"),
                f64(request, "x01"), f64(request, "x11"))));
            case "lerp3" -> result.addProperty("bits", Oracle.bits(Mth.lerp3(
                f64(request, "alpha1"), f64(request, "alpha2"),
                f64(request, "alpha3"), f64(request, "x000"),
                f64(request, "x100"), f64(request, "x010"),
                f64(request, "x110"), f64(request, "x001"),
                f64(request, "x101"), f64(request, "x011"),
                f64(request, "x111"))));
            case "catmullrom" -> result.addProperty("bits", Oracle.bits(Mth.catmullrom(
                Oracle.f32(request, "alpha"), Oracle.f32(request, "p0"),
                Oracle.f32(request, "p1"), Oracle.f32(request, "p2"),
                Oracle.f32(request, "p3"))));
            default -> throw new IllegalArgumentException("Unknown Mth interpolation function");
        }
        return result;
    }
}
