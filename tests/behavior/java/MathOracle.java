import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;
import org.joml.Vector3f;

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
        } else if (operation.equals("shape")) {
            String kind = request.get("kind").getAsString();
            if (kind.equals("single")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var events = new JsonArray();
                events.add(snapshot(shape));
                for (var entry : request.getAsJsonArray("steps")) {
                    JsonObject step = entry.getAsJsonObject();
                    if (step.get("op").getAsString().equals("move")) {
                        shape = shape.move(f64(step, "dx"), f64(step, "dy"), f64(step, "dz"));
                    } else {
                        throw new IllegalArgumentException("Unknown VoxelShape op");
                    }
                    events.add(snapshot(shape));
                }
                result.add("events", events);
            } else if (kind.equals("join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                BooleanOp op = switch (request.get("op").getAsString()) {
                    case "false" -> BooleanOp.FALSE;
                    case "or" -> BooleanOp.OR;
                    case "and" -> BooleanOp.AND;
                    case "only_first" -> BooleanOp.ONLY_FIRST;
                    case "only_second" -> BooleanOp.ONLY_SECOND;
                    case "not_same" -> BooleanOp.NOT_SAME;
                    case "second" -> BooleanOp.SECOND;
                    case "first" -> BooleanOp.FIRST;
                    default -> throw new IllegalArgumentException("Unknown VoxelShape boolean op");
                };
                result.add("shape", snapshot(Shapes.join(first, second, op)));
                result.addProperty("join_is_not_empty", Shapes.joinIsNotEmpty(first, second, op));
            } else if (kind.equals("collide")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                AABB moving = box(request.getAsJsonObject("moving"));
                Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown collision axis");
                }
                result.addProperty("distance", Oracle.bits(shape.collide(axis, moving, f64(request, "distance"))));
            } else if (kind.equals("collide_many")) {
                AABB moving = box(request.getAsJsonObject("moving"));
                Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown collision axis");
                }
                var shapes = new java.util.ArrayList<VoxelShape>();
                for (var entry : request.getAsJsonArray("shapes")) {
                    shapes.add(voxelShape(entry.getAsJsonObject()));
                }
                result.addProperty("distance", Oracle.bits(Shapes.collide(
                    axis, moving, shapes, f64(request, "distance"))));
            } else if (kind.equals("clip")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                var events = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    events.add(snapshot(shape.clip(
                        vec3(query.getAsJsonObject("from")),
                        vec3(query.getAsJsonObject("to")),
                        blockPos(query.getAsJsonObject("pos")))));
                }
                result.add("events", events);
            } else if (kind.equals("clip_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape shape = Shapes.join(first, second, BooleanOp.OR);
                var events = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    events.add(snapshot(shape.clip(
                        vec3(query.getAsJsonObject("from")),
                        vec3(query.getAsJsonObject("to")),
                        blockPos(query.getAsJsonObject("pos")))));
                }
                result.add("events", events);
            } else {
                throw new IllegalArgumentException("Unknown VoxelShape kind");
            }
        } else if (operation.equals("random")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            var events = new JsonArray();
            events.add(new JsonObject());
            for (var entry : request.getAsJsonArray("steps")) {
                events.add(randomState(random, entry.getAsJsonObject()));
            }
            result.add("events", events);
        } else if (operation.equals("random_fork")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("prefix_events", randomSteps(random, request.getAsJsonArray("prefix_steps")));
            RandomSource child = random.fork();
            result.add("child_events", randomSteps(child, request.getAsJsonArray("child_steps")));
            result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        } else if (operation.equals("random_positional")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            PositionalRandomFactory factory = random.forkPositional();
            var factoryEvents = new JsonArray();
            for (var entry : request.getAsJsonArray("factory_steps")) {
                JsonObject step = entry.getAsJsonObject();
                RandomSource child = switch (step.get("op").getAsString()) {
                    case "at" -> factory.at(step.get("x").getAsInt(), step.get("y").getAsInt(),
                        step.get("z").getAsInt());
                    case "fromHashOf" -> factory.fromHashOf(step.get("name").getAsString());
                    case "fromSeed" -> factory.fromSeed(Long.parseLong(step.get("seed").getAsString()));
                    default -> throw new IllegalArgumentException("Unknown positional random op");
                };
                var state = new JsonObject();
                state.addProperty("op", step.get("op").getAsString());
                state.add("events", randomSteps(child, step.getAsJsonArray("steps")));
                factoryEvents.add(state);
            }
            result.add("factory_events", factoryEvents);
            result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
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
                    case "contains_vec3" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.contains(vec3(step.getAsJsonObject("position"))));
                        events.add(state);
                    }
                    case "intersects_coords" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            f64(step, "min_x"), f64(step, "min_y"), f64(step, "min_z"),
                            f64(step, "max_x"), f64(step, "max_y"), f64(step, "max_z")));
                        events.add(state);
                    }
                    case "intersects_vec3" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            vec3(step.getAsJsonObject("min")),
                            vec3(step.getAsJsonObject("max"))));
                        events.add(state);
                    }
                    case "intersects_block_pos" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            blockPos(step.getAsJsonObject("position"))));
                        events.add(state);
                    }
                    case "collided_along_vector" -> {
                        var boxes = new java.util.ArrayList<AABB>();
                        for (var boxEntry : step.getAsJsonArray("boxes")) {
                            boxes.add(box(boxEntry.getAsJsonObject()));
                        }
                        var state = new JsonObject();
                        state.addProperty("hit", box.collidedAlongVector(
                            vec3(step.getAsJsonObject("vector")), boxes));
                        events.add(state);
                    }
                    case "from_block_pos" -> {
                        box = new AABB(blockPos(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "from_vec3" -> {
                        box = new AABB(
                            vec3(step.getAsJsonObject("begin")),
                            vec3(step.getAsJsonObject("end")));
                        events.add(snapshot(box));
                    }
                    case "unit_cube_from_lower_corner" -> {
                        box = AABB.unitCubeFromLowerCorner(
                            vec3(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "encapsulating_full_blocks" -> {
                        box = AABB.encapsulatingFullBlocks(
                            blockPos(step.getAsJsonObject("first")),
                            blockPos(step.getAsJsonObject("second")));
                        events.add(snapshot(box));
                    }
                    case "of_bounding_box" -> {
                        JsonObject boundingBox = step.getAsJsonObject("box");
                        box = AABB.of(new BoundingBox(
                            Integer.parseInt(boundingBox.get("min_x").getAsString()),
                            Integer.parseInt(boundingBox.get("min_y").getAsString()),
                            Integer.parseInt(boundingBox.get("min_z").getAsString()),
                            Integer.parseInt(boundingBox.get("max_x").getAsString()),
                            Integer.parseInt(boundingBox.get("max_y").getAsString()),
                            Integer.parseInt(boundingBox.get("max_z").getAsString())));
                        events.add(snapshot(box));
                    }
                    case "intersect" -> {
                        box = box.intersect(box(step.getAsJsonObject("box")));
                        events.add(snapshot(box));
                    }
                    case "minmax" -> {
                        box = box.minmax(box(step.getAsJsonObject("box")));
                        events.add(snapshot(box));
                    }
                    case "move" -> {
                        box = box.move(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "move_block_pos" -> {
                        box = box.move(blockPos(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "move_vec3" -> {
                        box = box.move(vec3(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "move_vector3f" -> {
                        JsonObject position = step.getAsJsonObject("position");
                        box = box.move(new Vector3f(
                            Oracle.f32(position, "x"),
                            Oracle.f32(position, "y"),
                            Oracle.f32(position, "z")));
                        events.add(snapshot(box));
                    }
                    case "sizes" -> {
                        var state = new JsonObject();
                        state.addProperty("x_size", Oracle.bits(box.getXsize()));
                        state.addProperty("y_size", Oracle.bits(box.getYsize()));
                        state.addProperty("z_size", Oracle.bits(box.getZsize()));
                        state.addProperty("size", Oracle.bits(box.getSize()));
                        events.add(state);
                    }
                    case "has_nan" -> {
                        var state = new JsonObject();
                        state.addProperty("has_nan", box.hasNaN());
                        events.add(state);
                    }
                    case "positions" -> {
                        var state = new JsonObject();
                        state.add("center", snapshot(box.getCenter()));
                        state.add("bottom_center", snapshot(box.getBottomCenter()));
                        state.add("min_position", snapshot(box.getMinPosition()));
                        state.add("max_position", snapshot(box.getMaxPosition()));
                        events.add(state);
                    }
                    case "min", "max" -> {
                        Direction.Axis axis = Direction.Axis.byName(step.get("axis").getAsString());
                        if (axis == null) {
                            throw new IllegalArgumentException("Unknown axis name");
                        }
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(step.get("op").getAsString().equals("min")
                            ? box.min(axis) : box.max(axis)));
                        events.add(state);
                    }
                    case "distance_point" -> {
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(box.distanceToSqr(
                            vec3(step.getAsJsonObject("point")))));
                        events.add(state);
                    }
                    case "distance_box" -> {
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(box.distanceToSqr(
                            box(step.getAsJsonObject("box")))));
                        events.add(state);
                    }
                    case "of_size" -> {
                        box = AABB.ofSize(
                            vec3(step.getAsJsonObject("center")),
                            f64(step, "size_x"),
                            f64(step, "size_y"),
                            f64(step, "size_z"));
                        events.add(snapshot(box));
                    }
                    case "set_min_x" -> {
                        box = box.setMinX(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_min_y" -> {
                        box = box.setMinY(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_min_z" -> {
                        box = box.setMinZ(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_x" -> {
                        box = box.setMaxX(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_y" -> {
                        box = box.setMaxY(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_z" -> {
                        box = box.setMaxZ(f64(step, "value"));
                        events.add(snapshot(box));
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
                    case "clip_coords" -> {
                        var hit = AABB.clip(
                            f64(step, "min_x"), f64(step, "min_y"), f64(step, "min_z"),
                            f64(step, "max_x"), f64(step, "max_y"), f64(step, "max_z"),
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
                    case "clip_many" -> {
                        var boxes = new ArrayList<AABB>();
                        for (var boxEntry : step.getAsJsonArray("boxes")) {
                            boxes.add(box(boxEntry.getAsJsonObject()));
                        }
                        var hit = AABB.clip(
                            boxes,
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")),
                            blockPos(step.getAsJsonObject("pos")));
                        events.add(snapshot(hit));
                    }
                    case "expand" -> {
                        box = box.expandTowards(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "contract" -> {
                        box = box.contract(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "inflate" -> {
                        box = box.inflate(f64(step, "amount"));
                        events.add(snapshot(box));
                    }
                    case "inflate_xyz" -> {
                        box = box.inflate(
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

    private static VoxelShape voxelShape(JsonObject object) {
        return Shapes.box(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
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

    private static JsonArray randomSteps(RandomSource random, JsonArray entries) {
        var events = new JsonArray();
        for (var entry : entries) {
            events.add(randomState(random, entry.getAsJsonObject()));
        }
        return events;
    }

    private static JsonObject randomState(RandomSource random, JsonObject step) {
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
            case "setSeed" -> {
                random.setSeed(Long.parseLong(step.get("seed").getAsString()));
                state.addProperty("op", "setSeed");
            }
            case "nextIntBetweenInclusive" -> state.addProperty("value", Integer.toString(
                random.nextIntBetweenInclusive(step.get("min").getAsInt(), step.get("max").getAsInt())));
            case "nextIntRange" -> state.addProperty("value", Integer.toString(
                random.nextInt(step.get("origin").getAsInt(), step.get("bound").getAsInt())));
            case "triangleDouble" -> state.addProperty("bits", Oracle.bits(
                random.triangle(f64(step, "mean"), f64(step, "spread"))));
            case "triangleFloat" -> state.addProperty("bits", Oracle.bits(
                random.triangle(Oracle.f32(step, "mean"), Oracle.f32(step, "spread"))));
            case "consumeCount" -> {
                random.consumeCount(step.get("rounds").getAsInt());
                state.addProperty("op", "consumeCount");
            }
            default -> throw new IllegalArgumentException("Unknown random op");
        }
        return state;
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

    private static JsonObject snapshot(VoxelShape shape) {
        var state = new JsonObject();
        state.addProperty("empty", shape.isEmpty());
        var boxes = new JsonArray();
        for (AABB box : shape.toAabbs()) {
            boxes.add(snapshot(box));
        }
        state.add("boxes", boxes);
        if (!shape.isEmpty()) {
            state.add("bounds", snapshot(shape.bounds()));
        }
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

    private static JsonObject snapshot(BlockHitResult hit) {
        var state = new JsonObject();
        if (hit == null) {
            state.addProperty("hit", false);
            return state;
        }
        state.addProperty("hit", true);
        state.addProperty("type", hit.getType().name());
        state.add("location", snapshotClip(hit.getLocation()));
        state.addProperty("direction", hit.getDirection().getName());
        state.add("block_pos", snapshot(hit.getBlockPos()));
        state.addProperty("inside", hit.isInside());
        state.addProperty("world_border_hit", hit.isWorldBorderHit());
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
