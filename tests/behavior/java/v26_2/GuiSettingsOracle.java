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
            if (current != null) current.resize(window.getGuiScaledWidth(), window.getGuiScaledHeight());
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
    /// The quality options as `options.txt` would have loaded them: the game is not running yet, so
    /// a set neither applies the preset nor turns it to Custom.
    /**
     * What the server told the world screen: the difficulty and whether it is locked, a hardcore world,
     * and the player's permission level (a player is made only when one is given).
     */
    static void worldFacts(StubGame minecraft, Object level, JsonObject facts) throws Exception {
        var data = new net.minecraft.client.multiplayer.ClientLevel.ClientLevelData(
            net.minecraft.world.Difficulty.valueOf(facts.get("difficulty").getAsString().toUpperCase()),
            facts.has("hardcore") && facts.get("hardcore").getAsBoolean(), false);
        data.setDifficultyLocked(facts.has("locked") && facts.get("locked").getAsBoolean());
        set(level, "clientLevelData", data);
        set(level, "levelData", data);
        var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
        set(player, "permissions", switch (facts.has("permission") ? facts.get("permission").getAsInt() : 0) {
            case 1 -> net.minecraft.server.permissions.LevelBasedPermissionSet.MODERATOR;
            case 2 -> net.minecraft.server.permissions.LevelBasedPermissionSet.GAMEMASTER;
            case 3 -> net.minecraft.server.permissions.LevelBasedPermissionSet.ADMIN;
            case 4 -> net.minecraft.server.permissions.LevelBasedPermissionSet.OWNER;
            default -> net.minecraft.server.permissions.PermissionSet.NO_PERMISSIONS;
        });
        minecraft.player = (net.minecraft.client.player.LocalPlayer) player;
    }

    static void storedQuality(StubGame minecraft, JsonObject values) throws Exception {
        var options = minecraft.options;
        set(minecraft, "running", false);
        if (values.has("preset")) {
            for (var preset : net.minecraft.client.GraphicsPreset.values()) {
                if (preset.getSerializedName().equals(values.get("preset").getAsString())) options.graphicsPreset().set(preset);
            }
        }
        if (values.has("blend")) options.biomeBlendRadius().set(values.get("blend").getAsInt());
        if (values.has("priority")) {
            options.prioritizeChunkUpdates().set(net.minecraft.client.PrioritizeChunkUpdates.values()[values.get("priority").getAsInt()]);
        }
        if (values.has("simulation")) options.simulationDistance().set(values.get("simulation").getAsInt());
        if (values.has("clouds")) options.cloudStatus().set(net.minecraft.client.CloudStatus.values()[values.get("clouds").getAsInt()]);
        if (values.has("mipmaps")) options.mipmapLevels().set(values.get("mipmaps").getAsInt());
        if (values.has("shadows")) options.entityShadows().set(values.get("shadows").getAsBoolean());
        if (values.has("entity_distance")) options.entityDistanceScaling().set(values.get("entity_distance").getAsDouble());
        if (values.has("cloud_range")) options.cloudRange().set(values.get("cloud_range").getAsInt());
        if (values.has("leaves")) options.cutoutLeaves().set(values.get("leaves").getAsBoolean());
        if (values.has("transparency")) options.improvedTransparency().set(values.get("transparency").getAsBoolean());
        if (values.has("weather")) options.weatherRadius().set(values.get("weather").getAsInt());
        if (values.has("anisotropy")) options.maxAnisotropyBit().set(values.get("anisotropy").getAsInt());
        if (values.has("filtering")) {
            options.textureFiltering().set(net.minecraft.client.TextureFilteringMethod.values()[values.get("filtering").getAsInt()]);
        }
        set(minecraft, "running", true);
    }

    static JsonObject settingsScreen(JsonObject request, String kind) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            setup();
            var minecraft = (StubGame) unsafe.allocateInstance(StubGame.class);
            var instance = Reflect.field(net.minecraft.client.Minecraft.class, "instance");
            instance.set(null, minecraft);
            var screen = prepare(minecraft, 0, 0, mouseX, mouseY);
            // A label too wide for its button scrolls by the clock: stop it at the start.
            var clock = Reflect.field(net.minecraft.util.Util.class, "timeSource");
            var clockBefore = clock.get(null);
            long[] now = {0};
            clock.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            set(minecraft, "musicManager", unsafe.allocateInstance(SilentMusic.class));
            // `GraphicsPreset.apply` asks for the device even for Custom, which does nothing with it.
            var deviceField = Reflect.field(com.mojang.blaze3d.systems.RenderSystem.class, "DEVICE");
            deviceField.set(null, unsafe.allocateInstance(com.mojang.blaze3d.systems.GpuDevice.class));
            set(minecraft, "framerateLimitTracker",
                unsafe.allocateInstance(com.mojang.blaze3d.platform.FramerateLimitTracker.class));
            var warnlist = unsafe.allocateInstance(net.minecraft.client.renderer.GpuWarnlistManager.class);
            set(warnlist, "warnings", com.google.common.collect.ImmutableMap.of());
            set(minecraft, "gpuWarnlistManager", warnlist);
            boolean inWorld = row.get("in_world").getAsBoolean();
            if (inWorld) {
                var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
                // The features the server enabled: `rotateWithMinecart` needs the minecart ones.
                var connection = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
                boolean minecart = row.has("options") && row.getAsJsonObject("options").has("minecart")
                    && row.getAsJsonObject("options").get("minecart").getAsBoolean();
                set(connection, "enabledFeatures", minecart
                    ? net.minecraft.world.flag.FeatureFlagSet.of(net.minecraft.world.flag.FeatureFlags.MINECART_IMPROVEMENTS)
                    : net.minecraft.world.flag.FeatureFlagSet.of());
                set(level, "connection", connection);
                set(minecraft, "level", level);
                if (row.has("world")) {
                    worldFacts(minecraft, level, row.getAsJsonObject("world"));
                }
            }
            // No resource packs are available, as in Photon.
            set(minecraft, "resourcePackRepository", new net.minecraft.server.packs.repository.PackRepository());
            var packSources = kind.equals("packs") ? syntheticPacks(row.getAsJsonArray("packs")) : null;
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
                if (row.getAsJsonObject("options").has("quality")) {
                    storedQuality(minecraft, row.getAsJsonObject("options").getAsJsonObject("quality"));
                }
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
            if (kind.equals("font")) {
                // The font options tell the font manager (no fonts loaded).
                var fonts = unsafe.allocateInstance(net.minecraft.client.gui.font.FontManager.class);
                set(fonts, "fontSets", new java.util.HashMap<>());
                set(minecraft, "fontManager", fonts);
                // The values the row stores; the variants are set whatever the JVM locale says.
                var stored = row.has("options") ? row.getAsJsonObject("options") : new JsonObject();
                minecraft.options.japaneseGlyphVariants().set(
                    stored.has("japanese_variants") && stored.get("japanese_variants").getAsBoolean());
                minecraft.options.forceUnicodeFont().set(
                    stored.has("force_unicode") && stored.get("force_unicode").getAsBoolean());
            }
            if (kind.equals("online")) {
                // The account's friends list: off unless the row turns it on.
                var social = unsafe.allocateInstance(net.minecraft.client.gui.screens.social.PlayerSocialManager.class);
                var friends = row.has("friends") ? row.getAsJsonObject("friends") : new JsonObject();
                set(social, "friendListEnabled", friends.has("enabled") && friends.get("enabled").getAsBoolean());
                set(social, "allowFriendRequests", friends.has("requests") && friends.get("requests").getAsBoolean());
                set(minecraft, "playerSocialManager", social);
            }
            minecraft.setLastInputType(net.minecraft.client.InputType.NONE);
            var languageBefore = net.minecraft.locale.Language.getInstance();
            if (kind.equals("language")) {
                net.minecraft.locale.Language.inject(clientLanguage(row.get("language").getAsString()));
            }
            net.minecraft.client.gui.screens.Screen video = switch (kind) {
                case "world" -> new net.minecraft.client.gui.screens.options.WorldOptionsScreen(null, minecraft.level);
                case "video" -> new net.minecraft.client.gui.screens.options.VideoSettingsScreen(
                    null, minecraft, minecraft.options);
                case "chat" -> new net.minecraft.client.gui.screens.options.ChatOptionsScreen(null, minecraft.options);
                case "accessibility" -> new net.minecraft.client.gui.screens.options.AccessibilityOptionsScreen(null, minecraft.options);
                case "mouse" -> new net.minecraft.client.gui.screens.options.MouseSettingsScreen(null, minecraft.options);
                case "controls" -> new net.minecraft.client.gui.screens.options.controls.ControlsScreen(null, minecraft.options);
                case "keybinds" -> new net.minecraft.client.gui.screens.options.controls.KeyBindsScreen(null, minecraft.options);
                case "sound" -> new net.minecraft.client.gui.screens.options.SoundOptionsScreen(null, minecraft.options);
                case "skin" -> new net.minecraft.client.gui.screens.options.SkinCustomizationScreen(null, minecraft.options);
                case "font" -> new net.minecraft.client.gui.screens.options.FontOptionsScreen(null, minecraft.options);
                case "online" -> new net.minecraft.client.gui.screens.options.OnlineOptionsScreen(null, minecraft.options);
                case "language" -> new net.minecraft.client.gui.screens.options.LanguageSelectScreen(
                    null, minecraft.options, languageManager(row.get("language").getAsString(), row.getAsJsonArray("codes")));
                case "packs" -> new net.minecraft.client.gui.screens.packs.PackSelectionScreen(
                    packSources, repository -> { }, java.nio.file.Path.of("/nonexistent-pack-folder"),
                    net.minecraft.network.chat.Component.translatable("resourcePack.title"));
                default -> throw new IllegalArgumentException(kind);
            };
            minecraft.current = video;
            set(minecraft.gui, "screen", video);
            set(video, "width", window.getGuiScaledWidth());
            set(video, "height", window.getGuiScaledHeight());
            var init = Reflect.method(kind.equals("world")
                ? net.minecraft.client.gui.screens.options.WorldOptionsScreen.class
                : kind.equals("packs")
                    ? net.minecraft.client.gui.screens.packs.PackSelectionScreen.class
                    : net.minecraft.client.gui.screens.options.OptionsSubScreen.class, "init");
            init.invoke(video);
            if (kind.equals("language") || kind.equals("packs")) {
                var focus = Reflect.method(net.minecraft.client.gui.screens.Screen.class, "setInitialFocus");
                focus.invoke(video);
            }
            video.tick();
            SilentSounds.ASKED.clear();
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
                        String bare = name.startsWith("shift+") ? name.substring(6) : name;
                        int code = switch (bare) {
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
                                if (!bare.startsWith("code:")) {
                                    throw new IllegalArgumentException(name);
                                }
                                yield Integer.parseInt(bare.substring(5));
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
            var background = Reflect.method(net.minecraft.client.gui.screens.Screen.class, "extractMenuBackground",
                GuiGraphicsExtractor.class);
            // A graphics made now knows the window's size, which the scissor is cut to.
            var finalState = new GuiRenderState();
            var graphics = new GuiGraphicsExtractor(minecraft, finalState, mouseX, mouseY);
            background.invoke(video, graphics);
            now[0] = row.has("ms") ? row.get("ms").getAsLong() : 0;
            video.extractRenderState(graphics, mouseX, mouseY, 0.0F);
            graphics.extractDeferredElements(mouseX, mouseY, 0.0F);
            double scrollAmount;
            if (kind.equals("keybinds")) {
                var keyList = Reflect.field(video.getClass(), "keyBindsList");
                scrollAmount = ((net.minecraft.client.gui.components.AbstractScrollArea) keyList.get(video))
                    .scrollAmount();
            } else if (kind.equals("language")) {
                var languageList = Reflect.field(video.getClass(), "languageSelectionList");
                scrollAmount = ((net.minecraft.client.gui.components.AbstractScrollArea) languageList.get(video))
                    .scrollAmount();
            } else if (kind.equals("world")) {
                scrollAmount = 0;
            } else if (kind.equals("packs")) {
                scrollAmount = packList(video, "availablePackList").scrollAmount();
            } else {
                var listField = Reflect.field(net.minecraft.client.gui.screens.options.OptionsSubScreen.class, "list");
                scrollAmount = ((net.minecraft.client.gui.components.OptionsList) listField.get(video)).scrollAmount();
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("width", window.getGuiScaledWidth());
            out.addProperty("height", window.getGuiScaledHeight());
            out.addProperty("gui_scale", minecraft.options.guiScale().get());
            out.addProperty("scroll", scrollAmount);
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            if (kind.equals("packs")) {
                packFacts(video, out);
            }
            if (kind.equals("language")) {
                var searchField = Reflect.field(video.getClass(), "search");
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
    /** A repository over the packs a row lists: no resources, only what the screen shows of each. */
    static net.minecraft.server.packs.repository.PackRepository syntheticPacks(JsonArray packs) {
        var list = new java.util.ArrayList<net.minecraft.server.packs.repository.Pack>();
        var selected = new java.util.ArrayList<String>();
        for (var element : packs) {
            var pack = element.getAsJsonObject();
            String id = pack.get("id").getAsString();
            var compatibility = net.minecraft.server.packs.repository.PackCompatibility.valueOf(
                pack.has("compatibility") ? pack.get("compatibility").getAsString() : "COMPATIBLE");
            boolean required = pack.has("required") && pack.get("required").getAsBoolean();
            boolean fixed = pack.has("fixed") && pack.get("fixed").getAsBoolean();
            var position = pack.has("top") && !pack.get("top").getAsBoolean()
                ? net.minecraft.server.packs.repository.Pack.Position.BOTTOM
                : net.minecraft.server.packs.repository.Pack.Position.TOP;
            var source = pack.has("builtin") && pack.get("builtin").getAsBoolean()
                ? net.minecraft.server.packs.repository.PackSource.BUILT_IN
                : net.minecraft.server.packs.repository.PackSource.DEFAULT;
            var title = pack.has("title_key")
                ? net.minecraft.network.chat.Component.translatable(pack.get("title_key").getAsString())
                : net.minecraft.network.chat.Component.literal(pack.get("title").getAsString());
            var description = pack.has("description_key")
                ? net.minecraft.network.chat.Component.translatable(pack.get("description_key").getAsString())
                : net.minecraft.network.chat.Component.literal(pack.get("description").getAsString());
            var location = new net.minecraft.server.packs.PackLocationInfo(id, title, source, java.util.Optional.empty());
            var resources = new net.minecraft.server.packs.repository.Pack.ResourcesSupplier() {
                public net.minecraft.server.packs.PackResources openPrimary(net.minecraft.server.packs.PackLocationInfo info) {
                    return noResources();
                }
                public net.minecraft.server.packs.PackResources openFull(net.minecraft.server.packs.PackLocationInfo info,
                    net.minecraft.server.packs.repository.Pack.Metadata metadata) {
                    return noResources();
                }
            };
            var metadata = new net.minecraft.server.packs.repository.Pack.Metadata(
                description, compatibility, net.minecraft.world.flag.FeatureFlagSet.of(), java.util.List.of());
            list.add(new net.minecraft.server.packs.repository.Pack(location, resources, metadata,
                new net.minecraft.server.packs.PackSelectionConfig(required, position, fixed)));
            if (pack.has("selected") && pack.get("selected").getAsBoolean()) {
                selected.add(id);
            }
        }
        var repository = new net.minecraft.server.packs.repository.PackRepository(
            (net.minecraft.server.packs.repository.RepositorySource) result -> list.forEach(result));
        repository.reload();
        repository.setSelected(selected);
        return repository;
    }

    /** A pack with no files: the icon is the default one. */
    static net.minecraft.server.packs.PackResources noResources() {
        return (net.minecraft.server.packs.PackResources) java.lang.reflect.Proxy.newProxyInstance(
            GuiSettingsOracle.class.getClassLoader(),
            new Class<?>[] {net.minecraft.server.packs.PackResources.class},
            (proxy, method, args) -> method.getReturnType().isPrimitive() && method.getReturnType() != void.class
                ? (method.getReturnType() == boolean.class ? (Object) false : (Object) 0)
                : null);
    }

    static net.minecraft.client.gui.components.AbstractScrollArea packList(
        net.minecraft.client.gui.screens.Screen screen, String field) throws Exception {
        return (net.minecraft.client.gui.components.AbstractScrollArea) Reflect.field(screen.getClass(), field).get(screen);
    }

    /** What the pack screen has focused and what each list has selected. */
    static void packFacts(net.minecraft.client.gui.screens.Screen screen, JsonObject out) throws Exception {
        var focused = screen.getFocused();
        String focus = "none";
        if (focused == Reflect.field(screen.getClass(), "search").get(screen)) {
            focus = "search";
        } else if (focused == packList(screen, "availablePackList")) {
            focus = "available";
        } else if (focused == packList(screen, "selectedPackList")) {
            focus = "selected";
        } else if (focused instanceof net.minecraft.client.gui.components.AbstractWidget button) {
            focus = button.getMessage().getContents() instanceof net.minecraft.network.chat.contents.TranslatableContents key
                && key.getKey().equals("pack.openFolder") ? "folder" : "done";
        }
        out.addProperty("focus", focus);
        var selected = new JsonArray();
        for (String name : new String[] {"availablePackList", "selectedPackList"}) {
            var list = (net.minecraft.client.gui.components.AbstractSelectionList<?>) packList(screen, name);
            var entry = list.getSelected();
            selected.add(entry == null ? "none"
                : (String) Reflect.method(net.minecraft.client.gui.screens.packs.TransferableSelectionList.Entry.class,
                    "getPackId").invoke(entry));
        }
        out.add("selected", selected);
    }

    static JsonObject containerScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            setup();
            MovementOracle.bootstrap();
            var open = MovementOracle.openMenu(row);
            var minecraft = open.scene().minecraft();
            var instance = Reflect.field(net.minecraft.client.Minecraft.class, "instance");
            instance.set(null, minecraft);
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var drawn = prepare(minecraft, width, height, row.get("mouse_x").getAsInt(),
                row.get("mouse_y").getAsInt());
            set(minecraft, "soundManager", unsafe.allocateInstance(GuiScreenOracle.SilentSounds.class));
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
                    case "minecraft:mount" -> mountScreen(
                        (net.minecraft.world.inventory.AbstractMountInventoryMenu) menu, inventory, minecraft,
                        row.getAsJsonObject("mount").get("columns").getAsInt());
                    default -> throw new IllegalArgumentException("Unknown menu screen");
                };
            set(screen, "minecraft", minecraft);
            set(screen, "font", font);
            set(screen, "width", width);
            set(screen, "height", height);
            var init = Reflect.method(net.minecraft.client.gui.screens.Screen.class, "init");
            init.invoke(screen);
            if (row.has("sync_slot")) {
                // The server's update of a slot, as the screen hears it.
                int slot = row.get("sync_slot").getAsInt();
                if (menu instanceof net.minecraft.world.inventory.LecternMenu) {
                    // The lectern's screen hears the server's update of the book.
                    var heard = Reflect.field(net.minecraft.client.gui.screens.inventory.LecternScreen.class, "listener");
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
                var tick = Reflect.method(net.minecraft.client.gui.screens.inventory.AbstractContainerScreen.class, "containerTick");
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
                // The mount's model as the screen asks for it: the background draws it with the
                // mouse the screen saw in the frame before (none yet, so the corner).
                var entities = new com.google.gson.JsonArray();
                var last = new net.minecraft.client.renderer.state.gui.pip.GuiEntityRenderState[1];
                frame.state().forEachPictureInPicture(pip -> {
                    if (pip instanceof net.minecraft.client.renderer.state.gui.pip.GuiEntityRenderState entity
) {
                        last[0] = entity;
                    }
                });
                if (last[0] != null && row.has("mount")) {
                    entities.add(entityState(last[0]));
                }
                step.add("entities", entities);
                steps.add(step);
            }
            out.add("steps", steps);
            // Clicks come last, so what they change does not reach the drawn steps; only the
            // sounds they ask for are read.
            GuiScreenOracle.SilentSounds.ASKED.clear();
            // A sound the player makes is placed against the camera.
            Access26_2.renderThrough(minecraft, new net.minecraft.client.Camera());
            for (var click : row.getAsJsonArray("clicks")) {
                minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                int before = GuiScreenOracle.SilentSounds.ASKED.size();
                try {
                    screen.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                        click.getAsJsonArray().get(0).getAsDouble(), click.getAsJsonArray().get(1).getAsDouble(),
                        new net.minecraft.client.input.MouseButtonInfo(0, 0)), false);
                } catch (NullPointerException stub) {
                    if (GuiScreenOracle.SilentSounds.ASKED.size() == before) {
                        throw stub;
                    }
                    // What a press does with the game (closing the container, telling the server) has
                    // no game to reach here; the sound came first.
                }
            }
            var asked = new JsonArray();
            GuiScreenOracle.SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            GuiScreenOracle.SilentSounds.ASKED.clear();
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

    /**
     * The renderer the mount screens' model preview asks the dispatcher for: it makes a render
     * state that has the entity's height and nothing else (the model is not recorded).
     */
    static final class StubRenderer extends net.minecraft.client.renderer.entity.EntityRenderer<
        net.minecraft.world.entity.LivingEntity,
        net.minecraft.client.renderer.entity.state.LivingEntityRenderState> {
        private StubRenderer() {
            super(null);
        }

        @Override
        public net.minecraft.client.renderer.entity.state.LivingEntityRenderState createRenderState() {
            return new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
        }

        @Override
        public void extractRenderState(net.minecraft.world.entity.LivingEntity entity,
                net.minecraft.client.renderer.entity.state.LivingEntityRenderState state, float partialTicks) {
            state.boundingBoxHeight = entity.getBbHeight();
        }

        @Override
        protected void finalizeRenderState(net.minecraft.world.entity.LivingEntity entity,
                net.minecraft.client.renderer.entity.state.LivingEntityRenderState state) {}
    }

    /**
     * The game mode switcher over the game's own client: for each step the mouse where it is, and
     * the switch key pressed or not; every step draws the screen and reports what it picked.
     */
    static JsonObject gameModeSwitcher(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            setup();
            MovementOracle.bootstrap();
            var open = MovementOracle.openMenu(row);
            var minecraft = open.scene().minecraft();
            var instance = Reflect.field(net.minecraft.client.Minecraft.class, "instance");
            instance.set(null, minecraft);
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var first = row.getAsJsonArray("steps").get(0).getAsJsonObject().getAsJsonArray("mouse");
            var drawn = prepare(minecraft, width, height, first.get(0).getAsInt(), first.get(1).getAsInt());
            set(minecraft, "itemModelResolver", unsafe.allocateInstance(NoItems.class));
            var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
            set(renderer, "lighting", unsafe.allocateInstance(NoLighting.class));
            set(minecraft, "gameRenderer", renderer);
            set(minecraft, "level", open.scene().level());
            // The mode played in before, which the screen starts from.
            var gameMode = minecraft.gameMode;
            set(gameMode, "previousLocalPlayerMode", row.has("previous")
                ? net.minecraft.world.level.GameType.byName(row.get("previous").getAsString()) : null);
            var screen = new net.minecraft.client.gui.screens.debug.GameModeSwitcherScreen();
            set(screen, "minecraft", minecraft);
            set(screen, "font", font);
            set(screen, "width", width);
            set(screen, "height", height);
            var init = Reflect.method(net.minecraft.client.gui.screens.Screen.class, "init");
            init.invoke(screen);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var steps = new JsonArray();
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                int mouseX = step.getAsJsonArray("mouse").get(0).getAsInt();
                int mouseY = step.getAsJsonArray("mouse").get(1).getAsInt();
                if (step.has("key")) {
                    // The switch key (F4).
                    screen.keyPressed(new net.minecraft.client.input.KeyEvent(293, 0, 0));
                }
                var frame = prepare(minecraft, width, height, mouseX, mouseY);
                frame.graphics().nextStratum();
                screen.extractBackground(frame.graphics(), mouseX, mouseY, 0.0F);
                frame.graphics().nextStratum();
                screen.extractRenderState(frame.graphics(), mouseX, mouseY, 0.0F);
                var done = new JsonObject();
                done.add("ops", record(frame.state()));
                var field = Reflect.field(net.minecraft.client.gui.screens.debug.GameModeSwitcherScreen.class, "currentlyHovered");
                done.addProperty("selected", field.get(screen).toString());
                steps.add(done);
            }
            out.add("steps", steps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A number the way the comparison reads it: five decimals, no negative zero. */
    static double rounded(double value) {
        return Math.round(value * 100000.0) / 100000.0 + 0.0;
    }

    /** What `GuiGraphicsExtractor.entity` was asked: the box, the scale, the lift and the turns. */
    static JsonObject entityState(net.minecraft.client.renderer.state.gui.pip.GuiEntityRenderState state) {
        var out = new JsonObject();
        out.addProperty("x0", state.x0());
        out.addProperty("y0", state.y0());
        out.addProperty("x1", state.x1());
        out.addProperty("y1", state.y1());
        out.addProperty("scale", rounded(state.scale()));
        var translation = state.translation();
        out.addProperty("translation_y", rounded(translation.y()));
        var rotation = state.rotation();
        var quaternion = new com.google.gson.JsonArray();
        quaternion.add(rounded(rotation.x()));
        quaternion.add(rounded(rotation.y()));
        quaternion.add(rounded(rotation.z()));
        quaternion.add(rounded(rotation.w()));
        out.add("rotation", quaternion);
        if (state.renderState() instanceof net.minecraft.client.renderer.entity.state.LivingEntityRenderState living) {
            out.addProperty("body_rot", rounded(living.bodyRot));
            out.addProperty("y_rot", rounded(living.yRot));
            out.addProperty("x_rot", rounded(living.xRot));
        }
        return out;
    }

    /** The screen `ClientPacketListener.handleMountScreenOpen` opens for a mount menu. */
    static net.minecraft.client.gui.screens.Screen mountScreen(
        net.minecraft.world.inventory.AbstractMountInventoryMenu menu,
        net.minecraft.world.entity.player.Inventory inventory, net.minecraft.client.Minecraft minecraft,
        int columns) throws Exception {
        var field = Reflect.field(net.minecraft.world.inventory.AbstractMountInventoryMenu.class, "mount");
        var mount = (net.minecraft.world.entity.LivingEntity) field.get(menu);
        // The title is the entity's display name, which asks its team of the level's scoreboard.
        var connection = Reflect.field(minecraft.level.getClass(), "connection");
        set(connection.get(minecraft.level), "scoreboard", new net.minecraft.world.scores.Scoreboard());
        var stub = unsafe.allocateInstance(StubRenderer.class);
        var dispatcher = unsafe.allocateInstance(net.minecraft.client.renderer.entity.EntityRenderDispatcher.class);
        set(dispatcher, "renderers", new java.util.HashMap<Object, Object>() {
            @Override
            public Object get(Object key) {
                return stub;
            }
        });
        set(minecraft, "entityRenderDispatcher", dispatcher);
        if (menu instanceof net.minecraft.world.inventory.HorseInventoryMenu horseMenu) {
            return new net.minecraft.client.gui.screens.inventory.HorseInventoryScreen(
                horseMenu, inventory, (net.minecraft.world.entity.animal.equine.AbstractHorse) mount, columns);
        }
        return new net.minecraft.client.gui.screens.inventory.NautilusInventoryScreen(
            (net.minecraft.world.inventory.NautilusInventoryMenu) menu, inventory,
            (net.minecraft.world.entity.animal.nautilus.AbstractNautilus) mount, columns);
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
        var order = Reflect.field(Reflect.type("net.minecraft.client.KeyMapping$Category"), "SORT_ORDER");
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
