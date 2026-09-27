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
                case "face_occlusion" -> rows.add(faceOcclusion(element.getAsJsonObject()));
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

    /**
     * Everything the face culling decision reads, as three tables.
     *
     * A face's occlusion shape is one of: the whole block, nothing, or one of
     * a small set of partial shapes. Two states rarely differ in all six, so
     * the six are grouped into tuples and a block names the tuple each of its
     * states uses. The last step of the decision asks whether one shape
     * sticks out past another, which is a pure function of the two shapes and
     * so is a table as well.
     */
    private static JsonObject faceOcclusion(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        // Shape kind zero is nothing and one is the whole block; the partial
        // shapes are numbered after them in the order they turn up.
        //
        // Keyed by the boxes the shape holds, not by the object. The reference
        // hands back a fresh instance per state and `VoxelShape` does not
        // compare by value, so keying by identity counts the same shape
        // thousands of times.
        var shapeIds = new java.util.LinkedHashMap<String, Integer>();
        var shapeByKey = new java.util.LinkedHashMap<
            String, net.minecraft.world.phys.shapes.VoxelShape>();
        var tupleIds = new java.util.LinkedHashMap<String, Integer>();
        var tuples = new JsonArray();
        var blocks = new JsonArray();
        for (var block : registry) {
            var perState = new JsonArray();
            for (var state : block.getStateDefinition().getPossibleStates()) {
                var key = new StringBuilder();
                var kinds = new int[6];
                int slot = 0;
                for (var direction : net.minecraft.core.Direction.values()) {
                    var shape = state.getFaceOcclusionShape(direction);
                    int kind;
                    if (shape == net.minecraft.world.phys.shapes.Shapes.empty()) {
                        kind = 0;
                    } else if (shape == net.minecraft.world.phys.shapes.Shapes.block()) {
                        kind = 1;
                    } else {
                        String shapeKey = shape.toAabbs().toString();
                        kind = shapeIds.computeIfAbsent(shapeKey, k -> shapeIds.size() + 2);
                        shapeByKey.putIfAbsent(shapeKey, shape);
                    }
                    kinds[slot++] = kind;
                    key.append(kind).append(',');
                }
                Integer tuple = tupleIds.get(key.toString());
                if (tuple == null) {
                    tuple = tupleIds.size();
                    tupleIds.put(key.toString(), tuple);
                    var described = new JsonArray();
                    for (int kind : kinds) {
                        described.add(kind);
                    }
                    tuples.add(described);
                }
                perState.add(tuple);
            }
            // A block whose states all share one tuple carries just that one.
            var distinct = new java.util.LinkedHashSet<com.google.gson.JsonElement>();
            perState.forEach(distinct::add);
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            var listed = new JsonArray();
            (distinct.size() == 1 ? distinct : perState).forEach(listed::add);
            described.add("tuples", listed);
            blocks.add(described);
        }
        entry.add("tuples", tuples);
        entry.add("blocks", blocks);

        // Whether the first shape has any part the second does not cover.
        var ordered = new java.util.ArrayList<net.minecraft.world.phys.shapes.VoxelShape>();
        ordered.add(net.minecraft.world.phys.shapes.Shapes.empty());
        ordered.add(net.minecraft.world.phys.shapes.Shapes.block());
        ordered.addAll(shapeByKey.values());
        var joins = new JsonArray();
        for (var shape : ordered) {
            var line = new StringBuilder();
            for (var occluder : ordered) {
                line.append(net.minecraft.world.phys.shapes.Shapes.joinIsNotEmpty(
                    shape, occluder, net.minecraft.world.phys.shapes.BooleanOp.ONLY_FIRST)
                    ? '1' : '0');
            }
            joins.add(line.toString());
        }
        entry.addProperty("shape_count", ordered.size());
        entry.add("joins", joins);

        // Which ordered pairs of blocks hide their shared face, and whether
        // that ever depends on the states rather than on the blocks alone.
        //
        // The default states are scanned first because almost every pair says
        // no; only the pairs that say yes are then checked across every state,
        // which keeps this from being a scan of every state against every
        // other state.
        var candidates = new java.util.ArrayList<
            net.minecraft.world.level.block.Block[]>();
        for (var block : registry) {
            var state = block.defaultBlockState();
            for (var other : registry) {
                var neighbour = other.defaultBlockState();
                for (var direction : net.minecraft.core.Direction.values()) {
                    if (state.skipRendering(neighbour, direction)) {
                        candidates.add(new net.minecraft.world.level.block.Block[] {block, other});
                        break;
                    }
                }
            }
        }
        // A pair is reported with what it actually depends on, because the
        // first measurement of this assumed "the blocks alone" and that turned
        // out to be wrong for some of them.
        var skips = new JsonArray();
        for (var pair : candidates) {
            boolean all = true;
            var directions = new java.util.TreeSet<String>();
            for (var state : pair[0].getStateDefinition().getPossibleStates()) {
                for (var neighbour : pair[1].getStateDefinition().getPossibleStates()) {
                    for (var direction : net.minecraft.core.Direction.values()) {
                        if (state.skipRendering(neighbour, direction)) {
                            directions.add(direction.getName());
                        } else {
                            all = false;
                        }
                    }
                }
            }
            var described = new JsonObject();
            described.addProperty("block", registry.getKey(pair[0]).toString());
            described.addProperty("neighbour", registry.getKey(pair[1]).toString());
            described.addProperty("always", all);
            var listed = new JsonArray();
            directions.forEach(listed::add);
            described.add("directions", listed);
            skips.add(described);
        }
        entry.add("skip_pairs", skips);
        boolean stateDependent = false;
        for (var element : skips) {
            if (!element.getAsJsonObject().get("always").getAsBoolean()) {
                stateDependent = true;
            }
        }
        entry.addProperty("skip_is_state_dependent", stateDependent);
        return entry;
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
