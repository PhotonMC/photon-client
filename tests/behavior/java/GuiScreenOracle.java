import net.minecraft.network.chat.Component;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.state.gui.GuiRenderState;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The pause, death, disconnected, confirm and options screens, and the widgets. */
class GuiScreenOracle extends GuiOverlayOracle {
    /** The game's own death screen, after some ticks, for a score and a cause. */
    static JsonObject deathScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            var scoreField = net.minecraft.world.entity.player.Player.class.getDeclaredField("DATA_SCORE_ID");
            scoreField.setAccessible(true);
            @SuppressWarnings("unchecked")
            var accessor = (net.minecraft.network.syncher.EntityDataAccessor<Integer>) scoreField.get(null);
            var data = (net.minecraft.network.syncher.SynchedEntityData)
                unsafe.allocateInstance(net.minecraft.network.syncher.SynchedEntityData.class);
            var items = new net.minecraft.network.syncher.SynchedEntityData.DataItem<?>[accessor.id() + 1];
            items[accessor.id()] = new net.minecraft.network.syncher.SynchedEntityData.DataItem<>(
                accessor, row.get("score").getAsInt());
            set(data, "itemsById", items);
            set(player, "entityData", data);
            set(minecraft, "player", player);
            set(minecraft, "reportingContext",
                unsafe.allocateInstance(net.minecraft.client.multiplayer.chat.report.ReportingContext.class));
            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(minecraft, "level", level);
            Component cause = row.has("cause") ? Component.literal(row.get("cause").getAsString()) : null;
            var death = new net.minecraft.client.gui.screens.DeathScreen(cause,
                row.get("hardcore").getAsBoolean(), (net.minecraft.client.player.LocalPlayer) player);
            set(death, "width", width);
            set(death, "height", height);
            var init = net.minecraft.client.gui.screens.DeathScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(death);
            for (int tick = 0; tick < row.get("ticks").getAsInt(); tick++) {
                death.tick();
            }
            death.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            death.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The game's own disconnected screen, for a reason. */
    static JsonObject disconnectedScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            set(minecraft, "allowsMultiplayer", true);
            set(minecraft, "userPropertiesFuture", java.util.concurrent.CompletableFuture.completedFuture(
                com.mojang.authlib.minecraft.UserApiService.OFFLINE_PROPERTIES));
            set(minecraft, "profileFuture", java.util.concurrent.CompletableFuture.completedFuture(null));
            var disconnected = new net.minecraft.client.gui.screens.DisconnectedScreen(null,
                Component.translatable("disconnect.lost"), Component.literal(row.get("reason").getAsString()));
            set(disconnected, "width", width);
            set(disconnected, "height", height);
            var init = net.minecraft.client.gui.screens.DisconnectedScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(disconnected);
            var background = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("extractMenuBackground",
                GuiGraphicsExtractor.class);
            background.setAccessible(true);
            background.invoke(disconnected, screen.graphics());
            disconnected.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    static JsonObject pauseScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            // The clock the tooltips wait by is the row's to move.
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "registryAccess", (net.minecraft.core.RegistryAccess.Frozen) Oracle.vanillaRegistries());
            set(listener, "serverLinks", net.minecraft.server.ServerLinks.EMPTY);
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(level, "connection", listener);
            set(minecraft, "level", level);
            set(minecraft, "playerSocialManager",
                unsafe.allocateInstance(net.minecraft.client.gui.screens.social.PlayerSocialManager.class));
            set(minecraft, "reportingContext",
                unsafe.allocateInstance(net.minecraft.client.multiplayer.chat.report.ReportingContext.class));
            var pause = new net.minecraft.client.gui.screens.PauseScreen(true);
            set(pause, "width", width);
            set(pause, "height", height);
            var init = net.minecraft.client.gui.screens.PauseScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(pause);
            if (row.has("steps")) {
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    if (step.get("do").getAsString().equals("frame")) {
                        // A frame drawn and thrown away, at a time.
                        now[0] = step.get("ms").getAsLong();
                        var frame = new GuiRenderState();
                        var graphics = new GuiGraphicsExtractor(minecraft, frame, mouseX, mouseY);
                        pause.extractRenderState(graphics, mouseX, mouseY, 0.0F);
                        graphics.extractDeferredElements(mouseX, mouseY, 0.0F);
                    } else {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "tab", "shift+tab" -> 258;
                            case "left" -> 263;
                            case "right" -> 262;
                            case "up" -> 265;
                            case "down" -> 264;
                            default -> throw new IllegalArgumentException(name);
                        };
                        minecraft.setLastInputType(code == 258 ? net.minecraft.client.InputType.KEYBOARD_TAB
                            : net.minecraft.client.InputType.KEYBOARD_ARROW);
                        pause.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                }
            }
            now[0] = row.has("ms") ? row.get("ms").getAsLong() : 0;
            pause.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            pause.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A sound manager that plays nothing, for the buttons that click when pressed. */
    static final class SilentSounds extends net.minecraft.client.sounds.SoundManager {
        SilentSounds() {
            super(null);
        }

        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
                net.minecraft.client.resources.sounds.SoundInstance instance) {
            return null;
        }

        // What the sound screen's widgets tell the manager when they change.
        @Override
        public void refreshCategoryVolume(net.minecraft.sounds.SoundSource category) {}

        @Override
        public void reload() {}

        @Override
        public boolean isActive(net.minecraft.client.resources.sounds.SoundInstance instance) {
            return false;
        }

        @Override
        public void stop(net.minecraft.client.resources.sounds.SoundInstance instance) {}

        @Override
        public java.util.List<String> getAvailableSoundDevices() {
            return java.util.List.of();
        }
    }

    /** A music manager with no music: the sound screen's sliders and cycles tell it. */
    static final class SilentMusic extends net.minecraft.client.sounds.MusicManager {
        SilentMusic() {
            super(null);
        }

        @Override
        public void showNowPlayingToastIfNeeded() {}

        @Override
        public void setMinutesBetweenSongs(net.minecraft.client.sounds.MusicManager.MusicFrequency frequency) {}
    }

    /**
     * The game's own confirmation screen (or the death screen's own), after some ticks and
     * some key presses and a click: what its callback was told, and what it draws.
     */
    static JsonObject confirmScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            if (row.get("in_world").getAsBoolean()) {
                set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            }
            var callbacks = new JsonArray();
            it.unimi.dsi.fastutil.booleans.BooleanConsumer callback =
                result -> callbacks.add(result ? "yes" : "no");
            Component title = Component.literal(row.get("title").getAsString());
            Component message = Component.literal(row.get("message").getAsString());
            Component yes = Component.literal(row.get("yes").getAsString());
            Component no = Component.literal(row.get("no").getAsString());
            if (row.has("pack")) {
                // The wording of ClientCommonPacketListenerImpl.PackConfirmScreen, which is private.
                var pack = row.getAsJsonObject("pack");
                boolean required = pack.get("required").getAsBoolean();
                Component prompt = pack.has("prompt") ? Component.literal(pack.get("prompt").getAsString()) : null;
                Component header = required
                    ? Component.translatable("multiplayer.requiredTexturePrompt.line2")
                        .withStyle(net.minecraft.ChatFormatting.YELLOW, net.minecraft.ChatFormatting.BOLD)
                    : Component.translatable("multiplayer.texturePrompt.line2");
                title = required
                    ? Component.translatable("multiplayer.requiredTexturePrompt.line1")
                    : Component.translatable("multiplayer.texturePrompt.line1");
                message = prompt == null ? header
                    : Component.translatable("multiplayer.texturePrompt.serverPrompt", header, prompt);
                yes = required ? net.minecraft.network.chat.CommonComponents.GUI_PROCEED
                    : net.minecraft.network.chat.CommonComponents.GUI_YES;
                no = required ? net.minecraft.network.chat.CommonComponents.GUI_DISCONNECT
                    : net.minecraft.network.chat.CommonComponents.GUI_NO;
            }
            net.minecraft.client.gui.screens.ConfirmScreen confirm =
                row.has("link")
                    ? new net.minecraft.client.gui.screens.ConfirmLinkScreen(callback,
                        row.getAsJsonObject("link").get("url").getAsString(),
                        row.getAsJsonObject("link").get("trusted").getAsBoolean())
                : row.get("death").getAsBoolean()
                    ? new net.minecraft.client.gui.screens.DeathScreen.TitleConfirmScreen(
                        callback, title, message, yes, no)
                    : new net.minecraft.client.gui.screens.ConfirmScreen(callback, title, message, yes, no);
            set(confirm, "width", width);
            set(confirm, "height", height);
            var init = net.minecraft.client.gui.screens.ConfirmScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(confirm);
            int delay = row.get("delay").getAsInt();
            if (delay > 0) {
                confirm.setDelay(delay);
            }
            for (int tick = 0; tick < row.get("ticks").getAsInt(); tick++) {
                confirm.tick();
            }
            for (var key : row.getAsJsonArray("keys")) {
                String name = key.getAsString();
                int code = switch (name) {
                    case "tab", "shift+tab" -> 258;
                    case "escape" -> 256;
                    case "enter" -> 257;
                    case "space" -> 32;
                    default -> throw new IllegalArgumentException(name);
                };
                confirm.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
            }
            if (row.get("click").getAsBoolean()) {
                confirm.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(mouseX, mouseY,
                    new net.minecraft.client.input.MouseButtonInfo(0, 0)), false);
            }
            if (row.get("death").getAsBoolean()) {
                confirm.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            } else {
                // Without a level the panorama would draw first, which is not a GUI drawing.
                var background = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod(
                    "extractMenuBackground", GuiGraphicsExtractor.class);
                background.setAccessible(true);
                background.invoke(confirm, screen.graphics());
            }
            confirm.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("callbacks", callbacks);
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The widgets an option makes (`OptionInstance.createButton`): a slider or a cycle button
     * for an option set to a value, after some focus, mouse and key steps: its drawing and the
     * option's value.
     */
    @SuppressWarnings({"unchecked", "rawtypes"})
    static JsonObject widgets(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(427, 240, mouseX, mouseY);
            var minecraft = screen.minecraft();
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            var options = minecraft.options;
            String kind = row.get("kind").getAsString();
            net.minecraft.client.OptionInstance instance = switch (kind) {
                case "fov" -> options.fov();
                case "sensitivity" -> options.sensitivity();
                case "chat_opacity" -> options.chatOpacity();
                case "invert_y" -> options.invertMouseY();
                case "chat_visibility" -> options.chatVisibility();
                default -> throw new IllegalArgumentException(kind);
            };
            var value = row.get("value");
            instance.set(switch (kind) {
                case "fov" -> (Object) value.getAsInt();
                case "invert_y" -> (Object) value.getAsBoolean();
                case "chat_visibility" ->
                    (Object) net.minecraft.world.entity.player.ChatVisiblity.values()[value.getAsInt()];
                default -> (Object) value.getAsDouble();
            });
            var widget = instance.createButton(options, 100, 50, row.get("width").getAsInt());
            widget.active = row.get("active").getAsBoolean();
            var info = new net.minecraft.client.input.MouseButtonInfo(0, 0);
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                String action = step.get("do").getAsString();
                switch (action) {
                    case "focus" -> {
                        minecraft.setLastInputType(net.minecraft.client.InputType.valueOf(
                            step.get("input").getAsString()));
                        widget.setFocused(true);
                    }
                    case "unfocus" -> widget.setFocused(false);
                    case "click" -> widget.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                        new net.minecraft.client.input.MouseButtonInfo(0, step.get("shift").getAsBoolean() ? 1 : 0)),
                        false);
                    case "drag" -> widget.mouseDragged(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), 60, info), 0, 0);
                    case "release" -> widget.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), 60, info));
                    case "key" -> {
                        int code = switch (step.get("key").getAsString()) {
                            case "left" -> 263;
                            case "right" -> 262;
                            case "enter" -> 257;
                            case "space" -> 32;
                            default -> throw new IllegalArgumentException(step.get("key").getAsString());
                        };
                        widget.keyPressed(new net.minecraft.client.input.KeyEvent(
                            code, 0, step.get("shift").getAsBoolean() ? 1 : 0));
                    }
                    case "scroll" -> widget.mouseScrolled(110, 60, 0, step.get("dy").getAsDouble());
                    default -> throw new IllegalArgumentException(action);
                }
            }
            widget.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            Object result = instance.get();
            out.addProperty("value", switch (kind) {
                case "fov" -> Integer.toString((Integer) result);
                case "invert_y" -> Boolean.toString((Boolean) result);
                case "chat_visibility" -> Integer.toString(((net.minecraft.world.entity.player.ChatVisiblity) result).ordinal());
                default -> String.format("%016x", Double.doubleToRawLongBits((Double) result));
            });
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own options screen, after some focus, key and mouse steps: what it draws and
     * the field of view the slider left. The buttons that open other screens are not pressed.
     */
    static JsonObject optionsScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            set(minecraft, "userPropertiesFuture", java.util.concurrent.CompletableFuture.completedFuture(
                com.mojang.authlib.minecraft.UserApiService.OFFLINE_PROPERTIES));
            boolean inWorld = row.get("in_world").getAsBoolean();
            if (inWorld) {
                set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            }
            minecraft.options.fov().set(row.get("fov").getAsInt());
            var options = new net.minecraft.client.gui.screens.options.OptionsScreen(null, minecraft.options, inWorld);
            set(options, "width", width);
            set(options, "height", height);
            var init = net.minecraft.client.gui.screens.options.OptionsScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(options);
            var info = new net.minecraft.client.input.MouseButtonInfo(0, 0);
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                String action = step.get("do").getAsString();
                switch (action) {
                    case "key" -> {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "tab", "shift+tab" -> 258;
                            case "left" -> 263;
                            case "right" -> 262;
                            case "up" -> 265;
                            case "down" -> 264;
                            case "enter" -> 257;
                            case "space" -> 32;
                            default -> throw new IllegalArgumentException(name);
                        };
                        // What `KeyboardHandler` notes of the keys before the screen has them.
                        if (code == 258) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                        } else if (code == 262 || code == 263 || code == 264 || code == 265) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_ARROW);
                        }
                        options.keyPressed(new net.minecraft.client.input.KeyEvent(
                            code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                    case "click" -> {
                        minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                        options.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                            step.get("x").getAsDouble(), step.get("y").getAsDouble(), info), false);
                    }
                    case "drag" -> options.mouseDragged(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(), info), 0, 0);
                    case "release" -> options.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(), info));
                    default -> throw new IllegalArgumentException(action);
                }
            }
            var background = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("extractMenuBackground",
                GuiGraphicsExtractor.class);
            background.setAccessible(true);
            background.invoke(options, screen.graphics());
            options.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("fov", minecraft.options.fov().get());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
