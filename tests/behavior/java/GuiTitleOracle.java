import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.state.gui.GuiRenderState;

/** The title screen and the server list, for a client that may play online but reaches no service. */
class GuiTitleOracle extends GuiSettingsOracle {
    /** The game's own title screen with a given splash, at a time, under a mouse, after keys. */
    static JsonObject titleScreen(JsonObject request) throws Exception {
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
            SilentSounds.ASKED.clear();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var panorama = menu(screen);
            var title = plainLogo(new net.minecraft.client.gui.screens.TitleScreen(false));
            panorama.holdSpin();
            var literal = net.minecraft.client.resources.SplashManager.class.getDeclaredMethod("literalSplash", String.class);
            literal.setAccessible(true);
            set(title, "splash", new net.minecraft.client.gui.components.SplashRenderer(
                (net.minecraft.network.chat.Component) literal.invoke(null, row.get("splash").getAsString())));
            minecraft.options.hideSplashTexts().set(row.has("hide_splash") && row.get("hide_splash").getAsBoolean());
            // The Realms notifications, for a client Realms does not answer: nothing to show.
            var realms = (com.mojang.realmsclient.gui.screens.RealmsNotificationsScreen)
                unsafe.allocateInstance(com.mojang.realmsclient.gui.screens.RealmsNotificationsScreen.class);
            set(realms, "validClient", java.util.concurrent.CompletableFuture.completedFuture(false));
            set(realms, "minecraft", minecraft);
            set(realms, "font", minecraft.font);
            set(realms, "renderables", new java.util.ArrayList<>());
            set(realms, "children", new java.util.ArrayList<>());
            set(realms, "narratables", new java.util.ArrayList<>());
            set(title, "realmsNotificationsScreen", realms);
            set(title, "width", width);
            set(title, "height", height);
            var init = net.minecraft.client.gui.screens.TitleScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(title);
            if (row.has("steps")) {
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    if (step.get("do").getAsString().equals("frame")) {
                        now[0] = step.get("ms").getAsLong();
                        var frame = new GuiRenderState();
                        var graphics = new GuiGraphicsExtractor(minecraft, frame, mouseX, mouseY);
                        title.extractRenderState(graphics, mouseX, mouseY, 0.0F);
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
                        title.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                }
            }
            now[0] = row.has("ms") ? row.get("ms").getAsLong() : 0;
            title.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            title.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
    /**
     * The game around a menu: online play allowed (the offline profile's properties, no ban, no
     * name change) and the panorama, which the caller holds still; its cube is drawn apart.
     */
    static net.minecraft.client.renderer.Panorama menu(Screen screen) throws Exception {
        var minecraft = screen.minecraft();
        set(minecraft, "allowsMultiplayer", true);
        // A pressed button clicks, heard by no one.
        set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
        var offline = com.mojang.authlib.minecraft.UserApiService.class.getField("OFFLINE_PROPERTIES").get(null);
        set(minecraft, "userPropertiesFuture", java.util.concurrent.CompletableFuture.completedFuture(offline));
        set(minecraft, "profileFuture", java.util.concurrent.CompletableFuture.completedFuture(null));
        set(minecraft, "playerSocialManager",
            unsafe.allocateInstance(net.minecraft.client.gui.screens.social.PlayerSocialManager.class));
        var gameRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        var panorama = new net.minecraft.client.renderer.Panorama();
        panorama.holdSpin();
        set(gameRenderer, "panorama", panorama);
        var gameState = unsafe.allocateInstance(net.minecraft.client.renderer.state.GameRenderState.class);
        set(gameState, "guiRenderState", screen.state());
        set(gameRenderer, "gameRenderState", gameState);
        set(minecraft, "gameRenderer", gameRenderer);
        return panorama;
    }

    /**
     * A server's data after its ping, as `ServerStatusPinger` leaves it: the answer read by the
     * game's own `ServerStatus.CODEC`, its fields set as `handleStatusResponse` sets them, the pong's
     * time; or the red reason of a ping that failed.
     */
    static void pinged(net.minecraft.client.multiplayer.ServerData server, JsonObject ping) throws Exception {
        if (ping.has("unreachable")) {
            var field = net.minecraft.client.gui.screens.multiplayer.ServerSelectionList.class.getDeclaredField(
                ping.get("unknown_host").getAsBoolean() ? "CANT_RESOLVE_TEXT" : "CANT_CONNECT_TEXT");
            field.setAccessible(true);
            server.setState(net.minecraft.client.multiplayer.ServerData.State.UNREACHABLE);
            server.motd = (net.minecraft.network.chat.Component) field.get(null);
            server.status = net.minecraft.network.chat.CommonComponents.EMPTY;
            return;
        }
        var status = net.minecraft.network.protocol.status.ServerStatus.CODEC.parse(
            com.mojang.serialization.JsonOps.INSTANCE,
            com.google.gson.JsonParser.parseString(ping.get("json").getAsString())).getOrThrow();
        server.motd = status.description();
        status.version().ifPresentOrElse(version -> {
            server.version = net.minecraft.network.chat.Component.literal(version.name());
            server.protocol = version.protocol();
        }, () -> {
            server.version = net.minecraft.network.chat.Component.translatable("multiplayer.status.old");
            server.protocol = 0;
        });
        status.players().ifPresentOrElse(players -> {
            server.status = net.minecraft.client.multiplayer.ServerStatusPinger.formatPlayerCount(players.online(), players.max());
            server.players = players;
            var names = new java.util.ArrayList<net.minecraft.network.chat.Component>();
            for (var profile : players.sample()) {
                names.add(profile.equals(net.minecraft.server.MinecraftServer.ANONYMOUS_PLAYER_PROFILE)
                    ? net.minecraft.network.chat.Component.translatable("multiplayer.status.anonymous_player")
                    : net.minecraft.network.chat.Component.literal(profile.name()));
            }
            if (!players.sample().isEmpty() && players.sample().size() < players.online()) {
                names.add(net.minecraft.network.chat.Component.translatable("multiplayer.status.and_more",
                    players.online() - players.sample().size()));
            }
            server.playerList = names;
        }, () -> server.status = net.minecraft.network.chat.Component.translatable("multiplayer.status.unknown")
            .withStyle(net.minecraft.ChatFormatting.DARK_GRAY));
        status.favicon().ifPresent(newIcon -> {
            if (!java.util.Arrays.equals(newIcon.iconBytes(), server.getIconBytes())) {
                server.setIconBytes(net.minecraft.client.multiplayer.ServerData.validateIcon(newIcon.iconBytes()));
            }
        });
        server.ping = ping.get("ping").getAsLong();
        server.setState(server.protocol == net.minecraft.SharedConstants.getCurrentVersion().protocolVersion()
            ? net.minecraft.client.multiplayer.ServerData.State.SUCCESSFUL
            : net.minecraft.client.multiplayer.ServerData.State.INCOMPATIBLE);
    }

    /** A texture that takes pixels and uploads nothing: the icon's GPU side, which a headless run has none of. */
    static class NoGpuTexture extends net.minecraft.client.renderer.texture.DynamicTexture {
        NoGpuTexture() {
            super(() -> "", new com.mojang.blaze3d.platform.NativeImage(1, 1, false));
        }

        @Override
        public void setPixels(com.mojang.blaze3d.platform.NativeImage pixels) {
        }

        @Override
        public void upload() {
        }

        @Override
        public void close() {
        }
    }

    /**
     * The entry's icon with a GPU-less texture in it where the game's upload would reach the GPU
     * (a PNG whose header says 64x64), so the rest of `FaviconTexture.upload` (the registration)
     * runs; elsewhere the icon is left as the game makes it and its own upload rejects the image.
     */
    static void presetIcon(Object entry, byte[] bytes) throws Exception {
        if (bytes == null || bytes.length < 24 || java.nio.ByteBuffer.wrap(bytes).getInt(16) != 64
            || java.nio.ByteBuffer.wrap(bytes).getInt(20) != 64) {
            return;
        }
        var iconField = entry.getClass().getDeclaredField("icon");
        iconField.setAccessible(true);
        var icon = iconField.get(entry);
        set(icon, "texture", unsafe.allocateInstance(NoGpuTexture.class));
    }

    /**
     * The game's own server list over a `servers.dat` of the row's names, each server still being
     * asked about (as the list makes it before its ping is sent), after clicks and scrolls.
     */
    static JsonObject joinMultiplayerScreen(JsonObject request) throws Exception {
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
            SilentSounds.ASKED.clear();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {row.get("ms").getAsLong()};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var panorama = menu(screen);
            // The saved servers, written where the list reads them.
            var saved = new net.minecraft.client.multiplayer.ServerList(minecraft);
            for (var name : row.getAsJsonArray("servers")) {
                saved.add(new net.minecraft.client.multiplayer.ServerData(name.getAsString(), "127.0.0.1:1",
                    net.minecraft.client.multiplayer.ServerData.Type.OTHER), false);
            }
            saved.save();
            var list = new net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen(
                plainLogo(new net.minecraft.client.gui.screens.TitleScreen(false)));
            // The title screen behind starts the panorama turning again.
            panorama.holdSpin();
            list.init(width, height);
            var pings = row.has("pings") ? row.getAsJsonArray("pings") : new JsonArray();
            var entriesField = net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen.class
                .getDeclaredField("serverSelectionList");
            entriesField.setAccessible(true);
            var selection = (net.minecraft.client.gui.screens.multiplayer.ServerSelectionList) entriesField.get(list);
            for (int i = 0; i < list.getServers().size(); i++) {
                var server = list.getServers().get(i);
                server.setState(net.minecraft.client.multiplayer.ServerData.State.PINGING);
                server.motd = net.minecraft.network.chat.CommonComponents.EMPTY;
                server.status = net.minecraft.network.chat.CommonComponents.EMPTY;
                if (i < pings.size() && !pings.get(i).isJsonNull()) {
                    pinged(server, pings.get(i).getAsJsonObject());
                    var refresh = net.minecraft.client.gui.screens.multiplayer.ServerSelectionList.OnlineServerEntry.class
                        .getDeclaredMethod("refreshStatus");
                    refresh.setAccessible(true);
                    refresh.invoke(selection.children().get(i));
                }
                presetIcon(selection.children().get(i), server.getIconBytes());
            }
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                double x = step.get("x").getAsDouble();
                double y = step.get("y").getAsDouble();
                switch (step.get("do").getAsString()) {
                    case "click" -> list.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(x, y,
                        new net.minecraft.client.input.MouseButtonInfo(0, 0)), false);
                    case "scroll" -> list.mouseScrolled(x, y, 0.0, step.get("dy").getAsDouble());
                    default -> throw new IllegalArgumentException(step.toString());
                }
            }
            list.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            list.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            list.removed();
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
    /**
     * The game's own forms of the server list (adding, editing, joining directly) for a server of
     * the row, after text typed, keys and clicks.
     */
    static JsonObject serverForm(JsonObject request) throws Exception {
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
            SilentSounds.ASKED.clear();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> 0L);
            var panorama = menu(screen);
            var last = plainLogo(new net.minecraft.client.gui.screens.TitleScreen(false));
            panorama.holdSpin();
            var data = new net.minecraft.client.multiplayer.ServerData(row.get("name").getAsString(),
                row.get("ip").getAsString(), net.minecraft.client.multiplayer.ServerData.Type.OTHER);
            data.setResourcePackStatus(net.minecraft.client.multiplayer.ServerData.ServerPackStatus.valueOf(
                row.get("pack").getAsString()));
            String kind = row.get("kind").getAsString();
            net.minecraft.client.gui.screens.Screen form;
            if (kind.equals("direct")) {
                minecraft.options.lastMpIp = row.get("ip").getAsString();
                form = new net.minecraft.client.gui.screens.DirectJoinServerScreen(last, accepted -> {}, data);
            } else {
                var title = net.minecraft.network.chat.Component.translatable(
                    kind.equals("add") ? "manageServer.add.title" : "manageServer.edit.title");
                form = new net.minecraft.client.gui.screens.ManageServerScreen(last, title, accepted -> {}, data);
            }
            form.init(width, height);
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                switch (step.get("do").getAsString()) {
                    case "type" -> step.get("text").getAsString().codePoints().forEach(cp ->
                        form.charTyped(new net.minecraft.client.input.CharacterEvent(cp)));
                    case "key" -> {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "tab", "shift+tab" -> 258;
                            case "backspace" -> 259;
                            case "enter" -> 257;
                            case "left" -> 263;
                            case "home" -> 268;
                            case "space" -> 32;
                            default -> throw new IllegalArgumentException(name);
                        };
                        minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                        form.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                    case "click" -> form.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                        new net.minecraft.client.input.MouseButtonInfo(0, 0)), false);
                    default -> throw new IllegalArgumentException(step.toString());
                }
            }
            form.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            form.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
    /** The game's own screens of a join: connecting (with a status) and loading the terrain. */
    static JsonObject connectScreen(JsonObject request) throws Exception {
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
            SilentSounds.ASKED.clear();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            var panorama = menu(screen);
            var last = plainLogo(new net.minecraft.client.gui.screens.TitleScreen(false));
            panorama.holdSpin();
            net.minecraft.client.gui.screens.Screen shown;
            if (row.get("screen").getAsString().equals("loading")) {
                var reason = row.has("reason")
                    ? net.minecraft.client.gui.screens.LevelLoadingScreen.Reason.valueOf(row.get("reason").getAsString().toUpperCase())
                    : net.minecraft.client.gui.screens.LevelLoadingScreen.Reason.OTHER;
                if (reason == net.minecraft.client.gui.screens.LevelLoadingScreen.Reason.NETHER_PORTAL) {
                    GuiHudOracle.stubPortalSprite(minecraft);
                }
                shown = new net.minecraft.client.gui.screens.LevelLoadingScreen(
                    new net.minecraft.client.multiplayer.LevelLoadTracker(), reason);
            } else if (row.get("screen").getAsString().equals("reconfiguring")) {
                var connection = new net.minecraft.network.Connection(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
                var reconfig = new net.minecraft.client.gui.screens.multiplayer.ServerReconfigScreen(
                    net.minecraft.network.chat.Component.translatable("connect.reconfiguring"), connection);
                shown = reconfig;
                shown.init(width, height);
                for (int tick = 0; tick < row.get("ticks").getAsInt(); tick++) {
                    var delay = net.minecraft.client.gui.screens.multiplayer.ServerReconfigScreen.class.getDeclaredField("delayTicker");
                    delay.setAccessible(true);
                    delay.setInt(reconfig, tick + 1);
                    if (tick + 1 == 600) {
                        var button = net.minecraft.client.gui.screens.multiplayer.ServerReconfigScreen.class.getDeclaredField("disconnectButton");
                        button.setAccessible(true);
                        ((net.minecraft.client.gui.components.Button) button.get(reconfig)).active = true;
                    }
                }
            } else {
                var constructor = net.minecraft.client.gui.screens.ConnectScreen.class.getDeclaredConstructor(
                    net.minecraft.client.gui.screens.Screen.class, net.minecraft.network.chat.Component.class);
                constructor.setAccessible(true);
                shown = constructor.newInstance(last, net.minecraft.network.chat.CommonComponents.CONNECT_FAILED);
                var update = net.minecraft.client.gui.screens.ConnectScreen.class.getDeclaredMethod(
                    "updateStatus", net.minecraft.network.chat.Component.class);
                update.setAccessible(true);
                update.invoke(shown, net.minecraft.network.chat.Component.translatable(row.get("status").getAsString()));
            }
            if (!row.get("screen").getAsString().equals("reconfiguring")) {
                shown.init(width, height);
            }
            shown.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            shown.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
