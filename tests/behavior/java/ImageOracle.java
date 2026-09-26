import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.NativeImage;
import java.io.ByteArrayInputStream;
import java.util.HexFormat;

/** Test glue only: decodes through the original NativeImage, never its own PNG reader. */
public final class ImageOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (!operation.equals("png_decode")) {
            throw new IllegalArgumentException("Unknown image operation: " + operation);
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            rows.add(decode(element.getAsJsonObject()));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static JsonObject decode(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        byte[] bytes = HexFormat.of().parseHex(row.get("png").getAsString());
        try (var stream = new ByteArrayInputStream(bytes);
             NativeImage image = NativeImage.read(stream)) {
            int width = image.getWidth();
            int height = image.getHeight();
            entry.addProperty("outcome", "decoded");
            entry.addProperty("width", width);
            entry.addProperty("height", height);
            // getPixel hands back ARGB; emit plain RGBA so the comparison is
            // about pixels rather than either side's in-memory packing.
            var rgba = new byte[width * height * 4];
            for (int y = 0; y < height; y++) {
                for (int x = 0; x < width; x++) {
                    int packed = image.getPixel(x, y);
                    int offset = (y * width + x) * 4;
                    rgba[offset] = (byte)(packed >> 16 & 0xFF);
                    rgba[offset + 1] = (byte)(packed >> 8 & 0xFF);
                    rgba[offset + 2] = (byte)(packed & 0xFF);
                    rgba[offset + 3] = (byte)(packed >> 24 & 0xFF);
                }
            }
            entry.addProperty("rgba", HexFormat.of().formatHex(rgba));
        } catch (Exception error) {
            entry.addProperty("outcome", "decode_failed");
        }
        return entry;
    }
}
