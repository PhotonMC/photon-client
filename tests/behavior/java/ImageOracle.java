import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.NativeImage;
import java.io.ByteArrayInputStream;
import java.util.HexFormat;

/** Test glue only: decodes through the original NativeImage, never its own PNG reader. */
public final class ImageOracle {
    public static JsonObject run(JsonObject request) throws Exception {
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "png_decode" -> rows.add(decode(element.getAsJsonObject()));
                case "srgb_tables" -> rows.add(srgbTables(element.getAsJsonObject()));
                case "mip_levels" -> rows.add(mipLevels(element.getAsJsonObject()));
                case "compute_transparency" -> rows.add(transparency(element.getAsJsonObject()));
                case "texture_metadata" -> rows.add(textureMetadata(element.getAsJsonObject()));
                case "sprite_uv" -> rows.add(spriteUv(element.getAsJsonObject()));
                case "colour_map" -> rows.add(colourMap(request, element.getAsJsonObject()));
                case "argb_multiply" -> rows.add(argbMultiply(element.getAsJsonObject()));
                case "tint_constants" -> rows.add(tintConstants(element.getAsJsonObject()));
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

    /** What the atlas measures from a sprite's pixels to resolve AUTO. */
    private static JsonObject transparency(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var bytes = HexFormat.of().parseHex(row.get("png").getAsString());
        try (var image = NativeImage.read(new ByteArrayInputStream(bytes))) {
            var measured = image.computeTransparency();
            entry.addProperty("outcome", "measured");
            entry.addProperty("transparent", measured.hasTransparent());
            entry.addProperty("translucent", measured.hasTranslucent());
            entry.addProperty("opaque", measured.isOpaque());
        } catch (Exception failure) {
            entry.addProperty("outcome", "failed");
        }
        return entry;
    }

    /** The texture section of a sprite's .mcmeta, which carries the strategy. */
    private static JsonObject textureMetadata(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        try {
            var json = com.google.gson.JsonParser.parseString(row.get("json").getAsString());
            var parsed = net.minecraft.client.resources.metadata.texture.TextureMetadataSection.CODEC
                .parse(com.mojang.serialization.JsonOps.INSTANCE, json)
                .getOrThrow();
            entry.addProperty("outcome", "parsed");
            entry.addProperty("blur", parsed.blur());
            entry.addProperty("clamp", parsed.clamp());
            entry.addProperty("strategy", parsed.mipmapStrategy().getSerializedName());
            entry.addProperty("bias", String.format(java.util.Locale.ROOT, "%08x",
                Float.floatToRawIntBits(parsed.alphaCutoffBias())));
        } catch (Exception failure) {
            entry.addProperty("outcome", "rejected");
        }
        return entry;
    }

    /**
     * Where a stitched sprite's texels land in the atlas.
     *
     * The constructor is protected and the class needs no GPU, so this opens
     * it rather than reimplementing the four divisions it performs.
     */
    private static JsonObject spriteUv(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        int width = row.get("width").getAsInt();
        int height = row.get("height").getAsInt();
        try (var image = new NativeImage(width, height, false)) {
            var contents = new net.minecraft.client.renderer.texture.SpriteContents(
                net.minecraft.resources.Identifier.parse("minecraft:block/parity"),
                new net.minecraft.client.resources.metadata.animation.FrameSize(width, height),
                image);
            var constructor = net.minecraft.client.renderer.texture.TextureAtlasSprite.class
                .getDeclaredConstructor(net.minecraft.resources.Identifier.class,
                    net.minecraft.client.renderer.texture.SpriteContents.class,
                    int.class, int.class, int.class, int.class, int.class);
            constructor.setAccessible(true);
            var sprite = constructor.newInstance(
                net.minecraft.resources.Identifier.parse("minecraft:textures/atlas/parity.png"),
                contents,
                row.get("atlas_width").getAsInt(),
                row.get("atlas_height").getAsInt(),
                row.get("x").getAsInt(),
                row.get("y").getAsInt(),
                row.get("padding").getAsInt());
            entry.addProperty("outcome", "placed");
            entry.addProperty("x", sprite.getX());
            entry.addProperty("y", sprite.getY());
            entry.addProperty("u0", Oracle.bits(sprite.getU0()));
            entry.addProperty("u1", Oracle.bits(sprite.getU1()));
            entry.addProperty("v0", Oracle.bits(sprite.getV0()));
            entry.addProperty("v1", Oracle.bits(sprite.getV1()));
            var interpolated = new JsonArray();
            for (var offsetElement : row.get("offsets").getAsJsonArray()) {
                float offset = Float.intBitsToFloat(
                    Integer.parseUnsignedInt(offsetElement.getAsString(), 16));
                var pair = new JsonObject();
                pair.addProperty("u", Oracle.bits(sprite.getU(offset)));
                pair.addProperty("v", Oracle.bits(sprite.getV(offset)));
                interpolated.add(pair);
            }
            entry.add("interpolated", interpolated);
        }
        return entry;
    }

    /** The colour map the three biome tints index, loaded once per request. */
    private static int[] colourMapPixels(JsonObject request) throws Exception {
        var bytes = HexFormat.of().parseHex(request.get("colour_map").getAsString());
        try (var image = NativeImage.read(new ByteArrayInputStream(bytes))) {
            return image.makePixelArray();
        }
    }

    /** One biome tint looked up by temperature and rainfall. */
    private static JsonObject colourMap(JsonObject request, JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        double temperature = Double.longBitsToDouble(
            Long.parseUnsignedLong(row.get("temperature").getAsString(), 16));
        double rain = Double.longBitsToDouble(
            Long.parseUnsignedLong(row.get("rain").getAsString(), 16));
        var pixels = colourMapPixels(request);
        net.minecraft.world.level.GrassColor.init(pixels);
        net.minecraft.world.level.FoliageColor.init(pixels);
        net.minecraft.world.level.DryFoliageColor.init(pixels);
        try {
            int colour = switch (row.get("kind").getAsString()) {
                case "grass" -> net.minecraft.world.level.GrassColor.get(temperature, rain);
                case "foliage" -> net.minecraft.world.level.FoliageColor.get(temperature, rain);
                case "dry_foliage" ->
                    net.minecraft.world.level.DryFoliageColor.get(temperature, rain);
                default -> throw new IllegalArgumentException("Unknown colour map kind");
            };
            entry.addProperty("outcome", "resolved");
            entry.addProperty("colour", String.format(java.util.Locale.ROOT, "%08x", colour));
        } catch (IndexOutOfBoundsException failure) {
            // The lookup guards the upper bound but not the lower one, so a
            // temperature above one indexes before the start of the map.
            entry.addProperty("outcome", "out_of_bounds");
        }
        return entry;
    }

    /** How a tint is folded into a quad's colour. */
    private static JsonObject argbMultiply(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        int lhs = Integer.parseUnsignedInt(row.get("lhs").getAsString(), 16);
        int rhs = Integer.parseUnsignedInt(row.get("rhs").getAsString(), 16);
        entry.addProperty("outcome", "multiplied");
        entry.addProperty("colour", String.format(java.util.Locale.ROOT, "%08x",
            net.minecraft.util.ARGB.multiply(lhs, rhs)));
        return entry;
    }

    /** The fixed tints, which are what a block drawn outside a world uses. */
    private static JsonObject tintConstants(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var values = new JsonObject();
        values.addProperty("foliage_default",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.world.level.FoliageColor.FOLIAGE_DEFAULT));
        values.addProperty("foliage_evergreen",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.world.level.FoliageColor.FOLIAGE_EVERGREEN));
        values.addProperty("foliage_birch",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.world.level.FoliageColor.FOLIAGE_BIRCH));
        values.addProperty("foliage_mangrove",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.world.level.FoliageColor.FOLIAGE_MANGROVE));
        values.addProperty("dry_foliage_default",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.world.level.DryFoliageColor.FOLIAGE_DRY_DEFAULT));
        values.addProperty("lily_pad_in_hand",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.client.color.block.BlockColors.LILY_PAD_DEFAULT));
        values.addProperty("lily_pad_in_world",
            String.format(java.util.Locale.ROOT, "%08x",
                net.minecraft.client.color.block.BlockColors.LILY_PAD_IN_WORLD));
        entry.add("values", values);
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
            var strategy = net.minecraft.client.renderer.texture.MipmapStrategy
                .valueOf(row.get("strategy").getAsString());
            float bias = Float.intBitsToFloat(
                Integer.parseUnsignedInt(row.get("bias").getAsString(), 16));
            var mips = net.minecraft.client.renderer.texture.MipmapGenerator.generateMipLevels(
                net.minecraft.resources.Identifier.parse(row.get("name").getAsString()),
                new NativeImage[] {image},
                levels,
                strategy,
                bias,
                com.mojang.blaze3d.platform.Transparency.of(
                    row.get("transparent").getAsBoolean(), row.get("translucent").getAsBoolean()));
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
