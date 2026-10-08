import java.util.ArrayList;
import net.minecraft.network.chat.Component;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.state.gui.GuiRenderState;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The settings screens and the container screen. */
class GuiSettingsOracle extends GuiScreenOracle {
    /** A game whose GUI scale can change, whose Control key is told, and whose monitor is none. */
    static final class StubGame extends net.minecraft.client.Minecraft {
        boolean control;
        net.minecraft.client.gui.screens.Screen current;

        StubGame() {
            super(null);
        }

        @Override
        public boolean hasControlDown() {
            return control;
        }

        /** A text field taking the focus tells the keyboard, of which there is none. */
        @Override
        public void onTextInputFocusChange(net.minecraft.client.gui.components.events.GuiEventListener element,
                boolean isFocused) {}

        /** `Minecraft.resizeGui` without the mouse handler. */
        @Override
        public void resizeGui() {
            var window = getWindow();
            window.setGuiScale(window.calculateScale(options.guiScale().get(), isEnforceUnicode()));
            current.resize(window.getGuiScaledWidth(), window.getGuiScaledHeight());
        }
    }

    static final class NoMonitors extends com.mojang.blaze3d.platform.MonitorManager {
        NoMonitors() {
            super();
        }

        @Override
        public com.mojang.blaze3d.platform.Monitor findBestMonitor(com.mojang.blaze3d.platform.Window window) {
            return null;
        }
    }

    /**
     * The game's own video settings screen in a window of a framebuffer size and a GUI scale:
     * what a press, a drag, a release and the wheel (with Control or without) do to it, then
     * what it draws. A screen is drawn between steps, as the game does, since the widgets
     * take their places as they are drawn.
     */
    static JsonObject settingsScreen(JsonObject request, String kind) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            setup();
            var minecraft = (StubGame) unsafe.allocateInstance(StubGame.class);
            var instance = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
            instance.setAccessible(true);
            instance.set(null, minecraft);
            var screen = prepare(minecraft, 0, 0, mouseX, mouseY);
            // A label too wide for its button scrolls by the clock: stop it at the start.
            var clock = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clock.setAccessible(true);
            var clockBefore = clock.get(null);
            long[] now = {0};
            clock.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            set(minecraft, "musicManager", unsafe.allocateInstance(SilentMusic.class));
            // `GraphicsPreset.apply` asks for the device even for Custom, which does nothing with it.
            var deviceField = com.mojang.blaze3d.systems.RenderSystem.class.getDeclaredField("DEVICE");
            deviceField.setAccessible(true);
            deviceField.set(null, unsafe.allocateInstance(com.mojang.blaze3d.systems.GpuDevice.class));
            set(minecraft, "framerateLimitTracker",
                unsafe.allocateInstance(com.mojang.blaze3d.platform.FramerateLimitTracker.class));
            var warnlist = unsafe.allocateInstance(net.minecraft.client.renderer.GpuWarnlistManager.class);
            set(warnlist, "warnings", com.google.common.collect.ImmutableMap.of());
            set(minecraft, "gpuWarnlistManager", warnlist);
            boolean inWorld = row.get("in_world").getAsBoolean();
            if (inWorld) {
                set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            }
            var window = minecraft.getWindow();
            set(window, "monitorManager", unsafe.allocateInstance(NoMonitors.class));
            set(window, "framebufferWidth", row.get("framebuffer").getAsJsonArray().get(0).getAsInt());
            set(window, "framebufferHeight", row.get("framebuffer").getAsJsonArray().get(1).getAsInt());
            minecraft.options.guiScale().set(row.get("gui_scale").getAsInt());
            set(minecraft, "running", true);
            window.setGuiScale(window.calculateScale(minecraft.options.guiScale().get(), false));
            // The chat the options screen asks to wrap its lines again.
            set(minecraft.gui.hud, "chat", new net.minecraft.client.gui.components.ChatComponent(minecraft));
            // The music toast the sound screen's choice tells.
            set(minecraft.gui, "toastManager",
                new net.minecraft.client.gui.components.toasts.ToastManager(minecraft, minecraft.options));
            if (row.has("options")) {
                chatOptions(minecraft.options, row.getAsJsonObject("options"));
            }
            if (row.has("bindings")) {
                // Keys bound before the screen opens, as `options.txt` would have them.
                for (var mapping : minecraft.options.keyMappings) {
                    var bound = row.getAsJsonObject("bindings").get(mapping.getName());
                    if (bound != null) {
                        mapping.setKey(com.mojang.blaze3d.platform.InputConstants.getKey(bound.getAsString()));
                    }
                }
                net.minecraft.client.KeyMapping.resetMapping();
            }
            minecraft.setLastInputType(net.minecraft.client.InputType.NONE);
            var languageBefore = net.minecraft.locale.Language.getInstance();
            if (kind.equals("language")) {
                net.minecraft.locale.Language.inject(clientLanguage(row.get("language").getAsString()));
            }
            net.minecraft.client.gui.screens.options.OptionsSubScreen video = switch (kind) {
                case "video" -> new net.minecraft.client.gui.screens.options.VideoSettingsScreen(
                    null, minecraft, minecraft.options);
                case "chat" -> new net.minecraft.client.gui.screens.options.ChatOptionsScreen(null, minecraft.options);
                case "mouse" -> new net.minecraft.client.gui.screens.options.MouseSettingsScreen(null, minecraft.options);
                case "controls" -> new net.minecraft.client.gui.screens.options.controls.ControlsScreen(null, minecraft.options);
                case "keybinds" -> new net.minecraft.client.gui.screens.options.controls.KeyBindsScreen(null, minecraft.options);
                case "sound" -> new net.minecraft.client.gui.screens.options.SoundOptionsScreen(null, minecraft.options);
                case "language" -> new net.minecraft.client.gui.screens.options.LanguageSelectScreen(
                    null, minecraft.options, languageManager(row.get("language").getAsString(), row.getAsJsonArray("codes")));
                default -> throw new IllegalArgumentException(kind);
            };
            minecraft.current = video;
            set(minecraft.gui, "screen", video);
            set(video, "width", window.getGuiScaledWidth());
            set(video, "height", window.getGuiScaledHeight());
            var init = net.minecraft.client.gui.screens.options.OptionsSubScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(video);
            if (kind.equals("language")) {
                var focus = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("setInitialFocus");
                focus.setAccessible(true);
                focus.invoke(video);
            }
            video.tick();
            var info = new net.minecraft.client.input.MouseButtonInfo(0, 0);
            double lastX = 0;
            double lastY = 0;
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                // A frame between one event and the next.
                var frame = new GuiRenderState();
                video.extractRenderState(new GuiGraphicsExtractor(minecraft, frame, mouseX, mouseY), mouseX, mouseY, 0.0F);
                String action = step.get("do").getAsString();
                if (action.equals("frame")) {
                    // A frame drawn and thrown away, at a time (the tooltips wait by it).
                    now[0] = step.get("ms").getAsLong();
                    var thrown = new GuiRenderState();
                    var thrownGraphics = new GuiGraphicsExtractor(minecraft, thrown, mouseX, mouseY);
                    video.extractRenderState(thrownGraphics, mouseX, mouseY, 0.0F);
                    thrownGraphics.extractDeferredElements(mouseX, mouseY, 0.0F);
                    continue;
                }
                double x = step.has("x") ? step.get("x").getAsDouble() : lastX;
                double y = step.has("y") ? step.get("y").getAsDouble() : lastY;
                switch (action) {
                    case "click" -> {
                        minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                        video.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(x, y,
                            new net.minecraft.client.input.MouseButtonInfo(
                                step.has("button") ? step.get("button").getAsInt() : 0, step.has("shift") && step.get("shift").getAsBoolean() ? 1 : 0)),
                            step.has("double") && step.get("double").getAsBoolean());
                    }
                    case "char" -> video.charTyped(new net.minecraft.client.input.CharacterEvent(
                        step.get("char").getAsString().codePointAt(0)));
                    case "key" -> {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "backspace" -> 259;
                            case "enter" -> 257;
                            case "space" -> 32;
                            case "tab", "shift+tab" -> 258;
                            case "left" -> 263;
                            case "right" -> 262;
                            case "up" -> 265;
                            case "down" -> 264;
                            case "escape" -> 256;
                            default -> {
                                // Any other key by its GLFW code.
                                if (!name.startsWith("code:")) {
                                    throw new IllegalArgumentException(name);
                                }
                                yield Integer.parseInt(name.substring(5));
                            }
                        };
                        // `KeyboardHandler.keyPress` notes what kind of key it was.
                        if (code == 258) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                        } else if (code >= 262 && code <= 265) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_ARROW);
                        }
                        video.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0,
                            name.startsWith("shift+") ? 1 : 0));
                    }
                    case "drag" -> video.mouseDragged(new net.minecraft.client.input.MouseButtonEvent(x, y, info),
                        x - lastX, y - lastY);
                    case "release" -> video.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(x, y, info));
                    case "scroll" -> {
                        minecraft.control = step.has("ctrl") && step.get("ctrl").getAsBoolean();
                        video.mouseScrolled(x, y, 0.0, step.get("dy").getAsDouble());
                        minecraft.control = false;
                    }
                    default -> throw new IllegalArgumentException(action);
                }
                lastX = x;
                lastY = y;
                video.tick();
            }
            var background = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("extractMenuBackground",
                GuiGraphicsExtractor.class);
            background.setAccessible(true);
            // A graphics made now knows the window's size, which the scissor is cut to.
            var finalState = new GuiRenderState();
            var graphics = new GuiGraphicsExtractor(minecraft, finalState, mouseX, mouseY);
            background.invoke(video, graphics);
            now[0] = row.has("ms") ? row.get("ms").getAsLong() : 0;
            video.extractRenderState(graphics, mouseX, mouseY, 0.0F);
            graphics.extractDeferredElements(mouseX, mouseY, 0.0F);
            double scrollAmount;
            if (kind.equals("keybinds")) {
                var keyList = video.getClass().getDeclaredField("keyBindsList");
                keyList.setAccessible(true);
                scrollAmount = ((net.minecraft.client.gui.components.AbstractScrollArea) keyList.get(video))
                    .scrollAmount();
            } else if (kind.equals("language")) {
                var languageList = video.getClass().getDeclaredField("languageSelectionList");
                languageList.setAccessible(true);
                scrollAmount = ((net.minecraft.client.gui.components.AbstractScrollArea) languageList.get(video))
                    .scrollAmount();
            } else {
                var listField = net.minecraft.client.gui.screens.options.OptionsSubScreen.class.getDeclaredField("list");
                listField.setAccessible(true);
                scrollAmount = ((net.minecraft.client.gui.components.OptionsList) listField.get(video)).scrollAmount();
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("width", window.getGuiScaledWidth());
            out.addProperty("height", window.getGuiScaledHeight());
            out.addProperty("gui_scale", minecraft.options.guiScale().get());
            out.addProperty("scroll", scrollAmount);
            if (kind.equals("language")) {
                var searchField = video.getClass().getDeclaredField("search");
                searchField.setAccessible(true);
                out.addProperty("search",
                    ((net.minecraft.client.gui.components.EditBox) searchField.get(video)).getValue());
            }
            out.add("ops", record(finalState));
            net.minecraft.locale.Language.inject(languageBefore);
            clock.set(null, clockBefore);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A menu's screen over the game's own container menu: its labels, sprites,
     * name box and what it shows of its data, with slots holding items that
     * draw nothing. After each row's `keys` and `typed` text.
     */
    static JsonObject containerScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            setup();
            MovementOracle.bootstrap();
            var open = MovementOracle.openMenu(row);
            var minecraft = open.scene().minecraft();
            var instance = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
            instance.setAccessible(true);
            instance.set(null, minecraft);
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var drawn = prepare(minecraft, width, height, row.get("mouse_x").getAsInt(),
                row.get("mouse_y").getAsInt());
            set(minecraft, "itemModelResolver", unsafe.allocateInstance(NoItems.class));
            // The entity models the loom's flag and the book of the enchanting
            // table are baked from.
            var modelManager = unsafe.allocateInstance(net.minecraft.client.resources.model.ModelManager.class);
            set(modelManager, "entityModelSet", net.minecraft.client.model.geom.EntityModelSet.vanilla());
            set(minecraft, "modelManager", modelManager);
            var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
            set(renderer, "lighting", unsafe.allocateInstance(NoLighting.class));
            set(minecraft, "gameRenderer", renderer);
            set(minecraft, "level", open.scene().level());
            set(minecraft, "deltaTracker", new net.minecraft.client.DeltaTracker.Timer(20.0F, 0L, millis -> 50L));
            // What a book asks of the account (whether text is filtered).
            set(minecraft, "userPropertiesFuture", java.util.concurrent.CompletableFuture.completedFuture(
                com.mojang.authlib.minecraft.UserApiService.OFFLINE_PROPERTIES));
            var menu = open.menu();
            var inventory = open.player().getInventory();
            net.minecraft.client.gui.screens.Screen screen =
                switch (row.get("menu").getAsString()) {
                    case "minecraft:anvil" -> new net.minecraft.client.gui.screens.inventory.AnvilScreen(
                        (net.minecraft.world.inventory.AnvilMenu) menu, inventory,
                        Component.translatable("container.repair"));
                    case "minecraft:stonecutter" -> new net.minecraft.client.gui.screens.inventory.StonecutterScreen(
                        (net.minecraft.world.inventory.StonecutterMenu) menu, inventory,
                        Component.translatable("container.stonecutter"));
                    case "minecraft:loom" -> new net.minecraft.client.gui.screens.inventory.LoomScreen(
                        (net.minecraft.world.inventory.LoomMenu) menu, inventory,
                        Component.translatable("container.loom"));
                    case "minecraft:grindstone" -> new net.minecraft.client.gui.screens.inventory.GrindstoneScreen(
                        (net.minecraft.world.inventory.GrindstoneMenu) menu, inventory,
                        Component.translatable("container.grindstone_title"));
                    case "minecraft:merchant" -> new net.minecraft.client.gui.screens.inventory.MerchantScreen(
                        (net.minecraft.world.inventory.MerchantMenu) menu, inventory,
                        Component.translatable("entity.minecraft.villager"));
                    case "minecraft:lectern" -> new net.minecraft.client.gui.screens.inventory.LecternScreen(
                        (net.minecraft.world.inventory.LecternMenu) menu, inventory,
                        Component.translatable("container.lectern"));
                    case "minecraft:crafter_3x3" -> new net.minecraft.client.gui.screens.inventory.CrafterScreen(
                        (net.minecraft.world.inventory.CrafterMenu) menu, inventory,
                        Component.translatable("container.crafter"));
                    case "minecraft:beacon" -> new net.minecraft.client.gui.screens.inventory.BeaconScreen(
                        (net.minecraft.world.inventory.BeaconMenu) menu, inventory,
                        Component.translatable("container.beacon"));
                    case "minecraft:brewing_stand" -> new net.minecraft.client.gui.screens.inventory.BrewingStandScreen(
                        (net.minecraft.world.inventory.BrewingStandMenu) menu, inventory,
                        Component.translatable("container.brewing"));
                    case "minecraft:cartography_table" -> new net.minecraft.client.gui.screens.inventory.CartographyTableScreen(
                        (net.minecraft.world.inventory.CartographyTableMenu) menu, inventory,
                        Component.translatable("container.cartography_table"));
                    case "minecraft:smithing" -> new net.minecraft.client.gui.screens.inventory.SmithingScreen(
                        (net.minecraft.world.inventory.SmithingMenu) menu, inventory,
                        Component.translatable("container.upgrade"));
                    case "minecraft:enchantment" -> new net.minecraft.client.gui.screens.inventory.EnchantmentScreen(
                        (net.minecraft.world.inventory.EnchantmentMenu) menu, inventory,
                        Component.translatable("container.enchant"));
                    default -> throw new IllegalArgumentException("Unknown menu screen");
                };
            set(screen, "minecraft", minecraft);
            set(screen, "font", font);
            set(screen, "width", width);
            set(screen, "height", height);
            var init = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(screen);
            if (row.has("sync_slot")) {
                // The server's update of a slot, as the screen hears it.
                int slot = row.get("sync_slot").getAsInt();
                if (menu instanceof net.minecraft.world.inventory.LecternMenu) {
                    // The lectern's screen hears the server's update of the book.
                    var heard = net.minecraft.client.gui.screens.inventory.LecternScreen.class
                        .getDeclaredField("listener");
                    heard.setAccessible(true);
                    ((net.minecraft.world.inventory.ContainerListener) heard.get(screen))
                        .slotChanged(menu, slot, menu.getSlot(slot).getItem());
                } else if (screen instanceof net.minecraft.world.inventory.ContainerListener listener) {
                    listener.slotChanged(menu, slot, menu.getSlot(slot).getItem());
                } else {
                    // The stonecutter and the loom hear their menu's own update.
                    menu.getSlot(slot).setChanged();
                }
            }
            if (row.has("data_after")) {
                // What the server sends once the screen is open: the screen hears the change.
                for (var pair : row.getAsJsonArray("data_after")) {
                    menu.setData(pair.getAsJsonArray().get(0).getAsInt(), pair.getAsJsonArray().get(1).getAsInt());
                }
            }
            if (menu instanceof net.minecraft.world.inventory.SmithingMenu
                || menu instanceof net.minecraft.world.inventory.BeaconMenu) {
                // Its empty slots show their icons, and a beacon's buttons their state, once the screen has ticked.
                var tick = net.minecraft.client.gui.screens.inventory.AbstractContainerScreen.class
                    .getDeclaredMethod("containerTick");
                tick.setAccessible(true);
                tick.invoke(screen);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var steps = new JsonArray();
            var keys = new ArrayList<String>();
            keys.add("");
            if (row.has("keys")) {
                row.getAsJsonArray("keys").forEach(key -> keys.add(key.getAsString()));
            }
            for (String key : keys) {
                var step = new JsonObject();
                step.addProperty("key", key);
                if (key.startsWith("type:")) {
                    for (char c : key.substring(5).toCharArray()) {
                        screen.charTyped(new net.minecraft.client.input.CharacterEvent(c));
                    }
                }
                var frame = prepare(minecraft, width, height, row.get("mouse_x").getAsInt(),
                    row.get("mouse_y").getAsInt());
                // The screen's own draw, without the deferred tooltip the game
                // draws last (tooltip text is the item's, which this run has no
                // items to give).
                frame.graphics().nextStratum();
                screen.extractBackground(frame.graphics(), row.get("mouse_x").getAsInt(),
                    row.get("mouse_y").getAsInt(), 0.0F);
                frame.graphics().nextStratum();
                screen.extractRenderState(frame.graphics(), row.get("mouse_x").getAsInt(),
                    row.get("mouse_y").getAsInt(), 0.0F);
                step.add("ops", record(frame.state()));
                steps.add(step);
            }
            out.add("steps", steps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** An item resolver that draws nothing: slots hold items, which are not recorded. */
    static final class NoItems extends net.minecraft.client.renderer.item.ItemModelResolver {
        NoItems() {
            super(null);
        }

        @Override
        public void updateForTopItem(net.minecraft.client.renderer.item.ItemStackRenderState output,
                net.minecraft.world.item.ItemStack item, net.minecraft.world.item.ItemDisplayContext context,
                net.minecraft.world.level.Level level, net.minecraft.world.entity.ItemOwner owner, int seed) {}
    }

    /** Lighting with no GPU behind it. */
    static final class NoLighting extends com.mojang.blaze3d.platform.Lighting {
        @Override
        public void setupFor(com.mojang.blaze3d.platform.Lighting.Entry entry) {}
    }

    /**
     * The key mappings of a fresh `Options`, in `keyMappings` order (name, category, default key,
     * order), the categories in their sort order, and every key `InputConstants` names.
     */
    static JsonObject keyMappings(JsonObject request) throws Exception {
        var options = screen(320, 240, 0, 0).minecraft().options;
        var mappings = new com.google.gson.JsonArray();
        for (var mapping : options.keyMappings) {
            var entry = new JsonObject();
            entry.addProperty("name", mapping.getName());
            entry.addProperty("category", mapping.getCategory().id().toString());
            entry.addProperty("key", mapping.getDefaultKey().getName());
            entry.addProperty("order", (int) get(mapping, "order"));
            mappings.add(entry);
        }
        var categories = new com.google.gson.JsonArray();
        var order = Class.forName("net.minecraft.client.KeyMapping$Category").getDeclaredField("SORT_ORDER");
        order.setAccessible(true);
        for (var category : (java.util.List<?>) order.get(null)) {
            categories.add(((net.minecraft.client.KeyMapping.Category) category).id().toString());
        }
        var keys = new com.google.gson.JsonArray();
        for (var type : com.mojang.blaze3d.platform.InputConstants.Type.values()) {
            @SuppressWarnings("unchecked")
            var map = (java.util.Map<Integer, com.mojang.blaze3d.platform.InputConstants.Key>) get(type, "map");
            map.keySet().stream().sorted().forEach(code -> {
                var key = new JsonObject();
                key.addProperty("type", type.name());
                key.addProperty("code", code);
                key.addProperty("name", map.get(code).getName());
                keys.add(key);
            });
        }
        var result = new JsonObject();
        result.add("mappings", mappings);
        result.add("categories", categories);
        result.add("keys", keys);
        return result;
    }
}
