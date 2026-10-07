import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

class MathBlockPosOracle extends MathAngleOracle {
    static JsonObject blockposCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject blockposPackedHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        long packed = pos.asLong();
        long neighborNode = Long.parseUnsignedLong(request.get("neighbor_node").getAsString(), 16);
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        int stepX = Integer.parseInt(request.get("step_x").getAsString());
        int stepY = Integer.parseInt(request.get("step_y").getAsString());
        int stepZ = Integer.parseInt(request.get("step_z").getAsString());
        var accessors = new JsonObject();
        accessors.addProperty("x", Integer.toString(BlockPos.getX(packed)));
        accessors.addProperty("y", Integer.toString(BlockPos.getY(packed)));
        accessors.addProperty("z", Integer.toString(BlockPos.getZ(packed)));
        result.add("zero", snapshot(BlockPos.ZERO));
        result.addProperty("packed_horizontal_length", BlockPos.PACKED_HORIZONTAL_LENGTH);
        result.addProperty("packed_y_length", BlockPos.PACKED_Y_LENGTH);
        result.addProperty("max_horizontal_coordinate", BlockPos.MAX_HORIZONTAL_COORDINATE);
        result.addProperty("packed", Long.toUnsignedString(packed, 16));
        result.addProperty("coordinate_packed", Long.toUnsignedString(BlockPos.asLong(
            pos.getX(), pos.getY(), pos.getZ()), 16));
        result.add("decoded", snapshot(BlockPos.of(packed)));
        result.add("accessors", accessors);
        result.addProperty("flat_index", Long.toUnsignedString(BlockPos.getFlatIndex(neighborNode), 16));
        result.addProperty("offset_direction", Long.toUnsignedString(
            BlockPos.offset(neighborNode, direction), 16));
        result.addProperty("offset_steps", Long.toUnsignedString(
            BlockPos.offset(neighborNode, stepX, stepY, stepZ), 16));
        return result;
    }

    static JsonObject blockposRelativeHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown direction name");
        }
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown axis name");
        }
        int steps = Integer.parseInt(request.get("steps").getAsString());
        var cardinal = new JsonObject();
        cardinal.add("above", snapshot(pos.above()));
        cardinal.add("above_steps", snapshot(pos.above(steps)));
        cardinal.add("below", snapshot(pos.below()));
        cardinal.add("below_steps", snapshot(pos.below(steps)));
        cardinal.add("north", snapshot(pos.north()));
        cardinal.add("north_steps", snapshot(pos.north(steps)));
        cardinal.add("south", snapshot(pos.south()));
        cardinal.add("south_steps", snapshot(pos.south(steps)));
        cardinal.add("west", snapshot(pos.west()));
        cardinal.add("west_steps", snapshot(pos.west(steps)));
        cardinal.add("east", snapshot(pos.east()));
        cardinal.add("east_steps", snapshot(pos.east(steps)));
        result.add("relative_direction", snapshot(pos.relative(direction)));
        result.add("relative_direction_steps", snapshot(pos.relative(direction, steps)));
        result.add("relative_axis_steps", snapshot(pos.relative(axis, steps)));
        result.add("cardinal", cardinal);
        return result;
    }

    static JsonObject blockposVectorHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        BlockPos delta = new BlockPos(
            Integer.parseInt(request.get("dx").getAsString()),
            Integer.parseInt(request.get("dy").getAsString()),
            Integer.parseInt(request.get("dz").getAsString()));
        int scale = Integer.parseInt(request.get("scale").getAsString());
        result.add("offset_vec3i", snapshot(pos.offset(delta)));
        result.add("subtract_vec3i", snapshot(pos.subtract(delta)));
        result.add("multiply", snapshot(pos.multiply(scale)));
        return result;
    }

    static JsonObject blockposGeometryHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        Rotation rotation = switch (request.get("rotation").getAsString()) {
            case "none" -> Rotation.NONE;
            case "clockwise_90" -> Rotation.CLOCKWISE_90;
            case "180" -> Rotation.CLOCKWISE_180;
            case "counterclockwise_90" -> Rotation.COUNTERCLOCKWISE_90;
            default -> throw new IllegalArgumentException("Unknown BlockPos rotation");
        };
        BlockPos upVector = new BlockPos(
            Integer.parseInt(request.get("up_x").getAsString()),
            Integer.parseInt(request.get("up_y").getAsString()),
            Integer.parseInt(request.get("up_z").getAsString()));
        int y = Integer.parseInt(request.get("at_y").getAsString());
        result.add("rotate", snapshot(pos.rotate(rotation)));
        result.add("cross", snapshot(pos.cross(upVector)));
        result.add("at_y", snapshot(pos.atY(y)));
        return result;
    }

    static JsonObject blockposDistanceHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        BlockPos other = blockPos(request.getAsJsonObject("other"));
        JsonObject point = request.getAsJsonObject("point");
        double distance = f64(request, "distance");
        result.addProperty("dist_sqr", Oracle.bits(pos.distSqr(other)));
        result.addProperty("dist_low_corner_sqr", Oracle.bits(pos.distToLowCornerSqr(
            f64(point, "x"), f64(point, "y"), f64(point, "z"))));
        result.addProperty("dist_center_sqr", Oracle.bits(pos.distToCenterSqr(
            f64(point, "x"), f64(point, "y"), f64(point, "z"))));
        result.addProperty("closer_than", pos.closerThan(other, distance));
        return result;
    }

    static JsonObject blockposIntegerHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        BlockPos other = blockPos(request.getAsJsonObject("other"));
        var selected = new JsonObject();
        selected.addProperty("x", pos.get(Direction.Axis.X));
        selected.addProperty("y", pos.get(Direction.Axis.Y));
        selected.addProperty("z", pos.get(Direction.Axis.Z));
        result.addProperty("dist_manhattan", pos.distManhattan(other));
        result.addProperty("dist_chessboard", pos.distChessboard(other));
        result.add("get", selected);
        return result;
    }

    static JsonObject blockposClampLocationCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        result.add("value", snapshot(pos.clampLocationWithin(
            vec3(request.getAsJsonObject("location")))));
        return result;
    }

    static JsonObject blockposBetweenClosedCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos from = blockPos(request.getAsJsonObject("from"));
        BlockPos to = blockPos(request.getAsJsonObject("to"));
        var fromTo = new JsonArray();
        for (BlockPos pos : BlockPos.betweenClosed(from, to)) {
            fromTo.add(snapshot(pos));
        }
        int minX = Integer.parseInt(request.get("min_x").getAsString());
        int minY = Integer.parseInt(request.get("min_y").getAsString());
        int minZ = Integer.parseInt(request.get("min_z").getAsString());
        int maxX = Integer.parseInt(request.get("max_x").getAsString());
        int maxY = Integer.parseInt(request.get("max_y").getAsString());
        int maxZ = Integer.parseInt(request.get("max_z").getAsString());
        var coordinates = new JsonArray();
        for (BlockPos pos : BlockPos.betweenClosed(minX, minY, minZ, maxX, maxY, maxZ)) {
            coordinates.add(snapshot(pos));
        }
        result.add("from_to", fromTo);
        result.add("coordinates", coordinates);
        return result;
    }

    static JsonObject blockposNeighborColumnCase(JsonObject request) {
        var result = new JsonObject();
        int startX = Integer.parseInt(request.get("start_x").getAsString());
        int startY = Integer.parseInt(request.get("start_y").getAsString());
        int startZ = Integer.parseInt(request.get("start_z").getAsString());
        int endY = Integer.parseInt(request.get("end_y").getAsString());
        var positions = new JsonArray();
        for (BlockPos pos : BlockPos.neighborColumn(startX, startY, startZ, endY)) {
            positions.add(snapshot(pos));
        }
        result.add("positions", positions);
        return result;
    }

    static JsonObject blockposSquareOutSouthEastCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos from = blockPos(request);
        var positions = new JsonArray();
        positions.add(snapshot(from));
        positions.add(snapshot(from.south()));
        positions.add(snapshot(from.east()));
        positions.add(snapshot(from.south().east()));
        result.add("positions", positions);
        return result;
    }

    static JsonObject blockposRandomRangesCase(JsonObject request) {
        var result = new JsonObject();
        long seed = Long.parseLong(request.get("seed").getAsString());
        int limit = Integer.parseInt(request.get("limit").getAsString());
        int minX = Integer.parseInt(request.get("min_x").getAsString());
        int minY = Integer.parseInt(request.get("min_y").getAsString());
        int minZ = Integer.parseInt(request.get("min_z").getAsString());
        int maxX = Integer.parseInt(request.get("max_x").getAsString());
        int maxY = Integer.parseInt(request.get("max_y").getAsString());
        int maxZ = Integer.parseInt(request.get("max_z").getAsString());
        RandomSource betweenRandom = RandomSource.create(seed);
        var between = new JsonArray();
        for (BlockPos pos : BlockPos.randomBetweenClosed(
            betweenRandom, limit, minX, minY, minZ, maxX, maxY, maxZ)) {
            between.add(snapshot(pos));
        }
        int afterBetween = betweenRandom.nextInt();
        BlockPos center = blockPos(request.getAsJsonObject("center"));
        int size = Integer.parseInt(request.get("size").getAsString());
        RandomSource cubeRandom = RandomSource.create(seed);
        var cube = new JsonArray();
        for (BlockPos pos : BlockPos.randomInCube(cubeRandom, limit, center, size)) {
            cube.add(snapshot(pos));
        }
        int afterCube = cubeRandom.nextInt();
        result.add("between", between);
        result.addProperty("after_between", Integer.toString(afterBetween));
        result.add("cube", cube);
        result.addProperty("after_cube", Integer.toString(afterCube));
        return result;
    }

    static JsonObject blockposBetweenClosedAabbCase(JsonObject request) {
        var result = new JsonObject();
        AABB aabb = box(request.getAsJsonObject("aabb"));
        var positions = new JsonArray();
        for (BlockPos pos : BlockPos.betweenClosed(aabb)) {
            positions.add(snapshot(pos));
        }
        result.add("positions", positions);
        return result;
    }

    static JsonObject blockposWithinManhattanCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos origin = blockPos(request.getAsJsonObject("origin"));
        int reachX = Integer.parseInt(request.get("reach_x").getAsString());
        int reachY = Integer.parseInt(request.get("reach_y").getAsString());
        int reachZ = Integer.parseInt(request.get("reach_z").getAsString());
        var positions = new JsonArray();
        for (BlockPos pos : BlockPos.withinManhattan(origin, reachX, reachY, reachZ)) {
            positions.add(snapshot(pos));
        }
        var streamPositions = new JsonArray();
        BlockPos.withinManhattanStream(origin, reachX, reachY, reachZ)
            .forEach(pos -> streamPositions.add(snapshot(pos)));
        result.add("positions", positions);
        result.add("stream_positions", streamPositions);
        return result;
    }

    static JsonObject blockposSpiralAroundCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos center = blockPos(request.getAsJsonObject("center"));
        int radius = Integer.parseInt(request.get("radius").getAsString());
        Direction firstDirection = Direction.byName(request.get("first_direction").getAsString());
        Direction secondDirection = Direction.byName(request.get("second_direction").getAsString());
        var positions = new JsonArray();
        for (BlockPos.MutableBlockPos pos : BlockPos.spiralAround(center, radius, firstDirection, secondDirection)) {
            positions.add(snapshot(pos));
        }
        result.add("positions", positions);
        return result;
    }

    static JsonObject blockposFindClosestMatchCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos startPos = blockPos(request.getAsJsonObject("start"));
        int horizontalRadius = Integer.parseInt(request.get("horizontal_radius").getAsString());
        int verticalRadius = Integer.parseInt(request.get("vertical_radius").getAsString());
        BlockPos target = blockPos(request.getAsJsonObject("target"));
        var match = BlockPos.findClosestMatch(
            startPos, horizontalRadius, verticalRadius, pos -> pos.equals(target));
        if (match.isPresent()) {
            result.add("value", snapshot(match.get()));
        } else {
            result.add("value", com.google.gson.JsonNull.INSTANCE);
        }
        return result;
    }

    static JsonObject blockposBetweenClosedStreamsCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos from = blockPos(request.getAsJsonObject("from"));
        BlockPos to = blockPos(request.getAsJsonObject("to"));
        var blockPositions = new JsonArray();
        BlockPos.betweenClosedStream(from, to)
            .forEach(pos -> blockPositions.add(snapshot(pos)));
        JsonObject bounds = request.getAsJsonObject("bounding_box");
        BoundingBox boundingBox = new BoundingBox(
            Integer.parseInt(bounds.get("min_x").getAsString()),
            Integer.parseInt(bounds.get("min_y").getAsString()),
            Integer.parseInt(bounds.get("min_z").getAsString()),
            Integer.parseInt(bounds.get("max_x").getAsString()),
            Integer.parseInt(bounds.get("max_y").getAsString()),
            Integer.parseInt(bounds.get("max_z").getAsString()));
        var boundingPositions = new JsonArray();
        BlockPos.betweenClosedStream(boundingBox)
            .forEach(pos -> boundingPositions.add(snapshot(pos)));
        AABB aabb = box(request.getAsJsonObject("aabb"));
        var aabbPositions = new JsonArray();
        BlockPos.betweenClosedStream(aabb)
            .forEach(pos -> aabbPositions.add(snapshot(pos)));
        int minX = Integer.parseInt(request.get("min_x").getAsString());
        int minY = Integer.parseInt(request.get("min_y").getAsString());
        int minZ = Integer.parseInt(request.get("min_z").getAsString());
        int maxX = Integer.parseInt(request.get("max_x").getAsString());
        int maxY = Integer.parseInt(request.get("max_y").getAsString());
        int maxZ = Integer.parseInt(request.get("max_z").getAsString());
        var coordinatePositions = new JsonArray();
        BlockPos.betweenClosedStream(minX, minY, minZ, maxX, maxY, maxZ)
            .forEach(pos -> coordinatePositions.add(snapshot(pos)));
        result.add("block", blockPositions);
        result.add("bounding_box", boundingPositions);
        result.add("aabb", aabbPositions);
        result.add("coordinates", coordinatePositions);
        return result;
    }

    static JsonObject blockposCoordinateHelpersCase(JsonObject request) {
        var result = new JsonObject();
        double x = f64(request, "x");
        double y = f64(request, "y");
        double z = f64(request, "z");
        Vec3 point = vec3(request.getAsJsonObject("point"));
        BlockPos first = blockPos(request.getAsJsonObject("first"));
        BlockPos second = blockPos(request.getAsJsonObject("second"));
        result.add("containing", snapshot(BlockPos.containing(x, y, z)));
        result.add("containing_position", snapshot(BlockPos.containing(point)));
        result.add("min", snapshot(BlockPos.min(first, second)));
        result.add("max", snapshot(BlockPos.max(first, second)));
        return result;
    }

    static JsonObject blockposBetweenCornersDirectionCase(JsonObject request) {
        var result = new JsonObject();
        AABB aabb = box(request.getAsJsonObject("aabb"));
        BlockPos first = blockPos(request.getAsJsonObject("first"));
        BlockPos second = blockPos(request.getAsJsonObject("second"));
        int firstX = Integer.parseInt(request.get("first_x").getAsString());
        int firstY = Integer.parseInt(request.get("first_y").getAsString());
        int firstZ = Integer.parseInt(request.get("first_z").getAsString());
        int secondX = Integer.parseInt(request.get("second_x").getAsString());
        int secondY = Integer.parseInt(request.get("second_y").getAsString());
        int secondZ = Integer.parseInt(request.get("second_z").getAsString());
        Vec3 direction = vec3(request.getAsJsonObject("direction"));
        var aabbPositions = new JsonArray();
        for (BlockPos pos : BlockPos.betweenCornersInDirection(aabb, direction)) {
            aabbPositions.add(snapshot(pos));
        }
        var blockPositions = new JsonArray();
        for (BlockPos pos : BlockPos.betweenCornersInDirection(first, second, direction)) {
            blockPositions.add(snapshot(pos));
        }
        var coordinatePositions = new JsonArray();
        for (BlockPos pos : BlockPos.betweenCornersInDirection(
            firstX, firstY, firstZ, secondX, secondY, secondZ, direction)) {
            coordinatePositions.add(snapshot(pos));
        }
        result.add("aabb", aabbPositions);
        result.add("block_pos", blockPositions);
        result.add("coordinates", coordinatePositions);
        return result;
    }

    static JsonObject blockposBreadthFirstTraversalCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos start = blockPos(request.getAsJsonObject("start"));
        int maxDepth = Integer.parseInt(request.get("max_depth").getAsString());
        int maxCount = Integer.parseInt(request.get("max_count").getAsString());
        JsonArray rules = request.getAsJsonArray("nodes");
        var processed = new JsonArray();
        var providerCalls = new JsonArray();
        int count = BlockPos.breadthFirstTraversal(
            start,
            maxDepth,
            maxCount,
            (pos, consumer) -> {
                providerCalls.add(snapshot(pos));
                for (JsonElement entry : rules) {
                    JsonObject rule = entry.getAsJsonObject();
                    if (pos.equals(blockPos(rule.getAsJsonObject("position")))) {
                        for (JsonElement neighbour : rule.getAsJsonArray("neighbors")) {
                            consumer.accept(blockPos(neighbour.getAsJsonObject()));
                        }
                        break;
                    }
                }
            },
            pos -> {
                processed.add(snapshot(pos));
                for (JsonElement entry : rules) {
                    JsonObject rule = entry.getAsJsonObject();
                    if (pos.equals(blockPos(rule.getAsJsonObject("position")))) {
                        return switch (rule.get("status").getAsString()) {
                            case "accept" -> BlockPos.TraversalNodeStatus.ACCEPT;
                            case "skip" -> BlockPos.TraversalNodeStatus.SKIP;
                            case "stop" -> BlockPos.TraversalNodeStatus.STOP;
                            default -> throw new IllegalArgumentException("Unknown traversal status");
                        };
                    }
                }
                return BlockPos.TraversalNodeStatus.ACCEPT;
            });
        result.addProperty("count", count);
        result.add("processed", processed);
        result.add("provider_calls", providerCalls);
        return result;
    }

    static JsonObject blockposMutableHelpersCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos initial = blockPos(request.getAsJsonObject("initial"));
        BlockPos.MutableBlockPos mutable = initial.mutable();
        var constructors = new JsonObject();
        constructors.add("immutable", snapshot(initial.immutable()));
        constructors.add("mutable", snapshot(initial.mutable()));
        constructors.add("double", snapshot(new BlockPos.MutableBlockPos(
            f64(request, "double_x"), f64(request, "double_y"), f64(request, "double_z"))));
        var events = new JsonArray();
        for (JsonElement entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            JsonObject event = new JsonObject();
            switch (step.get("op").getAsString()) {
                case "set" -> mutable.set(
                    Integer.parseInt(step.get("x").getAsString()),
                    Integer.parseInt(step.get("y").getAsString()),
                    Integer.parseInt(step.get("z").getAsString()));
                case "set_double" -> mutable.set(
                    f64(step, "x"), f64(step, "y"), f64(step, "z"));
                case "set_block_pos" -> mutable.set(blockPos(step.getAsJsonObject("value")));
                case "set_packed" -> mutable.set(Long.parseUnsignedLong(step.get("value").getAsString(), 16));
                case "set_with_offset_direction" -> {
                    BlockPos pos = blockPos(step.getAsJsonObject("position"));
                    Direction direction = Direction.byName(step.get("direction").getAsString());
                    mutable.setWithOffset(pos, direction);
                }
                case "set_with_offset_components" -> mutable.setWithOffset(
                    blockPos(step.getAsJsonObject("position")),
                    Integer.parseInt(step.get("x").getAsString()),
                    Integer.parseInt(step.get("y").getAsString()),
                    Integer.parseInt(step.get("z").getAsString()));
                case "set_with_offset_block_pos" -> mutable.setWithOffset(
                    blockPos(step.getAsJsonObject("position")),
                    blockPos(step.getAsJsonObject("offset")));
                case "move_direction" -> mutable.move(
                    Direction.byName(step.get("direction").getAsString()));
                case "move_direction_steps" -> mutable.move(
                    Direction.byName(step.get("direction").getAsString()),
                    Integer.parseInt(step.get("steps").getAsString()));
                case "move_components" -> mutable.move(
                    Integer.parseInt(step.get("x").getAsString()),
                    Integer.parseInt(step.get("y").getAsString()),
                    Integer.parseInt(step.get("z").getAsString()));
                case "move_block_pos" -> mutable.move(blockPos(step.getAsJsonObject("value")));
                case "clamp" -> mutable.clamp(
                    Direction.Axis.byName(step.get("axis").getAsString()),
                    Integer.parseInt(step.get("minimum").getAsString()),
                    Integer.parseInt(step.get("maximum").getAsString()));
                case "set_x" -> mutable.setX(Integer.parseInt(step.get("value").getAsString()));
                case "set_y" -> mutable.setY(Integer.parseInt(step.get("value").getAsString()));
                case "set_z" -> mutable.setZ(Integer.parseInt(step.get("value").getAsString()));
                case "offset_return" -> event.add("returned", snapshot(mutable.offset(
                    Integer.parseInt(step.get("x").getAsString()),
                    Integer.parseInt(step.get("y").getAsString()),
                    Integer.parseInt(step.get("z").getAsString()))));
                case "multiply_return" -> event.add("returned", snapshot(mutable.multiply(
                    Integer.parseInt(step.get("scale").getAsString()))));
                case "relative_return" -> event.add("returned", snapshot(mutable.relative(
                    Direction.byName(step.get("direction").getAsString()),
                    Integer.parseInt(step.get("steps").getAsString()))));
                case "relative_axis_return" -> event.add("returned", snapshot(mutable.relative(
                    Direction.Axis.byName(step.get("axis").getAsString()),
                    Integer.parseInt(step.get("steps").getAsString()))));
                case "rotate_return" -> {
                    Rotation rotation = switch (step.get("rotation").getAsString()) {
                        case "none" -> Rotation.NONE;
                        case "clockwise_90" -> Rotation.CLOCKWISE_90;
                        case "180" -> Rotation.CLOCKWISE_180;
                        case "counterclockwise_90" -> Rotation.COUNTERCLOCKWISE_90;
                        default -> throw new IllegalArgumentException("Unknown mutable rotation");
                    };
                    event.add("returned", snapshot(mutable.rotate(rotation)));
                }
                default -> throw new IllegalArgumentException("Unknown mutable BlockPos operation");
            }
            event.add("state", snapshot(mutable));
            events.add(event);
        }
        result.add("constructors", constructors);
        result.add("events", events);
        return result;
    }
}
