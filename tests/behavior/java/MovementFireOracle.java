import com.google.gson.JsonObject;

/** The blocks fire catches on: `FireBlock`'s ignite and burn odds, as its bootstrap sets them. */
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
}
