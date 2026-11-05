import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.client.DeltaTracker;

public final class TimerOracle {
    public static JsonObject run(JsonObject request) {
        if (!request.get("operation").getAsString().equals("trace")) {
            throw new IllegalArgumentException("Unknown timer operation");
        }
        // A mutable supplied tick duration is an input, not a replacement timer algorithm.
        float[] target = {Oracle.f32(request, "target_mspt")};
        var timer = new DeltaTracker.Timer(Oracle.f32(request, "tps"),
            Long.parseLong(request.get("start_ms").getAsString()), ignored -> target[0]);
        var events = new JsonArray();
        events.add(snapshot(timer));
        for (var entry : request.getAsJsonArray("steps")) {
            JsonObject step = entry.getAsJsonObject();
            Integer ticks = null;
            switch (step.get("action").getAsString()) {
                case "game" -> ticks = timer.advanceGameTime(Long.parseLong(step.get("ms").getAsString()));
                case "real" -> timer.advanceRealTime(Long.parseLong(step.get("ms").getAsString()));
                case "pause" -> timer.updatePauseState(step.get("value").getAsBoolean());
                case "freeze" -> timer.updateFrozenState(step.get("value").getAsBoolean());
                case "target" -> target[0] = Oracle.f32(step, "value");
                default -> throw new IllegalArgumentException("Unknown timer action");
            }
            JsonObject state = snapshot(timer);
            if (ticks != null) {
                state.addProperty("ticks", ticks);
            }
            events.add(state);
        }
        var result = new JsonObject();
        result.add("events", events);
        return result;
    }

    private static JsonObject snapshot(DeltaTracker.Timer timer) {
        var state = new JsonObject();
        state.addProperty("game_delta", Oracle.bits(timer.getGameTimeDeltaTicks()));
        state.addProperty("partial", Oracle.bits(timer.getGameTimeDeltaPartialTick(false)));
        state.addProperty("partial_ignore_frozen", Oracle.bits(timer.getGameTimeDeltaPartialTick(true)));
        state.addProperty("real_delta", Oracle.bits(timer.getRealtimeDeltaTicks()));
        return state;
    }
}
