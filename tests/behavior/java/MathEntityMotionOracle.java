import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;

class MathEntityMotionOracle extends MathProjectileFixtures {
    static JsonObject clientWorldEntityPlayerHeadRotationResult(JsonObject request) {
        int entityId = request.get("entity_id").getAsInt();
        JsonObject initialPosition = request.getAsJsonObject("initial_position");
        JsonObject initialRotation = request.getAsJsonObject("initial_rotation");
        TickRemotePlayerFixture player = TickRemotePlayerFixture.create(
            vec3(initialPosition),
            Oracle.f32(initialRotation, "y"),
            Oracle.f32(initialRotation, "x"),
            Oracle.f32(request, "initial_head"));

        var states = new JsonArray();
        states.add(tickRemotePlayerHeadState("initial", true, player));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("rotate_head")) {
                throw new IllegalArgumentException("Unknown remote player head action");
            }
            JsonElement headTarget = operation.get("head_target");
            if (headTarget != null && !headTarget.isJsonNull()) {
                player.lerpHeadTo(
                    Float.intBitsToFloat(Integer.parseUnsignedInt(headTarget.getAsString(), 16)),
                    3);
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                player.tickThroughClientLevel();
            }
            states.add(tickRemotePlayerHeadState(
                operation.get("label").getAsString(), true, player));
        }
        var result = new JsonObject();
        result.addProperty("entity_id", entityId);
        result.add("states", states);
        return result;
    }

    static JsonObject tickRemotePlayerHeadState(
        String label, boolean accepted, TickRemotePlayerFixture player
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("head_rotation", Oracle.bits(player.yHeadRot));
        state.addProperty("previous_head_rotation", Oracle.bits(player.yHeadRotO));
        state.addProperty("head_target", Oracle.bits(player.headTarget()));
        state.addProperty("head_lerp_steps", player.headSteps());
        state.addProperty("body_rotation_y", Oracle.bits(player.yBodyRot));
        return state;
    }

    static JsonObject clientWorldEntityPlayerInterpolationResult(JsonObject request) {
        int entityId = request.get("entity_id").getAsInt();
        JsonObject initialPosition = request.getAsJsonObject("initial_position");
        JsonObject initialRotation = request.getAsJsonObject("initial_rotation");
        RemotePlayerInterpolationFixture player = RemotePlayerInterpolationFixture.create(
            vec3(initialPosition),
            Oracle.f32(initialRotation, "y"),
            Oracle.f32(initialRotation, "x"),
            Oracle.f32(request, "initial_head"));

        var states = new JsonArray();
        states.add(remotePlayerInterpolationState("initial", player));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            switch (operation.get("action").getAsString()) {
                case "move" -> player.moveOrInterpolateTo(
                    vec3(operation.getAsJsonObject("target_position")),
                    Oracle.f32(operation.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(operation.getAsJsonObject("rotation"), "x"));
                case "tick" -> {
                    for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                        player.tickThroughClientLevel();
                    }
                }
                default -> throw new IllegalArgumentException(
                    "Unknown remote player interpolation action");
            }
            states.add(remotePlayerInterpolationState(
                operation.get("label").getAsString(), player));
        }
        var result = new JsonObject();
        result.addProperty("entity_id", entityId);
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPlayerSnapResult(JsonObject request) {
        int entityId = request.get("entity_id").getAsInt();
        JsonObject initialPosition = request.getAsJsonObject("initial_position");
        JsonObject initialRotation = request.getAsJsonObject("initial_rotation");
        RemotePlayerInterpolationFixture player = RemotePlayerInterpolationFixture.create(
            vec3(initialPosition),
            Oracle.f32(initialRotation, "y"),
            Oracle.f32(initialRotation, "x"),
            Oracle.f32(request, "initial_head"));

        var states = new JsonArray();
        states.add(remotePlayerInterpolationState("initial", player));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            switch (operation.get("action").getAsString()) {
                case "position_sync" -> player.applyPositionSync(
                    vec3(operation.getAsJsonObject("target_position")),
                    Oracle.f32(operation.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(operation.getAsJsonObject("rotation"), "x"));
                case "tick" -> {
                    for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                        player.tickThroughClientLevel();
                    }
                }
                default -> throw new IllegalArgumentException(
                    "Unknown remote player snap action");
            }
            states.add(remotePlayerInterpolationState(
                operation.get("label").getAsString(), player));
        }
        var result = new JsonObject();
        result.addProperty("entity_id", entityId);
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPlayerMotionResult(JsonObject request) {
        int entityId = request.get("entity_id").getAsInt();
        JsonObject initialPosition = request.getAsJsonObject("initial_position");
        JsonObject initialRotation = request.getAsJsonObject("initial_rotation");
        RemotePlayerInterpolationFixture player = RemotePlayerInterpolationFixture.create(
            vec3(initialPosition),
            Oracle.f32(initialRotation, "y"),
            Oracle.f32(initialRotation, "x"),
            Oracle.f32(request, "initial_head"),
            vec3(request.getAsJsonObject("initial_motion")));

        var states = new JsonArray();
        states.add(remotePlayerMotionState("initial", player));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            switch (operation.get("action").getAsString()) {
                case "motion" -> player.lerpMotion(
                    vec3(operation.getAsJsonObject("target_motion")));
                case "tick" -> {
                    for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                        player.tickThroughRemotePlayer();
                    }
                }
                default -> throw new IllegalArgumentException(
                    "Unknown remote player motion action");
            }
            states.add(remotePlayerMotionState(
                operation.get("label").getAsString(), player));
        }
        var result = new JsonObject();
        result.addProperty("entity_id", entityId);
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityTeleportResult(JsonObject request) {
        int entityId = request.get("entity_id").getAsInt();
        JsonObject initialPosition = request.getAsJsonObject("initial_position");
        JsonObject initialRotation = request.getAsJsonObject("initial_rotation");
        RemotePlayerInterpolationFixture player = RemotePlayerInterpolationFixture.create(
            vec3(initialPosition),
            Oracle.f32(initialRotation, "y"),
            Oracle.f32(initialRotation, "x"),
            Oracle.f32(request, "initial_head"),
            vec3(request.getAsJsonObject("initial_motion")));

        var states = new JsonArray();
        states.add(remotePlayerTeleportState("initial", player));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            switch (operation.get("action").getAsString()) {
                case "teleport" -> player.applyTeleport(
                    vec3(operation.getAsJsonObject("position")),
                    vec3(operation.getAsJsonObject("delta_motion")),
                    Oracle.f32(operation.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(operation.getAsJsonObject("rotation"), "x"),
                    operation.get("relative_flags").getAsInt(),
                    true);
                case "tick" -> {
                    for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                        player.tickThroughClientLevel();
                    }
                }
                default -> throw new IllegalArgumentException(
                    "Unknown remote entity teleport action");
            }
            states.add(remotePlayerTeleportState(
                operation.get("label").getAsString(), player));
        }
        var result = new JsonObject();
        result.addProperty("entity_id", entityId);
        result.add("states", states);
        return result;
    }

    static JsonObject remotePlayerInterpolationState(
        String label, RemotePlayerInterpolationFixture player
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        state.addProperty("tick_count", player.tickCount);
        state.add("position", snapshotClip(player.position()));
        state.add("previous_position", snapshotClip(player.oldPosition()));
        state.addProperty("rotation_y", Oracle.bits(player.getYRot()));
        state.addProperty("rotation_x", Oracle.bits(player.getXRot()));
        state.addProperty("previous_rotation_y", Oracle.bits(player.yRotO));
        state.addProperty("previous_rotation_x", Oracle.bits(player.xRotO));
        var interpolation = player.getInterpolation();
        boolean active = interpolation.hasActiveInterpolation();
        state.addProperty("interpolation_active", active);
        if (active) {
            state.add("interpolation_target_position", snapshotClip(interpolation.position()));
            var targetRotation = new JsonObject();
            targetRotation.addProperty("y", Oracle.bits(interpolation.yRot()));
            targetRotation.addProperty("x", Oracle.bits(interpolation.xRot()));
            state.add("interpolation_target_rotation", targetRotation);
        } else {
            state.add("interpolation_target_position", JsonNull.INSTANCE);
            state.add("interpolation_target_rotation", JsonNull.INSTANCE);
        }
        state.addProperty("interpolation_steps", player.interpolationSteps());
        return state;
    }

    static JsonObject remotePlayerMotionState(
        String label, RemotePlayerInterpolationFixture player
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        state.addProperty("tick_count", player.tickCount);
        state.add("position", snapshotClip(player.position()));
        state.add("previous_position", snapshotClip(player.oldPosition()));
        state.add("motion", snapshotClip(player.getDeltaMovement()));
        if (player.motionSteps() > 0) {
            state.add("motion_target", snapshotClip(player.motionTarget()));
        } else {
            state.add("motion_target", JsonNull.INSTANCE);
        }
        state.addProperty("motion_steps", player.motionSteps());
        return state;
    }

    static JsonObject remotePlayerTeleportState(
        String label, RemotePlayerInterpolationFixture player
    ) {
        var state = remotePlayerInterpolationState(label, player);
        state.add("motion", snapshotClip(player.getDeltaMovement()));
        return state;
    }
}
