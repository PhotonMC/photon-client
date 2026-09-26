import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * Test glue only: reads the original block registry, never rebuilds it.
 *
 * <p>Bootstrapping the registry replaces {@code System.out} with a stream that
 * feeds the game's logger, which would swallow every record written after it.
 * The original stream is put back as soon as the bootstrap returns, so the
 * other families in the same run are unaffected.
 */
public final class BlockOracle {
    private static boolean bootstrapped;

    private static synchronized void bootstrap() {
        if (bootstrapped) {
            return;
        }
        var out = System.out;
        var err = System.err;
        try {
            net.minecraft.SharedConstants.setVersion(Oracle.TEST_VERSION);
            net.minecraft.server.Bootstrap.bootStrap();
        } finally {
            System.setOut(out);
            System.setErr(err);
        }
        bootstrapped = true;
        // The grass and foliage tints read a colour map the game loads from
        // its own assets. Headless, that map is all zeroes, so those tints
        // would come back black. The maps are in the pinned jar, which is on
        // the classpath, so they are loaded from there.
        loadColourMap("grass", net.minecraft.world.level.GrassColor::init);
        loadColourMap("foliage", net.minecraft.world.level.FoliageColor::init);
        loadColourMap("dry_foliage", net.minecraft.world.level.DryFoliageColor::init);
    }

    private static void loadColourMap(String name, java.util.function.Consumer<int[]> install) {
        String path = "/assets/minecraft/textures/colormap/" + name + ".png";
        try (var stream = BlockOracle.class.getResourceAsStream(path)) {
            if (stream == null) {
                throw new IllegalStateException("Missing colour map: " + path);
            }
            try (var image = com.mojang.blaze3d.platform.NativeImage.read(stream)) {
                install.accept(image.makePixelArray());
            }
        } catch (java.io.IOException failure) {
            throw new IllegalStateException("Could not read colour map: " + path, failure);
        }
    }

    public static JsonObject run(JsonObject request) {
        bootstrap();
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "state_definitions" -> rows.add(stateDefinitions(element.getAsJsonObject()));
                case "block_tints" -> rows.add(blockTints(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown block operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Every block, its properties and the state it starts from. */
    private static JsonObject stateDefinitions(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        entry.addProperty("block_count", registry.size());
        var blocks = new JsonArray();
        int totalStates = 0;
        for (var block : registry) {
            var definition = block.getStateDefinition();
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            described.addProperty("id", registry.getId(block));
            var properties = new JsonArray();
            for (var property : definition.getProperties()) {
                var describedProperty = new JsonObject();
                describedProperty.addProperty("name", property.getName());
                describedProperty.addProperty("kind", switch (property) {
                    case net.minecraft.world.level.block.state.properties.BooleanProperty p -> "boolean";
                    case net.minecraft.world.level.block.state.properties.IntegerProperty p -> "integer";
                    case net.minecraft.world.level.block.state.properties.EnumProperty p -> "enum";
                    default -> "other";
                });
                var values = new JsonArray();
                for (var value : property.getPossibleValues()) {
                    values.add(name(property, value));
                }
                describedProperty.add("values", values);
                properties.add(describedProperty);
            }
            described.add("properties", properties);
            var defaults = new JsonArray();
            for (var property : definition.getProperties()) {
                defaults.add(name(property, block.defaultBlockState().getValue(property)));
            }
            described.add("default", defaults);
            described.addProperty("state_count", definition.getPossibleStates().size());
            // The protocol numbers block states across the whole registry, so
            // the first one a block owns is what a chunk section decodes to.
            described.addProperty("first_state_id", net.minecraft.world.level.block.Block
                .getId(definition.getPossibleStates().getFirst()));
            described.addProperty("default_state_id", net.minecraft.world.level.block.Block
                .getId(block.defaultBlockState()));
            totalStates += definition.getPossibleStates().size();
            blocks.add(described);
        }
        entry.addProperty("state_count", totalStates);
        entry.add("blocks", blocks);
        return entry;
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static String name(
        net.minecraft.world.level.block.state.properties.Property property, Comparable value) {
        return property.getName(value);
    }

    /** What each block's tint layers resolve to outside a world. */
    private static JsonObject blockTints(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var colors = net.minecraft.client.color.block.BlockColors.createDefault();
        var tinted = new JsonArray();
        for (var block : registry) {
            var state = block.defaultBlockState();
            var sources = colors.getTintSources(state);
            if (sources.isEmpty()) {
                continue;
            }
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            var layers = new JsonArray();
            for (var source : sources) {
                layers.add(String.format(java.util.Locale.ROOT, "%08x", source.color(state)));
            }
            described.add("layers", layers);
            tinted.add(described);
        }
        entry.add("tinted", tinted);
        return entry;
    }
}
