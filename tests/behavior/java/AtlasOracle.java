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
        if (operation.equals("sprite_ids")) {
            return spriteIds(request);
        }
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

    /** The ids an atlas definition lists in the game's own assets, as its sprite sources run. */
    private static JsonObject spriteIds(JsonObject request) {
        var rows = new JsonArray();
        try {
            // The registry of source types the definition files name is filled once, by the game.
            net.minecraft.client.renderer.texture.atlas.SpriteSources.bootstrap();
            var jar = java.nio.file.Path.of(Stitcher.class.getProtectionDomain().getCodeSource().getLocation().toURI());
            var jarSystem = java.nio.file.FileSystems.newFileSystem(jar);
            var pack = new net.minecraft.server.packs.VanillaPackResourcesBuilder()
                .pushAssetPath(net.minecraft.server.packs.PackType.CLIENT_RESOURCES, jarSystem.getPath("/assets"))
                .exposeNamespace("minecraft")
                .build(new net.minecraft.server.packs.PackLocationInfo("vanilla",
                    net.minecraft.network.chat.Component.literal("vanilla"),
                    net.minecraft.server.packs.repository.PackSource.BUILT_IN, java.util.Optional.empty()));
            var packsRoot = java.nio.file.Files.createTempDirectory("atlas-packs");
            int packNumber = 0;
            for (var element : request.get("rows").getAsJsonArray()) {
                var row = element.getAsJsonObject();
                // A row may put a pack of its own over the game's assets: files by id and hex bytes.
                var packs = new java.util.ArrayList<net.minecraft.server.packs.PackResources>(List.of(pack));
                if (row.has("pack")) {
                    var packRoot = java.nio.file.Files.createDirectories(packsRoot.resolve(Integer.toString(packNumber++)));
                    for (var fileElement : row.get("pack").getAsJsonArray()) {
                        var file = fileElement.getAsJsonObject();
                        var location = Identifier.parse(file.get("id").getAsString());
                        var path = packRoot.resolve("assets").resolve(location.getNamespace()).resolve(location.getPath());
                        java.nio.file.Files.createDirectories(path.getParent());
                        java.nio.file.Files.write(path, java.util.HexFormat.of().parseHex(file.get("hex").getAsString()));
                    }
                    packs.add(new net.minecraft.server.packs.PathPackResources(
                        new net.minecraft.server.packs.PackLocationInfo("test",
                            net.minecraft.network.chat.Component.literal("test"),
                            net.minecraft.server.packs.repository.PackSource.DEFAULT, java.util.Optional.empty()),
                        packRoot));
                }
                var manager = new net.minecraft.server.packs.resources.MultiPackResourceManager(
                    net.minecraft.server.packs.PackType.CLIENT_RESOURCES, packs);
                var list = net.minecraft.client.renderer.texture.atlas.SpriteSourceList.load(manager,
                    Identifier.parse(row.get("atlas").getAsString()));
                var field = list.getClass().getDeclaredField("sources");
                field.setAccessible(true);
                var found = new java.util.TreeSet<String>();
                var output = new net.minecraft.client.renderer.texture.atlas.SpriteSource.Output() {
                    @Override
                    public void add(Identifier id, net.minecraft.client.renderer.texture.atlas.SpriteSource.DiscardableLoader sprite) {
                        found.add(id.toString());
                    }

                    @Override
                    public void removeAll(java.util.function.Predicate<Identifier> predicate) {
                        found.removeIf(name -> predicate.test(Identifier.parse(name)));
                    }
                };
                for (var source : (List<net.minecraft.client.renderer.texture.atlas.SpriteSource>) field.get(list)) {
                    source.run(manager, output);
                }
                var entry = new JsonObject();
                entry.addProperty("label", row.get("label").getAsString());
                var ids = new JsonArray();
                found.forEach(ids::add);
                entry.add("ids", ids);
                rows.add(entry);
            }
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
