import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;

/** The conditions an item model decides from a stack alone: broken and has_component. */
class GuiItemOracle extends GuiDialogOracle {
    private static boolean propertiesBootstrapped;
    private static boolean numbersBootstrapped;

    private static net.minecraft.world.item.ItemStack itemStack(JsonObject row) {
        var id = net.minecraft.resources.Identifier.parse(row.get("item").getAsString());
        var stack = new net.minecraft.world.item.ItemStack(net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(id));
        if (row.has("patch")) {
            MovementPlayerOracle.applyWirePatch(stack, row.get("patch").getAsString(), Game26_2.vanillaRegistries());
        }
        if (row.has("count")) {
            stack.setCount(row.get("count").getAsInt());
        }
        if (row.has("max_stack_size")) {
            stack.set(net.minecraft.core.component.DataComponents.MAX_STACK_SIZE, row.get("max_stack_size").getAsInt());
        }
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
        if (row.has("block_state")) {
            var properties = new java.util.HashMap<String, String>();
            for (var entry : row.getAsJsonObject("block_state").entrySet()) {
                properties.put(entry.getKey(), entry.getValue().getAsString());
            }
            stack.set(net.minecraft.core.component.DataComponents.BLOCK_STATE,
                new net.minecraft.world.item.component.BlockItemStateProperties(properties));
        }
        if (row.has("remove_block_state")) {
            stack.remove(net.minecraft.core.component.DataComponents.BLOCK_STATE);
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
        // Binds the items' default components, which a stack reads.
        Game26_2.vanillaRegistries();
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

    /** The `damage` and `count` range properties' value for a stack; a NaN is reported as `nan`. */
    static JsonObject itemNumber(JsonObject request) {
        Game26_2.vanillaRegistries();
        if (!numbersBootstrapped) {
            net.minecraft.client.renderer.item.properties.numeric.RangeSelectItemModelProperties.bootstrap();
            numbersBootstrapped = true;
        }
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var property = net.minecraft.client.renderer.item.properties.numeric.RangeSelectItemModelProperties.MAP_CODEC
                .codec().parse(JsonOps.INSTANCE, row.get("property")).getOrThrow();
            float value = property.get(itemStack(row), null, null, 0);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (Float.isNaN(value)) {
                out.addProperty("nan", true);
            } else {
                out.addProperty("value", (double) value);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The colour a `custom_model_data` tint source gives a stack (ARGB, signed). */
    static JsonObject itemTint(JsonObject request) {
        Game26_2.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var tint = row.getAsJsonObject("tint");
            var source = new net.minecraft.client.color.item.CustomModelDataSource(
                tint.has("index") ? tint.get("index").getAsInt() : 0, tint.get("default").getAsInt());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("value", source.calculate(itemStack(row), null, null));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The `block_state` select's value for a stack: the property it reads, or null. */
    static JsonObject itemSelect(JsonObject request) {
        Game26_2.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            net.minecraft.client.renderer.item.properties.select.SelectItemModelProperty<String> property =
                row.has("custom_index")
                    ? new net.minecraft.client.renderer.item.properties.select.CustomModelDataProperty(
                        row.get("custom_index").getAsInt())
                    : new net.minecraft.client.renderer.item.properties.select.ItemBlockState(
                        row.get("property").getAsString());
            var value = property.get(itemStack(row), null, null, 0, net.minecraft.world.item.ItemDisplayContext.GUI);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("value", value);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The compass needle: the wobbler over a run of ticks, and the rotation towards a block. */
    static JsonObject itemNeedle(JsonObject request) throws Exception {
        Game26_2.bootstrap();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (row.get("kind").getAsString().equals("wobble")) {
                var wobbler = row.get("wobble").getAsBoolean()
                    ? net.minecraft.client.renderer.item.properties.numeric.NeedleDirectionHelper.standardWobbler(row.get("factor").getAsFloat())
                    : net.minecraft.client.renderer.item.properties.numeric.NeedleDirectionHelper.nonWobbler();
                var steps = new JsonArray();
                for (var step : row.getAsJsonArray("steps")) {
                    var object = step.getAsJsonObject();
                    long tick = object.get("tick").getAsLong();
                    var result = new JsonObject();
                    boolean update = wobbler.shouldUpdate(tick);
                    if (update) {
                        wobbler.update(tick, object.get("target").getAsFloat());
                    }
                    result.addProperty("updated", update);
                    result.addProperty("rotation", Float.floatToRawIntBits(wobbler.rotation()));
                    steps.add(result);
                }
                out.add("steps", steps);
            } else {
                var owner = row.getAsJsonObject("owner");
                var position = new net.minecraft.world.phys.Vec3(
                    owner.get("x").getAsDouble(), owner.get("y").getAsDouble(), owner.get("z").getAsDouble());
                float yaw = owner.get("yaw").getAsFloat();
                var itemOwner = (net.minecraft.world.entity.ItemOwner)java.lang.reflect.Proxy.newProxyInstance(
                    GuiItemOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.world.entity.ItemOwner.class},
                    (proxy, method, args) -> switch (method.getName()) {
                        case "position" -> position;
                        case "getVisualRotationYInDegrees" -> yaw;
                        case "asLivingEntity" -> null;
                        default -> throw new UnsupportedOperationException(method.getName());
                    });
                var block = row.getAsJsonArray("block");
                var state = new net.minecraft.client.renderer.item.properties.numeric.CompassAngleState(
                    true, net.minecraft.client.renderer.item.properties.numeric.CompassAngleState.CompassTarget.NONE);
                var method = Reflect.method(net.minecraft.client.renderer.item.properties.numeric.CompassAngleState.class, "getRotationTowardsCompassTarget",
                        net.minecraft.world.entity.ItemOwner.class, long.class, net.minecraft.core.BlockPos.class);
                float rotation = (Float)method.invoke(state, itemOwner, row.get("tick").getAsLong(),
                    new net.minecraft.core.BlockPos(block.get(0).getAsInt(), block.get(1).getAsInt(), block.get(2).getAsInt()));
                out.addProperty("rotation", Float.floatToRawIntBits(rotation));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
