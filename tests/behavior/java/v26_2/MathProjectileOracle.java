import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;

class MathProjectileOracle extends MathPlayerFixtures {
    static JsonObject clientWorldEntitySnowballTickResult(JsonObject request) {
        var snowballs = new ArrayList<TickSnowballFixture>();
        for (JsonElement entry : request.getAsJsonArray("entities")) {
            JsonObject input = entry.getAsJsonObject();
            snowballs.add(TickSnowballFixture.create(
                input.get("entity_id").getAsInt(),
                vec3(input.getAsJsonObject("position")),
                vec3(input.getAsJsonObject("movement")),
                Oracle.f32(input.getAsJsonObject("rotation"), "y"),
                Oracle.f32(input.getAsJsonObject("rotation"), "x"),
                input.get("no_gravity").getAsBoolean()));
        }

        var states = new JsonArray();
        states.add(tickSnowballState("initial", snowballs));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown snowball tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (TickSnowballFixture snowball : snowballs) {
                    snowball.tickThroughClientLevel();
                }
            }
            states.add(tickSnowballState(operation.get("label").getAsString(), snowballs));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickSnowballState(
        String label, List<TickSnowballFixture> snowballs
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (TickSnowballFixture snowball : snowballs) {
            var entity = new JsonObject();
            entity.addProperty("entity_id", snowball.entityId());
            entity.addProperty("tick_count", snowball.tickCount);
            entity.add("position", snapshotClip(snowball.position()));
            entity.add("previous_position", snapshotClip(snowball.previousPosition()));
            entity.add("motion", snapshotClip(snowball.getDeltaMovement()));
            entity.addProperty("rotation_y", Oracle.bits(snowball.getYRot()));
            entity.addProperty("rotation_x", Oracle.bits(snowball.getXRot()));
            entity.addProperty("previous_rotation_y", Oracle.bits(snowball.yRotO));
            entity.addProperty("previous_rotation_x", Oracle.bits(snowball.xRotO));
            entity.addProperty("no_gravity", snowball.noGravity);
            entities.add(entity);
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject clientWorldEntityArrowTickResult(JsonObject request) {
        var arrows = new ArrayList<TickArrowFixture>();
        for (JsonElement entry : request.getAsJsonArray("entities")) {
            JsonObject input = entry.getAsJsonObject();
            arrows.add(TickArrowFixture.create(
                input.get("entity_id").getAsInt(),
                vec3(input.getAsJsonObject("position")),
                vec3(input.getAsJsonObject("movement")),
                Oracle.f32(input.getAsJsonObject("rotation"), "y"),
                Oracle.f32(input.getAsJsonObject("rotation"), "x"),
                input.get("no_gravity").getAsBoolean()));
        }

        var states = new JsonArray();
        states.add(tickArrowState("initial", arrows));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown arrow tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (TickArrowFixture arrow : arrows) {
                    arrow.tickThroughClientLevel();
                }
            }
            states.add(tickArrowState(operation.get("label").getAsString(), arrows));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickArrowState(
        String label, List<TickArrowFixture> arrows
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (TickArrowFixture arrow : arrows) {
            var entity = new JsonObject();
            entity.addProperty("entity_id", arrow.entityId());
            entity.addProperty("tick_count", arrow.tickCount);
            entity.add("position", snapshotClip(arrow.position()));
            entity.add("previous_position", snapshotClip(arrow.previousPosition()));
            entity.add("motion", snapshotClip(arrow.getDeltaMovement()));
            entity.addProperty("rotation_y", Oracle.bits(arrow.getYRot()));
            entity.addProperty("rotation_x", Oracle.bits(arrow.getXRot()));
            entity.addProperty("previous_rotation_y", Oracle.bits(arrow.yRotO));
            entity.addProperty("previous_rotation_x", Oracle.bits(arrow.xRotO));
            entity.addProperty("no_gravity", arrow.noGravity);
            entities.add(entity);
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject clientWorldEntityExperienceOrbTickResult(JsonObject request) {
        var orbs = new ArrayList<TickExperienceOrbFixture>();
        for (JsonElement entry : request.getAsJsonArray("entities")) {
            JsonObject input = entry.getAsJsonObject();
            orbs.add(TickExperienceOrbFixture.create(
                input.get("entity_id").getAsInt(),
                vec3(input.getAsJsonObject("position")),
                vec3(input.getAsJsonObject("movement")),
                Oracle.f32(input.getAsJsonObject("rotation"), "y"),
                Oracle.f32(input.getAsJsonObject("rotation"), "x"),
                input.get("no_gravity").getAsBoolean()));
        }

        var states = new JsonArray();
        states.add(tickExperienceOrbState("initial", orbs));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown experience orb tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (TickExperienceOrbFixture orb : orbs) {
                    orb.tickThroughClientLevel();
                }
            }
            states.add(tickExperienceOrbState(operation.get("label").getAsString(), orbs));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickExperienceOrbState(
        String label, List<TickExperienceOrbFixture> orbs
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (TickExperienceOrbFixture orb : orbs) {
            var entity = new JsonObject();
            entity.addProperty("entity_id", orb.entityId());
            entity.addProperty("tick_count", orb.tickCount);
            entity.add("position", snapshotClip(orb.position()));
            entity.add("previous_position", snapshotClip(orb.previousPosition()));
            entity.add("motion", snapshotClip(orb.getDeltaMovement()));
            entity.addProperty("rotation_y", Oracle.bits(orb.getYRot()));
            entity.addProperty("rotation_x", Oracle.bits(orb.getXRot()));
            entity.addProperty("previous_rotation_y", Oracle.bits(orb.yRotO));
            entity.addProperty("previous_rotation_x", Oracle.bits(orb.xRotO));
            entity.addProperty("no_gravity", orb.noGravity);
            entities.add(entity);
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject clientWorldEntityPrimedTntTickResult(JsonObject request) {
        var tnts = new ArrayList<TickPrimedTntFixture>();
        for (JsonElement entry : request.getAsJsonArray("entities")) {
            JsonObject input = entry.getAsJsonObject();
            tnts.add(TickPrimedTntFixture.create(
                input.get("entity_id").getAsInt(),
                vec3(input.getAsJsonObject("position")),
                vec3(input.getAsJsonObject("movement")),
                Oracle.f32(input.getAsJsonObject("rotation"), "y"),
                Oracle.f32(input.getAsJsonObject("rotation"), "x"),
                input.get("no_gravity").getAsBoolean()));
        }

        var states = new JsonArray();
        states.add(tickPrimedTntState("initial", tnts));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown primed TNT tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (TickPrimedTntFixture tnt : tnts) {
                    tnt.tickThroughClientLevel();
                }
            }
            states.add(tickPrimedTntState(operation.get("label").getAsString(), tnts));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickPrimedTntState(
        String label, List<TickPrimedTntFixture> tnts
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (TickPrimedTntFixture tnt : tnts) {
            var entity = new JsonObject();
            entity.addProperty("entity_id", tnt.entityId());
            entity.addProperty("tick_count", tnt.tickCount);
            entity.add("position", snapshotClip(tnt.position()));
            entity.add("previous_position", snapshotClip(tnt.previousPosition()));
            entity.add("motion", snapshotClip(tnt.getDeltaMovement()));
            entity.addProperty("rotation_y", Oracle.bits(tnt.getYRot()));
            entity.addProperty("rotation_x", Oracle.bits(tnt.getXRot()));
            entity.addProperty("previous_rotation_y", Oracle.bits(tnt.yRotO));
            entity.addProperty("previous_rotation_x", Oracle.bits(tnt.xRotO));
            entity.addProperty("no_gravity", tnt.noGravity);
            entities.add(entity);
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject clientWorldEntityFallingBlockTickResult(JsonObject request) {
        var fallingBlocks = new ArrayList<TickFallingBlockFixture>();
        for (JsonElement entry : request.getAsJsonArray("entities")) {
            JsonObject input = entry.getAsJsonObject();
            fallingBlocks.add(TickFallingBlockFixture.create(
                input.get("entity_id").getAsInt(),
                vec3(input.getAsJsonObject("position")),
                vec3(input.getAsJsonObject("movement")),
                Oracle.f32(input.getAsJsonObject("rotation"), "y"),
                Oracle.f32(input.getAsJsonObject("rotation"), "x"),
                input.get("no_gravity").getAsBoolean()));
        }

        var states = new JsonArray();
        states.add(tickFallingBlockState("initial", fallingBlocks));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown falling block tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (TickFallingBlockFixture fallingBlock : fallingBlocks) {
                    fallingBlock.tickThroughClientLevel();
                }
            }
            states.add(tickFallingBlockState(operation.get("label").getAsString(), fallingBlocks));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickFallingBlockState(
        String label, List<TickFallingBlockFixture> fallingBlocks
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (TickFallingBlockFixture fallingBlock : fallingBlocks) {
            var entity = new JsonObject();
            entity.addProperty("entity_id", fallingBlock.entityId());
            entity.addProperty("tick_count", fallingBlock.tickCount);
            entity.add("position", snapshotClip(fallingBlock.position()));
            entity.add("previous_position", snapshotClip(fallingBlock.previousPosition()));
            entity.add("motion", snapshotClip(fallingBlock.getDeltaMovement()));
            entity.addProperty("rotation_y", Oracle.bits(fallingBlock.getYRot()));
            entity.addProperty("rotation_x", Oracle.bits(fallingBlock.getXRot()));
            entity.addProperty("previous_rotation_y", Oracle.bits(fallingBlock.yRotO));
            entity.addProperty("previous_rotation_x", Oracle.bits(fallingBlock.xRotO));
            entity.addProperty("no_gravity", fallingBlock.noGravity);
            entities.add(entity);
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject clientWorldEntityFireworkTickResult(JsonObject request) {
        var fireworks = new ArrayList<TickFireworkRocketFixture>();
        for (JsonElement entry : request.getAsJsonArray("entities")) {
            JsonObject input = entry.getAsJsonObject();
            fireworks.add(TickFireworkRocketFixture.create(
                input.get("entity_id").getAsInt(),
                vec3(input.getAsJsonObject("position")),
                vec3(input.getAsJsonObject("movement")),
                Oracle.f32(input.getAsJsonObject("rotation"), "y"),
                Oracle.f32(input.getAsJsonObject("rotation"), "x"),
                input.get("no_gravity").getAsBoolean()));
        }

        var states = new JsonArray();
        states.add(tickFireworkState("initial", fireworks));
        for (JsonElement entry : request.getAsJsonArray("operations")) {
            JsonObject operation = entry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown firework tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (TickFireworkRocketFixture firework : fireworks) {
                    firework.tickThroughClientLevel();
                }
            }
            states.add(tickFireworkState(operation.get("label").getAsString(), fireworks));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject tickFireworkState(
        String label, List<TickFireworkRocketFixture> fireworks
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (TickFireworkRocketFixture firework : fireworks) {
            var entity = new JsonObject();
            entity.addProperty("entity_id", firework.entityId());
            entity.addProperty("tick_count", firework.tickCount);
            entity.add("position", snapshotClip(firework.position()));
            entity.add("previous_position", snapshotClip(firework.previousPosition()));
            entity.add("motion", snapshotClip(firework.getDeltaMovement()));
            entity.addProperty("rotation_y", Oracle.bits(firework.getYRot()));
            entity.addProperty("rotation_x", Oracle.bits(firework.getXRot()));
            entity.addProperty("previous_rotation_y", Oracle.bits(firework.yRotO));
            entity.addProperty("previous_rotation_x", Oracle.bits(firework.xRotO));
            entity.addProperty("no_gravity", firework.noGravity);
            entities.add(entity);
        }
        state.add("entities", entities);
        return state;
    }
}
