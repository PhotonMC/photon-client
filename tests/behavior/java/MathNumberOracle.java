import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.util.Mth;
import org.apache.commons.lang3.math.Fraction;
import org.joml.Vector3f;

class MathNumberOracle extends MathLightOracle {
    static JsonObject mthConstantsCase(JsonObject request) {
        var result = new JsonObject();
        switch (request.get("name").getAsString()) {
            case "pi" -> result.addProperty("bits", Oracle.bits(Mth.PI));
            case "half_pi" -> result.addProperty("bits", Oracle.bits(Mth.HALF_PI));
            case "two_pi" -> result.addProperty("bits", Oracle.bits(Mth.TWO_PI));
            case "deg_to_rad" -> result.addProperty("bits", Oracle.bits(Mth.DEG_TO_RAD));
            case "rad_to_deg" -> result.addProperty("bits", Oracle.bits(Mth.RAD_TO_DEG));
            case "epsilon" -> result.addProperty("bits", Oracle.bits(Mth.EPSILON));
            case "sqrt_of_two" -> result.addProperty("bits", Oracle.bits(Mth.SQRT_OF_TWO));
            case "y_axis" -> result.add("value", snapshot(new Vector3f(Mth.Y_AXIS)));
            case "x_axis" -> result.add("value", snapshot(new Vector3f(Mth.X_AXIS)));
            case "z_axis" -> result.add("value", snapshot(new Vector3f(Mth.Z_AXIS)));
            default -> throw new IllegalArgumentException("Unknown Mth constant");
        }
        return result;
    }

    static JsonObject mthFloatingPrimitivesCase(JsonObject request) {
        var result = new JsonObject();
        String function = request.get("fn").getAsString();
        if (function.equals("sin")) {
            result.addProperty("bits", Oracle.bits(Mth.sin(f64(request, "value"))));
        } else if (function.equals("cos")) {
            result.addProperty("bits", Oracle.bits(Mth.cos(f64(request, "value"))));
        } else if (function.equals("sqrt")) {
            result.addProperty("bits", Oracle.bits(Mth.sqrt(Oracle.f32(request, "value"))));
        } else if (function.equals("abs")) {
            result.addProperty("bits", Oracle.bits(Mth.abs(Oracle.f32(request, "value"))));
        } else if (function.equals("clamp")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.clamp(
                    Oracle.f32(request, "value"), Oracle.f32(request, "min"),
                    Oracle.f32(request, "max"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.clamp(
                    f64(request, "value"), f64(request, "min"), f64(request, "max"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth clamp type");
            }
        } else {
            throw new IllegalArgumentException("Unknown Mth floating primitive");
        }
        return result;
    }

    static JsonObject mthPowerHelpersCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        String function = request.get("fn").getAsString();
        if (width.equals("i32")) {
            int input = Integer.parseInt(request.get("input").getAsString());
            if (function.equals("is_power_of_two")) {
                result.addProperty("value", Mth.isPowerOfTwo(input));
            } else {
                int value = switch (function) {
                    case "smallest_encompassing_power_of_two" -> Mth.smallestEncompassingPowerOfTwo(input);
                    case "smallest_square_side" -> Mth.smallestSquareSide(input);
                    case "ceillog2" -> Mth.ceillog2(input);
                    case "log2" -> Mth.log2(input);
                    default -> throw new IllegalArgumentException("Unknown Mth i32 power helper");
                };
                result.addProperty("value", Integer.toString(value));
            }
        } else if (width.equals("i64")) {
            if (!function.equals("is_power_of_two")) {
                throw new IllegalArgumentException("Unknown Mth i64 power helper");
            }
            result.addProperty("value", Mth.isPowerOfTwo(
                Long.parseLong(request.get("input").getAsString())));
        } else {
            throw new IllegalArgumentException("Unknown Mth power helper type");
        }
        return result;
    }

    static JsonObject mthFracCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (!request.get("fn").getAsString().equals("frac")) {
            throw new IllegalArgumentException("Unknown Mth frac function");
        }
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.frac(Oracle.f32(request, "input"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.frac(f64(request, "input"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth frac type");
        }
        return result;
    }

    static JsonObject mthGetIntCase(JsonObject request) {
        var result = new JsonObject();
        JsonElement inputElement = request.get("input");
        String input = inputElement.isJsonNull() ? null : inputElement.getAsString();
        int defaultValue = Integer.parseInt(request.get("default").getAsString());
        result.addProperty("value", Integer.toString(Mth.getInt(input, defaultValue)));
        return result;
    }

    static JsonObject mthIntegerHelpersCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        String function = request.get("fn").getAsString();
        if (width.equals("i32")) {
            int input = Integer.parseInt(request.get("input").getAsString());
            int divisor = Integer.parseInt(request.get("divisor").getAsString());
            int value = switch (function) {
                case "floor_div" -> Mth.floorDiv(input, divisor);
                case "positive_modulo" -> Mth.positiveModulo(input, divisor);
                case "positive_ceil_div" -> Mth.positiveCeilDiv(input, divisor);
                case "round_toward" -> Mth.roundToward(input, divisor);
                default -> throw new IllegalArgumentException("Unknown Mth integer function");
            };
            result.addProperty("value", Integer.toString(value));
        } else if (width.equals("i64")) {
            long input = Long.parseLong(request.get("input").getAsString());
            long divisor = Long.parseLong(request.get("divisor").getAsString());
            long value = switch (function) {
                case "positive_ceil_div" -> Mth.positiveCeilDiv(input, divisor);
                case "round_toward" -> Mth.roundToward(input, divisor);
                default -> throw new IllegalArgumentException("Unknown Mth long function");
            };
            result.addProperty("value", Long.toString(value));
        } else {
            throw new IllegalArgumentException("Unknown Mth integer type");
        }
        return result;
    }

    static JsonObject mthIntegerBasicsCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        String function = request.get("fn").getAsString();
        if (width.equals("i32")) {
            int input = Integer.parseInt(request.get("input").getAsString());
            if (function.equals("is_multiple")) {
                result.addProperty("value", Mth.isMultipleOf(input,
                    Integer.parseInt(request.get("divisor").getAsString())));
            } else {
                int value = switch (function) {
                    case "abs" -> Mth.abs(input);
                    case "clamp" -> Mth.clamp(input,
                        Integer.parseInt(request.get("min").getAsString()),
                        Integer.parseInt(request.get("max").getAsString()));
                    case "wrap_degrees" -> Mth.wrapDegrees(input);
                    default -> throw new IllegalArgumentException("Unknown Mth integer basic function");
                };
                result.addProperty("value", Integer.toString(value));
            }
        } else if (width.equals("i64")) {
            long input = Long.parseLong(request.get("input").getAsString());
            if (!function.equals("clamp")) {
                throw new IllegalArgumentException("Unknown Mth long basic function");
            }
            long value = Mth.clamp(input,
                Long.parseLong(request.get("min").getAsString()),
                Long.parseLong(request.get("max").getAsString()));
            result.addProperty("value", Long.toString(value));
        } else {
            throw new IllegalArgumentException("Unknown Mth integer basic type");
        }
        return result;
    }

    static JsonObject mthRoundingHelpersCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        String function = request.get("fn").getAsString();
        if (function.equals("ceil")) {
            int value;
            if (width.equals("f32")) {
                value = Mth.ceil(Oracle.f32(request, "input"));
            } else if (width.equals("f64")) {
                value = Mth.ceil(f64(request, "input"));
            } else {
                throw new IllegalArgumentException("Unknown Mth ceil type");
            }
            result.addProperty("value", Integer.toString(value));
        } else if (function.equals("lfloor")) {
            if (!width.equals("f64")) {
                throw new IllegalArgumentException("Unknown Mth lfloor type");
            }
            result.addProperty("value", Long.toString(Mth.lfloor(f64(request, "input"))));
        } else if (function.equals("ceil_long")) {
            if (!width.equals("f64")) {
                throw new IllegalArgumentException("Unknown Mth ceilLong type");
            }
            result.addProperty("value", Long.toString(Mth.ceilLong(f64(request, "input"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth rounding function");
        }
        return result;
    }

    static JsonObject mthPositiveModuloCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.positiveModulo(
                Oracle.f32(request, "input"), Oracle.f32(request, "mod"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.positiveModulo(
                f64(request, "input"), f64(request, "mod"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth positiveModulo type");
        }
        return result;
    }

    static JsonObject mthAbsMaxCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.absMax(
                Oracle.f32(request, "first"), Oracle.f32(request, "second"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.absMax(
                f64(request, "first"), f64(request, "second"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth absMax type");
        }
        return result;
    }

    static JsonObject mthEqualCase(JsonObject request) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            result.addProperty("equal", Mth.equal(
                Oracle.f32(request, "first"), Oracle.f32(request, "second")));
        } else if (width.equals("f64")) {
            result.addProperty("equal", Mth.equal(
                f64(request, "first"), f64(request, "second")));
        } else {
            throw new IllegalArgumentException("Unknown Mth equal type");
        }
        return result;
    }

    static JsonObject mthLengthCase(JsonObject request) {
        var result = new JsonObject();
        String type = request.get("type").getAsString();
        String function = request.get("fn").getAsString();
        if (type.equals("f64_2")) {
            double x = f64(request, "x");
            double y = f64(request, "y");
            double value = switch (function) {
                case "length_squared" -> Mth.lengthSquared(x, y);
                case "length" -> Mth.length(x, y);
                default -> throw new IllegalArgumentException("Unknown Mth length function");
            };
            result.addProperty("bits", Oracle.bits(value));
        } else if (type.equals("f32_2")) {
            if (!function.equals("length")) {
                throw new IllegalArgumentException("Unknown Mth length function");
            }
            result.addProperty("bits", Oracle.bits(Mth.length(
                Oracle.f32(request, "x"), Oracle.f32(request, "y"))));
        } else if (type.equals("f64_3")) {
            double x = f64(request, "x");
            double y = f64(request, "y");
            double z = f64(request, "z");
            double value = switch (function) {
                case "length_squared" -> Mth.lengthSquared(x, y, z);
                case "length" -> Mth.length(x, y, z);
                default -> throw new IllegalArgumentException("Unknown Mth length function");
            };
            result.addProperty("bits", Oracle.bits(value));
        } else if (type.equals("f32_3")) {
            if (!function.equals("length_squared")) {
                throw new IllegalArgumentException("Unknown Mth length function");
            }
            result.addProperty("bits", Oracle.bits(Mth.lengthSquared(
                Oracle.f32(request, "x"), Oracle.f32(request, "y"),
                Oracle.f32(request, "z"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth length type");
        }
        return result;
    }

    static JsonObject mthOutFromOriginCase(JsonObject request) {
        var result = new JsonObject();
        int origin = Integer.parseInt(request.get("origin").getAsString());
        int lowerBound = Integer.parseInt(request.get("lower").getAsString());
        int upperBound = Integer.parseInt(request.get("upper").getAsString());
        try {
            var positions = switch (request.get("mode").getAsString()) {
                case "default" -> Mth.outFromOrigin(origin, lowerBound, upperBound);
                case "step" -> Mth.outFromOrigin(origin, lowerBound, upperBound,
                    Integer.parseInt(request.get("step").getAsString()));
                default -> throw new IllegalArgumentException("Unknown Mth outFromOrigin mode");
            };
            var values = new JsonArray();
            positions.forEach(value -> values.add(value));
            result.addProperty("outcome", "value");
            result.add("value", values);
        } catch (IllegalArgumentException error) {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.getMessage());
            result.addProperty("java_exception", error.getClass().getName());
        }
        return result;
    }

    static JsonObject mthMulAndTruncateCase(JsonObject request) {
        var result = new JsonObject();
        Fraction fraction = Fraction.getFraction(
            Integer.parseInt(request.get("numerator").getAsString()),
            Integer.parseInt(request.get("denominator").getAsString()));
        result.addProperty("value", Integer.toString(Mth.mulAndTruncate(
            fraction, Integer.parseInt(request.get("factor").getAsString()))));
        return result;
    }

    static JsonObject mthSignCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("value", Integer.toString(Mth.sign(f64(request, "x"))));
        return result;
    }

    static JsonObject mthSquareF32Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.square(Oracle.f32(request, "x"))));
        return result;
    }

    static JsonObject mthCubeF32Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.cube(Oracle.f32(request, "x"))));
        return result;
    }

    static JsonObject mthSquareF64Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.square(f64(request, "x"))));
        return result;
    }

    static JsonObject mthSquareI32Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("value", Integer.toString(Mth.square(
            Integer.parseInt(request.get("x").getAsString()))));
        return result;
    }

    static JsonObject mthSquareI64Case(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("value", Long.toString(Mth.square(
            Long.parseLong(request.get("x").getAsString()))));
        return result;
    }

    static JsonObject ElseCase(JsonObject request, String operation) {
        var result = new JsonObject();
        String width = request.get("type").getAsString();
        if (width.equals("f32")) {
            float value = Oracle.f32(request, "value");
            switch (operation) {
                case "floor" -> result.addProperty("value", Integer.toString(Mth.floor(value)));
                case "wrap_degrees" -> result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(value)));
                case "clamp" -> result.addProperty("bits", Oracle.bits(Mth.clamp(value,
                    Oracle.f32(request, "min"), Oracle.f32(request, "max"))));
                default -> throw new IllegalArgumentException("Unknown math operation");
            }
        } else if (width.equals("f64")) {
            double value = f64(request, "value");
            switch (operation) {
                case "floor" -> result.addProperty("value", Integer.toString(Mth.floor(value)));
                case "wrap_degrees" -> result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(value)));
                case "clamp" -> result.addProperty("bits", Oracle.bits(Mth.clamp(value,
                    f64(request, "min"), f64(request, "max"))));
                default -> throw new IllegalArgumentException("Unknown math operation");
            }
        } else {
            throw new IllegalArgumentException("Unknown floating-point type");
        }

        return result;
    }
}
