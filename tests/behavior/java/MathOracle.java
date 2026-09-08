import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;

public final class MathOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        var result = new JsonObject();
        if (operation.equals("vec2")) {
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
        } else if (operation.equals("direction")) {
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
        } else if (operation.equals("coord")) {
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
        } else if (operation.equals("blockpos")) {
            BlockPos pos = blockPos(request);
            var events = new JsonArray();
            events.add(snapshot(pos));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                pos = switch (step.get("action").getAsString()) {
                    case "offset" -> pos.offset(
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()));
                    case "min" -> BlockPos.min(pos, blockPos(step));
                    case "max" -> BlockPos.max(pos, blockPos(step));
                    default -> throw new IllegalArgumentException("Unknown BlockPos action");
                };
                events.add(snapshot(pos));
            }
            result.add("events", events);
        } else if (operation.equals("section_relative")) {
            short relative = SectionPos.sectionRelativePos(blockPos(request));
            result.addProperty("relative", Integer.toString(Short.toUnsignedInt(relative)));
            result.addProperty("x", Integer.toString(SectionPos.sectionRelativeX(relative)));
            result.addProperty("y", Integer.toString(SectionPos.sectionRelativeY(relative)));
            result.addProperty("z", Integer.toString(SectionPos.sectionRelativeZ(relative)));
        } else if (operation.equals("vec3")) {
            Vec3 vector = new Vec3(f64(request, "x"), f64(request, "y"), f64(request, "z"));
            var events = new JsonArray();
            events.add(snapshot(vector));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                vector = switch (step.get("action").getAsString()) {
                    case "normalize" -> vector.normalize();
                    case "scale" -> vector.scale(f64(step, "factor"));
                    default -> throw new IllegalArgumentException("Unknown Vec3 action");
                };
                events.add(snapshot(vector));
            }
            result.add("events", events);
        } else if (operation.equals("random")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            var events = new JsonArray();
            events.add(new JsonObject());
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                var state = new JsonObject();
                switch (step.get("op").getAsString()) {
                    case "nextInt" -> state.addProperty("value", Integer.toString(random.nextInt()));
                    case "nextIntBound" -> state.addProperty("value",
                        Integer.toString(random.nextInt(step.get("bound").getAsInt())));
                    case "nextLong" -> state.addProperty("value", Long.toString(random.nextLong()));
                    case "nextBoolean" -> state.addProperty("value", Boolean.toString(random.nextBoolean()));
                    case "nextFloat" -> state.addProperty("bits", Oracle.bits(random.nextFloat()));
                    case "nextDouble" -> state.addProperty("bits", Oracle.bits(random.nextDouble()));
                    case "nextGaussian" -> state.addProperty("bits", Oracle.bits(random.nextGaussian()));
                    default -> throw new IllegalArgumentException("Unknown random op");
                }
                events.add(state);
            }
            result.add("events", events);
        } else if (operation.equals("aabb")) {
            AABB box = box(request);
            var events = new JsonArray();
            events.add(snapshot(box));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                switch (step.get("op").getAsString()) {
                    case "contains" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.contains(
                            f64(step, "x"), f64(step, "y"), f64(step, "z")));
                        events.add(state);
                    }
                    case "intersects" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(box(step.getAsJsonObject("box"))));
                        events.add(state);
                    }
                    case "clip" -> {
                        var hit = box.clip(
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")));
                        var state = new JsonObject();
                        if (hit.isPresent()) {
                            state.addProperty("hit", true);
                            state.add("point", snapshotClip(hit.get()));
                        } else {
                            state.addProperty("hit", false);
                        }
                        events.add(state);
                    }
                    case "expand" -> {
                        box = box.expandTowards(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "deflate" -> {
                        box = box.deflate(f64(step, "amount"));
                        events.add(snapshot(box));
                    }
                    default -> throw new IllegalArgumentException("Unknown AABB op");
                }
            }
            result.add("events", events);
        } else {
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
        }
        return result;
    }

    private static double f64(JsonObject object, String name) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(object.get(name).getAsString(), 16));
    }

    private static AABB box(JsonObject object) {
        return new AABB(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    private static BlockPos blockPos(JsonObject object) {
        return new BlockPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("y").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    private static Vec3 vec3(JsonObject object) {
        return new Vec3(f64(object, "x"), f64(object, "y"), f64(object, "z"));
    }

    private static JsonObject snapshot(AABB box) {
        var state = new JsonObject();
        state.addProperty("min_x", Oracle.bits(box.minX));
        state.addProperty("min_y", Oracle.bits(box.minY));
        state.addProperty("min_z", Oracle.bits(box.minZ));
        state.addProperty("max_x", Oracle.bits(box.maxX));
        state.addProperty("max_y", Oracle.bits(box.maxY));
        state.addProperty("max_z", Oracle.bits(box.maxZ));
        return state;
    }

    private static JsonObject snapshot(BlockPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.getX()));
        state.addProperty("y", Integer.toString(pos.getY()));
        state.addProperty("z", Integer.toString(pos.getZ()));
        return state;
    }

    private static JsonObject snapshot(Direction facing) {
        var state = new JsonObject();
        state.addProperty("name", facing.getName());
        state.addProperty("data3d", facing.get3DDataValue());
        state.addProperty("data2d", facing.get2DDataValue());
        state.addProperty("opposite", facing.getOpposite().getName());
        state.addProperty("axis", facing.getAxis().getName());
        state.addProperty("axis_direction", facing.getAxisDirection().getName());
        state.addProperty("step_x", facing.getStepX());
        state.addProperty("step_y", facing.getStepY());
        state.addProperty("step_z", facing.getStepZ());
        return state;
    }

    private static JsonObject snapshot(Vec3 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSqr()));
        return state;
    }

    private static JsonObject snapshotClip(Vec3 point) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(point.x));
        state.addProperty("y", Oracle.bits(point.y));
        state.addProperty("z", Oracle.bits(point.z));
        return state;
    }

    private static JsonObject snapshot(Vec2 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSquared()));
        return state;
    }
}
