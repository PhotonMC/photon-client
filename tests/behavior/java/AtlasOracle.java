import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.renderer.texture.Stitcher;
import net.minecraft.resources.Identifier;

/** Test glue only: drives the original atlas stitcher, never reimplements it. */
public final class AtlasOracle {
    private record Sprite(int width, int height, Identifier name) implements Stitcher.Entry {
    }

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (!operation.equals("stitch")) {
            throw new IllegalArgumentException("Unknown atlas operation: " + operation);
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            rows.add(stitch(element.getAsJsonObject()));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static JsonObject stitch(JsonObject row) {
        int maxWidth = row.get("max_width").getAsInt();
        int maxHeight = row.get("max_height").getAsInt();
        int mipLevel = row.get("mip_level").getAsInt();
        int anisotropyBit = row.get("anisotropy_bit").getAsInt();

        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var stitcher = new Stitcher<Sprite>(maxWidth, maxHeight, mipLevel, anisotropyBit);
        List<Sprite> sprites = new ArrayList<>();
        for (var element : row.get("sprites").getAsJsonArray()) {
            var sprite = element.getAsJsonObject();
            sprites.add(new Sprite(
                sprite.get("width").getAsInt(),
                sprite.get("height").getAsInt(),
                Identifier.parse(sprite.get("name").getAsString())
            ));
        }

        try {
            for (Sprite sprite : sprites) {
                stitcher.registerSprite(sprite);
            }
            stitcher.stitch();
        } catch (RuntimeException error) {
            // The exception class is diagnostic, not behaviour: comparing it
            // would ask the Rust side to reproduce a Java type name.
            entry.addProperty("outcome", "stitch_failed");
            return entry;
        }

        entry.addProperty("outcome", "stitched");
        entry.addProperty("width", stitcher.getWidth());
        entry.addProperty("height", stitcher.getHeight());
        var placements = new JsonArray();
        stitcher.gatherSprites((sprite, x, y, padding) -> {
            var placement = new JsonObject();
            placement.addProperty("name", sprite.name().toString());
            placement.addProperty("x", x);
            placement.addProperty("y", y);
            placement.addProperty("padding", padding);
            placements.add(placement);
        });
        entry.add("placements", placements);
        return entry;
    }
}
