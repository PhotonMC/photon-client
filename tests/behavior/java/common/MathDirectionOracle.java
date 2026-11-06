import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.Locale;
import net.minecraft.core.AxisCycle;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.util.RandomSource;
import org.joml.Vector3f;

class MathDirectionOracle extends MathChunkCacheOracle {
    static Direction directionOrElse(JsonObject object) {
        String name = object.get("or_else").getAsString();
        if (name.equals("none")) {
            return null;
        }
        Direction direction = Direction.byName(name);
        if (direction == null) {
            throw new IllegalArgumentException("Unknown fallback direction");
        }
        return direction;
    }

    static JsonObject directionCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        var events = new JsonArray();
        events.add(snapshot(facing));
        for (var entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            Direction.Axis axis = Direction.Axis.byName(step.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            facing = step.get("clockwise").getAsBoolean()
                ? facing.getClockWise(axis) : facing.getCounterClockWise(axis);
            events.add(snapshot(facing));
        }
        result.add("events", events);
        return result;
    }

    static JsonObject directionAxialRotationCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        result.add("clockwise", snapshot(facing.getClockWise(axis)));
        result.add("counter_clockwise", snapshot(facing.getCounterClockWise(axis)));
        return result;
    }

    static JsonObject directionYRotationCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        var clockwise = new JsonObject();
        try {
            clockwise.addProperty("outcome", "value");
            clockwise.add("value", snapshot(facing.getClockWise()));
        } catch (IllegalStateException error) {
            clockwise.addProperty("outcome", "error");
            clockwise.addProperty("error", error.getMessage());
        }
        var counterClockwise = new JsonObject();
        try {
            counterClockwise.addProperty("outcome", "value");
            counterClockwise.add("value", snapshot(facing.getCounterClockWise()));
        } catch (IllegalStateException error) {
            counterClockwise.addProperty("outcome", "error");
            counterClockwise.addProperty("error", error.getMessage());
        }
        result.add("clockwise", clockwise);
        result.add("counter_clockwise", counterClockwise);
        return result;
    }

    static JsonObject directionDataCase(JsonObject request) {
        var result = new JsonObject();
        int value = Integer.parseInt(request.get("value").getAsString());
        Direction facing = switch (request.get("type").getAsString()) {
            case "3d" -> Direction.from3DDataValue(value);
            case "2d" -> Direction.from2DDataValue(value);
            default -> throw new IllegalArgumentException("Unknown Direction data type");
        };
        result.add("value", snapshot(facing));
        return result;
    }

    static JsonObject directionYRotCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", snapshot(Direction.fromYRot(f64(request, "value"))));
        return result;
    }

    static JsonObject directionAxisCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        Direction.AxisDirection direction = switch (request.get("direction").getAsString()) {
            case "positive" -> Direction.AxisDirection.POSITIVE;
            case "negative" -> Direction.AxisDirection.NEGATIVE;
            default -> throw new IllegalArgumentException("Unknown axis direction");
        };
        result.add("value", snapshot(Direction.fromAxisAndDirection(axis, direction)));
        return result;
    }

    static JsonObject directionGetCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        Direction.AxisDirection direction = switch (request.get("direction").getAsString()) {
            case "positive" -> Direction.AxisDirection.POSITIVE;
            case "negative" -> Direction.AxisDirection.NEGATIVE;
            default -> throw new IllegalArgumentException("Unknown axis direction");
        };
        result.add("value", snapshot(Direction.get(direction, axis)));
        return result;
    }

    static JsonObject directionUnitVectorsCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        var unitVec3i = new JsonObject();
        unitVec3i.addProperty("x", facing.getUnitVec3i().getX());
        unitVec3i.addProperty("y", facing.getUnitVec3i().getY());
        unitVec3i.addProperty("z", facing.getUnitVec3i().getZ());
        result.add("unit_vec3i", unitVec3i);
        result.add("unit_vec3", snapshot(facing.getUnitVec3()));
        result.add("unit_vec3f", snapshot(new Vector3f(facing.getUnitVec3f())));
        return result;
    }

    static JsonObject directionStepCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.addProperty("step_x", facing.getStepX());
        result.addProperty("step_y", facing.getStepY());
        result.addProperty("step_z", facing.getStepZ());
        result.add("step", snapshot(facing.step()));
        return result;
    }

    static JsonObject directionAxisOrientationCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        result.addProperty("vertical", axis.isVertical());
        result.addProperty("horizontal", axis.isHorizontal());
        return result;
    }

    static JsonObject directionAxisDirectionsCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        var directions = new JsonArray();
        for (Direction direction : axis.getDirections()) {
            directions.add(snapshot(direction));
        }
        result.add("positive", snapshot(axis.getPositive()));
        result.add("negative", snapshot(axis.getNegative()));
        result.add("directions", directions);
        return result;
    }

    static JsonObject directionAxisDirectionValuesCase(JsonObject request) {
        var result = new JsonObject();
        Direction.AxisDirection direction = switch (request.get("direction").getAsString()) {
            case "positive" -> Direction.AxisDirection.POSITIVE;
            case "negative" -> Direction.AxisDirection.NEGATIVE;
            default -> throw new IllegalArgumentException("Unknown axis direction");
        };
        result.addProperty("step", direction.getStep());
        result.addProperty("name", direction.getName());
        result.addProperty("opposite", direction.opposite().getName());
        return result;
    }

    static JsonObject directionAxisChooseCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        int intValue = axis.choose(
            Integer.parseInt(request.get("int_x").getAsString()),
            Integer.parseInt(request.get("int_y").getAsString()),
            Integer.parseInt(request.get("int_z").getAsString()));
        double doubleValue = axis.choose(
            f64(request, "double_x"), f64(request, "double_y"), f64(request, "double_z"));
        boolean booleanValue = axis.choose(
            request.get("bool_x").getAsBoolean(), request.get("bool_y").getAsBoolean(),
            request.get("bool_z").getAsBoolean());
        result.addProperty("int", intValue);
        result.addProperty("double", Oracle.bits(doubleValue));
        result.addProperty("boolean", booleanValue);
        return result;
    }

    static JsonObject directionAxisTestCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        Direction input = request.get("direction").getAsString().equals("none")
            ? null : Direction.byName(request.get("direction").getAsString());
        if (!request.get("direction").getAsString().equals("none") && input == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.addProperty("value", axis.test(input));
        return result;
    }

    static JsonObject directionAxisPlaneCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        result.addProperty("plane", axis.getPlane() == Direction.Plane.HORIZONTAL
            ? "horizontal" : "vertical");
        return result;
    }

    static JsonObject directionPlaneTestCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Plane plane = switch (request.get("plane").getAsString()) {
            case "horizontal" -> Direction.Plane.HORIZONTAL;
            case "vertical" -> Direction.Plane.VERTICAL;
            default -> throw new IllegalArgumentException("Unknown plane");
        };
        Direction input = request.get("direction").getAsString().equals("none")
            ? null : Direction.byName(request.get("direction").getAsString());
        if (!request.get("direction").getAsString().equals("none") && input == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.addProperty("value", plane.test(input));
        return result;
    }

    static JsonObject directionPlaneOrderCase(JsonObject request) {
        var result = new JsonObject();
        Direction.Plane plane = switch (request.get("plane").getAsString()) {
            case "horizontal" -> Direction.Plane.HORIZONTAL;
            case "vertical" -> Direction.Plane.VERTICAL;
            default -> throw new IllegalArgumentException("Unknown plane");
        };
        var faces = new JsonArray();
        for (Direction direction : plane) {
            faces.add(direction.getName());
        }
        var streamFaces = new JsonArray();
        plane.stream().forEach(direction -> streamFaces.add(direction.getName()));
        var axes = new JsonArray();
        if (plane == Direction.Plane.HORIZONTAL) {
            axes.add(Direction.Axis.X.getName());
            axes.add(Direction.Axis.Z.getName());
        } else {
            axes.add(Direction.Axis.Y.getName());
        }
        result.add("faces", faces);
        result.add("stream_faces", streamFaces);
        result.add("axes", axes);
        result.addProperty("length", plane.length());
        return result;
    }

    static JsonObject directionRandomCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        Direction.Plane plane = switch (request.get("plane").getAsString()) {
            case "horizontal" -> Direction.Plane.HORIZONTAL;
            case "vertical" -> Direction.Plane.VERTICAL;
            default -> throw new IllegalArgumentException("Unknown plane");
        };
        result.add("direction", snapshot(Direction.getRandom(random)));
        result.addProperty("axis", Direction.Axis.getRandom(random).getName());
        result.add("plane_direction", snapshot(plane.getRandomDirection(random)));
        result.addProperty("plane_axis", plane.getRandomAxis(random).getName());
        result.addProperty("tail", Integer.toString(random.nextInt()));
        return result;
    }

    static JsonObject directionPlaneShuffleCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        Direction.Plane plane = switch (request.get("plane").getAsString()) {
            case "horizontal" -> Direction.Plane.HORIZONTAL;
            case "vertical" -> Direction.Plane.VERTICAL;
            default -> throw new IllegalArgumentException("Unknown plane");
        };
        var faces = new JsonArray();
        for (Direction direction : plane.shuffledCopy(random)) {
            faces.add(direction.getName());
        }
        result.add("faces", faces);
        result.addProperty("tail", Integer.toString(random.nextInt()));
        return result;
    }

    static JsonObject directionAllShuffleCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        var faces = new JsonArray();
        for (Direction direction : Direction.allShuffled(random)) {
            faces.add(direction.getName());
        }
        result.add("faces", faces);
        result.addProperty("tail", Integer.toString(random.nextInt()));
        return result;
    }

    static JsonObject directionStreamCase(JsonObject request) {
        var result = new JsonObject();
        var faces = new JsonArray();
        Direction.stream().forEach(direction -> faces.add(direction.getName()));
        result.add("faces", faces);
        return result;
    }

    static JsonObject directionToYRotCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.addProperty("bits", Oracle.bits(facing.toYRot()));
        return result;
    }

    static JsonObject directionGetYRotCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        try {
            result.addProperty("outcome", "value");
            result.addProperty("bits", Oracle.bits(Direction.getYRot(facing)));
        } catch (IllegalStateException error) {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.getMessage());
        }
        return result;
    }

    static JsonObject directionRotationCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.add("value", snapshot(facing.getRotation()));
        return result;
    }

    static JsonObject directionNameLookupCase(JsonObject request) {
        var result = new JsonObject();
        String name = request.get("name").getAsString();
        if (request.get("kind").getAsString().equals("direction")) {
            Direction facing = Direction.byName(name);
            result.addProperty("found", facing != null);
            if (facing != null) {
                result.addProperty("name", facing.getName());
                result.addProperty("serialized_name", facing.getSerializedName());
                result.addProperty("text", facing.toString());
            }
        } else if (request.get("kind").getAsString().equals("axis")) {
            Direction.Axis axis = Direction.Axis.byName(name);
            result.addProperty("found", axis != null);
            if (axis != null) {
                result.addProperty("name", axis.getName());
                result.addProperty("serialized_name", axis.getSerializedName());
                result.addProperty("text", axis.toString());
            }
        } else {
            throw new IllegalArgumentException("Unknown name lookup kind");
        }
        return result;
    }

    static JsonObject directionEnumTextCase(JsonObject request) {
        var result = new JsonObject();
        String name = request.get("name").getAsString();
        if (request.get("kind").getAsString().equals("axis_direction")) {
            Direction.AxisDirection direction = switch (name) {
                case "positive" -> Direction.AxisDirection.POSITIVE;
                case "negative" -> Direction.AxisDirection.NEGATIVE;
                default -> throw new IllegalArgumentException("Unknown axis direction");
            };
            result.addProperty("enum_name", direction.name());
            result.addProperty("name", direction.getName());
            result.addProperty("text", direction.toString());
        } else if (request.get("kind").getAsString().equals("plane")) {
            Direction.Plane plane = switch (name) {
                case "horizontal" -> Direction.Plane.HORIZONTAL;
                case "vertical" -> Direction.Plane.VERTICAL;
                default -> throw new IllegalArgumentException("Unknown plane");
            };
            result.addProperty("enum_name", plane.name());
            result.addProperty("text", plane.toString());
        } else {
            throw new IllegalArgumentException("Unknown enum text kind");
        }
        return result;
    }

    static JsonObject directionEnumNameCase(JsonObject request) {
        var result = new JsonObject();
        String name = request.get("name").getAsString();
        if (request.get("kind").getAsString().equals("direction")) {
            Direction facing = Direction.byName(name);
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.addProperty("enum_name", facing.name());
            result.addProperty("name", facing.getName());
            result.addProperty("serialized_name", facing.getSerializedName());
            result.addProperty("text", facing.toString());
        } else if (request.get("kind").getAsString().equals("axis")) {
            Direction.Axis axis = Direction.Axis.byName(name);
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.addProperty("enum_name", axis.name());
            result.addProperty("name", axis.getName());
            result.addProperty("serialized_name", axis.getSerializedName());
            result.addProperty("text", axis.toString());
        } else {
            throw new IllegalArgumentException("Unknown enum name kind");
        }
        return result;
    }

    static JsonObject directionApproximateNearestCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", snapshot(Direction.getApproximateNearest(
            f64(request, "x"), f64(request, "y"), f64(request, "z"))));
        return result;
    }

    static JsonObject directionApproximateNearestOverloadsCase(JsonObject request) {
        var result = new JsonObject();
        result.add("f32", snapshot(Direction.getApproximateNearest(
            Oracle.f32(request, "f32_x"), Oracle.f32(request, "f32_y"), Oracle.f32(request, "f32_z"))));
        result.add("f64", snapshot(Direction.getApproximateNearest(
            f64(request, "x"), f64(request, "y"), f64(request, "z"))));
        result.add("vec3", snapshot(Direction.getApproximateNearest(vec3(request))));
        return result;
    }

    static JsonObject directionNearestCase(JsonObject request) {
        var result = new JsonObject();
        Direction fallback = directionOrElse(request);
        Direction facing;
        if (request.get("kind").getAsString().equals("components")) {
            facing = Direction.getNearest(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()),
                fallback);
        } else if (request.get("kind").getAsString().equals("vec3i")) {
            facing = Direction.getNearest(blockPos(request), fallback);
        } else {
            throw new IllegalArgumentException("Unknown Direction nearest kind");
        }
        if (facing == null) {
            result.add("value", null);
        } else {
            result.add("value", snapshot(facing));
        }
        return result;
    }

    static JsonObject directionAxisStepOrderCase(JsonObject request) {
        var result = new JsonObject();
        var axes = new JsonArray();
        for (Direction.Axis axis : Direction.axisStepOrder(vec3(request))) {
            axes.add(axis.getName());
        }
        result.add("axes", axes);
        return result;
    }

    static JsonObject directionIsFacingAngleCase(JsonObject request) {
        var result = new JsonObject();
        Direction facing = Direction.byName(request.get("name").getAsString());
        if (facing == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        result.addProperty("value", facing.isFacingAngle(Oracle.f32(request, "angle")));
        return result;
    }

    static JsonObject axisCycleHelpersCase(JsonObject request) {
        var result = new JsonObject();
        int x = Integer.parseInt(request.get("x").getAsString());
        int y = Integer.parseInt(request.get("y").getAsString());
        int z = Integer.parseInt(request.get("z").getAsString());
        double doubleX = f64(request, "double_x");
        double doubleY = f64(request, "double_y");
        double doubleZ = f64(request, "double_z");
        var cycles = new JsonObject();
        for (AxisCycle cycle : AxisCycle.values()) {
            var value = new JsonObject();
            value.addProperty("int_x", cycle.cycle(x, y, z, Direction.Axis.X));
            value.addProperty("int_y", cycle.cycle(x, y, z, Direction.Axis.Y));
            value.addProperty("int_z", cycle.cycle(x, y, z, Direction.Axis.Z));
            value.addProperty("double_x", Oracle.bits(cycle.cycle(doubleX, doubleY, doubleZ, Direction.Axis.X)));
            value.addProperty("double_y", Oracle.bits(cycle.cycle(doubleX, doubleY, doubleZ, Direction.Axis.Y)));
            value.addProperty("double_z", Oracle.bits(cycle.cycle(doubleX, doubleY, doubleZ, Direction.Axis.Z)));
            value.addProperty("axis_x", cycle.cycle(Direction.Axis.X).getName());
            value.addProperty("axis_y", cycle.cycle(Direction.Axis.Y).getName());
            value.addProperty("axis_z", cycle.cycle(Direction.Axis.Z).getName());
            value.addProperty("inverse", cycle.inverse().name().toLowerCase(Locale.ROOT));
            cycles.add(cycle.name().toLowerCase(Locale.ROOT), value);
        }
        var between = new JsonArray();
        for (Direction.Axis from : AxisCycle.AXIS_VALUES) {
            for (Direction.Axis to : AxisCycle.AXIS_VALUES) {
                var value = new JsonObject();
                value.addProperty("from", from.getName());
                value.addProperty("to", to.getName());
                value.addProperty("cycle", AxisCycle.between(from, to).name().toLowerCase(Locale.ROOT));
                between.add(value);
            }
        }
        var mutableSet = new JsonObject();
        for (AxisCycle cycle : AxisCycle.values()) {
            BlockPos.MutableBlockPos value = new BlockPos.MutableBlockPos();
            value.set(cycle, x, y, z);
            mutableSet.add(cycle.name().toLowerCase(Locale.ROOT), snapshot(value));
        }
        result.add("cycles", cycles);
        result.add("between", between);
        result.add("mutable_set", mutableSet);
        return result;
    }
}
