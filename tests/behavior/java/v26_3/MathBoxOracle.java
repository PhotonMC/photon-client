import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import net.minecraft.core.Direction;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.phys.AABB;
import org.joml.Vector3f;

class MathBoxOracle extends MathBlockPosOracle {
    static JsonObject aabbBuilderCase(JsonObject request) {
        var result = new JsonObject();
        AABB.Builder builder = new AABB.Builder();
        var events = new JsonArray();
        var initial = new JsonObject();
        initial.addProperty("defined", builder.isDefined());
        events.add(initial);
        for (var entry : request.getAsJsonArray("points")) {
            JsonObject point = entry.getAsJsonObject();
            builder.include(new Vector3f(
                Oracle.f32(point, "x"),
                Oracle.f32(point, "y"),
                Oracle.f32(point, "z")
            ));
            var state = new JsonObject();
            state.addProperty("defined", builder.isDefined());
            events.add(state);
        }
        result.add("events", events);
        var build = new JsonObject();
        try {
            build.addProperty("outcome", "value");
            build.add("box", snapshot(builder.build()));
        } catch (IllegalStateException error) {
            build.addProperty("outcome", "error");
            build.addProperty("error", "undefined");
            result.addProperty("java_exception", error.getClass().getName());
        }
        result.add("build", build);
        return result;
    }

    static JsonObject aabbCase(JsonObject request) {
        var result = new JsonObject();
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
                case "expand_vec3" -> {
                    box = box.expandTowards(vec3(step.getAsJsonObject("delta")));
                    events.add(snapshot(box));
                }
                case "equals" -> {
                    var state = new JsonObject();
                    state.addProperty("equal", box.equals(box(step.getAsJsonObject("box"))));
                    events.add(state);
                }
                case "hash_code" -> {
                    var state = new JsonObject();
                    state.addProperty("hash_code", box.hashCode());
                    events.add(state);
                }
                case "to_string" -> {
                    var state = new JsonObject();
                    state.addProperty("value", box.toString());
                    events.add(state);
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
                case "deflate_xyz" -> {
                    box = box.deflate(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    events.add(snapshot(box));
                }
                default -> throw new IllegalArgumentException("Unknown AABB op");
            }
        }
        result.add("events", events);
        return result;
    }
}
