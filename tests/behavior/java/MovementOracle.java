import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * Test glue only: runs the original client player in a world the test builds.
 *
 * <p>The player here is the reference's own {@code LocalPlayer}, in the
 * reference's own {@code ClientLevel}, with the reference's own keyboard
 * input. Registries and tags come from the vanilla data pack through the same
 * loading path the server uses, and chunks arrive through the client chunk
 * cache's packet entry point. What this file supplies is only what a headless
 * run has no other way to get: the game singleton, an options object, a
 * connection that records what it is asked to send rather than sending it,
 * and a sound manager that plays nothing.
 */
public final class MovementOracle extends MovementParticleOracle {
    public static JsonObject run(JsonObject request) throws Exception {
        bootstrap();
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "local_player_ticks" -> rows.add(localPlayerTicks(element.getAsJsonObject()));
                case "item_entity_ticks" -> rows.add(itemEntityTicks(element.getAsJsonObject()));
                case "projectile_ticks" -> rows.add(projectileTicks(element.getAsJsonObject()));
                case "living_entity_ticks" -> rows.add(livingEntityTicks(element.getAsJsonObject()));
                case "entity_shadows" -> rows.add(entityShadows(element.getAsJsonObject()));
                case "entity_pick_table" -> rows.add(entityPickTable(element.getAsJsonObject()));
                case "light_changes" -> rows.add(lightChanges(element.getAsJsonObject()));
                case "sky_flash" -> rows.add(skyFlash(element.getAsJsonObject()));
                case "elytra_state" -> rows.add(elytraState(element.getAsJsonObject()));
                case "container_clicks" -> rows.add(containerClicks(element.getAsJsonObject()));
                case "particle_types" -> rows.add(particleTypes(element.getAsJsonObject()));
                case "particle_ticks" -> rows.add(particleTicks(element.getAsJsonObject()));
                case "particle_engine" -> rows.add(particleEngine(element.getAsJsonObject()));
                case "particle_quads" -> rows.add(particleQuads(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException(
                    "Unknown movement operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
