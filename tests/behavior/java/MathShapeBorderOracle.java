import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.SharedConstants;
import net.minecraft.world.level.border.WorldBorder;

class MathShapeBorderOracle extends MathSectionOracle {
    static JsonObject shapeworldBorderCollisionShapeCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
            border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
            border.setSize(f64(input, "size"));
            states.add(snapshot(border.getCollisionShape()));
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeworldBorderCollisionShapeMovingCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
            border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
            border.lerpSizeBetween(
                f64(input, "from"),
                f64(input, "to"),
                input.get("duration").getAsLong(),
                0L
            );
            var partialTicks = input.getAsJsonArray("partial_ticks");
            int tickCount = input.get("tick_count").getAsInt();
            var track = new JsonArray();
            for (int tick = 0; tick <= tickCount; tick++) {
                var samples = new JsonArray();
                for (var partialEntry : partialTicks) {
                    float partialTick = Float.intBitsToFloat(
                        Integer.parseUnsignedInt(partialEntry.getAsString(), 16));
                    var sample = new JsonObject();
                    sample.addProperty("partial_tick", Oracle.bits(partialTick));
                    sample.addProperty("min_x", Oracle.bits(border.getMinX(partialTick)));
                    sample.addProperty("max_x", Oracle.bits(border.getMaxX(partialTick)));
                    sample.addProperty("min_z", Oracle.bits(border.getMinZ(partialTick)));
                    sample.addProperty("max_z", Oracle.bits(border.getMaxZ(partialTick)));
                    samples.add(sample);
                }
                var state = new JsonObject();
                state.addProperty("size", Oracle.bits(border.getSize()));
                state.addProperty("lerp_time", border.getLerpTime());
                state.addProperty("lerp_target", Oracle.bits(border.getLerpTarget()));
                state.addProperty("status", border.getStatus().name());
                state.add("samples", samples);
                state.add("collision_shape", snapshot(border.getCollisionShape()));
                track.add(state);
                if (tick < tickCount) {
                    border.tick();
                }
            }
            states.add(track);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeclientWorldBorderStateCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var tracks = new JsonArray();
        for (var trackEntry : request.getAsJsonArray("inputs")) {
            WorldBorder border = new WorldBorder();
            long gameTime = 0L;
            var states = new JsonArray();
            for (var eventEntry : trackEntry.getAsJsonObject().getAsJsonArray("events")) {
                JsonObject event = eventEntry.getAsJsonObject();
                String eventKind = event.get("op").getAsString();
                switch (eventKind) {
                    case "initialize" -> {
                        border.setCenter(f64(event, "center_x"), f64(event, "center_z"));
                        long lerpTime = event.get("lerp_time").getAsLong();
                        if (lerpTime > 0L) {
                            border.lerpSizeBetween(
                                f64(event, "old_size"),
                                f64(event, "new_size"),
                                lerpTime,
                                gameTime
                            );
                        } else {
                            border.setSize(f64(event, "new_size"));
                        }
                        border.setAbsoluteMaxSize(event.get("absolute_max_size").getAsInt());
                        border.setWarningBlocks(event.get("warning_blocks").getAsInt());
                        border.setWarningTime(event.get("warning_time").getAsInt());
                    }
                    case "center" -> border.setCenter(
                        f64(event, "center_x"), f64(event, "center_z"));
                    case "lerp" -> border.lerpSizeBetween(
                        f64(event, "old_size"),
                        f64(event, "new_size"),
                        event.get("lerp_time").getAsLong(),
                        gameTime
                    );
                    case "size" -> border.setSize(f64(event, "size"));
                    case "warning_blocks" -> border.setWarningBlocks(event.get("value").getAsInt());
                    case "warning_time" -> border.setWarningTime(event.get("value").getAsInt());
                    case "tick" -> {
                        border.tick();
                        gameTime++;
                    }
                    default -> throw new IllegalArgumentException("Unknown world border event");
                }
                var samples = new JsonArray();
                for (float partialTick : new float[] {0.0F, 0.5F, 1.0F}) {
                    var sample = new JsonObject();
                    sample.addProperty("partial_tick", Oracle.bits(partialTick));
                    sample.addProperty("min_x", Oracle.bits(border.getMinX(partialTick)));
                    sample.addProperty("max_x", Oracle.bits(border.getMaxX(partialTick)));
                    sample.addProperty("min_z", Oracle.bits(border.getMinZ(partialTick)));
                    sample.addProperty("max_z", Oracle.bits(border.getMaxZ(partialTick)));
                    samples.add(sample);
                }
                var state = new JsonObject();
                state.addProperty("event", eventKind);
                state.addProperty("game_time", gameTime);
                state.addProperty("center_x", Oracle.bits(border.getCenterX()));
                state.addProperty("center_z", Oracle.bits(border.getCenterZ()));
                state.addProperty("absolute_max_size", border.getAbsoluteMaxSize());
                state.addProperty("warning_blocks", border.getWarningBlocks());
                state.addProperty("warning_time", border.getWarningTime());
                state.addProperty("size", Oracle.bits(border.getSize()));
                state.addProperty("lerp_time", border.getLerpTime());
                state.addProperty("lerp_target", Oracle.bits(border.getLerpTarget()));
                state.addProperty("status", border.getStatus().name());
                state.add("samples", samples);
                state.add("collision_shape", snapshot(border.getCollisionShape()));
                states.add(state);
            }
            tracks.add(states);
        }
        result.add("tracks", tracks);
        return result;
    }
}
