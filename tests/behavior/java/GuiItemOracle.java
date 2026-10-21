import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;

/** The conditions an item model decides from a stack alone: broken and has_component. */
class GuiItemOracle extends GuiDialogOracle {
    private static boolean propertiesBootstrapped;

    private static net.minecraft.world.item.ItemStack itemStack(JsonObject row) {
        var id = net.minecraft.resources.Identifier.parse(row.get("item").getAsString());
        var stack = new net.minecraft.world.item.ItemStack(net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(id));
        if (row.has("damage")) {
            stack.set(net.minecraft.core.component.DataComponents.DAMAGE, row.get("damage").getAsInt());
        }
        if (row.has("max_damage")) {
            stack.set(net.minecraft.core.component.DataComponents.MAX_DAMAGE, row.get("max_damage").getAsInt());
        }
        if (row.has("remove")) {
            for (var name : row.getAsJsonArray("remove")) {
                stack.remove(name.getAsString().equals("damage")
                    ? net.minecraft.core.component.DataComponents.DAMAGE
                    : net.minecraft.core.component.DataComponents.MAX_DAMAGE);
            }
        }
        if (row.has("unbreakable") && row.get("unbreakable").getAsBoolean()) {
            stack.set(net.minecraft.core.component.DataComponents.UNBREAKABLE, net.minecraft.util.Unit.INSTANCE);
        }
        for (var name : row.getAsJsonArray("components")) {
            switch (name.getAsString()) {
                case "minecraft:dyed_color" -> stack.set(net.minecraft.core.component.DataComponents.DYED_COLOR,
                    new net.minecraft.world.item.component.DyedItemColor(0xFF0000));
                case "minecraft:lodestone_tracker" -> stack.set(
                    net.minecraft.core.component.DataComponents.LODESTONE_TRACKER,
                    new net.minecraft.world.item.component.LodestoneTracker(java.util.Optional.empty(), true));
                default -> throw new IllegalArgumentException("component " + name);
            }
        }
        return stack;
    }

    static JsonObject itemCondition(JsonObject request) {
        Oracle.bootstrap();
        if (!propertiesBootstrapped) {
            net.minecraft.client.renderer.item.properties.conditional.ConditionalItemModelProperties.bootstrap();
            propertiesBootstrapped = true;
        }
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var property = net.minecraft.client.renderer.item.properties.conditional.ConditionalItemModelProperties.MAP_CODEC
                .codec().parse(JsonOps.INSTANCE, row.get("property")).getOrThrow();
            boolean value = property.get(itemStack(row), null, null, 0, net.minecraft.world.item.ItemDisplayContext.GUI);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("value", value);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
