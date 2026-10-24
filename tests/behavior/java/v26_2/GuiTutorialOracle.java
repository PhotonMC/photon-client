import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * The tutorial (`Tutorial` and its step instances), driven through scripts of events: ticks with
 * the keys held and the mouse moves, a block looked at or broken, the inventory opened, an item
 * got, the game mode. Each event is answered with the step the options keep and the tutorial
 * toasts of the real toast manager, in the order they were added.
 */
class GuiTutorialOracle extends GuiItemOracle {
    /** A player's inventory that holds the row's stacks and nothing else. */
    static class RowInventory extends net.minecraft.world.entity.player.Inventory {
        java.util.List<net.minecraft.world.item.ItemStack> held = new java.util.ArrayList<>();

        RowInventory() {
            super(null, null);
        }

        @Override
        public boolean contains(net.minecraft.tags.TagKey<net.minecraft.world.item.Item> tag) {
            return held.stream().anyMatch(stack -> stack.is(tag));
        }

        @Override
        public boolean hasAnyMatching(java.util.function.Predicate<net.minecraft.world.item.ItemStack> predicate) {
            return held.stream().anyMatch(predicate);
        }
    }

    /** A level whose every block is the one the row names. */
    static class RowLevel extends net.minecraft.client.multiplayer.ClientLevel {
        net.minecraft.world.level.block.state.BlockState state;

        RowLevel() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
        }

        @Override
        public net.minecraft.world.level.block.state.BlockState getBlockState(net.minecraft.core.BlockPos pos) {
            return state;
        }
    }

    /** A connection that hears the options being sent and does nothing with them. */
    static class QuietListener extends net.minecraft.client.multiplayer.ClientPacketListener {
        QuietListener() {
            super(null, null, null);
        }

        @Override
        public void broadcastClientInformation(net.minecraft.server.level.ClientInformation information) {}
    }

    private static net.minecraft.world.level.block.Block block(String name) {
        return net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(
            net.minecraft.resources.Identifier.parse(name));
    }

    private static net.minecraft.world.item.Item item(String name) {
        return net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
            net.minecraft.resources.Identifier.parse(name));
    }

    private static JsonArray toasts(net.minecraft.client.gui.components.toasts.ToastManager manager)
            throws Exception {
        var out = new JsonArray();
        var visible = (java.util.List<?>) field(manager, "visibleToasts");
        var all = new java.util.ArrayList<Object[]>();
        for (var instance : visible) {
            all.add(new Object[] {false, field(instance, "toast")});
        }
        for (var toast : (java.util.Collection<?>) field(manager, "queued")) {
            all.add(new Object[] {true, toast});
        }
        for (var entry : all) {
            if (!(entry[1] instanceof net.minecraft.client.gui.components.toasts.TutorialToast toast)) {
                continue;
            }
            var json = new JsonObject();
            json.addProperty("waiting", (boolean) entry[0]);
            json.addProperty("icon", ((Enum<?>) field(toast, "icon")).ordinal());
            json.addProperty("hidden", toast.getWantedVisibility()
                == net.minecraft.client.gui.components.toasts.Toast.Visibility.HIDE);
            json.addProperty("progress", Oracle.bits((float) field(toast, "progress")));
            json.addProperty("progressable", (boolean) field(toast, "progressable"));
            out.add(json);
        }
        return out;
    }

    private static Object field(Object target, String name) throws Exception {
        return Reflect.get(target, name);
    }

    static JsonObject tutorial(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(427, 240, 0, 0);
            var minecraft = screen.minecraft();
            var manager = new net.minecraft.client.gui.components.toasts.ToastManager(minecraft, minecraft.options);
            set(minecraft.gui, "toastManager", manager);
            var level = (RowLevel) unsafe.allocateInstance(RowLevel.class);
            var sounds = (GuiHudOracle.RecordingSounds) unsafe.allocateInstance(GuiHudOracle.RecordingSounds.class);
            set(sounds, "played", new java.util.ArrayList<String>());
            set(minecraft, "soundManager", sounds);
            set(minecraft, "level", level);
            set(minecraft, "userPropertiesFuture", java.util.concurrent.CompletableFuture.completedFuture(
                com.mojang.authlib.minecraft.UserApiService.OFFLINE_PROPERTIES));
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", unsafe.allocateInstance(QuietListener.class));
            var inventory = (RowInventory) unsafe.allocateInstance(RowInventory.class);
            inventory.held = new java.util.ArrayList<>();
            for (var name : row.getAsJsonArray("inventory")) {
                inventory.held.add(new net.minecraft.world.item.ItemStack(item(name.getAsString())));
            }
            set(player, "inventory", inventory);
            var stats = new net.minecraft.stats.StatsCounter();
            for (var name : row.getAsJsonArray("mined")) {
                stats.setValue(null, net.minecraft.stats.Stats.BLOCK_MINED.get(block(name.getAsString())), 1);
            }
            for (var name : row.getAsJsonArray("crafted")) {
                stats.setValue(null, net.minecraft.stats.Stats.ITEM_CRAFTED.get(item(name.getAsString())), 1);
            }
            set(player, "stats", stats);
            set(minecraft, "player", player);
            var mode = (net.minecraft.client.multiplayer.MultiPlayerGameMode)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.MultiPlayerGameMode.class);
            set(minecraft, "gameMode", mode);
            set(mode, "localPlayerMode", net.minecraft.world.level.GameType.valueOf(row.get("mode").getAsString()));
            minecraft.options.tutorialStep = net.minecraft.client.tutorial.TutorialSteps.getByName(
                row.get("step").getAsString());
            var tutorial = new net.minecraft.client.tutorial.Tutorial(minecraft, minecraft.options);
            var events = new JsonArray();
            for (var eventElement : row.getAsJsonArray("events")) {
                var event = eventElement.getAsJsonObject();
                var kind = event.get("do").getAsString();
                switch (kind) {
                    case "tick" -> {
                        var keys = event.getAsJsonArray("keys");
                        var held = new boolean[5];
                        var names = java.util.List.of("forward", "backward", "left", "right", "jump");
                        for (var key : keys) {
                            held[names.indexOf(key.getAsString())] = true;
                        }
                        for (int tick = 0; tick < event.get("n").getAsInt(); tick++) {
                            for (var move : event.getAsJsonArray("mouse")) {
                                tutorial.onMouse(move.getAsJsonArray().get(0).getAsDouble(),
                                    move.getAsJsonArray().get(1).getAsDouble());
                            }
                            var input = (net.minecraft.client.player.ClientInput)
                                unsafe.allocateInstance(net.minecraft.client.player.ClientInput.class);
                            input.keyPresses = new net.minecraft.world.entity.player.Input(
                                held[0], held[1], held[2], held[3], held[4], false, false);
                            tutorial.onInput(input);
                            tutorial.tick();
                        }
                    }
                    case "look" -> {
                        level.state = block(event.get("block").getAsString()).defaultBlockState();
                        var hit = new net.minecraft.world.phys.BlockHitResult(
                            net.minecraft.world.phys.Vec3.ZERO, net.minecraft.core.Direction.UP,
                            net.minecraft.core.BlockPos.ZERO, false);
                        tutorial.onLookAt(level, hit);
                    }
                    case "destroy" -> tutorial.onDestroyBlock(level, net.minecraft.core.BlockPos.ZERO,
                        block(event.get("block").getAsString()).defaultBlockState(),
                        event.get("percent").getAsFloat());
                    case "inventory" -> tutorial.onOpenInventory();
                    case "get_item" -> tutorial.onGetItem(
                        new net.minecraft.world.item.ItemStack(item(event.get("item").getAsString())));
                    case "mode" -> set(mode, "localPlayerMode",
                        net.minecraft.world.level.GameType.valueOf(event.get("value").getAsString()));
                    case "update" -> {
                        long at = event.get("at").getAsLong();
                        net.minecraft.util.Util.setTimeSource(() -> at * 1_000_000L);
                        manager.update();
                    }
                    case "stop" -> tutorial.stop();
                    default -> throw new IllegalArgumentException(kind);
                }
                var out = new JsonObject();
                out.addProperty("do", kind);
                out.addProperty("step", minecraft.options.tutorialStep.getName());
                var heard = new JsonArray();
                for (var name : sounds.played) {
                    heard.add(name);
                }
                sounds.played.clear();
                out.add("sounds", heard);
                out.add("toasts", toasts(manager));
                events.add(out);
            }
            net.minecraft.util.Util.setTimeSource(System::nanoTime);
            var result = new JsonObject();
            result.add("events", events);
            result.addProperty("label", row.get("label").getAsString());
            rows.add(result);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
