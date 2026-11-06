import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.Direction;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

class MathShapeBoxOracle extends MathShapeBorderOracle {
    static JsonObject snapshotWithCoords(VoxelShape shape) {
        var state = snapshot(shape);
        var coordinates = new JsonObject();
        for (Direction.Axis axis : Direction.Axis.values()) {
            var values = new JsonArray();
            var coords = shape.getCoords(axis);
            for (int i = 0; i < coords.size(); i++) {
                values.add(Oracle.bits(coords.getDouble(i)));
            }
            coordinates.add(axis.getName(), values);
        }
        state.add("coords", coordinates);
        return state;
    }

    static JsonObject shapesingleCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var events = new JsonArray();
        events.add(snapshot(shape));
        for (var entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            shape = switch (step.get("op").getAsString()) {
                case "move" -> shape.move(f64(step, "dx"), f64(step, "dy"), f64(step, "dz"));
                case "move_vec3" -> shape.move(vec3(step.getAsJsonObject("delta")));
                case "move_block_pos" -> shape.move(blockPos(step.getAsJsonObject("delta")));
                default -> throw new IllegalArgumentException("Unknown VoxelShape op");
            };
            events.add(snapshot(shape));
        }
        result.add("events", events);
        return result;
    }

    static JsonObject shapejoinCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject shapejoinManyCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("join_many needs at least one shape");
        }
        VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
        VoxelShape[] tail = new VoxelShape[entries.size() - 1];
        for (int i = 1; i < entries.size(); i++) {
            tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
        }
        result.add("shape", snapshot(Shapes.or(first, tail)));
        return result;
    }

    static JsonObject shapesingleEncompassingCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("single_encompassing needs at least one shape");
        }
        VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
        VoxelShape[] tail = new VoxelShape[entries.size() - 1];
        for (int i = 1; i < entries.size(); i++) {
            tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
        }
        result.add("shape", snapshot(Shapes.or(first, tail).singleEncompassing()));
        return result;
    }

    static JsonObject shapeoptimizeCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("optimize needs at least one shape");
        }
        VoxelShape shape = voxelShape(entries.get(0).getAsJsonObject());
        for (int i = 1; i < entries.size(); i++) {
            shape = Shapes.joinUnoptimized(
                shape,
                voxelShape(entries.get(i).getAsJsonObject()),
                BooleanOp.OR
            );
        }
        var events = new JsonArray();
        events.add(snapshotWithCoords(shape));
        events.add(snapshotWithCoords(shape.optimize()));
        result.add("events", events);
        return result;
    }

    static JsonObject shapeedgesCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("edges needs at least one shape");
        }
        VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
        VoxelShape[] tail = new VoxelShape[entries.size() - 1];
        for (int i = 1; i < entries.size(); i++) {
            tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
        }
        VoxelShape shape = Shapes.or(first, tail);
        var edges = new JsonArray();
        shape.forAllEdges((x1, y1, z1, x2, y2, z2) -> {
            var edge = new JsonArray();
            edge.add(Oracle.bits(x1));
            edge.add(Oracle.bits(y1));
            edge.add(Oracle.bits(z1));
            edge.add(Oracle.bits(x2));
            edge.add(Oracle.bits(y2));
            edge.add(Oracle.bits(z2));
            edges.add(edge);
        });
        result.add("edges", edges);
        return result;
    }

    static JsonObject shapejoinCoordsCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape first = voxelShape(request.getAsJsonObject("first"));
        VoxelShape second = voxelShape(request.getAsJsonObject("second"));
        BooleanOp op = switch (request.get("op").getAsString()) {
            case "or" -> BooleanOp.OR;
            case "only_first" -> BooleanOp.ONLY_FIRST;
            case "only_second" -> BooleanOp.ONLY_SECOND;
            case "and" -> BooleanOp.AND;
            case "not_same" -> BooleanOp.NOT_SAME;
            case "first" -> BooleanOp.FIRST;
            case "second" -> BooleanOp.SECOND;
            default -> throw new IllegalArgumentException("Unknown VoxelShape boolean op");
        };
        result.add("unoptimized", snapshotWithCoords(Shapes.joinUnoptimized(first, second, op)));
        result.add("optimized", snapshotWithCoords(Shapes.join(first, second, op)));
        return result;
    }

    static JsonObject shapeboxesCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("boxes needs at least one shape");
        }
        VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
        VoxelShape[] tail = new VoxelShape[entries.size() - 1];
        for (int i = 1; i < entries.size(); i++) {
            tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
        }
        VoxelShape shape = Shapes.or(first, tail);
        var boxes = new JsonArray();
        shape.forAllBoxes((x1, y1, z1, x2, y2, z2) -> {
            var box = new JsonArray();
            box.add(Oracle.bits(x1));
            box.add(Oracle.bits(y1));
            box.add(Oracle.bits(z1));
            box.add(Oracle.bits(x2));
            box.add(Oracle.bits(y2));
            box.add(Oracle.bits(z2));
            boxes.add(box);
        });
        result.add("boxes", boxes);
        return result;
    }

    static JsonObject shapecreateAabbCase(JsonObject request) {
        var result = new JsonObject();
        AABB aabb = box(request.getAsJsonObject("aabb"));
        result.add("shape", snapshot(Shapes.create(aabb)));
        return result;
    }

    static JsonObject shapecreateCoordsCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject box = request.getAsJsonObject("box");
        result.add("shape", snapshot(Shapes.create(
            f64(box, "min_x"), f64(box, "min_y"), f64(box, "min_z"),
            f64(box, "max_x"), f64(box, "max_y"), f64(box, "max_z"))));
        return result;
    }

    static JsonObject shapeinfinityCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape infinity = Shapes.INFINITY;
        String view = request.get("view").getAsString();
        if (view.equals("snapshot")) {
            result.add("shape", snapshot(infinity));
        } else if (view.equals("boxes")) {
            var boxes = new JsonArray();
            infinity.forAllBoxes((x1, y1, z1, x2, y2, z2) -> {
                var box = new JsonArray();
                box.add(Oracle.bits(x1));
                box.add(Oracle.bits(y1));
                box.add(Oracle.bits(z1));
                box.add(Oracle.bits(x2));
                box.add(Oracle.bits(y2));
                box.add(Oracle.bits(z2));
                boxes.add(box);
            });
            result.add("boxes", boxes);
        } else if (view.equals("not_same_self")) {
            result.add("shape", snapshot(Shapes.join(infinity, infinity, BooleanOp.NOT_SAME)));
        } else {
            throw new IllegalArgumentException("Unknown infinity view");
        }
        return result;
    }

    static JsonObject shapeconstantsCase(JsonObject request) {
        var result = new JsonObject();
        String name = request.get("name").getAsString();
        if (name.equals("epsilon")) {
            result.addProperty("epsilon", Oracle.bits(Shapes.EPSILON));
        } else if (name.equals("big_epsilon")) {
            result.addProperty("big_epsilon", Oracle.bits(Shapes.BIG_EPSILON));
        } else if (name.equals("both")) {
            result.addProperty("epsilon", Oracle.bits(Shapes.EPSILON));
            result.addProperty("big_epsilon", Oracle.bits(Shapes.BIG_EPSILON));
        } else {
            throw new IllegalArgumentException("Unknown shape constant");
        }
        return result;
    }

    static JsonObject shapeaxisLimitsCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("axis_limits needs at least one shape");
        }
        VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
        VoxelShape[] tail = new VoxelShape[entries.size() - 1];
        for (int i = 1; i < entries.size(); i++) {
            tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
        }
        VoxelShape shape = Shapes.or(first, tail);
        result.addProperty("min_x", Oracle.bits(shape.min(Direction.Axis.X)));
        result.addProperty("max_x", Oracle.bits(shape.max(Direction.Axis.X)));
        result.addProperty("min_y", Oracle.bits(shape.min(Direction.Axis.Y)));
        result.addProperty("max_y", Oracle.bits(shape.max(Direction.Axis.Y)));
        result.addProperty("min_z", Oracle.bits(shape.min(Direction.Axis.Z)));
        result.addProperty("max_z", Oracle.bits(shape.max(Direction.Axis.Z)));
        return result;
    }

    static JsonObject shapeaxisLimitsAtCase(JsonObject request) {
        var result = new JsonObject();
        var entries = request.getAsJsonArray("shapes");
        if (entries == null || entries.size() == 0) {
            throw new IllegalArgumentException("axis_limits_at needs at least one shape");
        }
        VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
        VoxelShape[] tail = new VoxelShape[entries.size() - 1];
        for (int i = 1; i < entries.size(); i++) {
            tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
        }
        VoxelShape shape = Shapes.or(first, tail);
        var values = new JsonArray();
        for (var entry : request.getAsJsonArray("queries")) {
            JsonObject query = entry.getAsJsonObject();
            Direction.Axis axis = Direction.Axis.byName(query.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown shape axis");
            }
            double value = query.get("op").getAsString().equals("min")
                ? shape.min(axis, f64(query, "b"), f64(query, "c"))
                : shape.max(axis, f64(query, "b"), f64(query, "c"));
            values.add(Oracle.bits(value));
        }
        result.add("values", values);
        return result;
    }

    static JsonObject shapeequalCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape first = voxelShape(request.getAsJsonObject("first"));
        VoxelShape second = voxelShape(request.getAsJsonObject("second"));
        result.addProperty("equal", Shapes.equal(first, second));
        return result;
    }

    static JsonObject shapeobjectEqualsCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        VoxelShape sameGeometry = voxelShape(request.getAsJsonObject("same"));
        VoxelShape differentGeometry = voxelShape(request.getAsJsonObject("different"));
        result.addProperty("self", shape.equals(shape));
        result.addProperty("same_geometry", shape.equals(sameGeometry));
        result.addProperty("different_geometry", shape.equals(differentGeometry));
        return result;
    }

    static JsonObject shapeobjectEqualsConstantsCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape empty = Shapes.empty();
        VoxelShape block = Shapes.block();
        VoxelShape infinity = Shapes.INFINITY;
        result.addProperty("empty_self", empty.equals(Shapes.empty()));
        result.addProperty("empty_thin", empty.equals(Shapes.box(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)));
        result.addProperty("empty_move", empty.equals(empty.move(1.0, 2.0, 3.0)));
        result.addProperty("block_self", block.equals(Shapes.block()));
        result.addProperty("block_box", block.equals(Shapes.box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)));
        result.addProperty("block_move", block.equals(block.move(0.0, 0.0, 0.0)));
        result.addProperty("infinity_self", infinity.equals(Shapes.INFINITY));
        result.addProperty("infinity_box", infinity.equals(Shapes.box(
            Double.NEGATIVE_INFINITY,
            Double.NEGATIVE_INFINITY,
            Double.NEGATIVE_INFINITY,
            Double.POSITIVE_INFINITY,
            Double.POSITIVE_INFINITY,
            Double.POSITIVE_INFINITY
        )));
        return result;
    }

    static JsonObject shapetoStringCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        result.addProperty("value", shape.toString());
        return result;
    }
}
