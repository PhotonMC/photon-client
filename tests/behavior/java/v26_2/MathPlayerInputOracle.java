import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.SharedConstants;
import net.minecraft.client.player.KeyboardInput;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.entity.ai.attributes.AttributeMap;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.phys.Vec2;
import sun.misc.Unsafe;

class MathPlayerInputOracle extends MathPlayerAttributeOracle {
    static JsonObject clientWorldPlayerSneakingInputResult(JsonObject request) {
        try {
            var output = System.out;
            var errorOutput = System.err;
            try {
                SharedConstants.setVersion(TEST_VERSION);
                Bootstrap.bootStrap();
            } finally {
                System.setOut(output);
                System.setErr(errorOutput);
            }

            int playerId = request.get("player_id").getAsInt();
            AttributeMap attributes = new AttributeMap(Player.createAttributes().build());
            SneakingSpeedInputFixture player = SneakingSpeedInputFixture.create(attributes);
            var states = new JsonArray();
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                boolean accepted = operation.get("player_id").getAsInt() == playerId;
                if (accepted) {
                    for (JsonElement snapshotEntry : operation.getAsJsonArray("attributes")) {
                        applyPlayerAttributeSnapshot(
                            attributes, snapshotEntry.getAsJsonObject());
                    }
                }
                player.configureApplyInput(
                    Oracle.f32(operation, "move_x"),
                    Oracle.f32(operation, "move_y"),
                    operation.get("jump").getAsBoolean(),
                    Oracle.f32(operation, "xxa"),
                    Oracle.f32(operation, "zza"),
                    operation.get("jumping").getAsBoolean(),
                    Oracle.f32(operation, "x_bob"),
                    Oracle.f32(operation, "y_bob"),
                    Oracle.f32(operation, "x_bob_o"),
                    Oracle.f32(operation, "y_bob_o"),
                    Oracle.f32(operation, "x_rot"),
                    Oracle.f32(operation, "y_rot"),
                    operation.get("controlled_camera").getAsBoolean(),
                    operation.get("using_item").getAsBoolean(),
                    operation.get("passenger").getAsBoolean(),
                    operation.get("moving_slowly").getAsBoolean());
                if (operation.get("using_item").getAsBoolean()) {
                    player.setItemUseSpeed(Oracle.f32(operation, "item_use_speed_multiplier"));
                }
                player.applyInput();

                var state = new JsonObject();
                state.addProperty("label", operation.get("label").getAsString());
                state.addProperty("accepted", accepted);
                state.addProperty(
                    "sneaking_speed",
                    Oracle.bits(player.getAttributeValue(Attributes.SNEAKING_SPEED)));
                state.add("movement", snapshot(new Vec2(player.xxa, player.zza)));
                state.addProperty("jumping", player.isJumping());
                state.addProperty("x_bob", Oracle.bits(player.xBob));
                state.addProperty("y_bob", Oracle.bits(player.yBob));
                state.addProperty("x_bob_o", Oracle.bits(player.xBobO));
                state.addProperty("y_bob_o", Oracle.bits(player.yBobO));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LocalPlayer sneaking speed attribute binding fixture failed", error);
        }
    }

    static JsonObject clientWorldPlayerKeyboardInputResult(JsonObject request) {
        try {
            var output = System.out;
            var errorOutput = System.err;
            try {
                SharedConstants.setVersion(TEST_VERSION);
                Bootstrap.bootStrap();
            } finally {
                System.setOut(output);
                System.setErr(errorOutput);
            }

            int playerId = request.get("player_id").getAsInt();
            AttributeMap attributes = new AttributeMap(Player.createAttributes().build());
            SneakingSpeedInputFixture player = SneakingSpeedInputFixture.create(attributes);
            var states = new JsonArray();
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                boolean accepted = operation.get("player_id").getAsInt() == playerId;
                if (accepted) {
                    for (JsonElement snapshotEntry : operation.getAsJsonArray("attributes")) {
                        applyPlayerAttributeSnapshot(
                            attributes, snapshotEntry.getAsJsonObject());
                    }
                }
                Input keys = new Input(
                    operation.get("up").getAsBoolean(),
                    operation.get("down").getAsBoolean(),
                    operation.get("left").getAsBoolean(),
                    operation.get("right").getAsBoolean(),
                    operation.get("jump").getAsBoolean(),
                    operation.get("shift").getAsBoolean(),
                    operation.get("sprint").getAsBoolean());
                KeyboardInput keyboard = new KeyboardInput(optionsFor(keys));
                keyboard.tick();
                player.configureKeyboardInput(
                    keyboard,
                    0.0F,
                    0.0F,
                    false,
                    0.0F,
                    0.0F,
                    0.0F,
                    0.0F,
                    0.0F,
                    0.0F,
                    operation.get("controlled_camera").getAsBoolean(),
                    operation.get("using_item").getAsBoolean(),
                    operation.get("passenger").getAsBoolean(),
                    operation.get("moving_slowly").getAsBoolean());
                if (operation.get("using_item").getAsBoolean()) {
                    player.setItemUseSpeed(Oracle.f32(operation, "item_use_speed_multiplier"));
                }
                player.applyInput();

                var state = new JsonObject();
                state.addProperty("label", operation.get("label").getAsString());
                state.addProperty("accepted", accepted);
                state.add("key_presses", snapshot(keyboard.keyPresses));
                state.add("move_vector", snapshot(keyboard.getMoveVector()));
                state.add("movement", snapshot(new Vec2(player.xxa, player.zza)));
                state.addProperty("jumping", player.isJumping());
                state.addProperty("x_bob", Oracle.bits(player.xBob));
                state.addProperty("y_bob", Oracle.bits(player.yBob));
                state.addProperty("x_bob_o", Oracle.bits(player.xBobO));
                state.addProperty("y_bob_o", Oracle.bits(player.yBobO));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LocalPlayer keyboard input binding fixture failed", error);
        }
    }

    static JsonObject clientWorldPlayerTickResult(JsonObject request) {
        int playerId = request.get("player_id").getAsInt();
        JsonObject initialPosition = request.getAsJsonObject("initial_position");
        JsonObject initialRotation = request.getAsJsonObject("initial_rotation");
        TickLocalPlayerFixture player = TickLocalPlayerFixture.create(
            vec3(initialPosition),
            Oracle.f32(initialRotation, "y"),
            Oracle.f32(initialRotation, "x"),
            Oracle.f32(request, "initial_head"));
        for (JsonElement entry : request.getAsJsonArray("initial_effects")) {
            player.replaceEffect(entry.getAsJsonObject());
        }

        var states = new JsonArray();
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown player tick action");
            }
            player.setFixtureState(
                vec3(operation.getAsJsonObject("position")),
                Oracle.f32(operation.getAsJsonObject("rotation"), "y"),
                Oracle.f32(operation.getAsJsonObject("rotation"), "x"));
            JsonElement headTarget = operation.get("head_target");
            if (headTarget != null && !headTarget.isJsonNull()) {
                player.lerpHeadTo(
                    Float.intBitsToFloat(Integer.parseUnsignedInt(headTarget.getAsString(), 16)),
                    3);
            }
            for (JsonElement entry : operation.getAsJsonArray("effects")) {
                player.replaceEffect(entry.getAsJsonObject());
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                player.tickThroughClientLevel();
            }
            states.add(tickLocalPlayerState(operation.get("label").getAsString(), player));
        }
        var result = new JsonObject();
        result.addProperty("player_id", playerId);
        result.add("states", states);
        return result;
    }

    static JsonObject tickLocalPlayerState(
        String label, TickLocalPlayerFixture player
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("tick_count", player.tickCount);
        state.add("position", snapshot(player.position()));
        state.add("previous_position", snapshot(player.oldPosition()));
        state.addProperty("rotation_y", Oracle.bits(player.getYRot()));
        state.addProperty("rotation_x", Oracle.bits(player.getXRot()));
        state.addProperty("previous_rotation_y", Oracle.bits(player.yRotO));
        state.addProperty("previous_rotation_x", Oracle.bits(player.xRotO));
        state.addProperty("head_rotation", Oracle.bits(player.yHeadRot));
        state.addProperty("previous_head_rotation", Oracle.bits(player.yHeadRotO));
        var effects = new JsonArray();
        for (TickEffect effect : player.effects) {
            var effectState = new JsonObject();
            effectState.addProperty("effect_id", effect.effectId);
            effectState.addProperty("amplifier", effect.value.getAmplifier());
            effectState.addProperty("duration_ticks", effect.value.getDuration());
            effectState.addProperty("flags", effect.flags);
            effects.add(effectState);
        }
        state.add("effects", effects);
        return state;
    }

    static JsonObject clientWorldPlayerSprintTriggerResult(JsonObject request) {
        try {
            var output = System.out;
            var errorOutput = System.err;
            try {
                SharedConstants.setVersion(TEST_VERSION);
                Bootstrap.bootStrap();
            } finally {
                System.setOut(output);
                System.setErr(errorOutput);
            }

            var canStartSprinting = Reflect.method(LocalPlayer.class, "canStartSprinting");
            Unsafe unsafe = unsafeInstance();
            int sprintWindow = request.get("sprint_window").getAsInt();
            JsonObject conditions = request.getAsJsonObject("conditions");
            var sequences = new JsonArray();
            for (JsonElement sequenceEntry : request.getAsJsonArray("sequences")) {
                JsonObject sequence = sequenceEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureSprintTrigger(Vec2.ZERO, Input.EMPTY, conditions);
                var states = new JsonArray();
                for (JsonElement operationEntry : sequence.getAsJsonArray("operations")) {
                    JsonObject operation = operationEntry.getAsJsonObject();
                    player.setSprintTriggerInput(
                        inputState(operation.getAsJsonObject("key_presses")),
                        vec2f(operation.getAsJsonObject("move_vector")));
                    for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                        player.tickSprintTrigger(canStartSprinting, sprintWindow);
                    }
                    states.add(sprintTriggerState(operation.get("label").getAsString(), player));
                }
                var sequenceState = new JsonObject();
                sequenceState.addProperty("label", sequence.get("label").getAsString());
                sequenceState.add("states", states);
                sequences.add(sequenceState);
            }
            var result = new JsonObject();
            result.addProperty("player_id", request.get("player_id").getAsInt());
            result.add("sequences", sequences);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LocalPlayer sprint trigger fixture failed", error);
        }
    }

    static Input inputState(JsonObject value) {
        return new Input(
            value.get("forward").getAsBoolean(),
            value.get("backward").getAsBoolean(),
            value.get("left").getAsBoolean(),
            value.get("right").getAsBoolean(),
            value.get("jump").getAsBoolean(),
            value.get("shift").getAsBoolean(),
            value.get("sprint").getAsBoolean());
    }

    static Vec2 vec2f(JsonObject value) {
        return new Vec2(Oracle.f32(value, "x"), Oracle.f32(value, "y"));
    }

    static JsonObject sprintTriggerState(
        String label, LocalPlayerInputFixture player
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("sprinting", player.isSprinting());
        state.addProperty("sprint_trigger_time", player.sprintTriggerTime());
        state.add("key_presses", snapshot(player.input.keyPresses));
        state.add("move_vector", snapshot(player.input.getMoveVector()));
        return state;
    }
}
