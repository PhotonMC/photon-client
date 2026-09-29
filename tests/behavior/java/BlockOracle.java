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
        // Tags matter here: movement reads the climbable, fence and wall tags,
        // and unbound tags would read as empty rather than as vanilla.
        Oracle.vanillaRegistries();
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
                case "collision_shapes" -> rows.add(collisionShapes(element.getAsJsonObject()));
                case "outline_shapes" -> rows.add(outlineShapes(element.getAsJsonObject()));
                case "movement_properties" -> rows.add(movementProperties(element.getAsJsonObject()));
                case "light_properties" -> rows.add(lightProperties(element.getAsJsonObject()));
                case "offset_properties" -> rows.add(offsetProperties(element.getAsJsonObject()));
                case "fluid_overlay_blocks" -> rows.add(fluidOverlayBlocks(element.getAsJsonObject()));
                case "motion_blocking" -> rows.add(motionBlocking(element.getAsJsonObject()));
                case "fluid_flow_properties" -> rows.add(fluidFlowProperties(element.getAsJsonObject()));
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

    /**
     * Every state's collision shape, as the world's collision walk asks for it
     * with no entity in the question.
     *
     * Shapes are keyed by their boxes rather than by object, because the
     * reference hands back fresh instances. A block is also reported with
     * whether its shape moves with its position (a random offset) and whether
     * the reference calls it dynamic, since for those the table is only the
     * starting point.
     */
    private static JsonObject collisionShapes(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var getter = net.minecraft.world.level.EmptyBlockGetter.INSTANCE;
        var context = net.minecraft.world.phys.shapes.CollisionContext.empty();
        var elsewhere = new net.minecraft.core.BlockPos[] {
            new net.minecraft.core.BlockPos(13, 70, -7),
            new net.minecraft.core.BlockPos(-200, 5, 999),
            new net.minecraft.core.BlockPos(1, 1, 1),
            new net.minecraft.core.BlockPos(-3, 64, 12),
        };
        var shapeIds = new java.util.LinkedHashMap<String, Integer>();
        var shapes = new JsonArray();
        var blocks = new JsonArray();
        for (var block : registry) {
            var perState = new JsonArray();
            boolean moves = false;
            for (var state : block.getStateDefinition().getPossibleStates()) {
                var shape = state.getCollisionShape(
                    getter, net.minecraft.core.BlockPos.ZERO, context);
                var boxes = shape.toAabbs();
                String key = boxes.toString();
                Integer id = shapeIds.get(key);
                if (id == null) {
                    id = shapeIds.size();
                    shapeIds.put(key, id);
                    var described = new JsonArray();
                    for (var box : boxes) {
                        var bounds = new JsonArray();
                        bounds.add(Oracle.bits(box.minX));
                        bounds.add(Oracle.bits(box.minY));
                        bounds.add(Oracle.bits(box.minZ));
                        bounds.add(Oracle.bits(box.maxX));
                        bounds.add(Oracle.bits(box.maxY));
                        bounds.add(Oracle.bits(box.maxZ));
                        described.add(bounds);
                    }
                    shapes.add(described);
                }
                perState.add(id);
                for (var pos : elsewhere) {
                    if (!state.getCollisionShape(getter, pos, context).toAabbs().equals(boxes)) {
                        moves = true;
                    }
                }
            }
            var distinct = new java.util.LinkedHashSet<com.google.gson.JsonElement>();
            perState.forEach(distinct::add);
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            var listed = new JsonArray();
            (distinct.size() == 1 ? distinct : perState).forEach(listed::add);
            described.add("shapes", listed);
            described.addProperty("position_dependent", moves);
            described.addProperty("dynamic", block.hasDynamicShape());
            blocks.add(described);
        }
        entry.add("shapes", shapes);
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * Every state's outline shape, the one picking a block aims at, with no
     * entity in the question.
     *
     * A block nudged off the grid by a random offset reports its shape with
     * the nudge at the origin taken back out, and whether the shape at other
     * positions is exactly that shape moved by the nudge there.
     */
    private static JsonObject outlineShapes(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var getter = net.minecraft.world.level.EmptyBlockGetter.INSTANCE;
        var context = net.minecraft.world.phys.shapes.CollisionContext.empty();
        var origin = net.minecraft.core.BlockPos.ZERO;
        var elsewhere = new net.minecraft.core.BlockPos[] {
            new net.minecraft.core.BlockPos(13, 70, -7),
            new net.minecraft.core.BlockPos(-200, 5, 999),
            new net.minecraft.core.BlockPos(1, 1, 1),
            new net.minecraft.core.BlockPos(-3, 64, 12),
        };
        var shapeIds = new java.util.LinkedHashMap<String, Integer>();
        var shapes = new JsonArray();
        var blocks = new JsonArray();
        for (var block : registry) {
            var perState = new JsonArray();
            boolean moves = false;
            boolean offsetExplained = true;
            for (var state : block.getStateDefinition().getPossibleStates()) {
                var nudge = state.getOffset(origin);
                var base = state.getShape(getter, origin, context);
                boolean shifts = false;
                for (var pos : elsewhere) {
                    if (!state.getShape(getter, pos, context).toAabbs().equals(base.toAabbs())) {
                        shifts = true;
                    }
                }
                if (shifts && !nudge.equals(net.minecraft.world.phys.Vec3.ZERO)) {
                    // Taking the nudge back out does not round back to the
                    // shape the block was built from; that shape's corners
                    // sit on the sixteenth grid, so they are snapped there.
                    var snapped = net.minecraft.world.phys.shapes.Shapes.empty();
                    for (var box : base.move(nudge.reverse()).toAabbs()) {
                        snapped = net.minecraft.world.phys.shapes.Shapes.or(snapped, net.minecraft.world.phys.shapes.Shapes.box(
                            Math.round(box.minX * 16.0) / 16.0, Math.round(box.minY * 16.0) / 16.0,
                            Math.round(box.minZ * 16.0) / 16.0, Math.round(box.maxX * 16.0) / 16.0,
                            Math.round(box.maxY * 16.0) / 16.0, Math.round(box.maxZ * 16.0) / 16.0));
                    }
                    base = snapped;
                }
                var boxes = base.toAabbs();
                String key = boxes.toString();
                Integer id = shapeIds.get(key);
                if (id == null) {
                    id = shapeIds.size();
                    shapeIds.put(key, id);
                    var described = new JsonArray();
                    for (var box : boxes) {
                        var bounds = new JsonArray();
                        bounds.add(Oracle.bits(box.minX));
                        bounds.add(Oracle.bits(box.minY));
                        bounds.add(Oracle.bits(box.minZ));
                        bounds.add(Oracle.bits(box.maxX));
                        bounds.add(Oracle.bits(box.maxY));
                        bounds.add(Oracle.bits(box.maxZ));
                        described.add(bounds);
                    }
                    shapes.add(described);
                }
                perState.add(id);
                for (var pos : elsewhere) {
                    var there = state.getShape(getter, pos, context).toAabbs();
                    if (!there.equals(boxes)) {
                        moves = true;
                    }
                    var expected = shifts ? base.move(state.getOffset(pos)).toAabbs() : boxes;
                    if (!there.equals(expected)) {
                        offsetExplained = false;
                    }
                }
            }
            var distinct = new java.util.LinkedHashSet<com.google.gson.JsonElement>();
            perState.forEach(distinct::add);
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            var listed = new JsonArray();
            (distinct.size() == 1 ? distinct : perState).forEach(listed::add);
            described.add("shapes", listed);
            described.addProperty("position_dependent", moves);
            described.addProperty("offset_explained", offsetExplained);
            described.addProperty("dynamic", block.hasDynamicShape());
            blocks.add(described);
        }
        entry.add("shapes", shapes);
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * What each block contributes to movement beyond its shape: the three
     * factors, the bounce, the tags and classes the movement code tests, and
     * for every state the fluid it holds.
     *
     * Tags are the vanilla data pack's, loaded the way the server loads them.
     * A real client learns them from the server's tag packet; these are what a
     * vanilla server sends.
     */
    private static JsonObject movementProperties(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var blocks = new JsonArray();
        for (var block : registry) {
            var state = block.defaultBlockState();
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            described.addProperty("friction", Oracle.bits(block.getFriction()));
            described.addProperty("speed_factor", Oracle.bits(block.getSpeedFactor()));
            described.addProperty("jump_factor", Oracle.bits(block.getJumpFactor()));
            described.addProperty("bounce_restitution", Oracle.bits(block.getBounceRestitution()));
            described.addProperty("climbable", state.is(net.minecraft.tags.BlockTags.CLIMBABLE));
            described.addProperty("fence", state.is(net.minecraft.tags.BlockTags.FENCES));
            described.addProperty("wall", state.is(net.minecraft.tags.BlockTags.WALLS));
            described.addProperty("can_glide_through",
                state.is(net.minecraft.tags.BlockTags.CAN_GLIDE_THROUGH));
            described.addProperty("suppresses_bounce",
                state.is(net.minecraft.tags.BlockTags.SUPPRESSES_BOUNCE));
            described.addProperty("fence_gate",
                block instanceof net.minecraft.world.level.block.FenceGateBlock);
            described.addProperty("trapdoor",
                block instanceof net.minecraft.world.level.block.TrapDoorBlock);
            described.addProperty("ladder",
                block instanceof net.minecraft.world.level.block.LadderBlock);
            var fluids = new JsonArray();
            var distinct = new java.util.LinkedHashSet<String>();
            var perState = new java.util.ArrayList<String>();
            for (var possible : block.getStateDefinition().getPossibleStates()) {
                var fluid = possible.getFluidState();
                // The last field is whether the state suffocates, which is what
                // the player's push out of a wall asks of each block.
                String key = net.minecraft.core.registries.BuiltInRegistries.FLUID
                    .getKey(fluid.getType()) + "|" + fluid.getAmount() + "|"
                    + (fluid.isEmpty() ? false : fluid.getValue(
                        net.minecraft.world.level.material.FlowingFluid.FALLING))
                    + "|" + Oracle.bits(fluid.getOwnHeight()) + "|" + fluid.isSource()
                    + "|" + possible.isSuffocating(
                        net.minecraft.world.level.EmptyBlockGetter.INSTANCE,
                        net.minecraft.core.BlockPos.ZERO);
                perState.add(key);
                distinct.add(key);
            }
            (distinct.size() == 1 ? distinct : perState).forEach(fluids::add);
            described.add("fluids", fluids);
            blocks.add(described);
        }
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * What the block renderer's lighting asks of each state, outside a world.
     *
     * Per state: the light it emits, whether it renders at full brightness,
     * the ambient occlusion shade it casts, whether it blocks the view, how
     * much light it dampens, whether it renders solid, and whether its
     * collision shape is the whole block. A block whose states all agree
     * carries one entry.
     */
    private static JsonObject lightProperties(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var getter = net.minecraft.world.level.EmptyBlockGetter.INSTANCE;
        var origin = net.minecraft.core.BlockPos.ZERO;
        var blocks = new JsonArray();
        for (var block : registry) {
            var distinct = new java.util.LinkedHashSet<String>();
            var perState = new java.util.ArrayList<String>();
            for (var state : block.getStateDefinition().getPossibleStates()) {
                String key = state.getLightEmission() + "|" + state.emissiveRendering()
                    + "|" + Oracle.bits(state.getShadeBrightness(getter, origin))
                    + "|" + state.isViewBlocking(getter, origin) + "|" + state.getLightDampening()
                    + "|" + state.isSolidRender() + "|" + state.isCollisionShapeFullBlock(getter, origin);
                perState.add(key);
                distinct.add(key);
            }
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            var states = new JsonArray();
            (distinct.size() == 1 ? distinct : perState).forEach(states::add);
            described.add("states", states);
            blocks.add(described);
        }
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * Which blocks the renderer nudges off the grid, and by how much at most.
     *
     * The offset is a function the block's properties hold, so its kind is
     * read from what it does: a vertical component anywhere means it moves in
     * all three axes. The two limits are the block's own.
     */
    private static JsonObject offsetProperties(JsonObject row) {
        try {
            return offsetPropertiesOf(row);
        } catch (ReflectiveOperationException failure) {
            throw new IllegalStateException(failure);
        }
    }

    private static JsonObject offsetPropertiesOf(JsonObject row) throws ReflectiveOperationException {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var horizontal = net.minecraft.world.level.block.state.BlockBehaviour.class
            .getDeclaredMethod("getMaxHorizontalOffset");
        var vertical = net.minecraft.world.level.block.state.BlockBehaviour.class
            .getDeclaredMethod("getMaxVerticalOffset");
        horizontal.setAccessible(true);
        vertical.setAccessible(true);
        var blocks = new JsonArray();
        for (var block : registry) {
            var state = block.defaultBlockState();
            if (!state.hasOffsetFunction()) {
                continue;
            }
            boolean vertically = false;
            for (int x = 0; x < 16 && !vertically; x++) {
                for (int z = 0; z < 16 && !vertically; z++) {
                    vertically = state.getOffset(new net.minecraft.core.BlockPos(x, 0, z)).y != 0.0;
                }
            }
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            described.addProperty("kind", vertically ? "xyz" : "xz");
            described.addProperty("max_horizontal", Oracle.bits((float) horizontal.invoke(block)));
            described.addProperty("max_vertical", Oracle.bits((float) vertical.invoke(block)));
            blocks.add(described);
        }
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * Which states the MOTION_BLOCKING heightmap counts: those that block
     * motion or hold a fluid. One bit per state in state id order, packed into
     * 64-bit words, lowest bit first, written as hex.
     */
    private static JsonObject motionBlocking(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        int count = net.minecraft.world.level.block.Block.BLOCK_STATE_REGISTRY.size();
        long[] words = new long[(count + 63) / 64];
        for (var state : net.minecraft.world.level.block.Block.BLOCK_STATE_REGISTRY) {
            int id = net.minecraft.world.level.block.Block.getId(state);
            var predicate = net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING.isOpaque();
            if (predicate.test(state)) {
                words[id / 64] |= 1L << (id % 64);
            }
        }
        entry.addProperty("state_count", count);
        var packed = new JsonArray();
        for (long word : words) {
            packed.add(String.format("%016x", word));
        }
        entry.add("words", packed);
        return entry;
    }

    /**
     * What a fluid's flow reads of each state: bits 0 to 5 are the faces, in
     * direction order, that are fully sturdy (never for an ice block, which the
     * fluid treats as not solid), bit 6 whether the state blocks motion. Runs of
     * equal values in state id order, each its first state and value.
     */
    private static JsonObject fluidFlowProperties(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var getter = net.minecraft.world.level.EmptyBlockGetter.INSTANCE;
        var origin = net.minecraft.core.BlockPos.ZERO;
        int count = net.minecraft.world.level.block.Block.BLOCK_STATE_REGISTRY.size();
        int[] values = new int[count];
        for (var state : net.minecraft.world.level.block.Block.BLOCK_STATE_REGISTRY) {
            int value = 0;
            if (!(state.getBlock() instanceof net.minecraft.world.level.block.IceBlock)) {
                for (var direction : net.minecraft.core.Direction.values()) {
                    if (state.isFaceSturdy(getter, origin, direction)) {
                        value |= 1 << direction.get3DDataValue();
                    }
                }
            }
            if (state.blocksMotion()) {
                value |= 1 << 6;
            }
            values[net.minecraft.world.level.block.Block.getId(state)] = value;
        }
        var runs = new JsonArray();
        for (int id = 0; id < count; id++) {
            if (id == 0 || values[id] != values[id - 1]) {
                runs.add(id + ":" + values[id]);
            }
        }
        entry.addProperty("state_count", count);
        entry.add("runs", runs);
        return entry;
    }

    /** The blocks a fluid's side shows its overlay sprite against. */
    private static JsonObject fluidOverlayBlocks(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var blocks = new JsonArray();
        for (var block : registry) {
            if (block instanceof net.minecraft.world.level.block.HalfTransparentBlock
                || block instanceof net.minecraft.world.level.block.LeavesBlock) {
                blocks.add(registry.getKey(block).toString());
            }
        }
        entry.add("blocks", blocks);
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
