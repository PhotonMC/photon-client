import com.google.gson.JsonObject;

/**
 * Block tables the client reads that are set when the game starts: the blocks fire catches on
 * (`FireBlock`'s odds), each state's map colour, and the falling blocks' dust colours.
 */
class MovementFireOracle extends MovementPickupOracle {
    /** Every block with odds, in registry order: its name, its ignite odds and its burn odds. */
    @SuppressWarnings("unchecked")
    static JsonObject fireFlammability(JsonObject row) throws Exception {
        var fire = net.minecraft.world.level.block.Blocks.FIRE;
        var ignite = (it.unimi.dsi.fastutil.objects.Object2IntMap<net.minecraft.world.level.block.Block>)
            getField(fire, "igniteOdds");
        var burn = (it.unimi.dsi.fastutil.objects.Object2IntMap<net.minecraft.world.level.block.Block>)
            getField(fire, "burnOdds");
        var blocks = new com.google.gson.JsonArray();
        for (var block : net.minecraft.core.registries.BuiltInRegistries.BLOCK) {
            if (ignite.getInt(block) == 0 && burn.getInt(block) == 0) {
                continue;
            }
            var entry = new JsonObject();
            entry.addProperty("name", net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(block).toString());
            entry.addProperty("ignite", ignite.getInt(block));
            entry.addProperty("burn", burn.getInt(block));
            blocks.add(entry);
        }
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.add("blocks", blocks);
        return entry;
    }

    /**
     * Every block's map colour (`BlockState.getMapColor`): its first state's, and each state whose
     * colour is another; a falling block's dust colour (`FallingBlock.getDustColor`).
     */
    static JsonObject blockMapColors(JsonObject row) throws Exception {
        var level = net.minecraft.world.level.EmptyBlockGetter.INSTANCE;
        var origin = net.minecraft.core.BlockPos.ZERO;
        var blocks = new com.google.gson.JsonArray();
        for (var block : net.minecraft.core.registries.BuiltInRegistries.BLOCK) {
            var states = block.getStateDefinition().getPossibleStates();
            int first = states.get(0).getMapColor(level, origin).id;
            var entry = new JsonObject();
            entry.addProperty("name", net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(block).toString());
            entry.addProperty("color", first);
            var others = new com.google.gson.JsonArray();
            for (var state : states) {
                int color = state.getMapColor(level, origin).id;
                if (color != first) {
                    var pair = new com.google.gson.JsonArray();
                    pair.add(net.minecraft.world.level.block.Block.getId(state));
                    pair.add(color);
                    others.add(pair);
                }
            }
            entry.add("states", others);
            if (block instanceof net.minecraft.world.level.block.FallingBlock falling) {
                entry.addProperty("dust", falling.getDustColor(block.defaultBlockState(), level, origin));
            }
            blocks.add(entry);
        }
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.add("blocks", blocks);
        return entry;
    }
}
