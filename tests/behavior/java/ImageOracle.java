import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.NativeImage;
import java.io.ByteArrayInputStream;
import java.util.HexFormat;

/** Test glue only: decodes through the original NativeImage, never its own PNG reader. */
public final class ImageOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "png_decode" -> rows.add(decode(element.getAsJsonObject()));
                case "srgb_tables" -> rows.add(srgbTables(element.getAsJsonObject()));
                case "mip_levels" -> rows.add(mipLevels(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown image operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The two fixed colour tables the mean blend reads. */
    private static JsonObject srgbTables(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var forward = new StringBuilder();
        for (int i = 0; i < 256; i++) {
            forward.append(String.format(java.util.Locale.ROOT, "%04x",
                Math.round(net.minecraft.util.ARGB.srgbToLinearChannel(i) * 1023.0F)));
        }
        entry.addProperty("srgb_to_linear", forward.toString());
        var back = new StringBuilder();
        for (int i = 0; i < 1024; i++) {
            back.append(String.format(java.util.Locale.ROOT, "%02x",
                net.minecraft.util.ARGB.linearToSrgbChannel(i / 1023.0F)));
        }
        entry.addProperty("linear_to_srgb", back.toString());
        return entry;
    }

    /** Mip levels generated from one image. */
    private static JsonObject mipLevels(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        byte[] bytes = HexFormat.of().parseHex(row.get("png").getAsString());
        int levels = row.get("levels").getAsInt();
        try (var stream = new ByteArrayInputStream(bytes);
             NativeImage image = NativeImage.read(stream)) {
            var mips = net.minecraft.client.renderer.texture.MipmapGenerator.generateMipLevels(
                net.minecraft.resources.Identifier.parse(row.get("name").getAsString()),
                new NativeImage[] {image},
                levels,
                net.minecraft.client.renderer.texture.MipmapStrategy.MEAN,
                0.0F,
                com.mojang.blaze3d.platform.Transparency.NONE);
            entry.addProperty("outcome", "generated");
            var described = new JsonArray();
            for (NativeImage mip : mips) {
                var level = new JsonObject();
                level.addProperty("width", mip.getWidth());
                level.addProperty("height", mip.getHeight());
                var rgba = new byte[mip.getWidth() * mip.getHeight() * 4];
                for (int y = 0; y < mip.getHeight(); y++) {
                    for (int x = 0; x < mip.getWidth(); x++) {
                        int packed = mip.getPixel(x, y);
                        int offset = (y * mip.getWidth() + x) * 4;
                        rgba[offset] = (byte)(packed >> 16 & 0xFF);
                        rgba[offset + 1] = (byte)(packed >> 8 & 0xFF);
                        rgba[offset + 2] = (byte)(packed & 0xFF);
                        rgba[offset + 3] = (byte)(packed >> 24 & 0xFF);
                    }
                }
                level.addProperty("rgba", HexFormat.of().formatHex(rgba));
                described.add(level);
            }
            entry.add("levels", described);
        } catch (Exception error) {
            entry.addProperty("outcome", "failed");
        }
        return entry;
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
