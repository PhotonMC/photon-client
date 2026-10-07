import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.JsonOps;
import java.util.EnumSet;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.util.RandomSource;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;

class MathVectorOracle extends MathTickFixtures {
    static JsonObject codecDecodeResult(DataResult<Vec3> decoded) {
        var result = new JsonObject();
        decoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            result.add("value", snapshot(value));
        });
        decoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
            error.partialValue().ifPresent(value -> result.add("partial", snapshot(value)));
        });
        return result;
    }

    static JsonObject codecEncodeResult(DataResult<JsonElement> encoded) {
        var result = new JsonObject();
        encoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            var values = new JsonArray();
            for (var entry : value.getAsJsonArray()) {
                values.add(Oracle.bits(entry.getAsDouble()));
            }
            result.add("value", values);
        });
        encoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
        });
        return result;
    }

    static JsonObject vec2Case(JsonObject request) {
        var result = new JsonObject();
        Vec2 vector = new Vec2(Oracle.f32(request, "x"), Oracle.f32(request, "y"));
        var events = new JsonArray();
        events.add(snapshot(vector));
        for (var entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            vector = switch (step.get("action").getAsString()) {
                case "normalize" -> vector.normalized();
                case "scale" -> vector.scale(Oracle.f32(step, "factor"));
                default -> throw new IllegalArgumentException("Unknown Vec2 action");
            };
            events.add(snapshot(vector));
        }
        result.add("events", events);
        return result;
    }

    static JsonObject coordCase(JsonObject request) {
        var result = new JsonObject();
        int value = Integer.parseInt(request.get("value").getAsString());
        String fn = request.get("fn").getAsString();
        int out = switch (fn) {
            case "block_to_section" -> SectionPos.blockToSectionCoord(value);
            case "section_to_block" -> SectionPos.sectionToBlockCoord(value);
            case "section_relative" -> SectionPos.sectionRelative(value);
            default -> throw new IllegalArgumentException("Unknown coord function");
        };
        result.addProperty("value", Integer.toString(out));
        return result;
    }

    static JsonObject vec3CodecCase(JsonObject request) {
        var result = new JsonObject();
        String mode = request.get("mode").getAsString();
        if (mode.equals("decode")) {
            var input = new JsonArray();
            for (var entry : request.getAsJsonArray("values")) {
                input.add(Double.longBitsToDouble(Long.parseUnsignedLong(entry.getAsString(), 16)));
            }
            DataResult<Vec3> decoded = Vec3.CODEC.parse(JsonOps.INSTANCE, input);
            result = codecDecodeResult(decoded);
        } else if (mode.equals("encode")) {
            DataResult<JsonElement> encoded = Vec3.CODEC.encodeStart(JsonOps.INSTANCE, vec3(request));
            result = codecEncodeResult(encoded);
        } else {
            throw new IllegalArgumentException("Unknown Vec3 codec mode");
        }
        return result;
    }

    static JsonObject vec3Case(JsonObject request) {
        var result = new JsonObject();
        Vec3 vector = new Vec3(f64(request, "x"), f64(request, "y"), f64(request, "z"));
        var events = new JsonArray();
        events.add(snapshot(vector));
        for (var entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            vector = switch (step.get("action").getAsString()) {
                case "normalize" -> vector.normalize();
                case "scale" -> vector.scale(f64(step, "factor"));
                case "reverse" -> vector.reverse();
                case "horizontal" -> vector.horizontal();
                case "multiply_vec3" -> vector.multiply(vec3(step.getAsJsonObject("scale")));
                case "multiply_components" -> vector.multiply(
                    f64(step, "x"), f64(step, "y"), f64(step, "z"));
                case "add_vec3" -> vector.add(vec3(step.getAsJsonObject("delta")));
                case "add_scalar" -> vector.add(f64(step, "value"));
                case "add_components" -> vector.add(
                    f64(step, "x"), f64(step, "y"), f64(step, "z"));
                case "subtract_vec3" -> vector.subtract(vec3(step.getAsJsonObject("delta")));
                case "subtract_scalar" -> vector.subtract(f64(step, "value"));
                case "subtract_components" -> vector.subtract(
                    f64(step, "x"), f64(step, "y"), f64(step, "z"));
                case "vector_to" -> vector.vectorTo(vec3(step.getAsJsonObject("target")));
                case "cross" -> vector.cross(vec3(step.getAsJsonObject("other")));
                default -> throw new IllegalArgumentException("Unknown Vec3 action");
            };
            events.add(snapshot(vector));
        }
        result.add("events", events);
        return result;
    }

    static JsonObject vec3DotCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 second = vec3(request.getAsJsonObject("second"));
        result.addProperty("value", Oracle.bits(first.dot(second)));
        return result;
    }

    static JsonObject vec3DistanceToSqrCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 second = vec3(request.getAsJsonObject("second"));
        result.addProperty("value", Oracle.bits(first.distanceToSqr(second)));
        return result;
    }

    static JsonObject vec3DistanceToCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 second = vec3(request.getAsJsonObject("second"));
        result.addProperty("value", Oracle.bits(first.distanceTo(second)));
        return result;
    }

    static JsonObject vec3DistanceToSqrComponentsCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.addProperty("value", Oracle.bits(first.distanceToSqr(
            f64(request, "x"), f64(request, "y"), f64(request, "z"))));
        return result;
    }

    static JsonObject vec3HorizontalDistancesCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.addProperty("distance", Oracle.bits(first.horizontalDistance()));
        result.addProperty("distance_squared", Oracle.bits(first.horizontalDistanceSqr()));
        return result;
    }

    static JsonObject vec3CloserThanCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 second = vec3(request.getAsJsonObject("second"));
        result.addProperty("position", first.closerThan(second, f64(request, "distance")));
        result.addProperty("xz_y", first.closerThan(
            second, f64(request, "distance_xz"), f64(request, "distance_y")));
        return result;
    }

    static JsonObject vec3OffsetRandomCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        float offset = Oracle.f32(request, "offset");
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        result.add("offset_random", snapshot(first.offsetRandom(random, offset)));
        RandomSource randomXZ = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        result.add("offset_random_xz", snapshot(first.offsetRandomXZ(randomXZ, offset)));
        return result;
    }

    static JsonObject vec3LerpCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 second = vec3(request.getAsJsonObject("second"));
        result.add("value", snapshot(first.lerp(second, f64(request, "alpha"))));
        return result;
    }

    static JsonObject vec3RotateClockwise90Case(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.add("value", snapshot(first.rotateClockwise90()));
        return result;
    }

    static JsonObject vec3IsFiniteCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.addProperty("value", first.isFinite());
        return result;
    }

    static JsonObject vec3GetCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        result.addProperty("value", Oracle.bits(first.get(axis)));
        return result;
    }

    static JsonObject vec3WithCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        result.add("value", snapshot(first.with(axis, f64(request, "value"))));
        return result;
    }

    static JsonObject vec3RelativeCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.add("value", snapshot(first.relative(direction, f64(request, "distance"))));
        return result;
    }

    static JsonObject vec3AlignCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        EnumSet<Direction.Axis> axes = EnumSet.noneOf(Direction.Axis.class);
        for (var entry : request.getAsJsonArray("axes")) {
            Direction.Axis axis = Direction.Axis.byName(entry.getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            axes.add(axis);
        }
        result.add("value", snapshot(first.align(axes)));
        return result;
    }

    static JsonObject vec3ProjectedOnCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 onto = vec3(request.getAsJsonObject("onto"));
        result.add("value", snapshot(first.projectedOn(onto)));
        return result;
    }

    static JsonObject vec3ToVector3fCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.add("value", snapshot(first.toVector3f()));
        return result;
    }

    static JsonObject vec3CoordinatesCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        String kind = request.get("kind").getAsString();
        Vec3 value = switch (kind) {
            case "lower_corner" -> Vec3.atLowerCornerOf(pos);
            case "lower_corner_offset" -> Vec3.atLowerCornerWithOffset(
                pos, f64(request, "offset_x"), f64(request, "offset_y"), f64(request, "offset_z"));
            case "center" -> Vec3.atCenterOf(pos);
            case "bottom_center" -> Vec3.atBottomCenterOf(pos);
            case "up_from_bottom_center" -> Vec3.upFromBottomCenterOf(pos, f64(request, "y_offset"));
            default -> throw new IllegalArgumentException("Unknown Vec3 coordinate helper");
        };
        result.add("value", snapshot(value));
        return result;
    }

    static JsonObject vec3EqualityCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        Vec3 second = vec3(request.getAsJsonObject("second"));
        result.addProperty("equals", first.equals(second));
        result.addProperty("first_hash", Integer.toString(first.hashCode()));
        result.addProperty("second_hash", Integer.toString(second.hashCode()));
        return result;
    }

    static JsonObject vec3ToStringCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.addProperty("value", first.toString());
        return result;
    }

    static JsonObject vec3RotateCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        float radians = Oracle.f32(request, "radians");
        Vec3 value = switch (request.get("axis").getAsString()) {
            case "x" -> first.xRot(radians);
            case "y" -> first.yRot(radians);
            case "z" -> first.zRot(radians);
            default -> throw new IllegalArgumentException("Unknown Vec3 rotation axis");
        };
        result.add("value", snapshot(value));
        return result;
    }

    static JsonObject vec3DirectionFromRotationCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 value = switch (request.get("kind").getAsString()) {
            case "components" -> Vec3.directionFromRotation(
                Oracle.f32(request, "rot_x"), Oracle.f32(request, "rot_y"));
            case "vec2" -> {
                JsonObject rotation = request.getAsJsonObject("rotation");
                yield Vec3.directionFromRotation(new Vec2(
                    Oracle.f32(rotation, "x"), Oracle.f32(rotation, "y")));
            }
            default -> throw new IllegalArgumentException("Unknown Vec3 direction rotation kind");
        };
        result.add("value", snapshot(value));
        return result;
    }

    static JsonObject vec3LocalCoordinatesCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 value = switch (request.get("kind").getAsString()) {
            case "apply" -> Vec3.applyLocalCoordinatesToRotation(
                new Vec2(
                    Oracle.f32(request.getAsJsonObject("rotation"), "x"),
                    Oracle.f32(request.getAsJsonObject("rotation"), "y")),
                vec3(request.getAsJsonObject("direction")));
            case "add" -> vec3(request.getAsJsonObject("first"))
                .addLocalCoordinates(vec3(request.getAsJsonObject("direction")));
            default -> throw new IllegalArgumentException("Unknown Vec3 local coordinate kind");
        };
        result.add("value", snapshot(value));
        return result;
    }

    static JsonObject vec3RotationCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 first = vec3(request.getAsJsonObject("first"));
        result.add("value", snapshot(first.rotation()));
        return result;
    }

    static JsonObject vec2RotateCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject first = request.getAsJsonObject("first");
        Vec2 vector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
        result.add("value", snapshot(vector.rotate(f64(request, "angle"))));
        return result;
    }

    static JsonObject vec2DotCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject first = request.getAsJsonObject("first");
        JsonObject second = request.getAsJsonObject("second");
        Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
        Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
        result.addProperty("value", Oracle.bits(firstVector.dot(secondVector)));
        return result;
    }

    static JsonObject vec2AddVec2Case(JsonObject request) {
        var result = new JsonObject();
        JsonObject first = request.getAsJsonObject("first");
        JsonObject second = request.getAsJsonObject("second");
        Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
        Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
        result.add("value", snapshot(firstVector.add(secondVector)));
        return result;
    }

    static JsonObject vec2AddScalarCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject first = request.getAsJsonObject("first");
        Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
        result.add("value", snapshot(firstVector.add(Oracle.f32(request, "value"))));
        return result;
    }

    static JsonObject vec2DistanceToSqrCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject first = request.getAsJsonObject("first");
        JsonObject second = request.getAsJsonObject("second");
        Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
        Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
        result.addProperty("value", Oracle.bits(firstVector.distanceToSqr(secondVector)));
        return result;
    }

    static JsonObject vec2NegatedCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject first = request.getAsJsonObject("first");
        Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
        result.add("value", snapshot(firstVector.negated()));
        return result;
    }

    static JsonObject vec2EqualityCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject firstObject = request.getAsJsonObject("first");
        JsonObject secondObject = request.getAsJsonObject("second");
        Vec2 first = new Vec2(Oracle.f32(firstObject, "x"), Oracle.f32(firstObject, "y"));
        Vec2 second = new Vec2(Oracle.f32(secondObject, "x"), Oracle.f32(secondObject, "y"));
        result.addProperty("equals", first.equals(second));
        result.addProperty("first_hash", Integer.toString(first.hashCode()));
        result.addProperty("second_hash", Integer.toString(second.hashCode()));
        return result;
    }

    static JsonObject vec2ConstantCase(JsonObject request) {
        var result = new JsonObject();
        Vec2 value = switch (request.get("name").getAsString()) {
            case "zero" -> Vec2.ZERO;
            case "one" -> Vec2.ONE;
            case "unit_x" -> Vec2.UNIT_X;
            case "neg_unit_x" -> Vec2.NEG_UNIT_X;
            case "unit_y" -> Vec2.UNIT_Y;
            case "neg_unit_y" -> Vec2.NEG_UNIT_Y;
            case "max" -> Vec2.MAX;
            case "min" -> Vec2.MIN;
            default -> throw new IllegalArgumentException("Unknown Vec2 constant");
        };
        result.add("value", snapshot(value));
        return result;
    }

    static JsonObject vec3ConstantCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 value = switch (request.get("name").getAsString()) {
            case "zero" -> Vec3.ZERO;
            case "x_axis" -> Vec3.X_AXIS;
            case "y_axis" -> Vec3.Y_AXIS;
            case "z_axis" -> Vec3.Z_AXIS;
            default -> throw new IllegalArgumentException("Unknown Vec3 constant");
        };
        result.add("value", snapshot(value));
        return result;
    }
}
