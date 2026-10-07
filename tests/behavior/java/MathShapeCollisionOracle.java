import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.core.Direction;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.BlockCollisions;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

class MathShapeCollisionOracle extends MathProjectileOracle {
    static JsonObject shapecollideCase(JsonObject request) {
        var result = new JsonObject();
        VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
        AABB moving = box(request.getAsJsonObject("moving"));
        Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
        if (axis == null) {
            throw new IllegalArgumentException("Unknown collision axis");
        }
        result.addProperty("distance", Oracle.bits(shape.collide(axis, moving, f64(request, "distance"))));
        return result;
    }

    static JsonObject shapecollideManyCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject shapecollideMovementCase(JsonObject request) {
        var result = new JsonObject();
        AABB moving = box(request.getAsJsonObject("moving"));
        var shapes = new java.util.ArrayList<VoxelShape>();
        for (var entry : request.getAsJsonArray("shapes")) {
            shapes.add(voxelShape(entry.getAsJsonObject()));
        }
        try {
            var method = Entity.class.getDeclaredMethod(
                "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
            method.setAccessible(true);
            var resolved = (Vec3) method.invoke(
                null, vec3(request.getAsJsonObject("movement")), moving, shapes);
            result.add("movement", snapshot(resolved));
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Entity.collideWithShapes reflection failed", error);
        }
        return result;
    }

    static JsonObject shapeentityMovementStateCase(JsonObject request) {
        var result = new JsonObject();
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            Vec3 movement = vec3(input.getAsJsonObject("movement"));
            AABB moving = box(input.getAsJsonObject("moving"));
            var shapes = new java.util.ArrayList<VoxelShape>();
            for (var shapeEntry : input.getAsJsonArray("shapes")) {
                shapes.add(voxelShape(shapeEntry.getAsJsonObject()));
            }
            try {
                var method = Entity.class.getDeclaredMethod(
                    "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
                method.setAccessible(true);
                var resolved = (Vec3) method.invoke(null, movement, moving, shapes);
                boolean xCollision = !Mth.equal(movement.x, resolved.x);
                boolean yCollision = movement.y != resolved.y;
                boolean zCollision = !Mth.equal(movement.z, resolved.z);
                boolean horizontalCollision = xCollision || zCollision;
                boolean verticalCollisionBelow = yCollision && movement.y < 0.0;
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(resolved));
                state.addProperty("x_collision", xCollision);
                state.addProperty("y_collision", yCollision);
                state.addProperty("z_collision", zCollision);
                state.addProperty("horizontal_collision", horizontalCollision);
                state.addProperty("vertical_collision", yCollision);
                state.addProperty("vertical_collision_below", verticalCollisionBelow);
                state.addProperty("on_ground", verticalCollisionBelow);
                states.add(state);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Entity.collideWithShapes reflection failed", error);
            }
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeentityMovementStateRetentionCase(JsonObject request) {
        var result = new JsonObject();
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            Vec3 delta = vec3(input.getAsJsonObject("delta"));
            AABB moving = box(input.getAsJsonObject("moving"));
            var shapes = new java.util.ArrayList<VoxelShape>();
            for (var shapeEntry : input.getAsJsonArray("shapes")) {
                shapes.add(voxelShape(shapeEntry.getAsJsonObject()));
            }
            try {
                var method = Entity.class.getDeclaredMethod(
                    "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
                method.setAccessible(true);
                var resolved = (Vec3)method.invoke(null, delta, moving, shapes);
                boolean xCollision = !Mth.equal(delta.x, resolved.x);
                boolean yCollision = delta.y != resolved.y;
                boolean zCollision = !Mth.equal(delta.z, resolved.z);
                boolean horizontalCollision = xCollision || zCollision;
                boolean verticalCollision = input.getAsJsonObject("previous")
                    .get("vertical_collision").getAsBoolean();
                boolean verticalCollisionBelow = input.getAsJsonObject("previous")
                    .get("vertical_collision_below").getAsBoolean();
                boolean onGround = input.getAsJsonObject("previous")
                    .get("on_ground").getAsBoolean();
                boolean movedVertically = Math.abs(delta.y) > 0.0;
                boolean authoritative = input.get("authoritative").getAsBoolean();
                if (movedVertically || authoritative) {
                    verticalCollision = yCollision;
                    verticalCollisionBelow = verticalCollision && delta.y < 0.0;
                    onGround = verticalCollisionBelow;
                }
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(resolved));
                state.addProperty("x_collision", xCollision);
                state.addProperty("y_collision", yCollision);
                state.addProperty("z_collision", zCollision);
                state.addProperty("horizontal_collision", horizontalCollision);
                state.addProperty("vertical_collision", verticalCollision);
                state.addProperty("vertical_collision_below", verticalCollisionBelow);
                state.addProperty("on_ground", onGround);
                states.add(state);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Entity movement state retention reflection failed", error);
            }
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeentityCollisionShapesCase(JsonObject request) {
        var result = new JsonObject();
        var output = System.out;
        var errorOutput = System.err;
        try {
            SharedConstants.setVersion(TEST_VERSION);
            Bootstrap.bootStrap();
        } finally {
            System.setOut(output);
            System.setErr(errorOutput);
        }
        AABB testArea = box(request.getAsJsonObject("test_area"));
        boolean sourcePresent = request.has("source_present")
            && request.get("source_present").getAsBoolean();
        CollisionFixtureEntity source = sourcePresent
            ? CollisionFixtureEntity.create(
                box(request.getAsJsonObject("source_box")), false, true, false)
            : null;
        var entities = new java.util.ArrayList<Entity>();
        for (var entry : request.getAsJsonArray("candidates")) {
            JsonObject candidate = entry.getAsJsonObject();
            boolean isSource = candidate.has("is_source")
                && candidate.get("is_source").getAsBoolean();
            entities.add(sourcePresent && isSource
                ? source
                : CollisionFixtureEntity.create(
                    box(candidate.getAsJsonObject("box")),
                    candidate.has("spectator") && candidate.get("spectator").getAsBoolean(),
                    candidate.has("can_be_collided_with")
                        && candidate.get("can_be_collided_with").getAsBoolean(),
                    candidate.has("same_vehicle") && candidate.get("same_vehicle").getAsBoolean()));
        }
        EntityCollisionFixture collisionFixture = new EntityCollisionFixture(entities);
        var shapes = collisionFixture.getEntityCollisions(source, testArea);
        var shapeSnapshots = new JsonArray();
        for (VoxelShape shape : shapes) {
            shapeSnapshots.add(snapshot(shape));
        }
        result.add("shapes", shapeSnapshots);
        return result;
    }

    static JsonObject shapecollectStepUpHeightsCase(JsonObject request) {
        var result = new JsonObject();
        AABB moving = box(request.getAsJsonObject("moving"));
        var shapes = new java.util.ArrayList<VoxelShape>();
        for (var entry : request.getAsJsonArray("shapes")) {
            shapes.add(voxelShape(entry.getAsJsonObject()));
        }
        try {
            var method = Entity.class.getDeclaredMethod(
                "collectCandidateStepUpHeights", AABB.class, java.util.List.class, float.class, float.class);
            method.setAccessible(true);
            var candidates = (float[]) method.invoke(
                null,
                moving,
                shapes,
                Oracle.f32(request, "max_step_height"),
                Oracle.f32(request, "step_height_to_skip"));
            var values = new JsonArray();
            for (float candidate : candidates) {
                values.add(Oracle.bits(candidate));
            }
            result.add("candidates", values);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Entity.collectCandidateStepUpHeights reflection failed", error);
        }
        return result;
    }

    static JsonObject shapeselectStepUpMovementCase(JsonObject request) {
        var result = new JsonObject();
        AABB boundingBox = box(request.getAsJsonObject("bounding_box"));
        AABB groundedBox = box(request.getAsJsonObject("grounded_box"));
        Vec3 movement = vec3(request.getAsJsonObject("movement"));
        Vec3 movementStep = vec3(request.getAsJsonObject("movement_step"));
        var shapes = new java.util.ArrayList<VoxelShape>();
        for (var entry : request.getAsJsonArray("shapes")) {
            shapes.add(voxelShape(entry.getAsJsonObject()));
        }
        try {
            var candidatesMethod = Entity.class.getDeclaredMethod(
                "collectCandidateStepUpHeights", AABB.class, java.util.List.class, float.class, float.class);
            var collideMethod = Entity.class.getDeclaredMethod(
                "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
            candidatesMethod.setAccessible(true);
            collideMethod.setAccessible(true);
            var candidates = (float[]) candidatesMethod.invoke(
                null,
                groundedBox,
                shapes,
                Oracle.f32(request, "max_step_height"),
                Oracle.f32(request, "step_height_to_skip"));
            Vec3 selected = movementStep;
            for (float candidate : candidates) {
                Vec3 stepFromGround = (Vec3)collideMethod.invoke(
                    null,
                    new Vec3(movement.x, candidate, movement.z),
                    groundedBox,
                    shapes);
                if (stepFromGround.horizontalDistanceSqr() > movementStep.horizontalDistanceSqr()) {
                    double distanceToGround = boundingBox.minY - groundedBox.minY;
                    selected = stepFromGround.subtract(0.0, distanceToGround, 0.0);
                    break;
                }
            }
            result.add("movement", snapshot(selected));
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Entity step-up movement selection reflection failed", error);
        }
        return result;
    }

    static JsonObject shapecollectBlockCollisionsCase(JsonObject request) {
        var result = new JsonObject();
        AABB query = box(request.getAsJsonObject("box"));
        CollisionGetter collisionGetter = collisionFixture(request.getAsJsonArray("placements"));
        var collisions = new JsonArray();
        var iterator = new BlockCollisions<JsonObject>(
            collisionGetter,
            (Entity)null,
            query,
            false,
            (pos, shape) -> {
                var collision = new JsonObject();
                collision.add("position", snapshot(pos));
                collision.add("shape", snapshot(shape));
                return collision;
            });
        while (iterator.hasNext()) {
            collisions.add(iterator.next());
        }
        result.add("collisions", collisions);
        return result;
    }
}
