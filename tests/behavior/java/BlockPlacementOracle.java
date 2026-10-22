import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * What placing a block does on scripted ground: the game's own block classes answer
 * `canBeReplaced`, `getStateForPlacement` and `canSurvive` for a click on a small world.
 */
class BlockPlacementOracle extends BlockItemOracle {
    /** A level whose blocks are the ones the row lists, and air everywhere else. */
    static final class ScriptedLevel extends net.minecraft.client.multiplayer.ClientLevel {
        java.util.Map<Long, net.minecraft.world.level.block.state.BlockState> states;

        ScriptedLevel() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
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
        public boolean isClientSide() {
            return true;
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
            return false;
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
        } catch (ReflectiveOperationException failure) {
            throw new IllegalStateException(failure);
        }
        level.states = new java.util.HashMap<>();
        for (var element : row.getAsJsonArray("world")) {
            var spec = element.getAsJsonObject();
            level.states.put(position(spec.getAsJsonArray("pos")).asLong(), scriptedState(spec));
        }
        viewer.yaw = Oracle.f32(row, "yaw");
        viewer.pitch = Oracle.f32(row, "pitch");
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
            var hit = new net.minecraft.world.phys.BlockHitResult(click, face, hitPos, false);
            var context = new Context(level, viewer, stack, hit);
            var described = new JsonObject();
            described.addProperty("block", name.getAsString());
            described.addProperty("replace_clicked", context.replacingClickedOnBlock());
            described.addProperty("can_place", context.canPlace());
            var placed = block.getStateForPlacement(context);
            // A refusal has no state.
            if (placed != null) {
                described.addProperty("state",
                    net.minecraft.commands.arguments.blocks.BlockStateParser.serialize(placed));
                described.addProperty("survives", placed.canSurvive(level, context.getClickedPos()));
            }
            results.add(described);
        }
        entry.add("placements", results);
        return entry;
    }
}
