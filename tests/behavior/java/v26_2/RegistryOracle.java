import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.registries.BuiltInRegistries;

/**
 * The built-in registries as the game numbers them: for each entry its name and the id the
 * network carries (the registry's own `getId`), in id order. Block states are listed with the
 * property values that name them.
 */
public final class RegistryOracle {
    private RegistryOracle() {
    }

    public static JsonObject run(JsonObject request) {
        if (!request.get("operation").getAsString().equals("entries")) {
            throw new IllegalArgumentException("Unknown registry operation");
        }
        Game26_2.vanillaRegistries();
        var rows = new JsonArray();
        for (var row : request.getAsJsonArray("rows")) {
            var label = row.getAsJsonObject().get("label").getAsString();
            var entry = new JsonObject();
            entry.addProperty("label", label);
            entry.add("entries", entries(label));
            rows.add(entry);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static JsonArray entries(String registry) {
        return switch (registry) {
            case "item" -> named(BuiltInRegistries.ITEM);
            case "block" -> named(BuiltInRegistries.BLOCK);
            case "entity_type" -> named(BuiltInRegistries.ENTITY_TYPE);
            case "sound_event" -> named(BuiltInRegistries.SOUND_EVENT);
            case "particle_type" -> named(BuiltInRegistries.PARTICLE_TYPE);
            case "menu" -> named(BuiltInRegistries.MENU);
            case "block_entity_type" -> named(BuiltInRegistries.BLOCK_ENTITY_TYPE);
            case "block_state" -> blockStates();
            default -> throw new IllegalArgumentException("Unknown registry");
        };
    }

    private static <T> JsonArray named(net.minecraft.core.Registry<T> registry) {
        var entries = new JsonArray();
        for (var value : registry) {
            var entry = new JsonObject();
            entry.addProperty("name", registry.getKey(value).toString());
            entry.addProperty("id", registry.getId(value));
            entries.add(entry);
        }
        return entries;
    }

    private static JsonArray blockStates() {
        var entries = new JsonArray();
        for (var state : net.minecraft.world.level.block.Block.BLOCK_STATE_REGISTRY) {
            var properties = new JsonArray();
            for (var property : state.getProperties()) {
                properties.add(property.getName() + "=" + state.getValue(property).toString().toLowerCase(java.util.Locale.ROOT));
            }
            var entry = new JsonObject();
            entry.addProperty("name", BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
            entry.addProperty("id", net.minecraft.world.level.block.Block.BLOCK_STATE_REGISTRY.getId(state));
            entry.add("properties", properties);
            entries.add(entry);
        }
        return entries;
    }
}
