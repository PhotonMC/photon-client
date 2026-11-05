import com.google.gson.JsonArray;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import net.minecraft.core.Direction;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

class MathShapeQueryOracle extends MathShapePacketOracle {
    static JsonObject shapeclipCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject shapeclipJoinCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject shapeclosestCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        var closest = shape.closestPointTo(vec3(request.getAsJsonObject("point")));
        result.add("point", closest.isPresent()
            ? snapshot(closest.get()) : com.google.gson.JsonNull.INSTANCE);
        return result;
    }

    static JsonObject shapeclosestJoinCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape first = voxelShape(request.getAsJsonObject("first"));
        VoxelShape second = voxelShape(request.getAsJsonObject("second"));
        VoxelShape shape = Shapes.join(first, second, BooleanOp.OR);
        var closest = shape.closestPointTo(vec3(request.getAsJsonObject("point")));
        result.add("point", closest.isPresent()
            ? snapshot(closest.get()) : com.google.gson.JsonNull.INSTANCE);
        return result;
    }

    static JsonObject shapefaceCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown face direction");
        }
        result.add("shape", snapshot(shape.getFaceShape(direction)));
        return result;
    }

    static JsonObject shapefaceJoinCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape first = voxelShape(request.getAsJsonObject("first"));
        VoxelShape second = voxelShape(request.getAsJsonObject("second"));
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown face direction");
        }
        result.add("shape", snapshot(Shapes.or(first, second).getFaceShape(direction)));
        return result;
    }

    static JsonObject shapeblockOccludesCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown occlusion direction");
        }
        result.addProperty("occludes", Shapes.blockOccludes(shape, occluder, direction));
        return result;
    }

    static JsonObject shapeblockOccludesJoinCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape first = voxelShape(request.getAsJsonObject("first"));
        VoxelShape second = voxelShape(request.getAsJsonObject("second"));
        VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown occlusion direction");
        }
        result.addProperty("occludes", Shapes.blockOccludes(Shapes.or(first, second), occluder, direction));
        return result;
    }

    static JsonObject shapemergedOccludesCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown merged occlusion direction");
        }
        result.addProperty("occludes", Shapes.mergedFaceOccludes(shape, occluder, direction));
        return result;
    }

    static JsonObject shapefaceOccludesCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
        result.addProperty("occludes", Shapes.faceShapeOccludes(shape, occluder));
        return result;
    }
}
