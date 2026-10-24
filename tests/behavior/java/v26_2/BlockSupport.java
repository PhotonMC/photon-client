import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The game's registries and the helpers every block row shares. */
class BlockSupport {
    static boolean bootstrapped;

    static synchronized void bootstrap() {
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

    static void loadColourMap(String name, java.util.function.Consumer<int[]> install) {
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

    @SuppressWarnings({"unchecked", "rawtypes"})
    static String name(
        net.minecraft.world.level.block.state.properties.Property property, Comparable value) {
        return property.getName(value);
    }

    /** The simple name of the class whose own method a class ends up running. */
    static String declaringClass(Class<?> type, String name, Class<?>... parameters) {
        for (Class<?> current = type; current != null; current = current.getSuperclass()) {
            try {
                current.getDeclaredMethod(name, parameters);
                return current.getSimpleName();
            } catch (NoSuchMethodException missing) {
                // Look further up.
            }
        }
        throw new IllegalStateException("No " + name + " on " + type);
    }

    static JsonObject sortedObject(JsonObject object) {
        var sorted = new JsonObject();
        object.keySet().stream().sorted().forEach(key -> sorted.add(key, object.get(key)));
        return sorted;
    }

    static com.google.gson.JsonElement componentJson(net.minecraft.network.chat.Component component) {
        return net.minecraft.network.chat.ComponentSerialization.CODEC
            .encodeStart(com.mojang.serialization.JsonOps.INSTANCE, component).getOrThrow();
    }

    static <T> JsonArray listRegistry(net.minecraft.core.RegistryAccess registries,
            net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<T>> key,
            java.util.function.BiConsumer<T, JsonObject> describe) {
        var registry = registries.lookupOrThrow(key);
        var list = new JsonArray();
        for (var value : registry) {
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(value).toString());
            described.addProperty("id", registry.getId(value));
            describe.accept(value, described);
            list.add(described);
        }
        return list;
    }

    static JsonObject toolJson(net.minecraft.world.item.component.Tool tool) {
        var toolJson = new JsonObject();
        var rules = new JsonArray();
        for (var rule : tool.rules()) {
            var ruleJson = new JsonObject();
            var tag = rule.blocks().unwrapKey();
            if (tag.isPresent()) {
                ruleJson.addProperty("blocks", "#" + tag.get().location());
            } else {
                var names = new JsonArray();
                for (var holder : rule.blocks()) {
                    names.add(holder.unwrapKey().orElseThrow().identifier().toString());
                }
                ruleJson.add("blocks", names);
            }
            rule.speed().ifPresent(speed -> ruleJson.addProperty("speed", Oracle.bits(speed)));
            rule.correctForDrops().ifPresent(correct -> ruleJson.addProperty("correct_for_drops", correct));
            rules.add(ruleJson);
        }
        toolJson.add("rules", rules);
        toolJson.addProperty("default_mining_speed", Oracle.bits(tool.defaultMiningSpeed()));
        toolJson.addProperty("damage_per_block", tool.damagePerBlock());
        toolJson.addProperty("can_destroy_blocks_in_creative", tool.canDestroyBlocksInCreative());
        return toolJson;
    }

    static String encodePatch(
            net.minecraft.network.codec.StreamCodec<net.minecraft.network.RegistryFriendlyByteBuf,
                net.minecraft.core.component.DataComponentPatch> codec,
            net.minecraft.core.component.DataComponentPatch patch,
            net.minecraft.core.RegistryAccess registries) {
        var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(io.netty.buffer.Unpooled.buffer(), registries);
        codec.encode(buffer, patch);
        var bytes = new byte[buffer.readableBytes()];
        buffer.readBytes(bytes);
        var hex = new StringBuilder();
        for (byte value : bytes) {
            hex.append(String.format("%02x", value));
        }
        return hex.toString();
    }

    /** A component's value as the network writes it, without its id. */
    static <T> String encodeComponentValue(
            net.minecraft.core.component.TypedDataComponent<T> component,
            net.minecraft.core.RegistryAccess registries) {
        var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(io.netty.buffer.Unpooled.buffer(), registries);
        component.type().streamCodec().encode(buffer, component.value());
        var bytes = new byte[buffer.readableBytes()];
        buffer.readBytes(bytes);
        var hex = new StringBuilder();
        for (byte value : bytes) {
            hex.append(String.format("%02x", value));
        }
        return hex.toString();
    }
}
