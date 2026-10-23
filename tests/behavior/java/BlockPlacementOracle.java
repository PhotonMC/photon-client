import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * What placing a block does on scripted ground: the game's own block classes answer
 * `canBeReplaced`, `getStateForPlacement` and `canSurvive` for a click on a small world.
 */
class BlockPlacementOracle extends BlockItemOracle {
    /** A level whose blocks are the ones the row lists, and air everywhere else. */
    static final class ScriptedLevel extends net.minecraft.client.multiplayer.ClientLevel {
        static final net.minecraft.util.RandomSource RANDOM = net.minecraft.util.RandomSource.create(0L);

        @Override
        public net.minecraft.util.RandomSource getRandom() {
            return RANDOM;
        }

        @Override
        public long getGameTime() {
            return 0L;
        }

        java.util.Map<Long, net.minecraft.world.level.block.state.BlockState> states;
        /** The blocks set since the last reset: position, state and flags. */
        java.util.List<Object[]> writes;
        net.minecraft.world.level.border.WorldBorder border;
        int rawLight;

        ScriptedLevel() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
        }

        /** Sounds are not what this oracle compares. */
        @Override
        public void playSound(net.minecraft.world.entity.Entity except, double x, double y,
                double z, net.minecraft.sounds.SoundEvent sound, net.minecraft.sounds.SoundSource source,
                float volume, float pitch) {
        }

        @Override
        public void playSeededSound(net.minecraft.world.entity.Entity except, double x, double y,
                double z, net.minecraft.core.Holder<net.minecraft.sounds.SoundEvent> sound,
                net.minecraft.sounds.SoundSource source, float volume, float pitch, long seed) {
        }

        @Override
        public void playSeededSound(net.minecraft.world.entity.Entity except,
                net.minecraft.world.entity.Entity entity,
                net.minecraft.core.Holder<net.minecraft.sounds.SoundEvent> sound,
                net.minecraft.sounds.SoundSource source, float volume, float pitch, long seed) {
        }

        @Override
        public net.minecraft.world.level.block.state.BlockState getBlockState(
            net.minecraft.core.BlockPos pos) {
            var state = this.states.get(pos.asLong());
            return state != null ? state : net.minecraft.world.level.block.Blocks.AIR.defaultBlockState();
        }

        @Override
        public net.minecraft.world.level.material.FluidState getFluidState(
            net.minecraft.core.BlockPos pos) {
            return this.getBlockState(pos).getFluidState();
        }

        @Override
        public net.minecraft.world.level.block.entity.BlockEntity getBlockEntity(
            net.minecraft.core.BlockPos pos) {
            return null;
        }

        @Override
        public boolean isClientSide() {
            return true;
        }

        @Override
        public boolean setBlock(net.minecraft.core.BlockPos pos,
                net.minecraft.world.level.block.state.BlockState state, int flags, int limit) {
            this.writes.add(new Object[] {pos.immutable(), state, flags});
            this.states.put(pos.asLong(), state);
            return true;
        }

        @Override
        public net.minecraft.world.level.border.WorldBorder getWorldBorder() {
            return this.border;
        }

        @Override
        public int getMaxY() {
            return 319;
        }

        @Override
        public boolean isUnobstructed(net.minecraft.world.entity.Entity source,
                net.minecraft.world.phys.shapes.VoxelShape shape) {
            return true;
        }

        @Override
        public java.util.List<net.minecraft.world.entity.Entity> getEntities(
                net.minecraft.world.entity.Entity except, net.minecraft.world.phys.AABB bb,
                java.util.function.Predicate<? super net.minecraft.world.entity.Entity> selector) {
            return java.util.List.of();
        }

        @Override
        public int getRawBrightness(net.minecraft.core.BlockPos pos, int darkening) {
            return this.rawLight;
        }

        @Override
        public net.minecraft.world.flag.FeatureFlagSet enabledFeatures() {
            return net.minecraft.world.flag.FeatureFlagSet.of();
        }
    }

    /** A player that only has a view direction. */
    static final class Viewer extends net.minecraft.client.player.LocalPlayer {
        float yaw;
        float pitch;
        boolean sneakRequested;
        boolean gameMaster;

        Viewer() {
            super(null, null, null, null, null, null, false, null);
        }

        @Override
        public float getYRot() {
            return this.yaw;
        }

        @Override
        public float getXRot() {
            return this.pitch;
        }

        @Override
        public boolean isSecondaryUseActive() {
            return this.sneakRequested;
        }

        @Override
        public boolean canUseGameMasterBlocks() {
            return this.gameMaster;
        }

        // The collision context of a placement reads these; the entity checks are stubbed.
        @Override
        public boolean isDescending() {
            return false;
        }

        @Override
        public net.minecraft.world.item.ItemStack getMainHandItem() {
            return net.minecraft.world.item.ItemStack.EMPTY;
        }
    }

    /** The context a block item builds when it is used on a block. */
    static final class Context extends net.minecraft.world.item.context.BlockPlaceContext {
        Context(net.minecraft.world.level.Level level, net.minecraft.world.entity.player.Player player,
                net.minecraft.world.item.ItemStack stack, net.minecraft.world.phys.BlockHitResult hit) {
            super(level, player, net.minecraft.world.InteractionHand.MAIN_HAND, stack, hit);
        }
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    static net.minecraft.world.level.block.state.BlockState scriptedState(JsonObject spec) {
        var block = net.minecraft.core.registries.BuiltInRegistries.BLOCK
            .getValue(net.minecraft.resources.Identifier.parse(spec.get("block").getAsString()));
        var state = block.defaultBlockState();
        if (spec.has("properties")) {
            for (var entry : spec.getAsJsonObject("properties").entrySet()) {
                net.minecraft.world.level.block.state.properties.Property property =
                    block.getStateDefinition().getProperty(entry.getKey());
                net.minecraft.world.level.block.state.StateHolder holder = state;
                state = (net.minecraft.world.level.block.state.BlockState) holder.setValue(property,
                    (Comparable) property.getValue(entry.getValue().getAsString()).orElseThrow());
            }
        }
        return state;
    }

    static net.minecraft.core.BlockPos position(JsonArray at) {
        return new net.minecraft.core.BlockPos(at.get(0).getAsInt(), at.get(1).getAsInt(), at.get(2).getAsInt());
    }

    /**
     * For each block of a row, as the item that places it is used on the scripted world: whether
     * the clicked block is replaced, whether the block may go in the clicked position, the state
     * it takes, and whether that state may stand there.
     */
    static JsonObject placementRules(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        sun.misc.Unsafe unsafe;
        ScriptedLevel level;
        Viewer viewer;
        try {
            var field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            unsafe = (sun.misc.Unsafe) field.get(null);
            level = (ScriptedLevel) unsafe.allocateInstance(ScriptedLevel.class);
            viewer = (Viewer) unsafe.allocateInstance(Viewer.class);
            // The placement's collision context reads the player's height.
            var position = net.minecraft.world.entity.Entity.class.getDeclaredField("position");
            position.setAccessible(true);
            position.set(viewer, net.minecraft.world.phys.Vec3.ZERO);
        } catch (ReflectiveOperationException failure) {
            throw new IllegalStateException(failure);
        }
        level.states = new java.util.HashMap<>();
        level.writes = new java.util.ArrayList<>();
        level.border = new net.minecraft.world.level.border.WorldBorder();
        for (var element : row.getAsJsonArray("world")) {
            var spec = element.getAsJsonObject();
            level.states.put(position(spec.getAsJsonArray("pos")).asLong(), scriptedState(spec));
        }
        level.rawLight = row.has("raw_light") ? row.get("raw_light").getAsInt() : 15;
        viewer.yaw = Oracle.f32(row, "yaw");
        viewer.pitch = Oracle.f32(row, "pitch");
        viewer.sneakRequested = row.has("sneak") && row.get("sneak").getAsBoolean();
        viewer.gameMaster = row.has("game_master") && row.get("game_master").getAsBoolean();
        var inside = row.has("inside") && row.get("inside").getAsBoolean();
        var location = row.getAsJsonArray("location");
        var click = new net.minecraft.world.phys.Vec3(
            Double.longBitsToDouble(Long.parseUnsignedLong(location.get(0).getAsString(), 16)),
            Double.longBitsToDouble(Long.parseUnsignedLong(location.get(1).getAsString(), 16)),
            Double.longBitsToDouble(Long.parseUnsignedLong(location.get(2).getAsString(), 16)));
        var face = net.minecraft.core.Direction.byName(row.get("face").getAsString());
        var hitPos = position(row.getAsJsonArray("hit"));
        var results = new JsonArray();
        for (var name : row.getAsJsonArray("blocks")) {
            var block = net.minecraft.core.registries.BuiltInRegistries.BLOCK
                .getValue(net.minecraft.resources.Identifier.parse(name.getAsString()));
            var stack = new net.minecraft.world.item.ItemStack(block.asItem());
            var hit = new net.minecraft.world.phys.BlockHitResult(click, face, hitPos, inside);
            var context = new Context(level, viewer, stack, hit);
            var described = new JsonObject();
            described.addProperty("block", name.getAsString());
            described.addProperty("replace_clicked", context.replacingClickedOnBlock());
            described.addProperty("can_place", context.canPlace());
            var placed = block.getStateForPlacement(context);
            // A growing plant's head takes a random age, which no client can reproduce.
            if (placed != null && block instanceof net.minecraft.world.level.block.GrowingPlantHeadBlock
                && placed.hasProperty(net.minecraft.world.level.block.state.properties.BlockStateProperties.AGE_25)) {
                placed = placed.setValue(
                    net.minecraft.world.level.block.state.properties.BlockStateProperties.AGE_25, 0);
            }
            // A refusal has no state.
            if (placed != null) {
                described.addProperty("state",
                    net.minecraft.commands.arguments.blocks.BlockStateParser.serialize(placed));
                described.addProperty("survives", placed.canSurvive(level, context.getClickedPos()));
                if (row.has("neighbours") && row.get("neighbours").getAsBoolean()) {
                    described.add("neighbours", neighbourShapes(level, context.getClickedPos(), placed));
                }
            }
            if (stack.getItem() instanceof net.minecraft.world.item.BlockItem item) {
                itemPlacement(level, context, stack, item, placed, described);
            }
            results.add(described);
        }
        entry.add("placements", results);
        return entry;
    }

    /**
     * {@code Level.updateNeighbourShapes} after {@code placed} is set at {@code pos}: the
     * neighbours, in the game's order, that change to another state that is not air.
     */
    static JsonArray neighbourShapes(ScriptedLevel level, net.minecraft.core.BlockPos pos,
            net.minecraft.world.level.block.state.BlockState placed) {
        var before = level.states.get(pos.asLong());
        level.states.put(pos.asLong(), placed);
        var changed = new JsonArray();
        net.minecraft.core.Direction[] order = {
            net.minecraft.core.Direction.WEST, net.minecraft.core.Direction.EAST,
            net.minecraft.core.Direction.NORTH, net.minecraft.core.Direction.SOUTH,
            net.minecraft.core.Direction.DOWN, net.minecraft.core.Direction.UP};
        for (var direction : order) {
            var at = pos.relative(direction);
            var current = level.getBlockState(at);
            var updated = current.updateShape(level, level, at, direction.getOpposite(), pos, placed,
                level.getRandom());
            if (updated != current && !updated.isAir()) {
                var entry = new JsonObject();
                var position = new JsonArray();
                position.add(at.getX());
                position.add(at.getY());
                position.add(at.getZ());
                entry.add("pos", position);
                entry.addProperty("state",
                    net.minecraft.commands.arguments.blocks.BlockStateParser.serialize(updated));
                changed.add(entry);
            }
        }
        if (before == null) {
            level.states.remove(pos.asLong());
        } else {
            level.states.put(pos.asLong(), before);
        }
        return changed;
    }

    /**
     * What the classes that extend {@code BlockItem} add: the state {@code getPlacementState}
     * chooses (the standing and wall blocks of a torch are tried in the look's order), and the
     * blocks {@code place} then sets, ending with {@code setPlacedBy}.
     */
    static void itemPlacement(ScriptedLevel level, Context context,
            net.minecraft.world.item.ItemStack stack, net.minecraft.world.item.BlockItem item,
            net.minecraft.world.level.block.state.BlockState placed, JsonObject described) {
        try {
            // Scaffolding moves the click along its stack.
            var used = item.updatePlacementContext(context);
            if (used == null) {
                return;
            }
            var method = net.minecraft.world.item.BlockItem.class.getDeclaredMethod(
                "getPlacementState", net.minecraft.world.item.context.BlockPlaceContext.class);
            method.setAccessible(true);
            var chosen = (net.minecraft.world.level.block.state.BlockState) method.invoke(item, used);
            if (chosen == null) {
                return;
            }
            // A growing plant's head takes a random age, which no client can reproduce.
            if (chosen.getBlock() instanceof net.minecraft.world.level.block.GrowingPlantHeadBlock
                && chosen.hasProperty(net.minecraft.world.level.block.state.properties.BlockStateProperties.AGE_25)) {
                chosen = chosen.setValue(
                    net.minecraft.world.level.block.state.properties.BlockStateProperties.AGE_25, 0);
            }
            described.addProperty("item_state",
                net.minecraft.commands.arguments.blocks.BlockStateParser.serialize(chosen));
            level.writes.clear();
            var before = new java.util.HashMap<>(level.states);
            var place = net.minecraft.world.item.BlockItem.class.getDeclaredMethod("placeBlock",
                net.minecraft.world.item.context.BlockPlaceContext.class,
                net.minecraft.world.level.block.state.BlockState.class);
            place.setAccessible(true);
            place.invoke(item, used, chosen);
            var pos = used.getClickedPos();
            var now = level.getBlockState(pos);
            if (now.is(chosen.getBlock())) {
                now.getBlock().setPlacedBy(level, pos, now, null, stack);
            }
            level.states = before;
            var writes = new JsonArray();
            for (var write : level.writes) {
                var at = (net.minecraft.core.BlockPos) write[0];
                var entry = new JsonObject();
                var position = new JsonArray();
                position.add(at.getX());
                position.add(at.getY());
                position.add(at.getZ());
                entry.add("pos", position);
                entry.addProperty("state", net.minecraft.commands.arguments.blocks.BlockStateParser
                    .serialize((net.minecraft.world.level.block.state.BlockState) write[1]));
                entry.addProperty("flags", (Integer) write[2]);
                writes.add(entry);
            }
            described.add("writes", writes);
        } catch (ReflectiveOperationException failure) {
            throw new IllegalStateException(failure);
        }
    }
}
