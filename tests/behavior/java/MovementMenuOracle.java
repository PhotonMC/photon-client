import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** A menu opened in a scene, and clicks on its slots. */
class MovementMenuOracle extends MovementPlayerOracle {
    /**
     * Clicks in the player's own inventory menu, as the inventory screen
     * sends them: `MultiPlayerGameMode.handleContainerInput` runs the menu's
     * click as the client predicts it and sends the click packet. After
     * every click the packets sent, as their codecs write them, every slot
     * and the carried stack are reported. A stack may carry a component
     * patch as the wire carries it; the click packet hashes it as
     * `ClientPacketListener.decoratedHashOpsGenerator` does.
     */
    /** A menu and the game around it: the scene, the player and the menu, slots and data set. */
    record OpenMenu(Scene scene, net.minecraft.client.player.LocalPlayer player,
                    net.minecraft.world.inventory.AbstractContainerMenu menu, int containerId) {}

    /**
     * The menu `ClientPacketListener.handleMountScreenOpen` builds for a mount: its entity made as
     * the data says (tame, baby), its chest a container of the columns' three rows.
     */
    static net.minecraft.world.inventory.AbstractContainerMenu mountMenu(
        JsonObject mount, int containerId, net.minecraft.client.player.LocalPlayer player,
        net.minecraft.client.multiplayer.ClientLevel level) throws Exception {
        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(mount.get("entity").getAsString()));
        var entity = type.create(level, net.minecraft.world.entity.EntitySpawnReason.LOAD);
        int columns = mount.get("columns").getAsInt();
        if (mount.get("baby").getAsBoolean()) {
            ((net.minecraft.world.entity.AgeableMob) entity).setBaby(true);
        }
        var container = new net.minecraft.world.SimpleContainer(
            net.minecraft.world.inventory.AbstractMountInventoryMenu.getInventorySize(columns));
        if (entity instanceof net.minecraft.world.entity.animal.equine.AbstractHorse horse) {
            horse.setTamed(mount.get("tame").getAsBoolean());
            return new net.minecraft.world.inventory.HorseInventoryMenu(
                containerId, player.getInventory(), container, horse, columns);
        }
        var nautilus = (net.minecraft.world.entity.animal.nautilus.AbstractNautilus) entity;
        nautilus.setTame(mount.get("tame").getAsBoolean(), false);
        return new net.minecraft.world.inventory.NautilusInventoryMenu(
            containerId, player.getInventory(), container, nautilus, columns);
    }

    static OpenMenu openMenu(JsonObject row) throws Exception {
        var scene = buildScene(row);
        var listener = scene.listener();
        var minecraft = scene.minecraft();
        var level = scene.level();
        // The client's own hasher, over the vanilla registries.
        setField(listener, "decoratedHashOpsGenerator", hashGenerator());
        var player = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        player.setId(1);
        setField(minecraft, "player", player);
        // A bundle's sounds are played where the player is, timed by the camera's distance.
        var camera = new net.minecraft.client.Camera();
        camera.setEntity(player);
        var gameRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        setField(gameRenderer, "mainCamera", camera);
        setField(minecraft, "gameRenderer", gameRenderer);
        var info = new net.minecraft.client.multiplayer.PlayerInfo(listener.getLocalGameProfile(), false);
        var gameType = net.minecraft.world.level.GameType.byName(row.get("game_mode").getAsString());
        var setMode = net.minecraft.client.multiplayer.PlayerInfo.class.getDeclaredMethod(
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.setAccessible(true);
        setMode.invoke(info, gameType);
        setField(player, "playerInfo", info);
        var gameMode = new net.minecraft.client.multiplayer.MultiPlayerGameMode(minecraft, listener);
        gameMode.setLocalMode(gameType);
        gameMode.adjustPlayer(player);
        setField(minecraft, "gameMode", gameMode);
        setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        player.getInventory().setSelectedSlot(row.get("selected").getAsInt());
        // Item cooldowns the server set, then the player's ticks since.
        if (row.has("cooldowns")) {
            for (var element : row.getAsJsonArray("cooldowns")) {
                var cooldown = element.getAsJsonObject();
                player.getCooldowns().addCooldown(
                    net.minecraft.resources.Identifier.parse(cooldown.get("group").getAsString()),
                    cooldown.get("duration").getAsInt());
            }
            for (int tick = 0; tick < row.get("cooldown_ticks").getAsInt(); tick++) {
                player.getCooldowns().tick();
            }
        }
        net.minecraft.world.inventory.AbstractContainerMenu menu = player.inventoryMenu;
        int containerId = 0;
        if (row.has("menu")) {
            // A menu the server opened: built the way the client builds it,
            // over the player's own inventory.
            containerId = row.get("container_id").getAsInt();
            // A furnace menu reads the items its recipes accept from the
            // server's recipe sets, and what burns from the client's fuel
            // values.
            setField(listener, "fuelValues", net.minecraft.world.level.block.entity.FuelValues
                .vanillaBurnTimes(Oracle.vanillaRegistries(), net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS));
            var accepted = new java.util.ArrayList<net.minecraft.world.item.crafting.Ingredient>();
            if (row.has("furnace_inputs")) {
                for (var name : row.getAsJsonArray("furnace_inputs")) {
                    accepted.add(net.minecraft.world.item.crafting.Ingredient.of(
                        net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                            net.minecraft.resources.Identifier.parse(name.getAsString()))));
                }
            }
            var inputs = net.minecraft.world.item.crafting.RecipePropertySet.create(accepted);
            var sets = new java.util.HashMap<net.minecraft.resources.ResourceKey<
                net.minecraft.world.item.crafting.RecipePropertySet>,
                net.minecraft.world.item.crafting.RecipePropertySet>();
            sets.put(net.minecraft.world.item.crafting.RecipePropertySet.FURNACE_INPUT, inputs);
            sets.put(net.minecraft.world.item.crafting.RecipePropertySet.BLAST_FURNACE_INPUT, inputs);
            sets.put(net.minecraft.world.item.crafting.RecipePropertySet.SMOKER_INPUT, inputs);
            if (row.has("recipe_sets")) {
                for (var setEntry : row.getAsJsonObject("recipe_sets").entrySet()) {
                    var members = new java.util.ArrayList<net.minecraft.world.item.crafting.Ingredient>();
                    for (var name : setEntry.getValue().getAsJsonArray()) {
                        members.add(net.minecraft.world.item.crafting.Ingredient.of(
                            net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                                net.minecraft.resources.Identifier.parse(name.getAsString()))));
                    }
                    sets.put(net.minecraft.resources.ResourceKey.create(
                            net.minecraft.world.item.crafting.RecipePropertySet.TYPE_KEY,
                            net.minecraft.resources.Identifier.parse(setEntry.getKey())),
                        net.minecraft.world.item.crafting.RecipePropertySet.create(members));
                }
            }
            var cutter = new java.util.ArrayList<net.minecraft.world.item.crafting.SelectableRecipe.SingleInputEntry<
                net.minecraft.world.item.crafting.StonecutterRecipe>>();
            if (row.has("stonecutter_inputs")) {
                for (var name : row.getAsJsonArray("stonecutter_inputs")) {
                    cutter.add(new net.minecraft.world.item.crafting.SelectableRecipe.SingleInputEntry<>(
                        net.minecraft.world.item.crafting.Ingredient.of(
                            net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                                net.minecraft.resources.Identifier.parse(name.getAsString()))),
                        new net.minecraft.world.item.crafting.SelectableRecipe<>(
                            net.minecraft.world.item.crafting.display.SlotDisplay.AnyFuel.INSTANCE,
                            java.util.Optional.empty())));
                }
            }
            setField(listener, "recipes", new net.minecraft.client.multiplayer.ClientRecipeContainer(
                sets, new net.minecraft.world.item.crafting.SelectableRecipe.SingleInputSet<>(cutter)));
            if (row.has("xp_level")) {
                player.experienceLevel = row.get("xp_level").getAsInt();
            }
            if (row.has("mount")) {
                menu = mountMenu(row.getAsJsonObject("mount"), containerId, player, level);
            } else {
                var type = net.minecraft.core.registries.BuiltInRegistries.MENU.getValue(
                    net.minecraft.resources.Identifier.parse(row.get("menu").getAsString()));
                menu = type.create(containerId, player.getInventory());
            }
            player.containerMenu = menu;
            if (menu instanceof net.minecraft.world.inventory.MerchantMenu merchantMenu) {
                // What the server's merchant offers, and how far it has levelled.
                var offers = new net.minecraft.world.item.trading.MerchantOffers();
                var items = net.minecraft.core.registries.BuiltInRegistries.ITEM;
                java.util.function.Function<com.google.gson.JsonElement, net.minecraft.world.item.trading.ItemCost> cost =
                    pair -> new net.minecraft.world.item.trading.ItemCost(
                        items.getValue(net.minecraft.resources.Identifier.parse(pair.getAsJsonArray().get(0).getAsString())),
                        pair.getAsJsonArray().get(1).getAsInt());
                if (row.has("offers")) {
                    for (var element : row.getAsJsonArray("offers")) {
                        var offer = element.getAsJsonObject();
                        var sell = offer.getAsJsonArray("sell");
                        var made = new net.minecraft.world.item.trading.MerchantOffer(
                            cost.apply(offer.get("buy")),
                            offer.has("buy_b") ? java.util.Optional.of(cost.apply(offer.get("buy_b"))) : java.util.Optional.empty(),
                            new net.minecraft.world.item.ItemStack(
                                items.getValue(net.minecraft.resources.Identifier.parse(sell.get(0).getAsString())),
                                sell.get(1).getAsInt()),
                            offer.get("uses").getAsInt(), offer.get("max_uses").getAsInt(), offer.get("xp").getAsInt(),
                            offer.get("multiplier").getAsFloat(), offer.get("demand").getAsInt());
                        made.setSpecialPriceDiff(offer.get("special").getAsInt());
                        offers.add(made);
                    }
                }
                merchantMenu.setOffers(offers);
                if (row.has("merchant")) {
                    var state = row.getAsJsonObject("merchant");
                    merchantMenu.setXp(state.get("xp").getAsInt());
                    merchantMenu.setMerchantLevel(state.get("level").getAsInt());
                    merchantMenu.setShowProgressBar(state.get("progress").getAsBoolean());
                    merchantMenu.setCanRestock(state.get("restock").getAsBoolean());
                }
            }
        }
        for (var element : row.getAsJsonArray("slots")) {
            var slot = element.getAsJsonObject();
            menu.getSlot(slot.get("slot").getAsInt()).set(stack(slot));
        }
        if (row.has("item_name") && menu instanceof net.minecraft.world.inventory.AnvilMenu anvilMenu) {
            anvilMenu.setItemName(row.get("item_name").getAsString());
        }
        if (row.has("carried")) {
            menu.setCarried(stack(row.getAsJsonObject("carried")));
        }
        if (row.has("select") && menu instanceof net.minecraft.world.inventory.MerchantMenu merchantMenu) {
            // A trade chosen on the screen: tried first, and its payments moved in.
            merchantMenu.setSelectionHint(row.get("select").getAsInt());
            merchantMenu.tryMoveItems(row.get("select").getAsInt());
        }
        // What the server last sent of the menu's data, after the slots are
        // set, which would make a menu work its own out.
        if (row.has("data")) {
            var values = row.getAsJsonArray("data");
            for (int index = 0; index < values.size(); index++) {
                menu.setData(index, values.get(index).getAsInt());
            }
        }
        setField(menu, "stateId", row.get("state_id").getAsInt());
        return new OpenMenu(scene, player, menu, containerId);
    }

    /**
     * A click packet with each stack's components in type id order: the
     * client writes them from an identity map, in an order no port can
     * follow, and the server reads them into a map.
     */
    @SuppressWarnings("unchecked")
    static net.minecraft.network.protocol.Packet<?> canonical(net.minecraft.network.protocol.Packet<?> packet)
        throws Exception {
        if (!(packet instanceof net.minecraft.network.protocol.game.ServerboundContainerClickPacket click)) {
            return packet;
        }
        // The packet keeps an unmodifiable view; the map under it keeps its own order of slots.
        java.lang.reflect.Field field = null;
        for (Class<?> type = click.changedSlots().getClass(); field == null && type != null; type = type.getSuperclass()) {
            try {
                field = type.getDeclaredField("map");
            } catch (NoSuchFieldException missing) {
                // Look further up.
            }
        }
        field.setAccessible(true);
        var slots = (it.unimi.dsi.fastutil.ints.Int2ObjectMap<net.minecraft.network.HashedStack>) field.get(click.changedSlots());
        slots.replaceAll((slot, stack) -> canonical(stack));
        return new net.minecraft.network.protocol.game.ServerboundContainerClickPacket(
            click.containerId(), click.stateId(), click.slotNum(), click.buttonNum(), click.containerInput(),
            slots, canonical(click.carriedItem()));
    }

    static net.minecraft.network.HashedStack canonical(net.minecraft.network.HashedStack stack) {
        if (!(stack instanceof net.minecraft.network.HashedStack.ActualItem item)) {
            return stack;
        }
        var types = net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE;
        java.util.Comparator<net.minecraft.core.component.DataComponentType<?>> byId =
            java.util.Comparator.comparingInt(types::getId);
        var added = new java.util.LinkedHashMap<net.minecraft.core.component.DataComponentType<?>, Integer>();
        item.components().addedComponents().entrySet().stream()
            .sorted(java.util.Map.Entry.comparingByKey(byId))
            .forEach(entry -> added.put(entry.getKey(), entry.getValue()));
        var removed = new java.util.LinkedHashSet<net.minecraft.core.component.DataComponentType<?>>();
        item.components().removedComponents().stream().sorted(byId).forEach(removed::add);
        return new net.minecraft.network.HashedStack.ActualItem(
            item.item(), item.count(), new net.minecraft.network.HashedPatchMap(added, removed));
    }

    /** `ClientPacketListener.decoratedHashOpsGenerator` over the vanilla registries. */
    static net.minecraft.network.HashedPatchMap.HashGenerator hashGenerator() {
        var ops = Oracle.vanillaRegistries().createSerializationContext(
            net.minecraft.util.HashOps.CRC32C_INSTANCE);
        return component -> component.encodeValue(ops).getOrThrow().asInt();
    }

    /**
     * The hash the client gives each component a stack's patch adds, and the
     * components it removes, by name.
     */
    static JsonObject componentHashes(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var split = stack(row).getComponentsPatch().split();
        var hasher = hashGenerator();
        var added = new java.util.TreeMap<String, Integer>();
        for (var component : split.added()) {
            added.put(net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE
                .getKey(component.type()).toString(), hasher.apply(component));
        }
        var hashes = new JsonObject();
        added.forEach(hashes::addProperty);
        entry.add("added", hashes);
        var removed = new java.util.TreeSet<String>();
        for (var type : split.removed()) {
            removed.add(net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_TYPE.getKey(type).toString());
        }
        var names = new JsonArray();
        removed.forEach(names::add);
        entry.add("removed", names);
        return entry;
    }

    static JsonObject containerClicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var open = openMenu(row);
        var scene = open.scene();
        var listener = scene.listener();
        var minecraft = scene.minecraft();
        var player = open.player();
        var menu = open.menu();
        int containerId = open.containerId();
        var gameMode = minecraft.gameMode;
        var wire = scene.wire();
        var clicks = new JsonArray();
        for (var element : row.getAsJsonArray("clicks")) {
            var click = element.getAsJsonObject();
            wire.sent.clear();
            SilentSoundManager.PLAYED.clear();
            var type = click.get("type").getAsString();
            if (type.equals("SCROLL") || type.equals("UNHOVER")) {
                // The screen's bundle actions over a slot: the wheel (by whole steps) or the
                // mouse leaving it.
                var actions = new net.minecraft.client.gui.BundleMouseActions(minecraft);
                var slot = menu.getSlot(click.get("slot").getAsInt());
                if (slot.hasItem() && actions.matches(slot)) {
                    if (type.equals("SCROLL")) {
                        actions.onMouseScrolled(0.0, click.get("button").getAsInt(), slot.index, slot.getItem());
                    } else {
                        actions.onStopHovering(slot);
                    }
                }
            } else {
                gameMode.handleContainerInput(containerId, click.get("slot").getAsInt(), click.get("button").getAsInt(),
                    net.minecraft.world.inventory.ContainerInput.valueOf(type), player);
            }
            var described = new JsonObject();
            var packets = new JsonArray();
            for (var sentPacket : wire.sent) {
                var packet = canonical(sentPacket);
                var packetJson = new JsonObject();
                packetJson.addProperty("type", packet.type().id().toString());
                packetJson.addProperty("bytes", encode(packet));
                packets.add(packetJson);
            }
            described.add("sent", packets);
            // A bundle's sounds, by name: their pitch is drawn from the level's unseeded random.
            if (!SilentSoundManager.PLAYED.isEmpty()) {
                var sounds = new JsonArray();
                for (var played : SilentSoundManager.PLAYED) {
                    sounds.add(played.substring(0, played.indexOf(',')));
                }
                described.add("sounds", sounds);
            }
            var slots = new JsonArray();
            for (var slot : menu.slots) {
                slots.add(describeStack(slot.getItem()));
            }
            described.add("slots", slots);
            // The bundles' selected items, which the wire does not carry: [slot, index] for each
            // with one (-1 for the carried stack).
            var selections = new JsonArray();
            for (var slot : menu.slots) {
                int index = net.minecraft.world.item.BundleItem.getSelectedItemIndex(slot.getItem());
                if (index != -1) {
                    var pair = new JsonArray();
                    pair.add(slot.index);
                    pair.add(index);
                    selections.add(pair);
                }
            }
            int carriedIndex = net.minecraft.world.item.BundleItem.getSelectedItemIndex(menu.getCarried());
            if (carriedIndex != -1) {
                var pair = new JsonArray();
                pair.add(-1);
                pair.add(carriedIndex);
                selections.add(pair);
            }
            if (!selections.isEmpty()) {
                described.add("bundle_selections", selections);
            }
            described.add("carried", describeStack(menu.getCarried()));
            described.addProperty("selected", player.getInventory().getSelectedSlot());
            clicks.add(described);
        }
        entry.addProperty("outcome", "clicked");
        entry.add("clicks", clicks);
        return entry;
    }
}
