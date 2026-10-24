import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.AttributeMap;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.phys.Vec3;

class MathPlayerAttributeOracle extends MathShapeOracle {
    static JsonObject playerAttributeQuery(AttributeMap attributes, JsonObject query) {
        int attributeId = query.get("attribute_id").getAsInt();
        AttributeInstance instance = attributes.getInstance(attributeHolder(attributeId));
        var state = new JsonObject();
        state.addProperty("label", query.get("label").getAsString());
        state.addProperty("attribute_id", attributeId);
        state.addProperty("found", instance != null);
        if (instance != null) {
            state.addProperty("base_value", Oracle.bits(instance.getBaseValue()));
            var modifiers = new ArrayList<>(instance.getModifiers());
            modifiers.sort((left, right) -> left.id().toString().compareTo(right.id().toString()));
            var modifierStates = new JsonArray();
            for (AttributeModifier modifier : modifiers) {
                var modifierState = new JsonObject();
                modifierState.addProperty("id", modifier.id().toString());
                modifierState.addProperty("amount", Oracle.bits(modifier.amount()));
                modifierState.addProperty("operation", modifier.operation().id());
                modifierStates.add(modifierState);
            }
            state.add("modifiers", modifierStates);
            state.addProperty("effective_value", Oracle.bits(instance.getValue()));
        }
        return state;
    }

    static JsonObject clientWorldPlayerAttributesResult(JsonObject request) {
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
            var events = new JsonArray();
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                String action = operation.get("action").getAsString();
                if (!action.equals("apply")) {
                    throw new IllegalArgumentException("Unknown player attribute action: " + action);
                }
                boolean accepted = operation.get("player_id").getAsInt() == playerId;
                if (accepted) {
                    for (JsonElement snapshotEntry : operation.getAsJsonArray("attributes")) {
                        applyPlayerAttributeSnapshot(
                            attributes, snapshotEntry.getAsJsonObject());
                    }
                }
                var queries = new JsonArray();
                for (JsonElement queryEntry : operation.getAsJsonArray("queries")) {
                    queries.add(playerAttributeQuery(
                        attributes, queryEntry.getAsJsonObject()));
                }
                var event = new JsonObject();
                event.addProperty("action", action);
                event.addProperty("label", operation.get("label").getAsString());
                event.addProperty("accepted", accepted);
                event.add("queries", queries);
                events.add(event);
            }
            var result = new JsonObject();
            result.add("events", events);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Player attribute fixture failed", error);
        }
    }

    static JsonObject clientWorldPlayerMovementSpeedResult(JsonObject request) {
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
            AttributeSpeedFixture player = AttributeSpeedFixture.create(attributes);
            var frictionMethod = Reflect.method(LivingEntity.class, 
                "getFrictionInfluencedSpeed", float.class);
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
                player.onGroundState = operation.get("on_ground").getAsBoolean();
                float movementSpeed = player.getSpeed();
                float frictionInfluencedSpeed = (float)frictionMethod.invoke(
                    player, Oracle.f32(operation, "block_friction"));
                var state = new JsonObject();
                state.addProperty("label", operation.get("label").getAsString());
                state.addProperty("accepted", accepted);
                state.addProperty("movement_speed", Oracle.bits(movementSpeed));
                state.addProperty(
                    "friction_influenced_speed", Oracle.bits(frictionInfluencedSpeed));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "Player.getSpeed attribute binding fixture failed", error);
        }
    }

    static JsonObject clientWorldPlayerAttributeWaterResult(JsonObject request) {
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
            AttributeSpeedFixture player = AttributeSpeedFixture.create(attributes);
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
                player.configureTravelInWater(
                    operation.get("sprinting").getAsBoolean(),
                    Oracle.f32(operation, "water_slow_down"),
                    Oracle.f32(operation, "water_efficiency"),
                    operation.get("on_ground").getAsBoolean(),
                    operation.get("dolphins_grace").getAsBoolean(),
                    operation.get("horizontal_collision").getAsBoolean(),
                    operation.get("on_climbable").getAsBoolean(),
                    operation.get("free_after_jump").getAsBoolean(),
                    Oracle.f32(operation, "y_rot"),
                    vec3(operation.getAsJsonObject("delta")));
                var travelMethod = Reflect.method(LivingEntity.class, 
                    "travelInWater", Vec3.class, double.class, boolean.class, double.class);
                travelMethod.invoke(
                    player,
                    vec3(operation.getAsJsonObject("input")),
                    f64(operation, "base_gravity"),
                    operation.get("is_falling").getAsBoolean(),
                    f64(operation, "old_y"));
                var state = new JsonObject();
                state.addProperty("label", operation.get("label").getAsString());
                state.addProperty("accepted", accepted);
                state.addProperty("movement_speed", Oracle.bits(player.getSpeed()));
                state.add("movement", snapshot(player.getDeltaMovement()));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "Player attribute water travel fixture failed", error);
        }
    }

    static JsonObject clientWorldPlayerSprintAttributeResult(JsonObject request) {
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
            SprintAttributeFixture player = SprintAttributeFixture.create(attributes);
            var states = new JsonArray();
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                boolean accepted = operation.get("player_id").getAsInt() == playerId;
                if (accepted) {
                    for (JsonElement snapshotEntry : operation.getAsJsonArray("attributes")) {
                        applyPlayerAttributeSnapshot(
                            attributes, snapshotEntry.getAsJsonObject());
                    }
                    player.setSprinting(operation.get("sprinting").getAsBoolean());
                }
                states.add(playerSprintAttributeState(
                    operation.get("label").getAsString(), accepted, player));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException(
                "LivingEntity.setSprinting attribute binding fixture failed", error);
        }
    }

    static JsonObject playerSprintAttributeState(
        String label, boolean accepted, SprintAttributeFixture player
    ) {
        AttributeInstance speed = player.getAttribute(Attributes.MOVEMENT_SPEED);
        if (speed == null) {
            throw new IllegalStateException("Player movement speed attribute is missing");
        }
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("sprinting", player.isSprinting());
        state.addProperty("base_value", Oracle.bits(speed.getBaseValue()));
        var modifiers = new ArrayList<>(speed.getModifiers());
        modifiers.sort((left, right) -> left.id().toString().compareTo(right.id().toString()));
        var modifierStates = new JsonArray();
        for (AttributeModifier modifier : modifiers) {
            var modifierState = new JsonObject();
            modifierState.addProperty("id", modifier.id().toString());
            modifierState.addProperty("amount", Oracle.bits(modifier.amount()));
            modifierState.addProperty("operation", modifier.operation().id());
            modifierStates.add(modifierState);
        }
        state.add("modifiers", modifierStates);
        state.addProperty("movement_speed", Oracle.bits(player.getSpeed()));
        return state;
    }
}
