import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.math.OctahedralGroup;
import net.minecraft.core.Direction;
import net.minecraft.world.level.block.state.properties.AttachFace;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

class MathShapeRotationOracle extends MathShapeQueryOracle {
    static JsonObject shaperotateCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
        result.add("shape", snapshot(Shapes.rotate(shape, rotation)));
        return result;
    }

    static JsonObject shaperotateJoinCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape first = voxelShape(request.getAsJsonObject("first"));
        VoxelShape second = voxelShape(request.getAsJsonObject("second"));
        OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
        result.add("shape", snapshot(Shapes.rotate(Shapes.or(first, second), rotation)));
        return result;
    }

    static JsonObject shaperotateCenterCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
        result.add("shape", snapshot(Shapes.rotate(
            shape, rotation, vec3(request.getAsJsonObject("center")))));
        return result;
    }

    static JsonObject shaperotateHorizontalAxisCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var axes = Shapes.rotateHorizontalAxis(shape);
        var snapshots = new JsonArray();
        for (Direction.Axis axis : Direction.Axis.values()) {
            if (axes.containsKey(axis)) {
                var entry = new JsonObject();
                entry.addProperty("axis", axis.getName());
                entry.add("shape", snapshot(axes.get(axis)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateHorizontalAxisCenterCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var axes = Shapes.rotateHorizontalAxis(shape, vec3(request.getAsJsonObject("center")));
        var snapshots = new JsonArray();
        for (Direction.Axis axis : Direction.Axis.values()) {
            if (axes.containsKey(axis)) {
                var entry = new JsonObject();
                entry.addProperty("axis", axis.getName());
                entry.add("shape", snapshot(axes.get(axis)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateAllAxisCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var axes = Shapes.rotateAllAxis(shape);
        var snapshots = new JsonArray();
        for (Direction.Axis axis : Direction.Axis.values()) {
            if (axes.containsKey(axis)) {
                var entry = new JsonObject();
                entry.addProperty("axis", axis.getName());
                entry.add("shape", snapshot(axes.get(axis)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateAllAxisCenterCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var axes = Shapes.rotateAllAxis(shape, vec3(request.getAsJsonObject("center")));
        var snapshots = new JsonArray();
        for (Direction.Axis axis : Direction.Axis.values()) {
            if (axes.containsKey(axis)) {
                var entry = new JsonObject();
                entry.addProperty("axis", axis.getName());
                entry.add("shape", snapshot(axes.get(axis)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateHorizontalCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var directions = Shapes.rotateHorizontal(shape);
        var snapshots = new JsonArray();
        for (Direction direction : Direction.values()) {
            if (directions.containsKey(direction)) {
                var entry = new JsonObject();
                entry.addProperty("direction", direction.getName());
                entry.add("shape", snapshot(directions.get(direction)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateHorizontalInitialCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
        var directions = Shapes.rotateHorizontal(shape, initial);
        var snapshots = new JsonArray();
        for (Direction direction : Direction.values()) {
            if (directions.containsKey(direction)) {
                var entry = new JsonObject();
                entry.addProperty("direction", direction.getName());
                entry.add("shape", snapshot(directions.get(direction)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateAllCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var directions = Shapes.rotateAll(shape);
        var snapshots = new JsonArray();
        for (Direction direction : Direction.values()) {
            if (directions.containsKey(direction)) {
                var entry = new JsonObject();
                entry.addProperty("direction", direction.getName());
                entry.add("shape", snapshot(directions.get(direction)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateAllCenterCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var directions = Shapes.rotateAll(shape, vec3(request.getAsJsonObject("center")));
        var snapshots = new JsonArray();
        for (Direction direction : Direction.values()) {
            if (directions.containsKey(direction)) {
                var entry = new JsonObject();
                entry.addProperty("direction", direction.getName());
                entry.add("shape", snapshot(directions.get(direction)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateAllInitialCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
        var directions = Shapes.rotateAll(shape, initial, vec3(request.getAsJsonObject("center")));
        var snapshots = new JsonArray();
        for (Direction direction : Direction.values()) {
            if (directions.containsKey(direction)) {
                var entry = new JsonObject();
                entry.addProperty("direction", direction.getName());
                entry.add("shape", snapshot(directions.get(direction)));
                snapshots.add(entry);
            }
        }
        result.add("shapes", snapshots);
        return result;
    }

    static JsonObject shaperotateAttachFaceCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        var faces = Shapes.rotateAttachFace(shape);
        var faceSnapshots = new JsonArray();
        for (AttachFace face : AttachFace.values()) {
            var directions = faces.get(face);
            var directionSnapshots = new JsonArray();
            for (Direction direction : Direction.values()) {
                if (directions.containsKey(direction)) {
                    var directionEntry = new JsonObject();
                    directionEntry.addProperty("direction", direction.getName());
                    directionEntry.add("shape", snapshot(directions.get(direction)));
                    directionSnapshots.add(directionEntry);
                }
            }
            var faceEntry = new JsonObject();
            faceEntry.addProperty("face", face.getSerializedName());
            faceEntry.add("shapes", directionSnapshots);
            faceSnapshots.add(faceEntry);
        }
        result.add("faces", faceSnapshots);
        return result;
    }

    static JsonObject shaperotateAttachFaceInitialCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
        OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
        var faces = Shapes.rotateAttachFace(shape, initial);
        var faceSnapshots = new JsonArray();
        for (AttachFace face : AttachFace.values()) {
            var directions = faces.get(face);
            var directionSnapshots = new JsonArray();
            for (Direction direction : Direction.values()) {
                if (directions.containsKey(direction)) {
                    var directionEntry = new JsonObject();
                    directionEntry.addProperty("direction", direction.getName());
                    directionEntry.add("shape", snapshot(directions.get(direction)));
                    directionSnapshots.add(directionEntry);
                }
            }
            var faceEntry = new JsonObject();
            faceEntry.addProperty("face", face.getSerializedName());
            faceEntry.add("shapes", directionSnapshots);
            faceSnapshots.add(faceEntry);
        }
        result.add("faces", faceSnapshots);
        return result;
    }
}
