import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;
import net.minecraft.client.resources.model.ModelDebugName;
import net.minecraft.client.resources.model.sprite.Material;
import net.minecraft.client.resources.model.sprite.TextureSlots;

/** Test glue only: drives the original texture slot resolver. */
public final class ModelOracle {
    private static final List<String> SLOT_PROBES = List.of(
        "all", "top", "bottom", "side", "north", "particle", "missing", "#all", "#top"
    );

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (!operation.equals("texture_slots")) {
            throw new IllegalArgumentException("Unknown model operation: " + operation);
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            rows.add(resolve(element.getAsJsonObject()));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static JsonObject resolve(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var resolver = new TextureSlots.Resolver();
        // The model chain is listed child first; the resolver is fed in that
        // same order, which is how ResolvedModel walks up to the root.
        for (var levelElement : row.get("chain").getAsJsonArray()) {
            var textures = new JsonObject();
            for (var slotElement : levelElement.getAsJsonArray()) {
                var slot = slotElement.getAsJsonObject();
                String name = slot.get("slot").getAsString();
                if (slot.has("raw")) {
                    textures.add(name, new JsonPrimitive(slot.get("raw").getAsString()));
                } else {
                    var material = new JsonObject();
                    material.addProperty("sprite", slot.get("sprite").getAsString());
                    material.addProperty("force_translucent", slot.get("force_translucent").getAsBoolean());
                    textures.add(name, material);
                }
            }
            resolver.addLast(TextureSlots.parseTextureMap(textures));
        }

        TextureSlots slots;
        try {
            slots = resolver.resolve((ModelDebugName)() -> "parity");
        } catch (RuntimeException error) {
            entry.addProperty("outcome", "resolve_failed");
            return entry;
        }

        entry.addProperty("outcome", "resolved");
        var lookups = new JsonArray();
        var ordered = new TreeMap<String, Material>();
        for (String probe : SLOT_PROBES) {
            Material material = slots.getMaterial(probe);
            if (material != null) {
                ordered.put(probe, material);
            }
        }
        for (var found : ordered.entrySet()) {
            var lookup = new JsonObject();
            lookup.addProperty("probe", found.getKey());
            lookup.addProperty("sprite", found.getValue().sprite().toString());
            lookup.addProperty("force_translucent", found.getValue().forceTranslucent());
            lookups.add(lookup);
        }
        entry.add("lookups", lookups);
        return entry;
    }
}
