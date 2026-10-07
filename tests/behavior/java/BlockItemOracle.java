import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Item tools, stacking, components, tooltips, potions and creative tabs. */
class BlockItemOracle extends BlockPropertyOracle {
    /**
     * Every item in registry order with the tool component it carries by
     * default. A rule's blocks are a tag (written with a leading #) or a list
     * of block names; speed and drop correctness are absent when the rule
     * leaves them out.
     */
    static JsonObject itemTools(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.ITEM;
        var items = new JsonArray();
        for (var item : registry) {
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(item).toString());
            var tool = item.components().get(net.minecraft.core.component.DataComponents.TOOL);
            if (tool != null) {
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
                described.add("tool", toolJson);
            }
            items.add(described);
        }
        entry.add("items", items);
        return entry;
    }

    /**
     * What container clicks read from every item's defaults: its stack
     * size, durability, equipment slot and whether a player may wear it,
     * the item left behind when it is crafted with, and which class decides
     * a click with it carried or clicked on.
     */
    static JsonObject itemStacking(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.ITEM;
        var playerType = net.minecraft.world.entity.EntityTypes.PLAYER.builtInRegistryHolder();
        var items = new JsonArray();
        for (var item : registry) {
            var components = item.components();
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(item).toString());
            described.addProperty("max_stack_size",
                components.getOrDefault(net.minecraft.core.component.DataComponents.MAX_STACK_SIZE, 1));
            described.addProperty("max_damage",
                components.getOrDefault(net.minecraft.core.component.DataComponents.MAX_DAMAGE, 0));
            var equippable = components.get(net.minecraft.core.component.DataComponents.EQUIPPABLE);
            if (equippable != null) {
                described.addProperty("equip_slot", equippable.slot().getSerializedName());
                described.addProperty("player_can_equip", equippable.canBeEquippedBy(playerType));
            }
            var remainder = item.getCraftingRemainder();
            if (remainder != null) {
                var stack = remainder.create();
                described.addProperty("crafting_remainder", registry.getKey(stack.getItem()).toString());
                described.addProperty("crafting_remainder_count", stack.getCount());
            }
            described.addProperty("stacked_on_other", declaringClass(item.getClass(), "overrideStackedOnOther",
                net.minecraft.world.item.ItemStack.class, net.minecraft.world.inventory.Slot.class,
                net.minecraft.world.inventory.ClickAction.class, net.minecraft.world.entity.player.Player.class));
            described.addProperty("other_stacked_on_me", declaringClass(item.getClass(), "overrideOtherStackedOnMe",
                net.minecraft.world.item.ItemStack.class, net.minecraft.world.item.ItemStack.class,
                net.minecraft.world.inventory.Slot.class, net.minecraft.world.inventory.ClickAction.class,
                net.minecraft.world.entity.player.Player.class, net.minecraft.world.entity.SlotAccess.class));
            items.add(described);
        }
        entry.add("items", items);
        return entry;
    }

    /**
     * Every item's default components as a component patch, in both forms a
     * stack carries them: as a server sends it, and as a client sends it
     * (each value behind its length). Components the network does not carry
     * are left out by the patch's own encoder.
     */
    static JsonObject itemComponents(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registries = Oracle.vanillaRegistries();
        var registry = net.minecraft.core.registries.BuiltInRegistries.ITEM;
        var items = new JsonArray();
        for (var item : registry) {
            // The components are set in the registry's order: the map they
            // come from iterates by identity hash, which differs by run.
            var builder = net.minecraft.core.component.DataComponentPatch.builder();
            var types = net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE;
            var typed = new java.util.ArrayList<net.minecraft.core.component.TypedDataComponent<?>>();
            item.components().forEach(typed::add);
            typed.sort(java.util.Comparator.comparingInt(component -> types.getId(component.type())));
            for (var component : typed) {
                builder.set(component);
            }
            var patch = builder.build();
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(item).toString());
            described.addProperty("wire", encodePatch(
                net.minecraft.core.component.DataComponentPatch.STREAM_CODEC, patch, registries));
            described.addProperty("delimited", encodePatch(
                net.minecraft.core.component.DataComponentPatch.DELIMITED_STREAM_CODEC, patch, registries));
            var ids = new JsonArray();
            for (var component : patch.entrySet()) {
                if (!component.getKey().isTransient()) {
                    ids.add(types.getId(component.getKey()));
                }
            }
            described.add("components", ids);
            items.add(described);
        }
        entry.add("items", items);
        return entry;
    }

    /**
     * The registries an item's tooltip reads its texts from, by registry id:
     * what each entry is called and, where the reference gives one, the
     * component it shows, as its JSON.
     */
    static JsonObject tooltipRegistries(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registries = Oracle.vanillaRegistries();
        entry.add("enchantments", listRegistry(registries, net.minecraft.core.registries.Registries.ENCHANTMENT,
            (enchantment, described) -> {
                described.add("description", componentJson(enchantment.description()));
                var holder = registries.lookupOrThrow(net.minecraft.core.registries.Registries.ENCHANTMENT)
                    .wrapAsHolder(enchantment);
                described.addProperty("curse", holder.is(net.minecraft.tags.EnchantmentTags.CURSE));
                described.addProperty("max_level", enchantment.getMaxLevel());
                int order = -1;
                var tag = registries.lookupOrThrow(net.minecraft.core.registries.Registries.ENCHANTMENT)
                    .get(net.minecraft.tags.EnchantmentTags.TOOLTIP_ORDER);
                if (tag.isPresent()) {
                    int index = 0;
                    for (var member : tag.get()) {
                        if (member.value() == enchantment) {
                            order = index;
                        }
                        index++;
                    }
                }
                described.addProperty("tooltip_order", order);
            }));
        entry.add("painting_variants", listRegistry(registries, net.minecraft.core.registries.Registries.PAINTING_VARIANT,
            (variant, described) -> {
                described.addProperty("width", variant.width());
                described.addProperty("height", variant.height());
                variant.title().ifPresent(title -> described.add("title", componentJson(title)));
                variant.author().ifPresent(author -> described.add("author", componentJson(author)));
            }));
        entry.add("instruments", listRegistry(registries, net.minecraft.core.registries.Registries.INSTRUMENT,
            (instrument, described) -> described.add("description", componentJson(instrument.description()))));
        entry.add("jukebox_songs", listRegistry(registries, net.minecraft.core.registries.Registries.JUKEBOX_SONG,
            (song, described) -> described.add("description", componentJson(song.description()))));
        entry.add("banner_patterns", listRegistry(registries, net.minecraft.core.registries.Registries.BANNER_PATTERN,
            (pattern, described) -> described.addProperty("translation_key", pattern.translationKey())));
        entry.add("trim_materials", listRegistry(registries, net.minecraft.core.registries.Registries.TRIM_MATERIAL,
            (material, described) -> described.add("description", componentJson(material.description()))));
        entry.add("trim_patterns", listRegistry(registries, net.minecraft.core.registries.Registries.TRIM_PATTERN,
            (pattern, described) -> described.add("description", componentJson(pattern.description()))));
        var attributes = new JsonArray();
        var attributeRegistry = net.minecraft.core.registries.BuiltInRegistries.ATTRIBUTE;
        for (var attribute : attributeRegistry) {
            var described = new JsonObject();
            described.addProperty("name", attributeRegistry.getKey(attribute).toString());
            described.addProperty("description_id", attribute.getDescriptionId());
            described.addProperty("style_up", attribute.getStyle(true).name());
            described.addProperty("style_down", attribute.getStyle(false).name());
            attributes.add(described);
        }
        entry.add("attributes", attributes);
        return entry;
    }

    /**
     * The potions and the mob effects they are made of: what a potion's
     * colour and name are read from, by registry id.
     */
    static JsonObject potions(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        Oracle.vanillaRegistries();
        var effectRegistry = net.minecraft.core.registries.BuiltInRegistries.MOB_EFFECT;
        var effects = new JsonArray();
        for (var effect : effectRegistry) {
            var described = new JsonObject();
            described.addProperty("name", effectRegistry.getKey(effect).toString());
            described.addProperty("color", effect.getColor());
            described.addProperty("description_id", effect.getDescriptionId());
            described.addProperty("style", effect.getCategory().getTooltipFormatting().name());
            var modifiers = new JsonArray();
            var attributeRegistry = net.minecraft.core.registries.BuiltInRegistries.ATTRIBUTE;
            effect.createModifiers(0, (attribute, modifier) -> {
                var described_modifier = new JsonObject();
                described_modifier.addProperty("attribute", attributeRegistry.getId(attribute.value()));
                described_modifier.addProperty("amount", modifier.amount());
                described_modifier.addProperty("operation", modifier.operation().id());
                modifiers.add(described_modifier);
            });
            described.add("modifiers", modifiers);
            effects.add(described);
        }
        entry.add("effects", effects);
        var potionRegistry = net.minecraft.core.registries.BuiltInRegistries.POTION;
        var potions = new JsonArray();
        for (var potion : potionRegistry) {
            var described = new JsonObject();
            described.addProperty("name", potionRegistry.getKey(potion).toString());
            described.addProperty("potion_name", potion.name());
            var list = new JsonArray();
            for (var instance : potion.getEffects()) {
                var item = new JsonObject();
                item.addProperty("effect", effectRegistry.getId(instance.getEffect().value()));
                item.addProperty("amplifier", instance.getAmplifier());
                item.addProperty("duration", instance.getDuration());
                item.addProperty("visible", instance.isVisible());
                list.add(item);
            }
            described.add("effects", list);
            potions.add(described);
        }
        entry.add("potions", potions);
        return entry;
    }

    /**
     * The creative inventory's tabs and what each shows, as the client builds
     * them for the default feature flags and a player without operator
     * permissions, with the client's registries. A stack is its item and
     * component patch as the network encodes it.
     */
    static JsonObject creativeTabs(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registries = Oracle.vanillaRegistries();
        net.minecraft.world.item.CreativeModeTabs.tryRebuildTabContents(
            net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS, false, registries);
        var registry = net.minecraft.core.registries.BuiltInRegistries.CREATIVE_MODE_TAB;
        entry.addProperty("default_tab", registry.getKey(net.minecraft.world.item.CreativeModeTabs.getDefaultTab()).toString());
        var tabs = new JsonArray();
        for (var tab : registry) {
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(tab).toString());
            described.addProperty("type", tab.getType().name());
            described.addProperty("row", tab.row().name());
            described.addProperty("column", tab.column());
            described.addProperty("aligned_right", tab.isAlignedRight());
            described.addProperty("show_title", tab.showTitle());
            described.addProperty("can_scroll", tab.canScroll());
            described.addProperty("should_display", tab.shouldDisplay());
            described.addProperty("background", tab.getBackgroundTexture().toString());
            var contents = tab.getDisplayName().getContents();
            described.addProperty("title", contents instanceof net.minecraft.network.chat.contents.TranslatableContents translatable
                ? translatable.getKey() : "");
            described.add("icon", creativeStack(tab.getIconItem(), registries));
            var items = new JsonArray();
            for (var stack : tab.getDisplayItems()) {
                items.add(creativeStack(stack, registries));
            }
            described.add("items", items);
            var searchItems = new JsonArray();
            for (var stack : tab.getSearchTabDisplayItems()) {
                searchItems.add(creativeStack(stack, registries));
            }
            described.add("search_items", searchItems);
            tabs.add(described);
        }
        entry.add("tabs", tabs);
        return entry;
    }

    static JsonObject creativeStack(net.minecraft.world.item.ItemStack stack,
            net.minecraft.core.RegistryAccess registries) {
        var described = new JsonObject();
        var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(io.netty.buffer.Unpooled.buffer(), registries);
        net.minecraft.world.item.ItemStack.OPTIONAL_STREAM_CODEC.encode(buffer, stack);
        var bytes = new byte[buffer.readableBytes()];
        buffer.readBytes(bytes);
        var hex = new StringBuilder();
        for (byte value : bytes) {
            hex.append(String.format("%02x", value));
        }
        described.addProperty("item", net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
        described.addProperty("wire", hex.toString());
        var positive = new JsonArray();
        var negative = new JsonArray();
        var types = net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE;
        for (var component : stack.getComponentsPatch().entrySet()) {
            (component.getValue().isPresent() ? positive : negative).add(types.getId(component.getKey()));
        }
        described.add("positive", positive);
        described.add("negative", negative);
        // What the patch adds, by component name, as the codec writes it.
        var ops = net.minecraft.resources.RegistryOps.create(com.mojang.serialization.JsonOps.INSTANCE, registries);
        var added = new JsonObject();
        for (var component : stack.getComponentsPatch().entrySet()) {
            if (component.getValue().isPresent()) {
                var typed = net.minecraft.core.component.TypedDataComponent.createUnchecked(
                    component.getKey(), component.getValue().get());
                added.add(types.getKey(component.getKey()).toString(),
                    typed.encodeValue(ops).result().orElse(com.google.gson.JsonNull.INSTANCE));
            }
        }
        described.add("components", sortedObject(added));
        return described;
    }
}
