import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.world.entity.Avatar;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

class MathShapeDimensionOracle extends MathShapeCollisionOracle {
    @SuppressWarnings("unchecked")
    static java.util.Map<Pose, EntityDimensions> avatarPoseDimensions() {
        try {
            var field = Reflect.field(Avatar.class, "POSES");
            return (java.util.Map<Pose, EntityDimensions>) field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Avatar.POSES reflection failed", error);
        }
    }

    static EntityDimensions dimensionsFromInput(JsonObject input) {
        float width = Oracle.f32(input, "width");
        float height = Oracle.f32(input, "height");
        boolean fixed = input.has("fixed") && input.get("fixed").getAsBoolean();
        EntityDimensions dimensions = fixed
            ? EntityDimensions.fixed(width, height)
            : EntityDimensions.scalable(width, height);
        if (input.has("eye_height")) {
            dimensions = dimensions.withEyeHeight(Oracle.f32(input, "eye_height"));
        }
        return dimensions;
    }

    static JsonObject shapeentityFitCase(JsonObject request) {
        var result = new JsonObject();
        JsonObject dimensionInput = request.getAsJsonObject("dimensions");
        float width = Oracle.f32(dimensionInput, "width");
        float height = Oracle.f32(dimensionInput, "height");
        boolean fixed = dimensionInput.has("fixed") && dimensionInput.get("fixed").getAsBoolean();
        EntityDimensions dimensions = fixed
            ? EntityDimensions.fixed(width, height)
            : EntityDimensions.scalable(width, height);
        if (dimensionInput.has("width_scale") || dimensionInput.has("height_scale")) {
            float widthScale = dimensionInput.has("width_scale")
                ? Oracle.f32(dimensionInput, "width_scale") : 1.0F;
            float heightScale = dimensionInput.has("height_scale")
                ? Oracle.f32(dimensionInput, "height_scale") : 1.0F;
            dimensions = dimensions.scale(widthScale, heightScale);
        }
        if (dimensionInput.has("eye_height")) {
            dimensions = dimensions.withEyeHeight(Oracle.f32(dimensionInput, "eye_height"));
        }
        Vec3 position = vec3(request.getAsJsonObject("position"));
        AABB boundingBox = dimensions.makeBoundingBox(position);
        AABB query = boundingBox.deflate(1.0E-7);
        CollisionGetter collisionGetter = collisionFixture(request.getAsJsonArray("placements"));
        var dimensionSnapshot = new JsonObject();
        dimensionSnapshot.addProperty("width", Oracle.bits(dimensions.width()));
        dimensionSnapshot.addProperty("height", Oracle.bits(dimensions.height()));
        dimensionSnapshot.addProperty("eye_height", Oracle.bits(dimensions.eyeHeight()));
        dimensionSnapshot.addProperty("fixed", dimensions.fixed());
        result.add("dimensions", dimensionSnapshot);
        result.add("box", snapshot(boundingBox));
        result.add("query", snapshot(query));
        result.addProperty("fits", collisionGetter.noCollision(null, query));
        return result;
    }

    static JsonObject shapelivingEntityDimensionsScaleCase(JsonObject request) {
        var result = new JsonObject();
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            int requestedPoseId = input.get("pose_id").getAsInt();
            Pose pose = Pose.BY_ID.apply(requestedPoseId);
            JsonObject dimensionInput = input.getAsJsonObject("default_dimensions");
            float width = Oracle.f32(dimensionInput, "width");
            float height = Oracle.f32(dimensionInput, "height");
            boolean fixed = dimensionInput.has("fixed") && dimensionInput.get("fixed").getAsBoolean();
            EntityDimensions defaultDimensions = fixed
                ? EntityDimensions.fixed(width, height)
                : EntityDimensions.scalable(width, height);
            if (dimensionInput.has("eye_height")) {
                defaultDimensions = defaultDimensions.withEyeHeight(
                    Oracle.f32(dimensionInput, "eye_height"));
            }
            float ageScale = Oracle.f32(input, "age_scale");
            float entityScale = Oracle.f32(input, "entity_scale");
            EntityDimensions dimensions = pose == Pose.SLEEPING
                ? EntityDimensions.fixed(0.2F, 0.2F).withEyeHeight(0.2F)
                : defaultDimensions.scale(ageScale).scale(entityScale);
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.addProperty("requested_pose_id", requestedPoseId);
            state.addProperty("resolved_pose_id", pose.id());
            state.add("dimensions", snapshot(dimensions));
            state.add("box", snapshot(dimensions.makeBoundingBox(
                vec3(input.getAsJsonObject("position")))));
            states.add(state);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeentityDimensionsRefreshCase(JsonObject request) {
        var result = new JsonObject();
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            JsonObject oldInput = input.getAsJsonObject("old_dimensions");
            JsonObject newInput = input.getAsJsonObject("new_dimensions");
            EntityDimensions oldDimensions = dimensionsFromInput(oldInput);
            EntityDimensions newDimensions = dimensionsFromInput(newInput);
            Vec3 position = vec3(input.getAsJsonObject("position"));
            boolean levelClientSide = input.get("level_client_side").getAsBoolean();
            boolean firstTick = input.get("first_tick").getAsBoolean();
            boolean noPhysics = input.get("no_physics").getAsBoolean();
            boolean isPlayer = input.get("is_player").getAsBoolean();
            boolean isSmall = newDimensions.width() <= 4.0F && newDimensions.height() <= 4.0F;
            boolean grew = newDimensions.width() > oldDimensions.width()
                || newDimensions.height() > oldDimensions.height();
            boolean shouldFudge = !levelClientSide
                && !firstTick
                && !noPhysics
                && isSmall
                && grew
                && !isPlayer;
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.add("old_dimensions", snapshot(oldDimensions));
            state.add("new_dimensions", snapshot(newDimensions));
            state.add("position_after_refresh", snapshot(position));
            state.add("bounding_box", snapshot(newDimensions.makeBoundingBox(position)));
            state.addProperty("should_fudge_position", shouldFudge);
            states.add(state);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeplayerPoseDimensionsCase(JsonObject request) {
        var result = new JsonObject();
        var avatarDimensions = avatarPoseDimensions();
        Vec3 position = vec3(request.getAsJsonObject("position"));
        var poses = new JsonArray();
        for (var entry : request.getAsJsonArray("pose_ids")) {
            int requestedId = entry.getAsInt();
            Pose pose = Pose.BY_ID.apply(requestedId);
            EntityDimensions dimensions = avatarDimensions.getOrDefault(
                pose, avatarDimensions.get(Pose.STANDING));
            var poseResult = new JsonObject();
            poseResult.addProperty("requested_id", requestedId);
            poseResult.addProperty("resolved_id", pose.id());
            poseResult.add("dimensions", snapshot(dimensions));
            poseResult.add("box", snapshot(dimensions.makeBoundingBox(position)));
            poses.add(poseResult);
        }
        result.add("poses", poses);
        return result;
    }

    static JsonObject shapeplayerPoseSelectionCase(JsonObject request) {
        var result = new JsonObject();
        var selections = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            var input = entry.getAsJsonObject();
            boolean sleeping = input.get("sleeping").getAsBoolean();
            boolean swimming = input.get("swimming").getAsBoolean();
            boolean fallFlying = input.get("fall_flying").getAsBoolean();
            boolean autoSpinAttack = input.get("auto_spin_attack").getAsBoolean();
            boolean shiftKeyDown = input.get("shift_key_down").getAsBoolean();
            boolean flying = input.get("flying").getAsBoolean();
            boolean spectator = input.get("spectator").getAsBoolean();
            boolean passenger = input.get("passenger").getAsBoolean();
            boolean swimmingFits = input.get("swimming_fits").getAsBoolean();
            boolean desiredFits = input.get("desired_fits").getAsBoolean();
            boolean crouchingFits = input.get("crouching_fits").getAsBoolean();

            Pose desiredPose;
            if (sleeping) {
                desiredPose = Pose.SLEEPING;
            } else if (swimming) {
                desiredPose = Pose.SWIMMING;
            } else if (fallFlying) {
                desiredPose = Pose.FALL_FLYING;
            } else if (autoSpinAttack) {
                desiredPose = Pose.SPIN_ATTACK;
            } else {
                desiredPose = shiftKeyDown && !flying ? Pose.CROUCHING : Pose.STANDING;
            }

            Pose actualPose = null;
            if (swimmingFits) {
                if (spectator || passenger || desiredFits) {
                    actualPose = desiredPose;
                } else if (crouchingFits) {
                    actualPose = Pose.CROUCHING;
                } else {
                    actualPose = Pose.SWIMMING;
                }
            }

            var selection = new JsonObject();
            selection.addProperty("label", input.get("label").getAsString());
            selection.addProperty("desired_id", desiredPose.id());
            selection.addProperty("actual_id", actualPose == null ? null : actualPose.id());
            selections.add(selection);
        }
        result.add("selections", selections);
        return result;
    }
}
