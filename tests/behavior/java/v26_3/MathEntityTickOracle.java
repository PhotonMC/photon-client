import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

class MathEntityTickOracle extends MathEntityStateOracle {
    static JsonObject clientWorldEntityBoatAirTickResult(JsonObject request) {
        JsonObject input = request.getAsJsonObject("input");
        JsonObject boatInput = request.getAsJsonObject("boat");
        TickBoatFixture boat = TickBoatFixture.create(
            boatInput.get("entity_id").getAsInt(),
            vec3(boatInput.getAsJsonObject("position")),
            vec3(boatInput.getAsJsonObject("movement")),
            Oracle.f32(boatInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(boatInput.getAsJsonObject("rotation"), "x"),
            boatInput.get("no_gravity").getAsBoolean(),
            true,
            input.get("left").getAsBoolean(),
            input.get("right").getAsBoolean(),
            input.get("forward").getAsBoolean(),
            input.get("backward").getAsBoolean());

        var states = new JsonArray();
        states.add(tickBoatState("initial", boat));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            switch (operation.get("action").getAsString()) {
                case "tick" -> {
                    for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                        boat.tickThroughClientLevel();
                    }
                }
                case "set_no_gravity" -> boat.setNoGravity(operation.get("value").getAsBoolean());
                default -> throw new IllegalArgumentException("Unknown boat air tick action");
            }
            states.add(tickBoatState(operation.get("label").getAsString(), boat));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityBoatRemoteTickResult(JsonObject request) {
        JsonObject input = request.getAsJsonObject("input");
        JsonObject boatInput = request.getAsJsonObject("boat");
        TickBoatFixture boat = TickBoatFixture.create(
            boatInput.get("entity_id").getAsInt(),
            vec3(boatInput.getAsJsonObject("position")),
            vec3(boatInput.getAsJsonObject("movement")),
            Oracle.f32(boatInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(boatInput.getAsJsonObject("rotation"), "x"),
            boatInput.get("no_gravity").getAsBoolean(),
            false,
            input.get("left").getAsBoolean(),
            input.get("right").getAsBoolean(),
            input.get("forward").getAsBoolean(),
            input.get("backward").getAsBoolean());

        var states = new JsonArray();
        states.add(tickBoatState("initial", boat));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown boat remote tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                boat.tickThroughClientLevel();
            }
            states.add(tickBoatState(operation.get("label").getAsString(), boat));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickBoatState(String label, TickBoatFixture boat) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        var entity = new JsonObject();
        entity.addProperty("entity_id", boat.entityId());
        entity.addProperty("tick_count", boat.tickCount);
        entity.add("position", snapshotClip(boat.position()));
        entity.add("previous_position", snapshotClip(boat.previousPosition()));
        entity.add("motion", snapshotClip(boat.motion));
        entity.addProperty("rotation_y", Oracle.bits(boat.yRot));
        entity.addProperty("rotation_x", Oracle.bits(boat.xRot));
        entity.addProperty("previous_rotation_y", Oracle.bits(boat.yRotO));
        entity.addProperty("previous_rotation_x", Oracle.bits(boat.xRotO));
        entity.addProperty("no_gravity", boat.noGravity);
        entity.addProperty("delta_rotation", Oracle.bits(boat.deltaRotation));
        entity.addProperty("paddle_left", boat.paddleLeft);
        entity.addProperty("paddle_right", boat.paddleRight);
        entities.add(entity);
        state.add("entities", entities);
        return state;
    }

    static JsonObject clientWorldEntityTickingEligibilityResult(JsonObject request) {
        JsonObject playerInput = request.getAsJsonObject("player");
        JsonObject remoteInput = request.getAsJsonObject("remote");
        TickEligibilityFixture fixture = TickEligibilityFixture.create(
            playerInput.get("entity_id").getAsInt(),
            remoteInput.get("entity_id").getAsInt());

        var states = new JsonArray();
        states.add(tickEligibilityState("initial", fixture));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            switch (operation.get("action").getAsString()) {
                case "tick" -> fixture.tick(operation.get("ticks").getAsInt());
                case "unload_chunk" -> fixture.setRemoteTicking(false);
                case "load_chunk" -> fixture.setRemoteTicking(true);
                default -> throw new IllegalArgumentException(
                    "Unknown entity ticking eligibility action");
            }
            states.add(tickEligibilityState(operation.get("label").getAsString(), fixture));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickEligibilityState(
        String label, TickEligibilityFixture fixture
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        state.addProperty("game_time", fixture.gameTime);

        var player = new JsonObject();
        player.addProperty("entity_id", fixture.localPlayerId);
        player.addProperty("tick_count", fixture.localPlayerTickCount);
        player.addProperty("always_ticking", true);
        player.addProperty("ticking", true);
        state.add("local_player", player);

        var entities = new JsonArray();
        var remote = new JsonObject();
        remote.addProperty("entity_id", fixture.remoteEntityId);
        remote.addProperty("tick_count", fixture.remoteTickCount);
        remote.addProperty("always_ticking", false);
        remote.addProperty("ticking", fixture.remoteTicking);
        entities.add(remote);
        state.add("entities", entities);
        return state;
    }

}
