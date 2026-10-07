import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Movement, light, offset, destroy and fluid properties, and block behaviours. */
class BlockPropertyOracle extends BlockShapeOracle {
    /**
     * What each block contributes to movement beyond its shape: the three
     * factors, the bounce, the tags and classes the movement code tests, and
     * for every state the fluid it holds.
     *
     * Tags are the vanilla data pack's, loaded the way the server loads them.
     * A real client learns them from the server's tag packet; these are what a
     * vanilla server sends.
     */
    static JsonObject movementProperties(JsonObject row) {
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
    static JsonObject lightProperties(JsonObject row) {
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
                    + "|" + state.isSolidRender() + "|" + state.isCollisionShapeFullBlock(getter, origin)
                    + "|" + state.useShapeForLightOcclusion() + "|" + state.canOcclude();
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
    static JsonObject offsetProperties(JsonObject row) {
        try {
            return offsetPropertiesOf(row);
        } catch (ReflectiveOperationException failure) {
            throw new IllegalStateException(failure);
        }
    }

    static JsonObject offsetPropertiesOf(JsonObject row) throws ReflectiveOperationException {
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
    /**
     * What breaking a block reads of each state: its destroy time (as float
     * bits; -1 cannot be broken), whether drops need the right tool, whether
     * it is air, and the state the client predicts in its place (the legacy
     * block of the fluid it holds). A block whose states all agree carries one
     * entry. Game master blocks are flagged per block.
     */
    static JsonObject destroyProperties(JsonObject row) {
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
                String key = Oracle.bits(state.getDestroySpeed(getter, origin))
                    + "|" + state.requiresCorrectToolForDrops() + "|" + state.isAir()
                    + "|" + net.minecraft.world.level.block.Block.getId(
                        state.getFluidState().createLegacyBlock());
                perState.add(key);
                distinct.add(key);
            }
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            described.addProperty("game_master",
                block instanceof net.minecraft.world.level.block.GameMasterBlock);
            var states = new JsonArray();
            (distinct.size() == 1 ? distinct : perState).forEach(states::add);
            described.add("states", states);
            blocks.add(described);
        }
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * What a fluid's flow reads of each state: bits 0 to 5 are the faces, in
     * direction order, that are fully sturdy (never for an ice block, which the
     * fluid treats as not solid), bit 6 whether the state blocks motion. Runs of
     * equal values in state id order, each its first state and value.
     */
    static JsonObject fluidFlowProperties(JsonObject row) {
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

    /**
     * Which class's own rule each block runs when an item is used on it and
     * when it is placed, and each state's replaceable flag. Items get their
     * class, the class whose use rules they run, and the block they place.
     */
    static JsonObject blockBehaviours(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        Class<?> stateType = net.minecraft.world.level.block.state.BlockState.class;
        Class<?> posType = net.minecraft.core.BlockPos.class;
        Class<?> levelType = net.minecraft.world.level.Level.class;
        Class<?> playerType = net.minecraft.world.entity.player.Player.class;
        Class<?> hitType = net.minecraft.world.phys.BlockHitResult.class;
        Class<?> placeType = net.minecraft.world.item.context.BlockPlaceContext.class;
        var blocks = new JsonArray();
        for (var block : registry) {
            var type = block.getClass();
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(block).toString());
            described.addProperty("class", type.getSimpleName());
            described.addProperty("placement", declaringClass(type, "getStateForPlacement", placeType));
            described.addProperty("can_be_replaced", declaringClass(type, "canBeReplaced", stateType, placeType));
            described.addProperty("can_survive", declaringClass(type, "canSurvive",
                stateType, net.minecraft.world.level.LevelReader.class, posType));
            described.addProperty("use_without_item", declaringClass(type, "useWithoutItem",
                stateType, levelType, posType, playerType, hitType));
            described.addProperty("use_item_on", declaringClass(type, "useItemOn",
                net.minecraft.world.item.ItemStack.class, stateType, levelType, posType, playerType,
                net.minecraft.world.InteractionHand.class, hitType));
            described.addProperty("update_shape", declaringClass(type, "updateShape",
                stateType, net.minecraft.world.level.LevelReader.class,
                net.minecraft.world.level.ScheduledTickAccess.class, posType,
                net.minecraft.core.Direction.class, posType, stateType,
                net.minecraft.util.RandomSource.class));
            var replaceable = new StringBuilder();
            for (var state : block.getStateDefinition().getPossibleStates()) {
                replaceable.append(state.canBeReplaced() ? '1' : '0');
            }
            described.addProperty("replaceable", replaceable.toString());
            described.addProperty("stairs", block instanceof net.minecraft.world.level.block.StairBlock);
            described.addProperty("item", net.minecraft.core.registries.BuiltInRegistries.ITEM
                .getKey(block.asItem()).toString());
            blocks.add(described);
        }
        entry.add("blocks", blocks);
        var items = new JsonArray();
        var itemRegistry = net.minecraft.core.registries.BuiltInRegistries.ITEM;
        for (var item : itemRegistry) {
            var type = item.getClass();
            var described = new JsonObject();
            described.addProperty("name", itemRegistry.getKey(item).toString());
            described.addProperty("class", type.getSimpleName());
            described.addProperty("use_on", declaringClass(type, "useOn",
                net.minecraft.world.item.context.UseOnContext.class));
            described.addProperty("use", declaringClass(type, "use",
                levelType, playerType, net.minecraft.world.InteractionHand.class));
            // `Item.use` does nothing for a stack with none of these.
            var components = item.components();
            var equippable = components.get(net.minecraft.core.component.DataComponents.EQUIPPABLE);
            described.addProperty("default_use_pass",
                !components.has(net.minecraft.core.component.DataComponents.CONSUMABLE)
                    && (equippable == null || !equippable.swappable())
                    && !components.has(net.minecraft.core.component.DataComponents.BLOCKS_ATTACKS)
                    && !components.has(net.minecraft.core.component.DataComponents.KINETIC_WEAPON));
            if (item instanceof net.minecraft.world.item.BlockItem blockItem) {
                described.addProperty("block", registry.getKey(blockItem.getBlock()).toString());
            }
            items.add(described);
        }
        entry.add("items", items);
        return entry;
    }
}
