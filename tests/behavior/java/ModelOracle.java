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
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "texture_slots" -> rows.add(resolve(element.getAsJsonObject()));
                case "face_geometry" -> rows.add(geometry(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown model operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Corner layout and default texture mapping for one cuboid face. */
    private static JsonObject geometry(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var from = vector(row.get("from").getAsJsonArray());
        var to = vector(row.get("to").getAsJsonArray());
        var facing = net.minecraft.core.Direction.byName(row.get("facing").getAsString());
        if (facing == null) {
            entry.addProperty("outcome", "unknown_facing");
            return entry;
        }
        entry.addProperty("outcome", "baked");

        try {
            var method = net.minecraft.client.resources.model.cuboid.FaceBakery.class
                .getDeclaredMethod("defaultFaceUV", org.joml.Vector3fc.class,
                                   org.joml.Vector3fc.class, net.minecraft.core.Direction.class);
            method.setAccessible(true);
            var uvs = method.invoke(null, from, to, facing);
            var uvClass = uvs.getClass();
            entry.addProperty("min_u", bits(uvClass, uvs, "minU"));
            entry.addProperty("min_v", bits(uvClass, uvs, "minV"));
            entry.addProperty("max_u", bits(uvClass, uvs, "maxU"));
            entry.addProperty("max_v", bits(uvClass, uvs, "maxV"));
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("default face UV unavailable", error);
        }

        var info = net.minecraft.client.renderer.FaceInfo.fromFacing(facing);
        var corners = new JsonArray();
        for (int index = 0; index < 4; index++) {
            // The bakery divides by 16 right after selecting the corner; the
            // division is kept here so the comparison covers it.
            var vertex = info.getVertexInfo(index).select(from, to).div(16.0F);
            var corner = new JsonObject();
            corner.addProperty("x", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.x())));
            corner.addProperty("y", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.y())));
            corner.addProperty("z", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.z())));
            corners.add(corner);
        }
        entry.add("corners", corners);
        return entry;
    }

    private static String bits(Class<?> owner, Object instance, String accessor) throws ReflectiveOperationException {
        var method = owner.getMethod(accessor);
        method.setAccessible(true);
        float value = (Float)method.invoke(instance);
        return String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value));
    }

    private static org.joml.Vector3f vector(JsonArray array) {
        return new org.joml.Vector3f(
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(0).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(1).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(2).getAsString(), 16))
        );
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
