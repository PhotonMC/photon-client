import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** What block and item classes declare, read through their own methods. */
class BlockDeclarationOracle extends BlockItemOracle {
    /** The block methods the client ports class by class, with their
     * parameters, in the order the declarations list them. */
    static final java.util.Map<String, Class<?>[]> TRACKED_METHODS = trackedMethods();

    static java.util.Map<String, Class<?>[]> trackedMethods() {
        Class<?> state = net.minecraft.world.level.block.state.BlockState.class;
        Class<?> pos = net.minecraft.core.BlockPos.class;
        Class<?> level = net.minecraft.world.level.Level.class;
        Class<?> reader = net.minecraft.world.level.LevelReader.class;
        Class<?> player = net.minecraft.world.entity.player.Player.class;
        Class<?> hit = net.minecraft.world.phys.BlockHitResult.class;
        var methods = new java.util.LinkedHashMap<String, Class<?>[]>();
        methods.put("getStateForPlacement",
            new Class<?>[] {net.minecraft.world.item.context.BlockPlaceContext.class});
        methods.put("canBeReplaced",
            new Class<?>[] {state, net.minecraft.world.item.context.BlockPlaceContext.class});
        methods.put("canSurvive", new Class<?>[] {state, reader, pos});
        methods.put("useWithoutItem", new Class<?>[] {state, level, pos, player, hit});
        methods.put("useItemOn", new Class<?>[] {
            net.minecraft.world.item.ItemStack.class, state, level, pos, player,
            net.minecraft.world.InteractionHand.class, hit});
        methods.put("updateShape", new Class<?>[] {
            state, reader, net.minecraft.world.level.ScheduledTickAccess.class, pos,
            net.minecraft.core.Direction.class, pos, state, net.minecraft.util.RandomSource.class});
        Class<?> getter = net.minecraft.world.level.BlockGetter.class;
        Class<?> context = net.minecraft.world.phys.shapes.CollisionContext.class;
        methods.put("getShape", new Class<?>[] {state, getter, pos, context});
        methods.put("getCollisionShape", new Class<?>[] {state, getter, pos, context});
        methods.put("getType", new Class<?>[] {});
        methods.put("getParticlePos", new Class<?>[] {pos});
        return methods;
    }

    /** `Properties.offsetType`, read from what the offset function does: a
     * vertical component anywhere means it moves in all three axes. */
    static String offsetType(net.minecraft.world.level.block.state.BlockState state) {
        if (!state.hasOffsetFunction()) {
            return "NONE";
        }
        for (int x = 0; x < 16; x++) {
            for (int z = 0; z < 16; z++) {
                if (state.getOffset(new net.minecraft.core.BlockPos(x, 0, z)).y != 0.0) {
                    return "XYZ";
                }
            }
        }
        return "XZ";
    }

    /** An interface and every interface it extends, each listed once. */
    static void collectInterface(
        Class<?> type, java.util.Map<String, java.util.List<String>> seen) {
        if (seen.containsKey(type.getSimpleName())) {
            return;
        }
        var parents = new java.util.ArrayList<String>();
        seen.put(type.getSimpleName(), parents);
        for (var parent : type.getInterfaces()) {
            parents.add(parent.getSimpleName());
            collectInterface(parent, seen);
        }
    }

    /** The static field a state property is kept in: `BlockStateProperties`
     * first, then the block's own classes. */
    static String propertyField(Object property, Class<?> type) {
        var owners = new java.util.ArrayList<Class<?>>();
        owners.add(net.minecraft.world.level.block.state.properties.BlockStateProperties.class);
        for (Class<?> current = type; current != null; current = current.getSuperclass()) {
            owners.add(current);
        }
        for (var owner : owners) {
            for (var field : owner.getDeclaredFields()) {
                if (!java.lang.reflect.Modifier.isStatic(field.getModifiers())) {
                    continue;
                }
                try {
                    field.setAccessible(true);
                    if (field.get(null) == property) {
                        return owner == owners.get(0) ? field.getName()
                            : owner.getSimpleName() + "." + field.getName();
                    }
                } catch (ReflectiveOperationException | RuntimeException skipped) {
                    // Not readable; not the one.
                }
            }
        }
        throw new IllegalStateException("No field holds " + property + " for " + type);
    }

    static Object declaredValue(Object properties, String field) throws ReflectiveOperationException {
        var reflected = net.minecraft.world.level.block.state.BlockBehaviour.Properties.class.getDeclaredField(field);
        reflected.setAccessible(true);
        return reflected.get(properties);
    }

    static boolean sameAsFresh(Object properties, Object fresh, String field)
        throws ReflectiveOperationException {
        return declaredValue(properties, field) == declaredValue(fresh, field);
    }

    static JsonArray intsOf(
        java.util.List<net.minecraft.world.level.block.state.BlockState> states,
        java.util.function.ToIntFunction<net.minecraft.world.level.block.state.BlockState> value) {
        var out = new JsonArray();
        states.forEach(state -> out.add(value.applyAsInt(state)));
        return out;
    }

    static JsonArray boolsOf(
        java.util.List<net.minecraft.world.level.block.state.BlockState> states,
        java.util.function.Predicate<net.minecraft.world.level.block.state.BlockState> value) {
        var out = new JsonArray();
        states.forEach(state -> out.add(value.test(state)));
        return out;
    }

    /** The item methods the client ports class by class. */
    static final java.util.Map<String, Class<?>[]> TRACKED_ITEM_METHODS = trackedItemMethods();

    static java.util.Map<String, Class<?>[]> trackedItemMethods() {
        Class<?> stack = net.minecraft.world.item.ItemStack.class;
        Class<?> slot = net.minecraft.world.inventory.Slot.class;
        Class<?> click = net.minecraft.world.inventory.ClickAction.class;
        Class<?> player = net.minecraft.world.entity.player.Player.class;
        var methods = new java.util.LinkedHashMap<String, Class<?>[]>();
        methods.put("useOn", new Class<?>[] {net.minecraft.world.item.context.UseOnContext.class});
        methods.put("use", new Class<?>[] {
            net.minecraft.world.level.Level.class, player, net.minecraft.world.InteractionHand.class});
        methods.put("overrideStackedOnOther", new Class<?>[] {stack, slot, click, player});
        methods.put("overrideOtherStackedOnMe", new Class<?>[] {
            stack, stack, slot, click, player, net.minecraft.world.entity.SlotAccess.class});
        return methods;
    }

    /** The default components the client reads from an item. */
    static final java.util.Set<String> READ_COMPONENTS = java.util.Set.of(
        "minecraft:item_name", "minecraft:rarity", "minecraft:attribute_modifiers",
        "minecraft:enchantment_glint_override", "minecraft:map_color", "minecraft:potion_contents",
        "minecraft:potion_duration_scale", "minecraft:fireworks", "minecraft:instrument",
        "minecraft:jukebox_playable", "minecraft:block_state", "minecraft:ominous_bottle_amplifier");

    /**
     * What each block declares, as `Blocks` builds it: its class and the
     * classes above it, its state properties by the field that holds them,
     * and its `Properties`. A state predicate or function reads as `default`
     * when it is the one `Properties.of()` starts with; otherwise its value
     * for every state. Nothing here is computed from shapes.
     */
    static JsonObject blockDeclarations(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        try {
            var propertiesField = net.minecraft.world.level.block.state.BlockBehaviour.class
                .getDeclaredField("properties");
            propertiesField.setAccessible(true);
            var fresh = net.minecraft.world.level.block.state.BlockBehaviour.Properties.of();
            var classes = new java.util.LinkedHashMap<String, String>();
            var classTypes = new java.util.HashMap<String, Class<?>>();
            var propertyFields = new java.util.IdentityHashMap<Object, String>();
            var propertyList = new JsonArray();
            var blocks = new JsonArray();
            for (var block : registry) {
                for (Class<?> type = block.getClass();
                     type != net.minecraft.world.level.block.state.BlockBehaviour.class;
                     type = type.getSuperclass()) {
                    var parent = type.getSuperclass();
                    classTypes.putIfAbsent(type.getSimpleName(), type);
                    classes.putIfAbsent(type.getSimpleName(),
                        parent == net.minecraft.world.level.block.state.BlockBehaviour.class
                            ? "" : parent.getSimpleName());
                }
                var described = new JsonObject();
                described.addProperty("name", registry.getKey(block).toString());
                described.addProperty("class", block.getClass().getSimpleName());
                var definition = block.getStateDefinition();
                var names = new JsonArray();
                var defaults = new JsonArray();
                for (var property : definition.getProperties()) {
                    String field = propertyFields.get(property);
                    if (field == null) {
                        field = propertyField(property, block.getClass());
                        propertyFields.put(property, field);
                        var describedProperty = new JsonObject();
                        describedProperty.addProperty("field", field);
                        describedProperty.addProperty("name", property.getName());
                        describedProperty.addProperty("kind", switch (property) {
                            case net.minecraft.world.level.block.state.properties.BooleanProperty p -> "boolean";
                            case net.minecraft.world.level.block.state.properties.IntegerProperty p -> "integer";
                            case net.minecraft.world.level.block.state.properties.EnumProperty p -> "enum";
                            default -> "other";
                        });
                        var values = new JsonArray();
                        for (var value : property.getPossibleValues()) {
                            values.add(name(property, (Comparable) value));
                        }
                        describedProperty.add("values", values);
                        propertyList.add(describedProperty);
                    }
                    names.add(field);
                    defaults.add(name(property, block.defaultBlockState().getValue(property)));
                }
                described.add("properties", names);
                described.add("default", defaults);
                var declared = (net.minecraft.world.level.block.state.BlockBehaviour.Properties)
                    propertiesField.get(block);
                var values = new JsonObject();
                for (String field : new String[] {
                    "destroyTime", "explosionResistance", "friction", "speedFactor", "jumpFactor",
                    "bounceRestitution",
                }) {
                    values.addProperty(field, Oracle.bits((float) declaredValue(declared, field)));
                }
                for (String field : new String[] {
                    "hasCollision", "requiresCorrectToolForDrops", "isRandomlyTicking", "canOcclude",
                    "isAir", "ignitedByLava", "liquid", "forceSolidOff", "forceSolidOn",
                    "spawnTerrainParticles", "replaceable", "dynamicShape",
                }) {
                    values.addProperty(field, (boolean) declaredValue(declared, field));
                }
                values.addProperty("pushReaction", declaredValue(declared, "pushReaction").toString());
                values.addProperty("offsetType", offsetType(block.defaultBlockState()));
                described.add("declared", values);
                described.addProperty("item", net.minecraft.core.registries.BuiltInRegistries.ITEM
                    .getKey(block.asItem()).toString());
                if (block instanceof net.minecraft.world.level.block.LiquidBlock) {
                    var fluid = net.minecraft.world.level.block.LiquidBlock.class.getDeclaredField("fluid");
                    fluid.setAccessible(true);
                    described.addProperty("fluid", net.minecraft.core.registries.BuiltInRegistries.FLUID
                        .getKey((net.minecraft.world.level.material.Fluid) fluid.get(block)).toString());
                }
                var states = definition.getPossibleStates();
                var perState = new JsonObject();
                var getter = net.minecraft.world.level.EmptyBlockGetter.INSTANCE;
                var origin = net.minecraft.core.BlockPos.ZERO;
                perState.add("lightEmission", sameAsFresh(declared, fresh, "lightEmission") ? null
                    : intsOf(states, state -> state.getLightEmission()));
                perState.add("emissiveRendering", sameAsFresh(declared, fresh, "emissiveRendering") ? null
                    : boolsOf(states, state -> state.emissiveRendering()));
                perState.add("isSuffocating", sameAsFresh(declared, fresh, "isSuffocating") ? null
                    : boolsOf(states, state -> state.isSuffocating(getter, origin)));
                perState.add("isViewBlocking", sameAsFresh(declared, fresh, "isViewBlocking") ? null
                    : boolsOf(states, state -> state.isViewBlocking(getter, origin)));
                perState.add("isRedstoneConductor", sameAsFresh(declared, fresh, "isRedstoneConductor") ? null
                    : boolsOf(states, state -> state.isRedstoneConductor(getter, origin)));
                described.add("per_state", perState);
                blocks.add(described);
            }
            var classList = new JsonArray();
            var interfaceParents = new java.util.LinkedHashMap<String, java.util.List<String>>();
            classes.forEach((name, parent) -> {
                var described = new JsonObject();
                described.addProperty("name", name);
                described.addProperty("parent", parent);
                var interfaces = new JsonArray();
                for (var type : classTypes.get(name).getInterfaces()) {
                    interfaces.add(type.getSimpleName());
                    collectInterface(type, interfaceParents);
                }
                described.add("interfaces", interfaces);
                var declares = new JsonArray();
                for (var method : TRACKED_METHODS.entrySet()) {
                    try {
                        var found = classTypes.get(name).getDeclaredMethod(method.getKey(), method.getValue());
                        // `getType` is also a trap door's and a skull's; only a bone meal type counts.
                        if (!method.getKey().equals("getType")
                            || found.getReturnType() == net.minecraft.world.level.block.BonemealableBlock.Type.class) {
                            declares.add(method.getKey());
                        }
                    } catch (NoSuchMethodException absent) {
                        // Inherited, not its own.
                    }
                }
                described.add("declares", declares);
                classList.add(described);
            });
            entry.add("classes", classList);
            var interfaceList = new JsonArray();
            interfaceParents.forEach((name, parents) -> {
                var described = new JsonObject();
                described.addProperty("name", name);
                var listed = new JsonArray();
                parents.forEach(listed::add);
                described.add("parents", listed);
                interfaceList.add(described);
            });
            entry.add("interfaces", interfaceList);
            entry.add("properties", propertyList);
            entry.add("blocks", blocks);
        } catch (ReflectiveOperationException failure) {
            throw new IllegalStateException(failure);
        }
        return entry;
    }

    /**
     * What each item declares, as `Items` builds it: its class with the
     * classes above it and the tracked methods each declares, and the
     * `Item.Properties` the client reads: stack size, durability, the
     * equippable and tool components, the crafting remainder, whether
     * using it does anything by default, and the block a block item places.
     */
    static JsonObject itemDeclarations(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "listed");
        var registry = net.minecraft.core.registries.BuiltInRegistries.ITEM;
        var blockRegistry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        var playerType = net.minecraft.world.entity.EntityTypes.PLAYER.builtInRegistryHolder();
        var components = net.minecraft.core.component.DataComponents.class;
        var classes = new java.util.LinkedHashMap<String, Class<?>>();
        var items = new JsonArray();
        // What the client burns an item for, from the registries and tags it
        // was given.
        var fuel = net.minecraft.world.level.block.entity.FuelValues.vanillaBurnTimes(
            Oracle.vanillaRegistries(), net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        // The ingredients of the brewing stand (the client's own `PotionBrewing`).
        var brewing = net.minecraft.world.item.alchemy.PotionBrewing.bootstrap(
            net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        for (var item : registry) {
            for (Class<?> type = item.getClass();
                 type != null && type != Object.class;
                 type = type.getSuperclass()) {
                classes.putIfAbsent(type.getSimpleName(), type);
                if (type == net.minecraft.world.item.Item.class) {
                    break;
                }
            }
            var data = item.components();
            var described = new JsonObject();
            described.addProperty("name", registry.getKey(item).toString());
            described.addProperty("class", item.getClass().getSimpleName());
            described.addProperty("max_stack_size",
                data.getOrDefault(net.minecraft.core.component.DataComponents.MAX_STACK_SIZE, 1));
            described.addProperty("max_damage",
                data.getOrDefault(net.minecraft.core.component.DataComponents.MAX_DAMAGE, 0));
            var equippable = data.get(net.minecraft.core.component.DataComponents.EQUIPPABLE);
            if (equippable != null) {
                var equip = new JsonObject();
                equip.addProperty("slot", equippable.slot().getSerializedName());
                equip.addProperty("player", equippable.canBeEquippedBy(playerType));
                equip.addProperty("swappable", equippable.swappable());
                equippable.assetId().ifPresent(asset -> equip.addProperty("asset", asset.identifier().toString()));
                equippable.cameraOverlay().ifPresent(overlay -> equip.addProperty("camera_overlay", overlay.toString()));
                described.add("equippable", equip);
            }
            var remainder = item.getCraftingRemainder();
            if (remainder != null) {
                var stack = remainder.create();
                var left = new JsonObject();
                left.addProperty("item", registry.getKey(stack.getItem()).toString());
                left.addProperty("count", stack.getCount());
                described.add("crafting_remainder", left);
            }
            var tool = data.get(net.minecraft.core.component.DataComponents.TOOL);
            if (tool != null) {
                described.add("tool", toolJson(tool));
            }
            if (brewing.isIngredient(new net.minecraft.world.item.ItemStack(item))) {
                described.addProperty("brewing_ingredient", true);
            }
            int burn = fuel.burnDuration(new net.minecraft.world.item.ItemStack(item));
            if (burn > 0) {
                described.addProperty("burn_time", burn);
            }
            described.addProperty("consumable",
                data.has(net.minecraft.core.component.DataComponents.CONSUMABLE));
            var consumable = data.get(net.minecraft.core.component.DataComponents.CONSUMABLE);
            if (consumable != null) {
                described.addProperty("consume_seconds", consumable.consumeSeconds());
                described.addProperty("use_animation", consumable.animation().getSerializedName());
                described.addProperty("consume_sound", consumable.sound().getRegisteredName());
                described.addProperty("consume_particles", consumable.hasConsumeParticles());
            }
            var useEffects = data.get(net.minecraft.core.component.DataComponents.USE_EFFECTS);
            if (useEffects != null) {
                described.addProperty("use_can_sprint", useEffects.canSprint());
                described.addProperty("use_speed", useEffects.speedMultiplier());
            }
            described.addProperty("blocks_attacks",
                data.has(net.minecraft.core.component.DataComponents.BLOCKS_ATTACKS));
            described.addProperty("kinetic_weapon",
                data.has(net.minecraft.core.component.DataComponents.KINETIC_WEAPON));
            var kinetic = data.get(net.minecraft.core.component.DataComponents.KINETIC_WEAPON);
            if (kinetic != null && kinetic.hitSound().isPresent()) {
                described.addProperty("kinetic_hit_sound", kinetic.hitSound().get().getRegisteredName());
            }
            described.addProperty("glider",
                data.has(net.minecraft.core.component.DataComponents.GLIDER));
            if (item instanceof net.minecraft.world.item.BlockItem blockItem) {
                described.addProperty("block", blockRegistry.getKey(blockItem.getBlock()).toString());
            }
            // The default components the client reads, as the component's own
            // codec writes them and as the network writes them. A value the
            // client cannot tell from the component being absent (common
            // rarity, no attribute modifiers) is left out.
            var componentJson = new JsonObject();
            var componentWire = new JsonObject();
            var registries = Oracle.vanillaRegistries();
            var ops = net.minecraft.resources.RegistryOps.create(
                com.mojang.serialization.JsonOps.INSTANCE, registries);
            for (var component : data) {
                var key = net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE
                    .getKey(component.type()).toString();
                if (!READ_COMPONENTS.contains(key)) {
                    continue;
                }
                var encoded = component.encodeValue(ops).result().orElseThrow();
                if (key.equals("minecraft:rarity") && encoded.getAsString().equals("common")) {
                    continue;
                }
                if (key.equals("minecraft:attribute_modifiers") && encoded.getAsJsonArray().isEmpty()) {
                    continue;
                }
                componentJson.add(key, encoded);
                componentWire.addProperty(key, encodeComponentValue(component, registries));
            }
            described.add("components", sortedObject(componentJson));
            described.add("component_wire", sortedObject(componentWire));
            items.add(described);
        }
        var classList = new JsonArray();
        classes.forEach((name, type) -> {
            var described = new JsonObject();
            described.addProperty("name", name);
            described.addProperty("parent",
                type == net.minecraft.world.item.Item.class ? "" : type.getSuperclass().getSimpleName());
            var declares = new JsonArray();
            for (var method : TRACKED_ITEM_METHODS.entrySet()) {
                try {
                    type.getDeclaredMethod(method.getKey(), method.getValue());
                    declares.add(method.getKey());
                } catch (NoSuchMethodException absent) {
                    // Inherited, not its own.
                }
            }
            described.add("declares", declares);
            classList.add(described);
        });
        entry.add("classes", classList);
        entry.add("items", items);
        var componentTypes = new JsonArray();
        for (var type : net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE) {
            componentTypes.add(net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE
                .getKey(type).toString());
        }
        entry.add("component_types", componentTypes);
        return entry;
    }
}
